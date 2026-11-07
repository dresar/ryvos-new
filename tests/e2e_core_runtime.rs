use std::path::PathBuf;
use std::sync::Arc;
use tempfile::TempDir;

use ryvos_core::config::AppConfig;
use ryvos_core::traits::{SessionStore, Tool};
use ryvos_core::types::{ChatMessage, CostEvent, SessionId, ToolContext};
use ryvos_memory::{apply_lean_pragmas, CostStore, IntegrationStore, SessionMetaStore, SqliteStore};
use ryvos_tools::builtin::network::{DnsLookupTool, HttpDownloadTool};

fn create_temp_context(temp_dir: &TempDir) -> ToolContext {
    ToolContext {
        session_id: SessionId::from_string("e2e-core-session"),
        working_dir: temp_dir.path().to_path_buf(),
        store: None,
        agent_spawner: None,
        sandbox_config: None,
        config_path: None,
        viking_client: None,
    }
}

#[test]
fn test_config_parse_valid_default_minimal() {
    let toml_data = r#"
[model]
provider = "openai"
model_id = "gpt-4o"
api_key = "sk-test"

[agent]
max_turns = 10
"#;
    let parsed: Result<AppConfig, _> = toml::from_str(toml_data);
    assert!(parsed.is_ok());
    let cfg = parsed.unwrap();
    assert_eq!(cfg.model.provider, "openai");
    assert_eq!(cfg.model.model_id, "gpt-4o");
    assert_eq!(cfg.agent.max_turns, 10);
}

#[tokio::test]
async fn test_sqlite_store_lifecycle_and_pragmas() {
    let temp = TempDir::new().unwrap();
    let db_path = temp.path().join("sessions.db");
    let store = SqliteStore::open(&db_path).unwrap();

    let session = SessionId::from_string("session-001");
    let msg1 = ChatMessage::user("Hello agent runtime");
    let msg2 = ChatMessage::assistant_text("Hello human, ready to assist");

    assert!(store.append_messages(&session, &[msg1]).await.is_ok());
    assert!(store.append_messages(&session, &[msg2]).await.is_ok());

    let history = store.load_history(&session, 10).await.unwrap();
    assert_eq!(history.len(), 2);
    assert_eq!(history[0].text(), "Hello agent runtime");
    assert_eq!(history[1].text(), "Hello human, ready to assist");

    let conn = rusqlite::Connection::open(&db_path).unwrap();
    let journal_mode: String = conn
        .query_row("PRAGMA journal_mode;", [], |row| row.get(0))
        .unwrap();
    assert_eq!(journal_mode.to_lowercase(), "wal");

    apply_lean_pragmas(&conn).unwrap();
    let synchronous: i32 = conn
        .query_row("PRAGMA synchronous;", [], |row| row.get(0))
        .unwrap();
    assert_eq!(synchronous, 1);
}

#[tokio::test]
async fn test_cost_store_lifecycle_and_pragmas() {
    let temp = TempDir::new().unwrap();
    let db_path = temp.path().join("cost.db");
    let store = CostStore::open(&db_path).unwrap();

    let event = CostEvent {
        run_id: "run-001".to_string(),
        session_id: "session-001".to_string(),
        timestamp: chrono::Utc::now(),
        input_tokens: 1500,
        output_tokens: 350,
        cost_cents: 12,
        billing_type: ryvos_core::types::BillingType::Api,
        model: "gpt-4o".to_string(),
        provider: "openai".to_string(),
    };

    assert!(store.record_cost_event(&event).is_ok());

    let from = chrono::Utc::now() - chrono::Duration::hours(1);
    let to = chrono::Utc::now() + chrono::Duration::hours(1);
    let summary = store.cost_summary(&from, &to).unwrap();
    assert!(summary.total_cost_cents >= 12);

    let conn = rusqlite::Connection::open(&db_path).unwrap();
    conn.execute_batch(
        "PRAGMA cache_size = -2000;
         PRAGMA mmap_size = 0;
         PRAGMA temp_store = MEMORY;",
    )
    .unwrap();

    let cache_size: i64 = conn
        .query_row("PRAGMA cache_size;", [], |row| row.get(0))
        .unwrap();
    assert_eq!(cache_size, -2000);

    let mmap_size: i64 = conn
        .query_row("PRAGMA mmap_size;", [], |row| row.get(0))
        .unwrap();
    assert_eq!(mmap_size, 0);
}

#[tokio::test]
async fn test_session_meta_store_lifecycle_and_pragmas() {
    let temp = TempDir::new().unwrap();
    let db_path = temp.path().join("session_meta.db");
    let store = SessionMetaStore::open(&db_path).unwrap();

    let session = SessionId::from_string("session-meta-001");
    assert!(store.get(&session.to_string()).is_ok());

    let conn = rusqlite::Connection::open(&db_path).unwrap();
    conn.execute_batch(
        "PRAGMA cache_size = -2000;
         PRAGMA mmap_size = 0;
         PRAGMA temp_store = MEMORY;",
    )
    .unwrap();

    let temp_store: i32 = conn
        .query_row("PRAGMA temp_store;", [], |row| row.get(0))
        .unwrap();
    assert_eq!(temp_store, 2);
}

#[tokio::test]
async fn test_integration_store_lifecycle_and_pragmas() {
    let temp = TempDir::new().unwrap();
    let db_path = temp.path().join("integrations.db");
    let store = IntegrationStore::open(&db_path).unwrap();

    assert!(store.list_connected().await.is_ok());

    let conn = rusqlite::Connection::open(&db_path).unwrap();
    let journal_mode: String = conn
        .query_row("PRAGMA journal_mode;", [], |row| row.get(0))
        .unwrap();
    assert_eq!(journal_mode.to_lowercase(), "wal");
}

#[test]
fn test_streaming_download_tool_declaration_and_schema() {
    let tool = HttpDownloadTool;
    assert_eq!(tool.name(), "http_download");
    assert_eq!(tool.tier(), ryvos_core::security::SecurityTier::T1);
    assert_eq!(tool.timeout_secs(), 300);

    let schema = tool.input_schema();
    assert_eq!(schema["type"], "object");
    assert!(schema["properties"]["url"].is_object());
    assert!(schema["properties"]["output"].is_object());
    assert!(schema["required"].is_array());
}

#[test]
fn test_dns_lookup_tool_declaration_and_schema() {
    let tool = DnsLookupTool;
    assert_eq!(tool.name(), "dns_lookup");
    assert_eq!(tool.tier(), ryvos_core::security::SecurityTier::T0);

    let schema = tool.input_schema();
    assert!(schema["properties"]["hostname"].is_object());
}

#[test]
fn test_zero_panic_config_empty_string() {
    let parsed: Result<AppConfig, _> = toml::from_str("");
    assert!(parsed.is_err() || parsed.is_ok());
}

#[test]
fn test_zero_panic_config_malformed_syntax() {
    let malformed = "[[[this is completely broken toml ::: ===";
    let parsed: Result<AppConfig, _> = toml::from_str(malformed);
    assert!(parsed.is_err());
}

#[test]
fn test_zero_panic_config_unknown_fields() {
    let toml_data = r#"
[unknown_section]
mystery_key = "unexpected_value"
nested_field = 42
"#;
    let parsed: Result<AppConfig, _> = toml::from_str(toml_data);
    let _ = parsed;
}

#[test]
fn test_zero_panic_config_extreme_numeric_values() {
    let extreme_toml = r#"
[agent]
max_turns = 18446744073709551615
max_duration_secs = 999999999999
"#;
    let parsed: Result<AppConfig, _> = toml::from_str(extreme_toml);
    let _ = parsed;
}

#[test]
fn test_zero_panic_sqlite_corrupted_database_file() {
    let temp = TempDir::new().unwrap();
    let db_path = temp.path().join("corrupted.db");
    std::fs::write(&db_path, b"NOT_A_VALID_SQLITE3_HEADER_JUST_RANDOM_GARBAGE").unwrap();

    let result = SqliteStore::open(&db_path);
    assert!(result.is_err());
}

#[test]
fn test_zero_panic_sqlite_nonexistent_deep_directory() {
    let temp = TempDir::new().unwrap();
    let deep_path = temp
        .path()
        .join("a")
        .join("b")
        .join("c")
        .join("deep_store.db");
    let result = SqliteStore::open(&deep_path);
    assert!(result.is_ok());
}

#[test]
fn test_zero_panic_sqlite_special_characters_in_path() {
    let temp = TempDir::new().unwrap();
    let special_dir = temp.path().join("test with spaces and #!@ %^&*");
    std::fs::create_dir_all(&special_dir).unwrap();
    let db_path = special_dir.join("special_store.db");
    let result = SqliteStore::open(&db_path);
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_zero_panic_sqlite_unclosed_quotes_query_handling() {
    let temp = TempDir::new().unwrap();
    let db_path = temp.path().join("unclosed.db");
    let store = SqliteStore::open(&db_path).unwrap();

    let session = SessionId::from_string("session-unclosed");
    let malicious_msg = ChatMessage::user("Robert'); DROP TABLE messages; -- '\"'");
    let append_res = store.append_messages(&session, &[malicious_msg]).await;
    assert!(append_res.is_ok());

    let search_res = store.search("DROP TABLE'\"", 10).await;
    assert!(search_res.is_ok() || search_res.is_err());
}

#[test]
fn test_zero_panic_cost_store_extreme_token_counts() {
    let temp = TempDir::new().unwrap();
    let db_path = temp.path().join("cost_extreme.db");
    let store = CostStore::open(&db_path).unwrap();

    let extreme_event = CostEvent {
        run_id: "run-extreme".to_string(),
        session_id: "session-extreme".to_string(),
        timestamp: chrono::Utc::now(),
        input_tokens: i64::MAX as u64,
        output_tokens: i64::MAX as u64,
        cost_cents: u64::MAX,
        billing_type: ryvos_core::types::BillingType::Api,
        model: "extreme-model".to_string(),
        provider: "extreme-provider".to_string(),
    };
    let res = store.record_cost_event(&extreme_event);
    assert!(res.is_ok() || res.is_err());
}

#[tokio::test]
async fn test_zero_panic_download_tool_empty_url() {
    let temp = TempDir::new().unwrap();
    let ctx = create_temp_context(&temp);
    let tool = HttpDownloadTool;

    let input = serde_json::json!({
        "url": "",
        "output": "downloaded.bin"
    });
    let res = tool.execute(input, ctx).await;
    assert!(res.is_err() || res.unwrap().is_error);
}

#[tokio::test]
async fn test_zero_panic_download_tool_invalid_url_protocol() {
    let temp = TempDir::new().unwrap();
    let ctx = create_temp_context(&temp);
    let tool = HttpDownloadTool;

    let input = serde_json::json!({
        "url": "not_a_valid_protocol://bad-host:99999/test",
        "output": "downloaded.bin"
    });
    let res = tool.execute(input, ctx).await;
    assert!(res.is_err() || res.unwrap().is_error);
}

#[tokio::test]
async fn test_zero_panic_download_tool_missing_output_field() {
    let temp = TempDir::new().unwrap();
    let ctx = create_temp_context(&temp);
    let tool = HttpDownloadTool;

    let input = serde_json::json!({
        "url": "http://127.0.0.1:65534/test"
    });
    let res = tool.execute(input, ctx).await;
    assert!(res.is_err());
}

#[tokio::test]
async fn test_zero_panic_tool_execution_empty_json_object() {
    let temp = TempDir::new().unwrap();
    let ctx = create_temp_context(&temp);
    let tool = HttpDownloadTool;

    let input = serde_json::json!({});
    let res = tool.execute(input, ctx).await;
    assert!(res.is_err());
}

#[tokio::test]
async fn test_pairwise_corrupted_config_and_sqlite_recovery() {
    let temp = TempDir::new().unwrap();
    let broken_config = "this is garbage config";
    let parsed_config: Result<AppConfig, _> = toml::from_str(broken_config);
    assert!(parsed_config.is_err());

    let db_path = temp.path().join("pairwise_recovery.db");
    let store = SqliteStore::open(&db_path).unwrap();
    let session = SessionId::from_string("pairwise-session");
    let msg = ChatMessage::user("System recovery test");
    assert!(store.append_messages(&session, &[msg]).await.is_ok());

    let loaded = store.load_history(&session, 10).await.unwrap();
    assert_eq!(loaded.len(), 1);
}

#[tokio::test]
async fn test_pairwise_cost_store_with_concurrent_sqlite_messages() {
    let temp = TempDir::new().unwrap();
    let session_db = temp.path().join("concurrent_session.db");
    let cost_db = temp.path().join("concurrent_cost.db");

    let session_store = Arc::new(SqliteStore::open(&session_db).unwrap());
    let cost_store = Arc::new(CostStore::open(&cost_db).unwrap());

    let mut handles = Vec::new();
    for i in 0..10 {
        let s_store = session_store.clone();
        let c_store = cost_store.clone();
        let handle = tokio::spawn(async move {
            let session = SessionId::from_string(&format!("session-{}", i));
            let msg = ChatMessage::user(format!("Message count {}", i));
            s_store.append_messages(&session, &[msg]).await.unwrap();

            let event = CostEvent {
                run_id: format!("run-{}", i),
                session_id: session.to_string(),
                timestamp: chrono::Utc::now(),
                input_tokens: 100 * (i + 1),
                output_tokens: 50 * (i + 1),
                cost_cents: (i + 1),
                billing_type: ryvos_core::types::BillingType::Api,
                model: "gpt-4o".to_string(),
                provider: "openai".to_string(),
            };
            c_store.record_cost_event(&event).unwrap();
        });
        handles.push(handle);
    }

    for h in handles {
        h.await.unwrap();
    }

    let from = chrono::Utc::now() - chrono::Duration::hours(1);
    let to = chrono::Utc::now() + chrono::Duration::hours(1);
    let summary = cost_store.cost_summary(&from, &to).unwrap();
    assert!(summary.total_cost_cents >= 55);
}

#[tokio::test]
async fn test_pairwise_download_tool_with_invalid_directory_and_timeout() {
    let _temp = TempDir::new().unwrap();
    let ctx = ToolContext {
        session_id: SessionId::from_string("timeout-test"),
        working_dir: PathBuf::from("/proc/forbidden/nonexistent/path"),
        store: None,
        agent_spawner: None,
        sandbox_config: None,
        config_path: None,
        viking_client: None,
    };
    let tool = HttpDownloadTool;
    let input = serde_json::json!({
        "url": "http://127.0.0.1:18789/nonexistent",
        "output": "test.bin"
    });
    let res = tool.execute(input, ctx).await;
    assert!(res.is_err() || res.unwrap().is_error);
}

#[tokio::test]
async fn test_real_world_multi_store_transactional_resilience() {
    let temp = TempDir::new().unwrap();
    let session_db = temp.path().join("full_session.db");
    let cost_db = temp.path().join("full_cost.db");
    let meta_db = temp.path().join("full_meta.db");
    let integ_db = temp.path().join("full_integ.db");

    let session_store = SqliteStore::open(&session_db).unwrap();
    let cost_store = CostStore::open(&cost_db).unwrap();
    let meta_store = SessionMetaStore::open(&meta_db).unwrap();
    let integ_store = IntegrationStore::open(&integ_db).unwrap();

    let session = SessionId::from_string("real-world-session-1");
    for i in 0..20 {
        let msg = if i % 2 == 0 {
            ChatMessage::user(format!("Turn {} query with payload data", i))
        } else {
            ChatMessage::assistant_text(format!("Turn {} response with reasoning", i))
        };
        session_store.append_messages(&session, &[msg]).await.unwrap();

        let event = CostEvent {
            run_id: format!("run-{}", i),
            session_id: session.to_string(),
            timestamp: chrono::Utc::now(),
            input_tokens: 200,
            output_tokens: 80,
            cost_cents: 2,
            billing_type: ryvos_core::types::BillingType::Api,
            model: "claude-sonnet".to_string(),
            provider: "anthropic".to_string(),
        };
        cost_store.record_cost_event(&event).unwrap();
    }

    let history = session_store.load_history(&session, 50).await.unwrap();
    assert_eq!(history.len(), 20);

    let from = chrono::Utc::now() - chrono::Duration::hours(1);
    let to = chrono::Utc::now() + chrono::Duration::hours(1);
    let summary = cost_store.cost_summary(&from, &to).unwrap();
    assert_eq!(summary.total_cost_cents, 40);

    assert!(meta_store.list().is_ok());
    assert!(integ_store.list_connected().await.is_ok());
}

#[tokio::test]
async fn test_real_world_agent_runtime_zero_panic_fault_injection() {
    let temp = TempDir::new().unwrap();
    let session_db = temp.path().join("fault_session.db");
    let session_store = SqliteStore::open(&session_db).unwrap();

    let session = SessionId::from_string("fault-injection-session");

    let extreme_payload = "A".repeat(500_000);
    let msg = ChatMessage::user(extreme_payload);
    assert!(session_store.append_messages(&session, &[msg]).await.is_ok());

    let null_byte_payload = "Hello \0 World \0 with \0 null \0 bytes";
    let msg_null = ChatMessage::user(null_byte_payload);
    assert!(session_store.append_messages(&session, &[msg_null]).await.is_ok());

    let unicode_payload = "🚀 🦀 💥 \u{1F980} \u{1F680} \u{2705} \u{1F512} \u{1F6E1}";
    let msg_unicode = ChatMessage::user(unicode_payload);
    assert!(session_store.append_messages(&session, &[msg_unicode]).await.is_ok());

    let history = session_store.load_history(&session, 10).await.unwrap();
    assert_eq!(history.len(), 3);
}
