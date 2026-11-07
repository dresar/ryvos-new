use std::net::IpAddr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use futures::future::BoxFuture;
use futures::stream::{self, BoxStream, StreamExt};
use reqwest::header::{HeaderMap, HeaderValue};
use ryvos_agent::gate::SecurityGate;
use ryvos_agent::ApprovalBroker;
use ryvos_core::config::{ModelConfig, RetryConfig};
use ryvos_core::error::{Result, RyvosError};
use ryvos_core::event::EventBus;
use ryvos_core::security::{SecurityPolicy, SecurityTier};
use ryvos_core::traits::{LlmClient, Tool};
use ryvos_core::types::*;
use ryvos_llm::retry::{CircuitBreaker, CircuitState, RetryingClient};
use ryvos_llm::streaming::{SseParser, SseStream};
use ryvos_tools::egress::is_ip_blocked_for_egress;
use ryvos_tools::registry::ToolRegistry;

struct MockTool {
    name: String,
    tier: SecurityTier,
    calls: Arc<AtomicUsize>,
}

impl Tool for MockTool {
    fn name(&self) -> &str {
        &self.name
    }

    fn tier(&self) -> SecurityTier {
        self.tier
    }

    fn description(&self) -> &str {
        "Mock tool for testing"
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "payload": { "type": "string" }
            }
        })
    }

    fn execute(
        &self,
        _input: serde_json::Value,
        _ctx: ToolContext,
    ) -> BoxFuture<'_, Result<ToolResult>> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Box::pin(async move {
            Ok(ToolResult {
                content: "mock executed".to_string(),
                is_error: false,
                trust: TrustLevel::Trusted,
            })
        })
    }
}

struct ScriptedLlmClient {
    responses: Mutex<Vec<Result<Vec<StreamDelta>>>>,
    call_count: AtomicUsize,
}

impl ScriptedLlmClient {
    fn new(responses: Vec<Result<Vec<StreamDelta>>>) -> Self {
        Self {
            responses: Mutex::new(responses),
            call_count: AtomicUsize::new(0),
        }
    }
}

impl LlmClient for ScriptedLlmClient {
    fn chat_stream(
        &self,
        _config: &ModelConfig,
        _messages: Vec<ChatMessage>,
        _tools: &[ToolDefinition],
    ) -> BoxFuture<'_, Result<BoxStream<'_, Result<StreamDelta>>>> {
        self.call_count.fetch_add(1, Ordering::SeqCst);
        let mut guard = self.responses.lock().unwrap();
        let res = if guard.is_empty() {
            Err(RyvosError::LlmRequest("No responses configured".into()))
        } else {
            guard.remove(0)
        };

        Box::pin(async move {
            match res {
                Ok(deltas) => Ok(stream::iter(deltas.into_iter().map(Ok)).boxed()),
                Err(e) => Err(e),
            }
        })
    }
}

fn test_model_config(provider: &str, model: &str) -> ModelConfig {
    ModelConfig {
        provider: provider.to_string(),
        model_id: model.to_string(),
        api_key: Some("key".to_string()),
        base_url: None,
        max_tokens: 1024,
        temperature: 0.7,
        thinking: ThinkingLevel::Off,
        retry: None,
        azure_resource: None,
        azure_deployment: None,
        azure_api_version: None,
        aws_region: None,
        extra_headers: std::collections::HashMap::new(),
        claude_command: None,
        cli_allowed_tools: vec![],
        cli_permission_mode: None,
        copilot_command: None,
        cli_session_id: None,
    }
}

#[test]
fn test_adversarial_sse_single_byte_feeding_astral_plane() {
    let mut parser = SseParser::new();
    let text = "data: 🦀\n\n";
    let bytes = text.as_bytes();

    let mut events = Vec::new();
    for &b in bytes {
        events.extend(parser.feed_bytes(&[b]));
    }

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].data, "🦀");
}

#[test]
fn test_adversarial_sse_zwj_sequence_byte_by_byte() {
    let mut parser = SseParser::new();
    let text = "event: family\ndata: 👨‍👩‍👧‍👦\n\n";
    let bytes = text.as_bytes();

    let mut events = Vec::new();
    for &b in bytes {
        events.extend(parser.feed_bytes(&[b]));
    }

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type.as_deref(), Some("family"));
    assert_eq!(events[0].data, "👨‍👩‍👧‍👦");
}

#[test]
fn test_adversarial_sse_multiline_data_concatenation() {
    let mut parser = SseParser::new();
    let payload = "event: delta\ndata: line one\ndata: line two\ndata: line three\n\n";
    let events = parser.feed(payload);

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].data, "line one\nline two\nline three");
}

#[test]
fn test_adversarial_sse_mixed_cr_lf_crlf_torture() {
    let mut parser = SseParser::new();
    let stream_parts = [
        "event: first\r\ndata: 1\r\n\r\n",
        "event: second\ndata: 2\n\n",
        "event: third\rdata: 3\r\r",
        "event: fourth\r\ndata: 4\n\r\n",
        "event: fifth\ndata: 5\r\n\n",
    ];

    for part in stream_parts {
        let events = parser.feed(part);
        assert_eq!(events.len(), 1, "Failed on part: {:?}", part);
    }
}

#[test]
fn test_adversarial_sse_boundary_split_on_delimiter_bytes() {
    let mut parser = SseParser::new();

    let ev1 = parser.feed_bytes(b"data: message\r\n\r");
    assert!(ev1.is_empty());

    let ev2 = parser.feed_bytes(b"\n");
    assert_eq!(ev2.len(), 1);
    assert_eq!(ev2[0].data, "message");
}

#[tokio::test]
async fn test_adversarial_sse_stream_idle_timeout() {
    let pending_stream = stream::pending::<std::result::Result<tokio_util::bytes::Bytes, reqwest::Error>>();
    let mut sse_stream = SseStream::new(pending_stream).with_idle_timeout(Duration::from_millis(50));

    let res = sse_stream.next_chunk_timeout().await;
    assert!(res.is_err());
    match res.unwrap_err() {
        RyvosError::LlmStream(msg) => {
            assert!(msg.contains("idle timeout"));
        }
        other => panic!("Expected LlmStream timeout error, got {:?}", other),
    }
}

#[test]
fn test_adversarial_circuit_breaker_429_immediate_trip() {
    let cb = CircuitBreaker::new(10, Duration::from_secs(60));
    assert_eq!(cb.state(), CircuitState::Closed);
    assert!(cb.allow_request());

    cb.record_failure(true, None);
    assert_eq!(cb.state(), CircuitState::Open);
    assert!(!cb.allow_request());
}

#[test]
fn test_adversarial_circuit_breaker_retry_after_header_formats() {
    let mut h1 = HeaderMap::new();
    h1.insert("retry-after", HeaderValue::from_static("45"));
    assert_eq!(CircuitBreaker::parse_retry_after_headers(&h1), Some(Duration::from_secs(45)));

    let mut h2 = HeaderMap::new();
    h2.insert("retry-after", HeaderValue::from_static("1.5"));
    assert_eq!(CircuitBreaker::parse_retry_after_headers(&h2), Some(Duration::from_millis(1500)));

    let mut h3 = HeaderMap::new();
    h3.insert("x-ratelimit-reset-requests", HeaderValue::from_static("100ms"));
    assert_eq!(CircuitBreaker::parse_retry_after_headers(&h3), Some(Duration::from_millis(100)));

    let str_dur = CircuitBreaker::parse_retry_after_str("Rate limit exceeded. retry-after: 12s");
    assert_eq!(str_dur, Some(Duration::from_secs(12)));
}

#[test]
fn test_adversarial_circuit_breaker_half_open_probe_behavior() {
    let cb = CircuitBreaker::with_window(3, Duration::from_millis(30), Duration::from_millis(60));
    cb.record_failure(true, None);
    assert_eq!(cb.state(), CircuitState::Open);

    std::thread::sleep(Duration::from_millis(40));
    assert_eq!(cb.state(), CircuitState::HalfOpen);
    assert!(cb.allow_request());

    cb.record_success();
    assert_eq!(cb.state(), CircuitState::Closed);
    assert!(cb.allow_request());
}

#[test]
fn test_adversarial_circuit_breaker_probe_failure_window_edge_case() {
    let cb = CircuitBreaker::with_window(3, Duration::from_millis(20), Duration::from_millis(40));
    cb.record_failure(true, None);
    assert_eq!(cb.state(), CircuitState::Open);

    std::thread::sleep(Duration::from_millis(50));
    assert_eq!(cb.state(), CircuitState::HalfOpen);

    cb.record_failure(false, None);
    let state_after = cb.state();
    println!("State after HalfOpen probe failure with expired window: {:?}", state_after);
}

#[tokio::test]
async fn test_adversarial_retrying_client_failover_chain_resilience() {
    let primary = Box::new(ScriptedLlmClient::new(vec![Err(RyvosError::LlmRequest(
        "429 rate limit exceeded".into(),
    ))]));

    let sec_cfg = test_model_config("anthropic", "claude-sonnet");
    let secondary = Box::new(ScriptedLlmClient::new(vec![Err(RyvosError::LlmRequest(
        "500 Internal Server Error".into(),
    ))]));

    let tert_cfg = test_model_config("gemini", "gemini-1.5-pro");
    let tertiary = Box::new(ScriptedLlmClient::new(vec![Ok(vec![
        StreamDelta::TextDelta("tertiary recovered".to_string()),
        StreamDelta::Stop(StopReason::EndTurn),
    ])]));

    let retry_cfg = RetryConfig {
        max_retries: 0,
        initial_backoff_ms: 1,
        max_backoff_ms: 2,
    };

    let client = RetryingClient::new(
        primary,
        vec![(sec_cfg, secondary), (tert_cfg, tertiary)],
        retry_cfg,
    );

    let cfg = test_model_config("openai", "gpt-4o");
    let mut stream = client
        .chat_stream(&cfg, vec![ChatMessage::user("hi")], &[])
        .await
        .unwrap();

    let mut output = String::new();
    while let Some(Ok(delta)) = stream.next().await {
        if let StreamDelta::TextDelta(t) = delta {
            output.push_str(&t);
        }
    }
    assert_eq!(output, "tertiary recovered");
}

#[test]
fn test_adversarial_ssrf_ip_ranges_and_edges() {
    let blocked = [
        "127.0.0.1",
        "127.255.255.254",
        "169.254.169.254",
        "169.254.0.1",
        "10.0.0.1",
        "10.255.255.255",
        "172.16.0.1",
        "172.31.255.255",
        "192.168.0.1",
        "192.168.255.255",
        "0.0.0.0",
        "224.0.0.1",
        "255.255.255.255",
        "::1",
        "::ffff:127.0.0.1",
        "::ffff:169.254.169.254",
        "::ffff:10.0.0.1",
        "fd00::1",
        "fe80::1",
    ];

    for ip_str in blocked {
        let ip: IpAddr = ip_str.parse().unwrap();
        assert!(
            is_ip_blocked_for_egress(&ip),
            "Expected blocked: {}",
            ip_str
        );
    }

    let allowed = [
        "1.1.1.1",
        "8.8.8.8",
        "172.15.255.255",
        "172.32.0.1",
        "192.167.255.255",
        "192.169.0.1",
        "2606:4700:4700::1111",
    ];

    for ip_str in allowed {
        let ip: IpAddr = ip_str.parse().unwrap();
        assert!(
            !is_ip_blocked_for_egress(&ip),
            "Expected allowed: {}",
            ip_str
        );
    }
}

#[test]
fn test_adversarial_ssrf_ipv4_compatible_ipv6_check() {
    let ip_compatible: IpAddr = "::127.0.0.1".parse().unwrap();
    let is_blocked = is_ip_blocked_for_egress(&ip_compatible);
    println!("IPv4-compatible ::127.0.0.1 is_blocked = {}", is_blocked);
}

#[tokio::test]
async fn test_adversarial_taint_tracking_and_security_gate_enforcement() {
    let event_bus = Arc::new(EventBus::new(32));
    let broker = Arc::new(ApprovalBroker::new(event_bus.clone()));
    let policy = SecurityPolicy::default();

    let calls = Arc::new(AtomicUsize::new(0));
    let mut registry = ToolRegistry::new();
    registry.register(MockTool {
        name: "safe_reader".to_string(),
        tier: SecurityTier::T1,
        calls: calls.clone(),
    });
    registry.register(MockTool {
        name: "dangerous_writer".to_string(),
        tier: SecurityTier::T2,
        calls: calls.clone(),
    });

    let tools = Arc::new(tokio::sync::RwLock::new(registry));
    let gate = SecurityGate::new(policy, tools, broker.clone(), event_bus);

    let session_id = "test-session";
    let ctx = ToolContext {
        session_id: SessionId::from_string(session_id),
        working_dir: std::path::PathBuf::from("/tmp"),
        store: None,
        agent_spawner: None,
        sandbox_config: None,
        config_path: None,
        viking_client: None,
    };

    assert!(!gate.is_session_tainted(session_id).await);

    let res_read = gate.execute("safe_reader", serde_json::json!({}), ctx.clone()).await;
    assert!(res_read.is_ok());

    gate.mark_session_tainted(session_id).await;
    assert!(gate.is_session_tainted(session_id).await);

    let res_read_tainted = gate.execute("safe_reader", serde_json::json!({}), ctx.clone()).await;
    assert!(res_read_tainted.is_ok());

    let broker_clone = broker.clone();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(20)).await;
        let pending = broker_clone.pending_requests().await;
        if let Some(req) = pending.first() {
            broker_clone
                .respond(
                    &req.id,
                    ryvos_core::security::ApprovalDecision::Denied {
                        reason: "Blocked untrusted injection attempt".to_string(),
                    },
                )
                .await;
        }
    });

    let res_write_tainted = gate
        .execute("dangerous_writer", serde_json::json!({}), ctx.clone())
        .await;

    assert!(res_write_tainted.is_err());
    match res_write_tainted.unwrap_err() {
        RyvosError::ApprovalDenied { reason, .. } => {
            assert!(reason.contains("Blocked untrusted injection attempt"));
        }
        other => panic!("Expected ApprovalDenied error, got {:?}", other),
    }
}
