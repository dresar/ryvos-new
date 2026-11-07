use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use tracing::{debug, info, warn};
use uuid::Uuid;

use ryvos_core::error::{Result, RyvosError};
use ryvos_core::event::EventBus;
use ryvos_core::security::{
    summarize_input, tool_has_side_effects, ApprovalDecision, ApprovalRequest, SecurityPolicy,
    SecurityTier,
};
use ryvos_core::traits::Tool;
use ryvos_core::types::{ToolContext, ToolDefinition, ToolIndexEntry, ToolResult, TrustLevel};
use ryvos_tools::ToolRegistry;

use crate::approval::ApprovalBroker;
use crate::audit::{AuditEntry, AuditTrail};
use crate::safety_memory::{assess_outcome, SafetyMemory, SafetyOutcome};

pub struct SecurityGate {
    policy: SecurityPolicy,
    tools: Arc<tokio::sync::RwLock<ToolRegistry>>,
    broker: Arc<ApprovalBroker>,
    #[allow(dead_code)]
    event_bus: Arc<EventBus>,
    safety_memory: Option<Arc<SafetyMemory>>,
    audit_trail: Option<Arc<AuditTrail>>,
    tainted_sessions: Arc<tokio::sync::RwLock<HashSet<String>>>,
}

impl SecurityGate {
    pub fn new(
        policy: SecurityPolicy,
        tools: Arc<tokio::sync::RwLock<ToolRegistry>>,
        broker: Arc<ApprovalBroker>,
        event_bus: Arc<EventBus>,
    ) -> Self {
        Self {
            policy,
            tools,
            broker,
            event_bus,
            safety_memory: None,
            audit_trail: None,
            tainted_sessions: Arc::new(tokio::sync::RwLock::new(HashSet::new())),
        }
    }

    pub fn set_safety_memory(&mut self, memory: Arc<SafetyMemory>) {
        self.safety_memory = Some(memory);
    }

    pub fn set_audit_trail(&mut self, trail: Arc<AuditTrail>) {
        self.audit_trail = Some(trail);
    }

    pub async fn mark_session_tainted(&self, session_id: &str) {
        let mut lock = self.tainted_sessions.write().await;
        lock.insert(session_id.to_string());
    }

    pub async fn is_session_tainted(&self, session_id: &str) -> bool {
        let lock = self.tainted_sessions.read().await;
        lock.contains(session_id)
    }

    pub async fn clear_session_taint(&self, session_id: &str) {
        let mut lock = self.tainted_sessions.write().await;
        lock.remove(session_id);
    }

    pub async fn execute(
        &self,
        name: &str,
        input: serde_json::Value,
        ctx: ToolContext,
    ) -> Result<ToolResult> {
        let tool = {
            let tools = self.tools.read().await;
            tools
                .get(name)
                .ok_or_else(|| RyvosError::ToolNotFound(name.to_string()))?
        };

        let input_summary = summarize_input(name, &input);

        let mut lesson_ids = Vec::new();
        if let Some(ref memory) = self.safety_memory {
            if let Ok(lessons) = memory.relevant_lessons(name, 3).await {
                if !lessons.is_empty() {
                    info!(
                        tool = name,
                        lesson_count = lessons.len(),
                        "Safety memory: relevant lessons available"
                    );
                    lesson_ids = lessons.iter().map(|l| l.id.clone()).collect();
                }
            }
        }

        let safety_reasoning = Some(match (tool_has_side_effects(name), lesson_ids.len()) {
            (false, 0) => "Read-only, no prior incidents".to_string(),
            (true, 0) => format!("Side-effect tool ({})", name),
            (false, n) => format!("{} lesson(s) from past experience", n),
            (true, n) => format!("Side-effect tool ({}); {} lesson(s) available", name, n),
        });

        let session_id_str = ctx.session_id.to_string();
        let is_tainted = self.is_session_tainted(&session_id_str).await;

        if is_tainted && (tool.tier() >= SecurityTier::T2 || tool_has_side_effects(name)) {
            let req = ApprovalRequest {
                id: Uuid::new_v4().to_string(),
                tool_name: name.to_string(),
                tier: tool.tier(),
                input_summary: format!("Execution of {} under tainted context", name),
                session_id: session_id_str.clone(),
                timestamp: Utc::now(),
            };

            let rx = self.broker.request(req).await;
            let timeout = Duration::from_secs(self.policy.approval_timeout_secs);
            match tokio::time::timeout(timeout, rx).await {
                Ok(Ok(ApprovalDecision::Approved)) => {
                    debug!(tool = name, "Tainted execution approved");
                }
                Ok(Ok(ApprovalDecision::Denied { reason })) => {
                    warn!(tool = name, reason = %reason, "Tainted execution denied by policy");
                    return Err(RyvosError::ApprovalDenied {
                        tool: name.to_string(),
                        reason: format!("Denied by policy: {}", reason),
                    });
                }
                _ => {
                    warn!(tool = name, "Tainted execution timed out waiting for approval");
                    return Err(RyvosError::ApprovalDenied {
                        tool: name.to_string(),
                        reason: "Tainted execution timed out waiting for approval".to_string(),
                    });
                }
            }
        } else if self.policy.should_pause(name) && tool_has_side_effects(name) {
            let req = ApprovalRequest {
                id: Uuid::new_v4().to_string(),
                tool_name: name.to_string(),
                tier: tool.tier(),
                input_summary: input_summary.clone(),
                session_id: session_id_str.clone(),
                timestamp: Utc::now(),
            };

            let rx = self.broker.request(req).await;
            let timeout = Duration::from_secs(self.policy.approval_timeout_secs);

            match tokio::time::timeout(timeout, rx).await {
                Ok(Ok(ApprovalDecision::Denied { reason })) => {
                    warn!(tool = name, reason = %reason, "User denied soft checkpoint");
                    return Err(RyvosError::ApprovalDenied {
                        tool: name.to_string(),
                        reason,
                    });
                }
                Ok(Ok(ApprovalDecision::Approved)) => {
                    debug!(tool = name, "Soft checkpoint approved");
                }
                Ok(Err(_)) | Err(_) => {
                    debug!(tool = name, "Soft checkpoint timed out — proceeding");
                }
            }
        }

        let result = self
            .execute_tool_direct(&tool, name, input.clone(), ctx.clone())
            .await;

        if let Ok(ref tool_result) = result {
            if tool_result.trust == TrustLevel::Untrusted
                || tool_result.content.contains("<untrusted_content>")
            {
                self.mark_session_tainted(&session_id_str).await;
            }
        }

        match &result {
            Ok(tool_result) => {
                let outcome =
                    assess_outcome(name, &input, &tool_result.content, tool_result.is_error);

                if let Some(ref trail) = self.audit_trail {
                    let output_preview: String = tool_result.content.chars().take(200).collect();
                    let entry = AuditEntry {
                        timestamp: Utc::now(),
                        session_id: ctx.session_id.to_string(),
                        tool_name: name.to_string(),
                        input_summary: input_summary.clone(),
                        output_summary: output_preview,
                        safety_reasoning: safety_reasoning.clone(),
                        outcome: outcome.clone(),
                        lessons_available: lesson_ids.clone(),
                    };
                    if let Err(e) = trail.log_tool_call(&entry).await {
                        debug!(error = %e, "Failed to log post-execution audit entry");
                    }
                }

                if let Some(ref memory) = self.safety_memory {
                    match &outcome {
                        SafetyOutcome::Incident {
                            what_happened,
                            severity,
                        } => {
                            let lesson = crate::safety_memory::SafetyLesson {
                                id: Uuid::new_v4().to_string(),
                                timestamp: Utc::now(),
                                action: format!("{}({})", name, input_summary),
                                outcome: outcome.clone(),
                                reflection: format!(
                                    "Tool {} resulted in {:?} incident: {}",
                                    name, severity, what_happened
                                ),
                                principle_violated: None,
                                corrective_rule: format!(
                                    "Be cautious with {} — verify preconditions before executing",
                                    name
                                ),
                                confidence: match severity {
                                    crate::safety_memory::Severity::Critical => 1.0,
                                    crate::safety_memory::Severity::High => 0.95,
                                    crate::safety_memory::Severity::Medium => 0.8,
                                    crate::safety_memory::Severity::Low => 0.6,
                                },
                                times_applied: 0,
                            };
                            if let Err(e) = memory.record_lesson(&lesson).await {
                                debug!(error = %e, "Failed to record safety lesson");
                            }
                        }
                        SafetyOutcome::NearMiss { .. } => {
                            for id in &lesson_ids {
                                if let Err(e) = memory.reinforce(id).await {
                                    debug!(error = %e, lesson_id = %id, "Failed to reinforce lesson");
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            Err(e) => {
                if let Some(ref trail) = self.audit_trail {
                    let entry = AuditEntry {
                        timestamp: Utc::now(),
                        session_id: ctx.session_id.to_string(),
                        tool_name: name.to_string(),
                        input_summary,
                        output_summary: format!("ERROR: {}", e),
                        safety_reasoning: safety_reasoning.clone(),
                        outcome: SafetyOutcome::Incident {
                            what_happened: e.to_string(),
                            severity: crate::safety_memory::Severity::Low,
                        },
                        lessons_available: lesson_ids,
                    };
                    if let Err(e) = trail.log_tool_call(&entry).await {
                        debug!(error = %e, "Failed to log error audit entry");
                    }
                }
            }
        }

        result
    }

    async fn execute_tool_direct(
        &self,
        tool: &Arc<dyn Tool>,
        name: &str,
        input: serde_json::Value,
        ctx: ToolContext,
    ) -> Result<ToolResult> {
        let timeout = Duration::from_secs(tool.timeout_secs());
        match tokio::time::timeout(timeout, tool.execute(input, ctx)).await {
            Ok(result) => result,
            Err(_) => Err(RyvosError::ToolTimeout {
                tool: name.to_string(),
                timeout_secs: tool.timeout_secs(),
            }),
        }
    }

    pub async fn definitions(&self) -> Vec<ToolDefinition> {
        self.tools.read().await.definitions()
    }

    pub async fn l1_index(&self) -> Vec<ToolIndexEntry> {
        self.tools.read().await.l1_index()
    }

    pub async fn l2_schema(&self, name: &str) -> Option<serde_json::Value> {
        self.tools.read().await.l2_schema(name)
    }

    pub async fn l2_definition(&self, name: &str) -> Option<ToolDefinition> {
        self.tools.read().await.l2_definition(name)
    }

    pub fn tools_lock(&self) -> &Arc<tokio::sync::RwLock<ToolRegistry>> {
        &self.tools
    }

    pub fn policy(&self) -> &SecurityPolicy {
        &self.policy
    }

    pub fn safety_memory(&self) -> Option<&Arc<SafetyMemory>> {
        self.safety_memory.as_ref()
    }

    pub fn audit_trail(&self) -> Option<&Arc<AuditTrail>> {
        self.audit_trail.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ryvos_core::security::{SecurityPolicy, SecurityTier};
    use ryvos_core::types::SessionId;
    use ryvos_tools::ToolRegistry;

    fn test_ctx() -> ToolContext {
        ToolContext {
            session_id: SessionId::new(),
            working_dir: std::env::temp_dir(),
            store: None,
            agent_spawner: None,
            sandbox_config: None,
            config_path: None,
            viking_client: None,
        }
    }

    fn make_gate(policy: SecurityPolicy) -> SecurityGate {
        let tools = Arc::new(tokio::sync::RwLock::new(ToolRegistry::with_builtins()));
        let event_bus = Arc::new(EventBus::default());
        let broker = Arc::new(ApprovalBroker::new(event_bus.clone()));
        SecurityGate::new(policy, tools, broker, event_bus)
    }

    #[tokio::test]
    async fn all_tools_execute_freely() {
        let gate = make_gate(SecurityPolicy::default());
        let input = serde_json::json!({"command": "echo hello"});
        let result = gate.execute("bash", input, test_ctx()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn read_tool_executes() {
        let gate = make_gate(SecurityPolicy::default());
        let input = serde_json::json!({"file_path": "/tmp/test.txt"});
        let result = gate.execute("read", input, test_ctx()).await;
        assert!(
            result.is_ok() || matches!(result, Err(RyvosError::ToolExecution { .. })),
            "T0 tool should never be blocked"
        );
    }

    #[tokio::test]
    async fn no_blocking_on_any_tier() {
        let policy = SecurityPolicy {
            deny_above: Some(SecurityTier::T1),
            ..Default::default()
        };
        let gate = make_gate(policy);
        let input = serde_json::json!({"command": "echo hello"});
        let result = gate.execute("bash", input, test_ctx()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn tool_not_found_still_errors() {
        let gate = make_gate(SecurityPolicy::default());
        let result = gate
            .execute("nonexistent_tool", serde_json::Value::Null, test_ctx())
            .await;
        assert!(matches!(result, Err(RyvosError::ToolNotFound(_))));
    }

    #[tokio::test]
    async fn pause_before_timeout_proceeds() {
        let policy = SecurityPolicy {
            pause_before: vec!["bash".to_string()],
            approval_timeout_secs: 0,
            ..Default::default()
        };
        let gate = make_gate(policy);
        let input = serde_json::json!({"command": "echo hello"});
        let result = gate.execute("bash", input, test_ctx()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn tainted_session_requires_approval() {
        let event_bus = Arc::new(EventBus::default());
        let broker = Arc::new(ApprovalBroker::new(event_bus.clone()));
        let tools = Arc::new(tokio::sync::RwLock::new(ToolRegistry::with_builtins()));
        let policy = SecurityPolicy {
            approval_timeout_secs: 1,
            ..Default::default()
        };
        let gate = SecurityGate::new(policy, tools, broker.clone(), event_bus);

        let ctx = test_ctx();
        let session_id = ctx.session_id.to_string();
        gate.mark_session_tainted(&session_id).await;
        assert!(gate.is_session_tainted(&session_id).await);

        let broker_clone = broker.clone();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(20)).await;
            let pending = broker_clone.pending_requests().await;
            if let Some(first) = pending.first() {
                let _ = broker_clone
                    .respond(&first.id, ApprovalDecision::Approved)
                    .await;
            }
        });

        let input = serde_json::json!({"command": "echo approved"});
        let result = gate.execute("bash", input, ctx).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn tainted_session_denies_on_rejection() {
        let event_bus = Arc::new(EventBus::default());
        let broker = Arc::new(ApprovalBroker::new(event_bus.clone()));
        let tools = Arc::new(tokio::sync::RwLock::new(ToolRegistry::with_builtins()));
        let policy = SecurityPolicy {
            approval_timeout_secs: 1,
            ..Default::default()
        };
        let gate = SecurityGate::new(policy, tools, broker.clone(), event_bus);

        let ctx = test_ctx();
        let session_id = ctx.session_id.to_string();
        gate.mark_session_tainted(&session_id).await;

        let broker_clone = broker.clone();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(20)).await;
            let pending = broker_clone.pending_requests().await;
            if let Some(first) = pending.first() {
                let _ = broker_clone
                    .respond(
                        &first.id,
                        ApprovalDecision::Denied {
                            reason: "Denied by policy test".to_string(),
                        },
                    )
                    .await;
            }
        });

        let input = serde_json::json!({"command": "echo denied"});
        let result = gate.execute("bash", input, ctx).await;
        assert!(result.is_err());
        assert!(matches!(result, Err(RyvosError::ApprovalDenied { .. })));
    }
}
