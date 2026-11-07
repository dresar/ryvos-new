use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::ws::{WebSocket, WebSocketUpgrade};
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;
use serde_json::Value;
use tracing::{debug, info};

use ryvos_core::types::SessionId;

use crate::auth;
use crate::connection;
use crate::middleware::Authenticated;
use crate::state::AppState;

pub async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

pub async fn list_sessions(
    Authenticated(auth_result): Authenticated,
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if !auth::has_viewer_access(&auth_result.role) {
        return Err(StatusCode::FORBIDDEN);
    }
    let in_memory_keys = state.session_mgr.list();
    let mut session_map: std::collections::HashMap<String, serde_json::Value> =
        std::collections::HashMap::new();

    if let Some(ref meta_store) = state.session_meta {
        if let Ok(meta_list) = meta_store.list() {
            for m in meta_list {
                session_map.insert(
                    m.session_key.clone(),
                    serde_json::json!({
                        "id": m.session_key,
                        "session_id": m.session_id,
                        "channel": m.channel,
                        "last_active": m.last_active,
                        "total_runs": m.total_runs,
                        "total_tokens": m.total_tokens,
                    }),
                );
            }
        }
    }

    for key in in_memory_keys {
        session_map.entry(key.clone()).or_insert_with(|| {
            serde_json::json!({ "id": key })
        });
    }

    let sessions: Vec<serde_json::Value> = session_map.into_values().collect();
    Ok(Json(serde_json::json!({ "sessions": sessions })))
}

#[derive(Deserialize)]
pub struct HistoryQuery {
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_limit() -> usize {
    50
}

pub async fn session_history(
    Authenticated(auth_result): Authenticated,
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Query(q): Query<HistoryQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if !auth::has_viewer_access(&auth_result.role) {
        return Err(StatusCode::FORBIDDEN);
    }

    let resolved_id = if let Some(ref meta_store) = state.session_meta {
        meta_store
            .get(&id)
            .ok()
            .flatten()
            .map(|m| m.session_id)
            .unwrap_or_else(|| id.clone())
    } else {
        id.clone()
    };
    let session_id = SessionId::from_string(&resolved_id);
    match state.store.load_history(&session_id, q.limit).await {
        Ok(messages) => {
            let msgs: Vec<serde_json::Value> = messages
                .iter()
                .map(|m| {
                    serde_json::json!({
                        "role": m.role,
                        "text": m.text(),
                        "timestamp": m.timestamp,
                    })
                })
                .collect();
            Ok(Json(serde_json::json!({ "messages": msgs })))
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[derive(Deserialize)]
pub struct SendMessageBody {
    pub message: String,
}

pub async fn send_message(
    Authenticated(auth_result): Authenticated,
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<SendMessageBody>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if !auth::has_operator_access(&auth_result.role) {
        return Err(StatusCode::FORBIDDEN);
    }

    if body.message.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let session_id = SessionId::from_string(&id);
    let session_key = id.clone();
    if let Some(ref meta_store) = state.session_meta {
        let _ = meta_store.get_or_create(&session_key, &session_id.to_string(), "webui");
    }

    let run_res = state.runtime.run(&session_id, &body.message).await;
    state.session_mgr.record_run_stats(&session_key, 0, "standard");
    if let Some(ref meta_store) = state.session_meta {
        let _ = meta_store.record_run_stats(&session_key, 0, "standard");
    }

    match run_res {
        Ok(response) => Ok(Json(serde_json::json!({
            "session_id": session_id.to_string(),
            "response": response,
        }))),
        Err(e) => Ok(Json(serde_json::json!({
            "session_id": session_id.to_string(),
            "error": e.to_string(),
        }))),
    }
}


#[derive(Deserialize)]
pub struct RunsQuery {
    #[serde(default = "default_runs_limit")]
    pub limit: u64,
    #[serde(default)]
    pub offset: u64,
}

fn default_runs_limit() -> u64 {
    50
}

#[derive(Deserialize)]
pub struct CostsQuery {
    #[serde(default)]
    pub from: Option<String>,
    #[serde(default)]
    pub to: Option<String>,
    #[serde(default = "default_group_by")]
    pub group_by: String,
}

fn default_group_by() -> String {
    "model".to_string()
}

pub async fn metrics(
    Authenticated(auth_result): Authenticated,
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if !auth::has_viewer_access(&auth_result.role) {
        return Err(StatusCode::FORBIDDEN);
    }

    let sessions = state.session_mgr.list();
    let uptime_secs = state.start_time.elapsed().as_secs();

    let (total_runs, total_tokens, total_cost_cents) =
        if let Some(ref cost_store) = state.cost_store {
            let total_count = cost_store.run_history(1, 0).map(|(_, t)| t).unwrap_or(0);
            let tokens = cost_store.total_tokens_all_time().unwrap_or(0);
            let monthly = cost_store.monthly_spend_cents().unwrap_or(0);
            (total_count, tokens, monthly)
        } else {
            (0, 0, 0)
        };

    let monthly_budget_cents = state
        .budget_config
        .as_ref()
        .map(|b| b.monthly_budget_cents)
        .unwrap_or(0);

    let budget_utilization_pct = if monthly_budget_cents > 0 {
        (total_cost_cents as f64 / monthly_budget_cents as f64 * 100.0) as u64
    } else {
        0
    };

    Ok(Json(serde_json::json!({
        "total_runs": total_runs,
        "active_sessions": sessions.len(),
        "total_tokens": total_tokens,
        "total_cost_cents": total_cost_cents,
        "monthly_budget_cents": monthly_budget_cents,
        "budget_utilization_pct": budget_utilization_pct,
        "uptime_secs": uptime_secs,
    })))
}

pub async fn runs(
    Authenticated(auth_result): Authenticated,
    State(state): State<Arc<AppState>>,
    Query(q): Query<RunsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if !auth::has_viewer_access(&auth_result.role) {
        return Err(StatusCode::FORBIDDEN);
    }

    let Some(cost_store) = state.cost_store.as_ref() else {
        return Ok(Json(serde_json::json!({
            "runs": [],
            "total": 0,
            "offset": q.offset,
            "limit": q.limit,
            "note": "Budget tracking not configured. Add [budget] to config.toml to enable."
        })));
    };
    let (runs, total) = cost_store
        .run_history(q.limit, q.offset)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({
        "runs": runs,
        "total": total,
        "offset": q.offset,
        "limit": q.limit,
    })))
}

pub async fn costs(
    Authenticated(auth_result): Authenticated,
    State(state): State<Arc<AppState>>,
    Query(q): Query<CostsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if !auth::has_viewer_access(&auth_result.role) {
        return Err(StatusCode::FORBIDDEN);
    }

    let Some(cost_store) = state.cost_store.as_ref() else {
        return Ok(Json(serde_json::json!({
            "summary": { "total_cost_cents": 0, "total_input_tokens": 0, "total_output_tokens": 0, "total_events": 0 },
            "breakdown": [],
            "from": "",
            "to": "",
            "group_by": q.group_by,
            "note": "Budget tracking not configured. Add [budget] to config.toml to enable."
        })));
    };

    let from = q
        .from
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
        .map(|d| d.with_timezone(&chrono::Utc))
        .unwrap_or_else(|| chrono::Utc::now() - chrono::Duration::days(30));

    let to =
        q.to.and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
            .map(|d| d.with_timezone(&chrono::Utc))
            .unwrap_or_else(chrono::Utc::now);

    let summary = cost_store
        .cost_summary(&from, &to)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let breakdown = cost_store
        .cost_by_group(&from, &to, &q.group_by)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let breakdown_json: Vec<serde_json::Value> = breakdown
        .into_iter()
        .map(|(key, cost, input, output)| {
            serde_json::json!({
                "key": key,
                "cost_cents": cost,
                "input_tokens": input,
                "output_tokens": output,
            })
        })
        .collect();

    Ok(Json(serde_json::json!({
        "summary": summary,
        "breakdown": breakdown_json,
        "from": from.to_rfc3339(),
        "to": to.to_rfc3339(),
        "group_by": q.group_by,
    })))
}


#[derive(Deserialize)]
pub struct WebhookPayload {
    pub prompt: String,
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub channel: Option<String>,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
    #[serde(default)]
    pub callback_url: Option<String>,
}

pub async fn webhook_wake(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(body): Json<WebhookPayload>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let webhook_config = state
        .config
        .webhooks
        .as_ref()
        .filter(|w| w.enabled)
        .ok_or(StatusCode::NOT_FOUND)?;

    if let Some(ref expected_token) = webhook_config.token {
        let auth_header = headers
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or(StatusCode::UNAUTHORIZED)?;

        if auth_header != expected_token {
            return Err(StatusCode::UNAUTHORIZED);
        }
    }

    if body.prompt.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let session_id = body
        .session_id
        .map(|s| SessionId::from_string(&s))
        .unwrap_or_else(SessionId::new);

    info!(session_id = %session_id, "Webhook wake triggered");

    let callback_url = body.callback_url.clone();
    let channel = body.channel.clone();
    let metadata = body.metadata.clone();

    match state.runtime.run(&session_id, &body.prompt).await {
        Ok(response) => {
            if let Some(url) = callback_url.clone() {
                let client = reqwest::Client::new();
                let md = metadata.clone();
                let cb_body = serde_json::json!({
                    "session_id": session_id.to_string(),
                    "response": response,
                    "metadata": md,
                });
                tokio::spawn(async move {
                    if let Err(e) = client.post(&url).json(&cb_body).send().await {
                        tracing::warn!(url = %url, error = %e, "Webhook callback failed");
                    }
                });
            }

            Ok(Json(serde_json::json!({
                "session_id": session_id.to_string(),
                "response": response,
                "channel": channel,
                "metadata": metadata,
            })))
        }
        Err(e) => {
            if let Some(url) = callback_url {
                let client = reqwest::Client::new();
                let cb_body = serde_json::json!({
                    "session_id": session_id.to_string(),
                    "error": e.to_string(),
                    "metadata": metadata,
                });
                tokio::spawn(async move {
                    if let Err(e) = client.post(&url).json(&cb_body).send().await {
                        tracing::warn!(url = %url, error = %e, "Webhook callback failed");
                    }
                });
            }

            Ok(Json(serde_json::json!({
                "session_id": session_id.to_string(),
                "error": e.to_string(),
            })))
        }
    }
}


#[derive(Deserialize)]
pub struct WhatsAppVerifyQuery {
    #[serde(rename = "hub.mode", default)]
    pub mode: String,
    #[serde(rename = "hub.verify_token", default)]
    pub verify_token: String,
    #[serde(rename = "hub.challenge", default)]
    pub challenge: String,
}

pub async fn whatsapp_verify(
    State(state): State<Arc<AppState>>,
    Query(q): Query<WhatsAppVerifyQuery>,
) -> Result<String, StatusCode> {
    let handle = state
        .whatsapp_handle
        .as_ref()
        .ok_or(StatusCode::NOT_FOUND)?;

    handle
        .verify_webhook(&q.mode, &q.verify_token, &q.challenge)
        .ok_or(StatusCode::FORBIDDEN)
}

pub async fn whatsapp_incoming(
    State(state): State<Arc<AppState>>,
    Json(body): Json<serde_json::Value>,
) -> StatusCode {
    if let Some(ref handle) = state.whatsapp_handle {
        handle.process_webhook(body).await;
    }
    StatusCode::OK
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Authenticated(_auth_result): Authenticated,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_ws(socket, state))
}

async fn handle_ws(socket: WebSocket, state: Arc<AppState>) {
    info!("WebSocket client connected");
    connection::handle_connection(
        socket,
        state.runtime.clone(),
        state.event_bus.clone(),
        state.store.clone(),
        state.session_mgr.clone(),
        state.broker.clone(),
        state.session_meta.clone(),
    )
    .await;
    debug!("WebSocket client disconnected");
}


#[derive(Deserialize)]
pub struct AuditQuery {
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub tool: Option<String>,
    #[serde(default = "default_audit_limit")]
    pub limit: usize,
}

fn default_audit_limit() -> usize {
    50
}

pub async fn audit_entries(
    Authenticated(auth_result): Authenticated,
    State(state): State<Arc<AppState>>,
    Query(q): Query<AuditQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if !auth::has_viewer_access(&auth_result.role) {
        return Err(StatusCode::FORBIDDEN);
    }
    let trail = state.audit_trail.as_ref().ok_or(StatusCode::NOT_FOUND)?;
    let entries = if let Some(ref tool) = q.tool {
        trail
            .entries_by_tool(tool, q.limit)
            .await
            .unwrap_or_default()
    } else if let Some(ref sid) = q.session_id {
        trail.recent_entries(sid, q.limit).await.unwrap_or_default()
    } else {
        trail.recent_entries("", q.limit).await.unwrap_or_default()
    };
    Ok(Json(serde_json::json!({ "entries": entries })))
}

pub async fn audit_stats(
    Authenticated(auth_result): Authenticated,
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if !auth::has_viewer_access(&auth_result.role) {
        return Err(StatusCode::FORBIDDEN);
    }
    let trail = state.audit_trail.as_ref().ok_or(StatusCode::NOT_FOUND)?;
    let total = trail.total_entries().await.unwrap_or(0);
    let tool_breakdown = trail.tool_breakdown().await.unwrap_or_default();
    let heartbeat_sessions = trail.heartbeat_session_count().await.unwrap_or(0);

    let tools: Vec<serde_json::Value> = tool_breakdown
        .into_iter()
        .map(|(tool, count)| serde_json::json!({"tool": tool, "count": count}))
        .collect();

    let viking_entries = if let Some(ref vc) = state.viking_client {
        vc.list_directory("viking://")
            .await
            .map(|v| v.len() as u64)
            .unwrap_or(0)
    } else {
        0
    };

    Ok(Json(serde_json::json!({
        "total_entries": total,
        "tool_breakdown": tools,
        "heartbeat_sessions": heartbeat_sessions,
        "viking_entries": viking_entries,
    })))
}


#[derive(Deserialize)]
pub struct VikingListQuery {
    #[serde(default = "default_viking_path")]
    pub path: String,
}

fn default_viking_path() -> String {
    "viking://".to_string()
}

#[derive(Deserialize)]
pub struct VikingReadQuery {
    pub path: String,
    #[serde(default = "default_viking_level")]
    pub level: String,
}

fn default_viking_level() -> String {
    "L1".to_string()
}

#[derive(Deserialize)]
pub struct VikingSearchQuery {
    pub query: String,
    #[serde(default)]
    pub directory: Option<String>,
    #[serde(default = "default_viking_search_limit")]
    pub limit: usize,
}

fn default_viking_search_limit() -> usize {
    10
}

pub async fn viking_list(
    Authenticated(auth_result): Authenticated,
    State(state): State<Arc<AppState>>,
    Query(q): Query<VikingListQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if !auth::has_viewer_access(&auth_result.role) {
        return Err(StatusCode::FORBIDDEN);
    }
    let viking = state.viking_client.as_ref().ok_or(StatusCode::NOT_FOUND)?;
    match viking.list_directory(&q.path).await {
        Ok(entries) => Ok(Json(serde_json::json!(entries))),
        Err(e) => Ok(Json(serde_json::json!({ "error": e }))),
    }
}

pub async fn viking_read(
    Authenticated(auth_result): Authenticated,
    State(state): State<Arc<AppState>>,
    Query(q): Query<VikingReadQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if !auth::has_viewer_access(&auth_result.role) {
        return Err(StatusCode::FORBIDDEN);
    }
    let viking = state.viking_client.as_ref().ok_or(StatusCode::NOT_FOUND)?;
    let level = match q.level.as_str() {
        "L0" => ryvos_memory::viking::ContextLevel::L0,
        "L2" => ryvos_memory::viking::ContextLevel::L2,
        _ => ryvos_memory::viking::ContextLevel::L1,
    };
    match viking.read_memory(&q.path, level).await {
        Ok(result) => Ok(Json(serde_json::json!(result))),
        Err(e) => Ok(Json(serde_json::json!({ "error": e }))),
    }
}

pub async fn viking_search(
    Authenticated(auth_result): Authenticated,
    State(state): State<Arc<AppState>>,
    Query(q): Query<VikingSearchQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if !auth::has_viewer_access(&auth_result.role) {
        return Err(StatusCode::FORBIDDEN);
    }
    let viking = state.viking_client.as_ref().ok_or(StatusCode::NOT_FOUND)?;
    match viking
        .search(&q.query, q.directory.as_deref(), q.limit)
        .await
    {
        Ok(results) => Ok(Json(serde_json::json!(results))),
        Err(e) => Ok(Json(serde_json::json!({ "error": e }))),
    }
}


pub async fn get_config(
    Authenticated(auth_result): Authenticated,
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if auth_result.role != ryvos_core::config::ApiKeyRole::Admin {
        return Err(StatusCode::FORBIDDEN);
    }
    let path = state.config_path.as_ref().ok_or(StatusCode::NOT_FOUND)?;
    match tokio::fs::read_to_string(path).await {
        Ok(content) => Ok(Json(serde_json::json!({
            "path": path.display().to_string(),
            "content": content,
        }))),
        Err(e) => Ok(Json(serde_json::json!({ "error": e.to_string() }))),
    }
}

pub async fn put_config(
    Authenticated(auth_result): Authenticated,
    State(state): State<Arc<AppState>>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if auth_result.role != ryvos_core::config::ApiKeyRole::Admin {
        return Err(StatusCode::FORBIDDEN);
    }
    let path = state.config_path.as_ref().ok_or(StatusCode::NOT_FOUND)?;
    let content = body["content"].as_str().ok_or(StatusCode::BAD_REQUEST)?;

    if toml::from_str::<ryvos_core::config::AppConfig>(content).is_err() {
        return Ok(Json(serde_json::json!({ "error": "Invalid TOML config" })));
    }

    match tokio::fs::write(path, content).await {
        Ok(()) => Ok(Json(serde_json::json!({ "ok": true }))),
        Err(e) => Ok(Json(serde_json::json!({ "error": e.to_string() }))),
    }
}


pub async fn channels_status(
    Authenticated(auth_result): Authenticated,
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if !auth::has_viewer_access(&auth_result.role) {
        return Err(StatusCode::FORBIDDEN);
    }

    let mut channels = Vec::new();

    let has_whatsapp = state.whatsapp_handle.is_some();
    let session_list = state.session_mgr.list();

    let has_telegram = session_list.iter().any(|s| s.starts_with("telegram:"));
    let has_discord = session_list.iter().any(|s| s.starts_with("discord:"));
    let has_slack = session_list.iter().any(|s| s.starts_with("slack:"));

    channels.push(serde_json::json!({ "name": "Telegram", "type": "telegram", "status": if has_telegram { "active" } else { "not_configured" } }));
    channels.push(serde_json::json!({ "name": "Discord", "type": "discord", "status": if has_discord { "active" } else { "not_configured" } }));
    channels.push(serde_json::json!({ "name": "Slack", "type": "slack", "status": if has_slack { "active" } else { "not_configured" } }));
    channels.push(serde_json::json!({ "name": "WhatsApp", "type": "whatsapp", "status": if has_whatsapp { "configured" } else { "not_configured" } }));
    channels.push(serde_json::json!({ "name": "Web UI", "type": "webui", "status": "active" }));
    channels.push(serde_json::json!({ "name": "Gateway", "type": "gateway", "status": "active" }));

    Ok(Json(serde_json::json!({ "channels": channels })))
}


pub async fn list_approvals(
    Authenticated(auth_result): Authenticated,
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if !auth::has_viewer_access(&auth_result.role) {
        return Err(StatusCode::FORBIDDEN);
    }
    let pending = state.broker.pending_requests().await;
    Ok(Json(serde_json::json!({ "approvals": pending })))
}

pub async fn approve_request(
    Authenticated(auth_result): Authenticated,
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if !auth::has_operator_access(&auth_result.role) {
        return Err(StatusCode::FORBIDDEN);
    }
    let found = state
        .broker
        .respond(&id, ryvos_core::security::ApprovalDecision::Approved)
        .await;
    Ok(Json(serde_json::json!({ "approved": found })))
}

pub async fn deny_request(
    Authenticated(auth_result): Authenticated,
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if !auth::has_operator_access(&auth_result.role) {
        return Err(StatusCode::FORBIDDEN);
    }
    let found = state
        .broker
        .respond(
            &id,
            ryvos_core::security::ApprovalDecision::Denied {
                reason: "Denied via Web UI".to_string(),
            },
        )
        .await;
    Ok(Json(serde_json::json!({ "denied": found })))
}


pub async fn list_cron(
    Authenticated(auth_result): Authenticated,
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if !auth::has_viewer_access(&auth_result.role) {
        return Err(StatusCode::FORBIDDEN);
    }
    let path = state.config_path.as_ref().ok_or(StatusCode::NOT_FOUND)?;
    let content = tokio::fs::read_to_string(path)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let config: toml::Value =
        toml::from_str(&content).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let jobs = config
        .get("cron")
        .and_then(|c| c.get("jobs"))
        .and_then(|j| j.as_array())
        .cloned()
        .unwrap_or_default();
    Ok(Json(serde_json::json!({ "jobs": jobs })))
}

#[derive(Deserialize)]
pub struct AddCronBody {
    name: String,
    schedule: String,
    prompt: String,
    channel: Option<String>,
    goal: Option<String>,
}

pub async fn add_cron(
    Authenticated(auth_result): Authenticated,
    State(state): State<Arc<AppState>>,
    Json(body): Json<AddCronBody>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if !auth::has_operator_access(&auth_result.role) {
        return Err(StatusCode::FORBIDDEN);
    }
    let path = state.config_path.as_ref().ok_or(StatusCode::NOT_FOUND)?;
    let mut content = tokio::fs::read_to_string(path)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    content.push_str("\n\n[[cron.jobs]]\n");
    content.push_str(&format!("name = {:?}\n", body.name));
    content.push_str(&format!("schedule = {:?}\n", body.schedule));
    content.push_str(&format!("prompt = {:?}\n", body.prompt));
    if let Some(ref ch) = body.channel {
        content.push_str(&format!("channel = {:?}\n", ch));
    }
    if let Some(ref goal) = body.goal {
        content.push_str(&format!("goal = {:?}\n", goal));
    }

    if toml::from_str::<ryvos_core::config::AppConfig>(&content).is_err() {
        return Ok(Json(
            serde_json::json!({ "error": "Invalid config after adding job" }),
        ));
    }

    tokio::fs::write(path, &content)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(
        serde_json::json!({ "ok": true, "note": "Restart required for changes to take effect" }),
    ))
}

pub async fn delete_cron(
    Authenticated(auth_result): Authenticated,
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if !auth::has_operator_access(&auth_result.role) {
        return Err(StatusCode::FORBIDDEN);
    }
    let path = state.config_path.as_ref().ok_or(StatusCode::NOT_FOUND)?;
    let content = tokio::fs::read_to_string(path)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut config: toml::Value =
        toml::from_str(&content).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some(cron) = config.get_mut("cron") {
        if let Some(jobs) = cron.get_mut("jobs") {
            if let Some(arr) = jobs.as_array_mut() {
                arr.retain(|j| j.get("name").and_then(|n| n.as_str()) != Some(&name));
            }
        }
    }

    let new_content =
        toml::to_string_pretty(&config).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    tokio::fs::write(path, &new_content)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(
        serde_json::json!({ "ok": true, "note": "Restart required for changes to take effect" }),
    ))
}


pub async fn get_budget(
    Authenticated(auth_result): Authenticated,
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if !auth::has_viewer_access(&auth_result.role) {
        return Err(StatusCode::FORBIDDEN);
    }
    match &state.budget_config {
        Some(bc) => Ok(Json(serde_json::json!({
            "monthly_budget_cents": bc.monthly_budget_cents,
            "warn_pct": bc.warn_pct,
        }))),
        None => Ok(Json(serde_json::json!({
            "configured": false,
            "note": "No [budget] section in config.toml"
        }))),
    }
}

#[derive(Deserialize)]
pub struct UpdateBudgetBody {
    monthly_budget_cents: Option<u64>,
    warn_pct: Option<u8>,
}

pub async fn put_budget(
    Authenticated(auth_result): Authenticated,
    State(state): State<Arc<AppState>>,
    Json(body): Json<UpdateBudgetBody>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if !auth::has_operator_access(&auth_result.role) {
        return Err(StatusCode::FORBIDDEN);
    }
    let path = state.config_path.as_ref().ok_or(StatusCode::NOT_FOUND)?;
    let content = tokio::fs::read_to_string(path)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut config: toml::Value =
        toml::from_str(&content).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let budget = config
        .as_table_mut()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?
        .entry("budget")
        .or_insert_with(|| toml::Value::Table(toml::map::Map::new()));

    if let Some(table) = budget.as_table_mut() {
        if let Some(v) = body.monthly_budget_cents {
            table.insert(
                "monthly_budget_cents".to_string(),
                toml::Value::Integer(v as i64),
            );
        }
        if let Some(v) = body.warn_pct {
            table.insert("warn_pct".to_string(), toml::Value::Integer(v as i64));
        }
    }

    let new_content =
        toml::to_string_pretty(&config).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    tokio::fs::write(path, &new_content)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(
        serde_json::json!({ "ok": true, "note": "Restart required for changes to take effect" }),
    ))
}


pub async fn get_model(
    Authenticated(auth_result): Authenticated,
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if !auth::has_viewer_access(&auth_result.role) {
        return Err(StatusCode::FORBIDDEN);
    }
    let path = state.config_path.as_ref().ok_or(StatusCode::NOT_FOUND)?;
    let content = tokio::fs::read_to_string(path)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let config: toml::Value =
        toml::from_str(&content).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut model = config
        .get("model")
        .cloned()
        .unwrap_or(toml::Value::Table(toml::map::Map::new()));

    if let Some(table) = model.as_table_mut() {
        table.remove("api_key");
        table.remove("token");
    }

    let mut resp = match serde_json::to_value(&model) {
        Ok(serde_json::Value::Object(map)) => map,
        _ => serde_json::Map::new(),
    };
    resp.insert(
        "model".to_string(),
        serde_json::to_value(&model).unwrap_or_default(),
    );

    Ok(Json(serde_json::Value::Object(resp)))
}

#[derive(Deserialize)]
pub struct ModelsQuery {
    #[serde(default)]
    provider: Option<String>,
}

pub async fn list_models(
    Authenticated(auth_result): Authenticated,
    State(state): State<Arc<AppState>>,
    Query(q): Query<ModelsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if !auth::has_viewer_access(&auth_result.role) {
        return Err(StatusCode::FORBIDDEN);
    }

    let current_mc = state.runtime.current_model_config();
    let current_provider = current_mc.provider.clone();
    let current_model_id = current_mc.model_id.clone();
    let key_count = current_mc.get_api_key_pool().len();

    let provider = q.provider.unwrap_or(current_provider);

    let models = match provider.as_str() {
        "anthropic" | "claude" | "claude-code" | "claude-cli" | "claude-sub" => {
            vec![
                serde_json::json!({"id": "claude-3-7-sonnet", "name": "Claude 3.7 Sonnet (2026)"}),
                serde_json::json!({"id": "claude-sonnet-4-20250514", "name": "Claude Sonnet 4"}),
                serde_json::json!({"id": "claude-opus-4-20250514", "name": "Claude Opus 4"}),
                serde_json::json!({"id": "claude-3-5-sonnet-20241022", "name": "Claude 3.5 Sonnet"}),
                serde_json::json!({"id": "claude-3-5-haiku-20241022", "name": "Claude 3.5 Haiku"}),
            ]
        }
        "openai" => {
            vec![
                serde_json::json!({"id": "gpt-4.5-preview", "name": "GPT-4.5 Preview"}),
                serde_json::json!({"id": "gpt-4o", "name": "GPT-4o"}),
                serde_json::json!({"id": "gpt-4o-mini", "name": "GPT-4o Mini"}),
                serde_json::json!({"id": "o3-mini", "name": "o3-mini"}),
                serde_json::json!({"id": "o1", "name": "o1"}),
            ]
        }
        "gemini" | "google" => {
            vec![
                serde_json::json!({"id": "gemini-3.6-flash", "name": "Gemini 3.6 Flash (2026)"}),
                serde_json::json!({"id": "gemini-2.5-flash", "name": "Gemini 2.5 Flash"}),
                serde_json::json!({"id": "gemini-2.5-pro", "name": "Gemini 2.5 Pro"}),
                serde_json::json!({"id": "gemini-2.0-flash", "name": "Gemini 2.0 Flash"}),
            ]
        }
        "deepseek" => {
            vec![
                serde_json::json!({"id": "deepseek-chat", "name": "DeepSeek-V3"}),
                serde_json::json!({"id": "deepseek-reasoner", "name": "DeepSeek-R1"}),
            ]
        }
        "groq" => {
            vec![
                serde_json::json!({"id": "llama-3.3-70b-versatile", "name": "Llama 3.3 70B"}),
                serde_json::json!({"id": "llama-3.1-8b-instant", "name": "Llama 3.1 8B"}),
                serde_json::json!({"id": "mixtral-8x7b-32768", "name": "Mixtral 8x7B"}),
            ]
        }
        "openrouter" => {
            vec![
                serde_json::json!({"id": "google/gemini-2.5-flash", "name": "OpenRouter: Gemini 2.5 Flash"}),
                serde_json::json!({"id": "anthropic/claude-3.7-sonnet", "name": "OpenRouter: Claude 3.7 Sonnet"}),
                serde_json::json!({"id": "deepseek/deepseek-chat", "name": "OpenRouter: DeepSeek-V3"}),
            ]
        }
        "ollama" => {
            vec![
                serde_json::json!({"id": "llama3.2", "name": "Llama 3.2 (Local)"}),
                serde_json::json!({"id": "qwen2.5:coder", "name": "Qwen 2.5 Coder (Local)"}),
            ]
        }
        _ => vec![serde_json::json!({"id": "custom", "name": "Custom Model"})],
    };

    Ok(Json(serde_json::json!({
        "provider": provider,
        "model_id": current_model_id,
        "models": models,
        "key_count": key_count,
        "all_providers": [
            {"id": "gemini", "name": "Google Gemini"},
            {"id": "anthropic", "name": "Anthropic Claude"},
            {"id": "openai", "name": "OpenAI"},
            {"id": "deepseek", "name": "DeepSeek"},
            {"id": "groq", "name": "Groq"},
            {"id": "openrouter", "name": "OpenRouter"},
            {"id": "ollama", "name": "Ollama (Local)"}
        ]
    })))
}

#[derive(Deserialize)]
pub struct UpdateModelBody {
    provider: Option<String>,
    model_id: Option<String>,
    api_key: Option<String>,
    api_keys: Option<Vec<String>>,
    temperature: Option<f32>,
    max_tokens: Option<u32>,
    thinking: Option<String>,
}

pub async fn put_model(
    Authenticated(auth_result): Authenticated,
    State(state): State<Arc<AppState>>,
    Json(body): Json<UpdateModelBody>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if !auth::has_operator_access(&auth_result.role) {
        return Err(StatusCode::FORBIDDEN);
    }

    let mut current_mc = state.runtime.current_model_config();
    if let Some(ref p) = body.provider {
        current_mc.provider = p.clone();
    }
    if let Some(ref m) = body.model_id {
        current_mc.model_id = m.clone();
    }
    if let Some(ref k) = body.api_key {
        current_mc.api_key = Some(k.clone());
    }
    if let Some(ref arr) = body.api_keys {
        current_mc.api_keys = arr.clone();
    }
    if let Some(t) = body.temperature {
        current_mc.temperature = t;
    }
    if let Some(m) = body.max_tokens {
        current_mc.max_tokens = m;
    }

    state.runtime.update_model_config(current_mc.clone());

    if let Some(ref path) = state.config_path {
        if let Ok(content) = tokio::fs::read_to_string(path).await {
            if let Ok(mut config) = toml::from_str::<toml::Value>(&content) {
                if let Some(model) = config.get_mut("model").and_then(|m| m.as_table_mut()) {
                    if let Some(ref v) = body.provider {
                        model.insert("provider".to_string(), toml::Value::String(v.clone()));
                    }
                    if let Some(ref v) = body.model_id {
                        model.insert("model_id".to_string(), toml::Value::String(v.clone()));
                    }
                    if let Some(ref v) = body.api_key {
                        model.insert("api_key".to_string(), toml::Value::String(v.clone()));
                    }
                    if let Some(ref arr) = body.api_keys {
                        let toml_arr = arr.iter().map(|s| toml::Value::String(s.clone())).collect();
                        model.insert("api_keys".to_string(), toml::Value::Array(toml_arr));
                    }
                    if let Some(v) = body.temperature {
                        model.insert("temperature".to_string(), toml::Value::Float(v as f64));
                    }
                    if let Some(v) = body.max_tokens {
                        model.insert("max_tokens".to_string(), toml::Value::Integer(v as i64));
                    }
                    if let Some(v) = &body.thinking {
                        model.insert("thinking".to_string(), toml::Value::String(v.clone()));
                    }
                }
                if let Ok(new_content) = toml::to_string_pretty(&config) {
                    let _ = tokio::fs::write(path, &new_content).await;
                }
            }
        }
    }

    Ok(Json(serde_json::json!({
        "ok": true,
        "provider": current_mc.provider,
        "model_id": current_mc.model_id,
        "key_count": current_mc.get_api_key_pool().len(),
    })))
}


fn integration_apps() -> Vec<serde_json::Value> {
    vec![
        serde_json::json!({"id": "gmail", "name": "Gmail", "provider": "google", "actions": 23, "icon": "mail"}),
        serde_json::json!({"id": "calendar", "name": "Google Calendar", "provider": "google", "actions": 46, "icon": "calendar"}),
        serde_json::json!({"id": "drive", "name": "Google Drive", "provider": "google", "actions": 50, "icon": "hard-drive"}),
        serde_json::json!({"id": "slack", "name": "Slack", "provider": "slack", "actions": 74, "icon": "message-square"}),
        serde_json::json!({"id": "notion", "name": "Notion", "provider": "notion", "actions": 48, "icon": "book"}),
        serde_json::json!({"id": "github", "name": "GitHub", "provider": "github", "actions": 50, "icon": "github"}),
        serde_json::json!({"id": "jira", "name": "Jira", "provider": "jira", "actions": 97, "icon": "clipboard"}),
        serde_json::json!({"id": "linear", "name": "Linear", "provider": "linear", "actions": 33, "icon": "trending-up"}),
    ]
}

pub async fn list_integrations(
    Authenticated(auth_result): Authenticated,
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if !auth::has_viewer_access(&auth_result.role) {
        return Err(StatusCode::FORBIDDEN);
    }
    let integrations = &state.integrations_config;
    let store = &state.integration_store;
    let mut apps = Vec::new();
    for app in integration_apps() {
        let id = app["id"].as_str().unwrap_or("");
        let configured = match id {
            "gmail" | "calendar" | "drive" => integrations.gmail.is_some(),
            "slack" => integrations.slack.is_some(),
            "notion" => integrations.notion.is_some(),
            "github" => integrations.github.is_some(),
            "jira" => integrations.jira.is_some(),
            "linear" => integrations.linear.is_some(),
            _ => false,
        };
        let store_key = match id {
            "calendar" | "drive" => "gmail",
            other => other,
        };
        let connected = if let Some(ref s) = store {
            s.is_connected(store_key).await
        } else {
            false
        };
        apps.push(serde_json::json!({
            "id": app["id"], "name": app["name"], "provider": app["provider"],
            "actions": app["actions"], "icon": app["icon"],
            "configured": configured, "connected": connected,
        }));
    }
    Ok(Json(serde_json::json!({ "apps": apps })))
}

pub async fn connect_integration(
    Authenticated(auth_result): Authenticated,
    State(state): State<Arc<AppState>>,
    Path(app_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if !auth::has_operator_access(&auth_result.role) {
        return Err(StatusCode::FORBIDDEN);
    }
    if app_id == "notion" {
        if let Some(ref notion) = state.integrations_config.notion {
            if let Some(ref store) = state.integration_store {
                let token = ryvos_memory::IntegrationToken {
                    app_id: "notion".into(),
                    provider: "notion".into(),
                    access_token: notion.api_key.clone(),
                    refresh_token: None,
                    token_expiry: None,
                    scopes: "all".into(),
                    connected_at: chrono::Utc::now().to_rfc3339(),
                };
                store
                    .save_token(&token)
                    .await
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            }
            return Ok(Json(
                serde_json::json!({ "connected": true, "app": "notion" }),
            ));
        }
        return Ok(Json(
            serde_json::json!({ "error": "Notion not configured" }),
        ));
    }
    let provider = crate::oauth::get_provider(&app_id, &state.integrations_config);
    let Some(provider_config) = provider else {
        return Ok(Json(
            serde_json::json!({ "error": format!("{} not configured", app_id) }),
        ));
    };
    let bind = &state.config.bind;
    let redirect_uri = format!("http://{}/api/integrations/callback", bind);
    let auth_url = crate::oauth::generate_auth_url(&provider_config, &redirect_uri, &app_id);
    Ok(Json(
        serde_json::json!({ "redirect_url": auth_url, "app": app_id }),
    ))
}

pub async fn integration_callback(
    State(state): State<Arc<AppState>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let code = params.get("code").cloned().unwrap_or_default();
    let app_id = params.get("state").cloned().unwrap_or_default();
    if code.is_empty() || app_id.is_empty() {
        return axum::response::Redirect::to("/#/integrations?error=missing_code").into_response();
    }
    let provider = crate::oauth::get_provider(&app_id, &state.integrations_config);
    let Some(provider_config) = provider else {
        return axum::response::Redirect::to("/#/integrations?error=not_configured")
            .into_response();
    };
    let bind = &state.config.bind;
    let redirect_uri = format!("http://{}/api/integrations/callback", bind);
    match crate::oauth::exchange_code(&provider_config, &code, &redirect_uri).await {
        Ok(token_resp) => {
            if let Some(ref store) = state.integration_store {
                let token = ryvos_memory::IntegrationToken {
                    app_id: app_id.clone(),
                    provider: app_id.clone(),
                    access_token: token_resp.access_token,
                    refresh_token: token_resp.refresh_token,
                    token_expiry: token_resp.expires_in.map(|e| {
                        (chrono::Utc::now() + chrono::Duration::seconds(e as i64)).to_rfc3339()
                    }),
                    scopes: token_resp.scope.unwrap_or_default(),
                    connected_at: chrono::Utc::now().to_rfc3339(),
                };
                let _ = store.save_token(&token).await;
            }
            axum::response::Redirect::to(&format!("/#/integrations?connected={}", app_id))
                .into_response()
        }
        Err(e) => {
            tracing::error!(app = %app_id, error = %e, "OAuth token exchange failed");
            axum::response::Redirect::to(&format!("/#/integrations?error=exchange&app={}", app_id))
                .into_response()
        }
    }
}

pub async fn disconnect_integration(
    Authenticated(auth_result): Authenticated,
    State(state): State<Arc<AppState>>,
    Path(app_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if !auth::has_operator_access(&auth_result.role) {
        return Err(StatusCode::FORBIDDEN);
    }
    if let Some(ref store) = state.integration_store {
        let deleted = store.delete(&app_id).await.unwrap_or(false);
        Ok(Json(
            serde_json::json!({ "disconnected": deleted, "app": app_id }),
        ))
    } else {
        Ok(Json(
            serde_json::json!({ "error": "Integration store not available" }),
        ))
    }
}


#[derive(Deserialize)]
pub struct RunGoalBody {
    pub description: String,
    #[serde(default)]
    pub prompt: Option<String>,
    #[serde(default)]
    pub channel: Option<String>,
}

pub async fn run_goal(
    Authenticated(auth_result): Authenticated,
    State(state): State<Arc<AppState>>,
    Json(body): Json<RunGoalBody>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if !auth::has_operator_access(&auth_result.role) {
        return Err(StatusCode::FORBIDDEN);
    }

    if body.description.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let now = chrono::Utc::now();
    let session_id = SessionId::from_string(&format!("goal:{}", now.format("%Y%m%d-%H%M%S")));
    let prompt = body.prompt.unwrap_or_else(|| body.description.clone());

    let goal = ryvos_core::goal::Goal {
        description: body.description.clone(),
        success_criteria: vec![ryvos_core::goal::SuccessCriterion {
            id: "llm_judge".into(),
            criterion_type: ryvos_core::goal::CriterionType::LlmJudge {
                prompt: format!("Did the agent achieve this goal: {}?", body.description),
            },
            weight: 1.0,
            description: "Goal achievement".into(),
        }],
        constraints: vec![],
        success_threshold: 0.7,
        version: 0,
        metrics: Default::default(),
    };

    info!(
        session = %session_id,
        goal = %body.description,
        "Goal execution requested via API"
    );

    let runtime = state.runtime.clone();
    let event_bus = state.event_bus.clone();
    let channel = body.channel.clone();
    let sid = session_id.clone();
    let goal_desc = body.description.clone();

    tokio::spawn(async move {
        match runtime.run_with_goal(&sid, &prompt, Some(&goal)).await {
            Ok(response) => {
                info!(session = %sid, "Goal completed");
                if let Some(ch) = channel {
                    event_bus.publish(ryvos_core::types::AgentEvent::CronJobComplete {
                        name: format!("goal: {}", goal_desc),
                        response,
                        channel: Some(ch),
                    });
                }
            }
            Err(e) => {
                tracing::error!(session = %sid, error = %e, "Goal execution failed");
            }
        }
    });

    Ok(Json(serde_json::json!({
        "session_id": session_id.to_string(),
        "status": "running",
        "goal": body.description,
    })))
}

pub async fn goal_history(
    Authenticated(auth_result): Authenticated,
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if !auth::has_viewer_access(&auth_result.role) {
        return Err(StatusCode::FORBIDDEN);
    }

    if let Some(ref cost_store) = state.cost_store {
        let (runs, _total) = cost_store
            .run_history(100, 0)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let goal_runs: Vec<_> = runs
            .into_iter()
            .filter(|r| {
                let sid = r.get("session_id").and_then(|v| v.as_str()).unwrap_or("");
                sid.starts_with("goal:") || sid.starts_with("cron:")
            })
            .collect();

        Ok(Json(serde_json::json!({
            "runs": goal_runs,
        })))
    } else {
        Ok(Json(serde_json::json!({ "runs": [] })))
    }
}


pub async fn list_skills(
    Authenticated(auth_result): Authenticated,
    State(_state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if !auth::has_viewer_access(&auth_result.role) {
        return Err(StatusCode::FORBIDDEN);
    }

    let skills_dir = dirs_home().join(".ryvos/skills");
    let mut skills = Vec::new();

    if skills_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&skills_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let manifest_path = path.join("skill.toml");
                    if manifest_path.exists() {
                        if let Ok(content) = std::fs::read_to_string(&manifest_path) {
                            if let Ok(manifest) = toml::from_str::<toml::Value>(&content) {
                                skills.push(serde_json::json!({
                                    "name": manifest.get("name").and_then(|v| v.as_str()).unwrap_or("unknown"),
                                    "description": manifest.get("description").and_then(|v| v.as_str()).unwrap_or(""),
                                    "command": manifest.get("command").and_then(|v| v.as_str()).unwrap_or(""),
                                    "timeout_secs": manifest.get("timeout_secs").and_then(|v| v.as_integer()).unwrap_or(30),
                                    "tier": manifest.get("tier").and_then(|v| v.as_str()).unwrap_or("t2"),
                                    "enabled": true,
                                }));
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(Json(serde_json::json!({ "skills": skills })))
}


pub async fn heartbeat_history(
    Authenticated(auth_result): Authenticated,
    State(state): State<Arc<AppState>>,
    Query(q): Query<HistoryQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if !auth::has_viewer_access(&auth_result.role) {
        return Err(StatusCode::FORBIDDEN);
    }

    let trail = state.audit_trail.as_ref().ok_or(StatusCode::NOT_FOUND)?;

    let all_entries = trail
        .recent_entries("", q.limit.min(100))
        .await
        .unwrap_or_default();

    let heartbeat_events: Vec<serde_json::Value> = all_entries
        .into_iter()
        .filter(|e| e.session_id.starts_with("heartbeat:"))
        .map(|e| {
            serde_json::json!({
                "timestamp": e.timestamp.to_rfc3339(),
                "session_id": e.session_id,
                "tool_name": e.tool_name,
                "input_summary": e.input_summary,
                "outcome": format!("{:?}", e.outcome),
            })
        })
        .collect();

    Ok(Json(serde_json::json!({ "events": heartbeat_events })))
}


pub async fn safety_lessons(
    Authenticated(auth_result): Authenticated,
    State(state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Value>, StatusCode> {
    if !auth::has_viewer_access(&auth_result.role) {
        return Err(StatusCode::FORBIDDEN);
    }
    let limit = params
        .get("limit")
        .and_then(|v| v.parse().ok())
        .unwrap_or(20);
    let sm = state.safety_memory.as_ref().ok_or(StatusCode::NOT_FOUND)?;
    if let Some(search) = params.get("search") {
        let lessons = sm
            .search_lessons(search, limit)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let items: Vec<Value> = lessons
            .iter()
            .map(|l| {
                serde_json::json!({
                    "id": l.id,
                    "action": l.action,
                    "corrective_rule": l.corrective_rule,
                    "confidence": l.confidence,
                    "times_applied": l.times_applied,
                    "reflection": l.reflection,
                    "principle_violated": l.principle_violated,
                    "timestamp": l.timestamp.to_rfc3339()
                })
            })
            .collect();
        Ok(Json(serde_json::json!({
            "lessons": items,
            "total": sm.count_lessons().await.unwrap_or(0)
        })))
    } else {
        let lessons = sm
            .list_lessons(limit, 0)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let items: Vec<Value> = lessons
            .iter()
            .map(|l| {
                serde_json::json!({
                    "id": l.id,
                    "action": l.action,
                    "corrective_rule": l.corrective_rule,
                    "confidence": l.confidence,
                    "times_applied": l.times_applied,
                    "reflection": l.reflection,
                    "principle_violated": l.principle_violated,
                    "timestamp": l.timestamp.to_rfc3339()
                })
            })
            .collect();
        Ok(Json(serde_json::json!({
            "lessons": items,
            "total": sm.count_lessons().await.unwrap_or(0)
        })))
    }
}

pub async fn list_decisions(
    Authenticated(auth_result): Authenticated,
    State(state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Value>, StatusCode> {
    if !auth::has_viewer_access(&auth_result.role) {
        return Err(StatusCode::FORBIDDEN);
    }
    let limit = params
        .get("limit")
        .and_then(|v| v.parse().ok())
        .unwrap_or(20);
    let fj = state
        .failure_journal
        .as_ref()
        .ok_or(StatusCode::NOT_FOUND)?;
    if let Some(session_id) = params.get("session_id") {
        let decisions = fj
            .load_decisions(session_id, limit)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let items: Vec<Value> = decisions
            .iter()
            .map(|d| {
                serde_json::json!({
                    "id": d.id,
                    "timestamp": d.timestamp.to_rfc3339(),
                    "session_id": d.session_id,
                    "turn": d.turn,
                    "description": d.description,
                    "chosen_option": d.chosen_option,
                    "alternatives": d.alternatives,
                    "outcome": d.outcome,
                })
            })
            .collect();
        Ok(Json(serde_json::json!({
            "decisions": items,
            "total": fj.count_decisions().unwrap_or(0)
        })))
    } else {
        let decisions = fj
            .list_decisions(limit, 0)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let items: Vec<Value> = decisions
            .iter()
            .map(|d| {
                serde_json::json!({
                    "id": d.id,
                    "timestamp": d.timestamp.to_rfc3339(),
                    "session_id": d.session_id,
                    "turn": d.turn,
                    "description": d.description,
                    "chosen_option": d.chosen_option,
                    "alternatives": d.alternatives,
                    "outcome": d.outcome,
                })
            })
            .collect();
        Ok(Json(serde_json::json!({
            "decisions": items,
            "total": fj.count_decisions().unwrap_or(0)
        })))
    }
}

pub async fn list_failures(
    Authenticated(auth_result): Authenticated,
    State(state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Value>, StatusCode> {
    if !auth::has_viewer_access(&auth_result.role) {
        return Err(StatusCode::FORBIDDEN);
    }
    let limit = params
        .get("limit")
        .and_then(|v| v.parse().ok())
        .unwrap_or(20);
    let fj = state
        .failure_journal
        .as_ref()
        .ok_or(StatusCode::NOT_FOUND)?;
    let pattern = params.get("pattern").map(|s| s.as_str());
    let tool = params.get("tool").map(|s| s.as_str());
    let failures = fj
        .search_failures(pattern, tool, limit)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let items: Vec<Value> = failures
        .iter()
        .map(|f| {
            serde_json::json!({
                "timestamp": f.timestamp.to_rfc3339(),
                "session_id": f.session_id,
                "tool_name": f.tool_name,
                "error": f.error,
                "input_summary": f.input_summary,
                "turn": f.turn,
            })
        })
        .collect();
    Ok(Json(serde_json::json!({
        "failures": items,
        "total": fj.count_failures().unwrap_or(0)
    })))
}

fn dirs_home() -> std::path::PathBuf {
    std::env::var("HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from("/tmp"))
}

#[derive(Deserialize, Default)]
pub struct MemorySearchQuery {
    pub query: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Deserialize, Default)]
pub struct MemoryListQuery {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

async fn find_supermemory_server(manager: &ryvos_mcp::McpClientManager) -> Option<String> {
    let servers = manager.connected_servers().await;
    if servers.contains(&"supermemory".to_string()) {
        return Some("supermemory".to_string());
    }
    for server in &servers {
        if server.to_lowercase().contains("supermemory") {
            return Some(server.clone());
        }
    }
    servers.into_iter().next()
}

pub async fn memory_search_get(
    Authenticated(auth_result): Authenticated,
    State(state): State<Arc<AppState>>,
    Query(q): Query<MemorySearchQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if !auth::has_viewer_access(&auth_result.role) {
        return Err(StatusCode::FORBIDDEN);
    }
    let manager = state.mcp_manager.as_ref().ok_or(StatusCode::NOT_FOUND)?;
    let mut args = serde_json::Map::new();
    if let Some(query) = q.query {
        args.insert("query".to_string(), serde_json::Value::String(query));
    }
    if let Some(limit) = q.limit {
        args.insert("limit".to_string(), serde_json::json!(limit));
    }
    let server_name = find_supermemory_server(manager)
        .await
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    match manager
        .call_tool(&server_name, "search_memory", Some(args))
        .await
    {
        Ok(out) => {
            let val = serde_json::from_str::<serde_json::Value>(&out)
                .unwrap_or_else(|_| serde_json::json!({ "result": out }));
            Ok(Json(val))
        }
        Err(e) => Ok(Json(serde_json::json!({ "error": e.to_string() }))),
    }
}

pub async fn memory_search_post(
    Authenticated(auth_result): Authenticated,
    State(state): State<Arc<AppState>>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if !auth::has_viewer_access(&auth_result.role) {
        return Err(StatusCode::FORBIDDEN);
    }
    let manager = state.mcp_manager.as_ref().ok_or(StatusCode::NOT_FOUND)?;
    let mut args = serde_json::Map::new();
    if let Some(obj) = body.as_object() {
        for (k, v) in obj {
            args.insert(k.clone(), v.clone());
        }
    } else if let Some(text) = body.as_str() {
        args.insert("query".to_string(), serde_json::Value::String(text.to_string()));
    }
    let server_name = find_supermemory_server(manager)
        .await
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    match manager
        .call_tool(&server_name, "search_memory", Some(args))
        .await
    {
        Ok(out) => {
            let val = serde_json::from_str::<serde_json::Value>(&out)
                .unwrap_or_else(|_| serde_json::json!({ "result": out }));
            Ok(Json(val))
        }
        Err(e) => Ok(Json(serde_json::json!({ "error": e.to_string() }))),
    }
}

pub async fn memory_list(
    Authenticated(auth_result): Authenticated,
    State(state): State<Arc<AppState>>,
    Query(q): Query<MemoryListQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if !auth::has_viewer_access(&auth_result.role) {
        return Err(StatusCode::FORBIDDEN);
    }
    let manager = state.mcp_manager.as_ref().ok_or(StatusCode::NOT_FOUND)?;
    let mut args = serde_json::Map::new();
    if let Some(limit) = q.limit {
        args.insert("limit".to_string(), serde_json::json!(limit));
    }
    if let Some(offset) = q.offset {
        args.insert("offset".to_string(), serde_json::json!(offset));
    }
    let server_name = find_supermemory_server(manager)
        .await
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    match manager
        .call_tool(&server_name, "list_memories", Some(args))
        .await
    {
        Ok(out) => {
            let val = serde_json::from_str::<serde_json::Value>(&out)
                .unwrap_or_else(|_| serde_json::json!({ "result": out }));
            Ok(Json(val))
        }
        Err(e) => Ok(Json(serde_json::json!({ "error": e.to_string() }))),
    }
}

pub async fn memory_add(
    Authenticated(auth_result): Authenticated,
    State(state): State<Arc<AppState>>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if !auth::has_operator_access(&auth_result.role) {
        return Err(StatusCode::FORBIDDEN);
    }
    let manager = state.mcp_manager.as_ref().ok_or(StatusCode::NOT_FOUND)?;
    let mut args = serde_json::Map::new();
    if let Some(obj) = body.as_object() {
        for (k, v) in obj {
            args.insert(k.clone(), v.clone());
        }
    } else if let Some(text) = body.as_str() {
        args.insert("content".to_string(), serde_json::Value::String(text.to_string()));
    }
    let server_name = find_supermemory_server(manager)
        .await
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    match manager
        .call_tool(&server_name, "add_memory", Some(args))
        .await
    {
        Ok(out) => {
            let val = serde_json::from_str::<serde_json::Value>(&out)
                .unwrap_or_else(|_| serde_json::json!({ "result": out }));
            Ok(Json(val))
        }
        Err(e) => Ok(Json(serde_json::json!({ "error": e.to_string() }))),
    }
}
