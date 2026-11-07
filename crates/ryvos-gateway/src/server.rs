use std::sync::Arc;
use std::time::Instant;

use axum::routing::{get, post};
use axum::Router;
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;
use tower_http::cors::CorsLayer;
use tracing::info;

use ryvos_agent::{
    AgentRuntime, ApprovalBroker, AuditTrail, FailureJournal, SafetyMemory, SessionManager,
};
use ryvos_channels::WhatsAppWebhookHandle;
use ryvos_core::config::{BudgetConfig, GatewayConfig, IntegrationsConfig};
use ryvos_core::event::EventBus;
use ryvos_core::traits::SessionStore;
use ryvos_memory::{CostStore, IntegrationStore, SessionMetaStore, VikingClient};

use crate::routes;
use crate::state::AppState;
use crate::static_files;

pub struct GatewayServer {
    config: GatewayConfig,
    runtime: Arc<AgentRuntime>,
    event_bus: Arc<EventBus>,
    store: Arc<dyn SessionStore>,
    session_mgr: Arc<SessionManager>,
    broker: Arc<ApprovalBroker>,
    whatsapp_handle: Option<WhatsAppWebhookHandle>,
    cost_store: Option<Arc<CostStore>>,
    budget_config: Option<BudgetConfig>,
    start_time: Instant,
    audit_trail: Option<Arc<AuditTrail>>,
    viking_client: Option<Arc<VikingClient>>,
    config_path: Option<std::path::PathBuf>,
    session_meta: Option<Arc<SessionMetaStore>>,
    integration_store: Option<Arc<IntegrationStore>>,
    integrations_config: IntegrationsConfig,
    safety_memory: Option<Arc<SafetyMemory>>,
    failure_journal: Option<Arc<FailureJournal>>,
    mcp_manager: Option<Arc<ryvos_mcp::McpClientManager>>,
}

impl GatewayServer {
    pub fn new(
        config: GatewayConfig,
        runtime: Arc<AgentRuntime>,
        event_bus: Arc<EventBus>,
        store: Arc<dyn SessionStore>,
        session_mgr: Arc<SessionManager>,
        broker: Arc<ApprovalBroker>,
    ) -> Self {
        Self {
            config,
            runtime,
            event_bus,
            store,
            session_mgr,
            broker,
            whatsapp_handle: None,
            cost_store: None,
            budget_config: None,
            start_time: Instant::now(),
            audit_trail: None,
            viking_client: None,
            config_path: None,
            session_meta: None,
            integration_store: None,
            integrations_config: IntegrationsConfig::default(),
            safety_memory: None,
            failure_journal: None,
            mcp_manager: None,
        }
    }

    pub fn set_whatsapp_handle(&mut self, handle: WhatsAppWebhookHandle) {
        self.whatsapp_handle = Some(handle);
    }

    pub fn set_cost_store(
        &mut self,
        cost_store: Arc<CostStore>,
        budget_config: Option<BudgetConfig>,
    ) {
        self.cost_store = Some(cost_store);
        self.budget_config = budget_config;
    }

    pub fn set_audit_trail(&mut self, trail: Arc<AuditTrail>) {
        self.audit_trail = Some(trail);
    }

    pub fn set_viking_client(&mut self, client: Arc<VikingClient>) {
        self.viking_client = Some(client);
    }

    pub fn set_config_path(&mut self, path: std::path::PathBuf) {
        self.config_path = Some(path);
    }

    pub fn set_session_meta(&mut self, meta: Arc<SessionMetaStore>) {
        self.session_meta = Some(meta);
    }

    pub fn set_integration_store(&mut self, store: Arc<IntegrationStore>) {
        self.integration_store = Some(store);
    }

    pub fn set_integrations_config(&mut self, config: IntegrationsConfig) {
        self.integrations_config = config;
    }

    pub fn set_safety_memory(&mut self, memory: Arc<SafetyMemory>) {
        self.safety_memory = Some(memory);
    }

    pub fn set_failure_journal(&mut self, journal: Arc<FailureJournal>) {
        self.failure_journal = Some(journal);
    }

    pub fn set_mcp_manager(&mut self, manager: Arc<ryvos_mcp::McpClientManager>) {
        self.mcp_manager = Some(manager);
    }

    pub async fn run(&self, shutdown: CancellationToken) -> anyhow::Result<()> {
        let state = Arc::new(AppState {
            config: self.config.clone(),
            runtime: self.runtime.clone(),
            event_bus: self.event_bus.clone(),
            store: self.store.clone(),
            session_mgr: self.session_mgr.clone(),
            broker: self.broker.clone(),
            whatsapp_handle: self.whatsapp_handle.clone(),
            cost_store: self.cost_store.clone(),
            budget_config: self.budget_config.clone(),
            start_time: self.start_time,
            audit_trail: self.audit_trail.clone(),
            viking_client: self.viking_client.clone(),
            config_path: self.config_path.clone(),
            session_meta: self.session_meta.clone(),
            integration_store: self.integration_store.clone(),
            integrations_config: self.integrations_config.clone(),
            safety_memory: self.safety_memory.clone(),
            failure_journal: self.failure_journal.clone(),
            mcp_manager: self.mcp_manager.clone(),
        });

        let app = Router::new()
            .route("/ws", get(routes::ws_handler))
            .route("/api/health", get(routes::health))
            .route("/api/sessions", get(routes::list_sessions))
            .route("/api/sessions/{id}/history", get(routes::session_history))
            .route("/api/sessions/{id}/messages", post(routes::send_message))
            .route("/api/metrics", get(routes::metrics))
            .route("/api/runs", get(routes::runs))
            .route("/api/costs", get(routes::costs))
            .route("/api/audit", get(routes::audit_entries))
            .route("/api/audit/stats", get(routes::audit_stats))
            .route("/api/viking/list", get(routes::viking_list))
            .route("/api/viking/read", get(routes::viking_read))
            .route("/api/viking/search", get(routes::viking_search))
            .route(
                "/api/memory/search",
                get(routes::memory_search_get).post(routes::memory_search_post),
            )
            .route(
                "/api/memory/search_memory",
                get(routes::memory_search_get).post(routes::memory_search_post),
            )
            .route("/api/memory/list", get(routes::memory_list))
            .route("/api/memory/list_memories", get(routes::memory_list))
            .route("/api/memory/add", post(routes::memory_add))
            .route("/api/memory/add_memory", post(routes::memory_add))
            .route(
                "/api/config",
                get(routes::get_config).put(routes::put_config),
            )
            .route("/api/channels", get(routes::channels_status))
            .route("/api/approvals", get(routes::list_approvals))
            .route("/api/approvals/{id}/approve", post(routes::approve_request))
            .route("/api/approvals/{id}/deny", post(routes::deny_request))
            .route("/api/cron", get(routes::list_cron).post(routes::add_cron))
            .route(
                "/api/cron/{name}",
                axum::routing::delete(routes::delete_cron),
            )
            .route(
                "/api/budget",
                get(routes::get_budget).put(routes::put_budget),
            )
            .route("/api/model", get(routes::get_model).put(routes::put_model))
            .route("/api/models/available", get(routes::list_models))
            .route("/api/integrations", get(routes::list_integrations))
            .route(
                "/api/integrations/callback",
                get(routes::integration_callback),
            )
            .route(
                "/api/integrations/{app}/connect",
                post(routes::connect_integration),
            )
            .route(
                "/api/integrations/{app}",
                axum::routing::delete(routes::disconnect_integration),
            )
            .route("/api/skills", get(routes::list_skills))
            .route("/api/heartbeat/history", get(routes::heartbeat_history))
            .route("/api/safety/lessons", get(routes::safety_lessons))
            .route("/api/decisions", get(routes::list_decisions))
            .route("/api/failures", get(routes::list_failures))
            .route("/api/goals/run", post(routes::run_goal))
            .route("/api/goals/history", get(routes::goal_history))
            .route("/api/hooks/wake", post(routes::webhook_wake))
            .route(
                "/api/whatsapp/webhook",
                get(routes::whatsapp_verify).post(routes::whatsapp_incoming),
            )
            .route("/", get(static_files::index))
            .route("/index.html", get(static_files::index))
            .route("/assets/{*path}", get(static_files::static_file))
            .fallback(static_files::index)
            .layer(CorsLayer::permissive())
            .with_state(state);

        let listener = TcpListener::bind(&self.config.bind).await?;
        info!(bind = %self.config.bind, "Gateway listening");

        axum::serve(listener, app)
            .with_graceful_shutdown(async move { shutdown.cancelled().await })
            .await?;

        info!("Gateway shut down");
        Ok(())
    }
}
