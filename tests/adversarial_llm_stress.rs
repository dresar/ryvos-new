use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use futures::future::BoxFuture;
use futures::stream::{self, BoxStream, StreamExt};
use reqwest::header::{HeaderMap, HeaderValue};
use ryvos_agent::agent_loop::AgentRuntime;
use ryvos_core::config::{AppConfig, ModelConfig, RetryConfig};
use ryvos_core::error::{Result, RyvosError};
use ryvos_core::event::EventBus;
use ryvos_core::traits::{LlmClient, Tool};
use ryvos_core::types::*;
use ryvos_llm::retry::{CircuitBreaker, CircuitState, RetryingClient};
use ryvos_llm::streaming::SseParser;
use ryvos_memory::store::SqliteStore;
use ryvos_tools::registry::ToolRegistry;

struct CountingTool {
    calls: Arc<AtomicUsize>,
}

impl Tool for CountingTool {
    fn name(&self) -> &str {
        "counting_tool"
    }

    fn description(&self) -> &str {
        "Counting tool for stress testing"
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "count": { "type": "integer" }
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
                content: "ok".to_string(),
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

    #[allow(dead_code)]
    fn calls(&self) -> usize {
        self.call_count.load(Ordering::SeqCst)
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

fn test_app_config() -> AppConfig {
    let toml_str = r#"
        [model]
        provider = "openai"
        model_id = "gpt-4o"
        api_key = "test-key"

        [agent]
        max_turns = 5
    "#;
    toml::from_str(toml_str).unwrap()
}

fn test_model_config(provider: &str, model: &str) -> ModelConfig {
    ModelConfig {
        provider: provider.to_string(),
        model_id: model.to_string(),
        api_key: Some("test-key".to_string()),
        base_url: None,
        max_tokens: 2048,
        temperature: 0.7,
        thinking: ryvos_core::types::ThinkingLevel::Off,
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
fn test_stress_utf8_byte_by_byte_arbitrary_multilingual() {
    let mut parser = SseParser::new();
    let text = "event: message\ndata: Hello 🌍 世界 café 🦀! \u{1F680}\u{1F389}\n\n";
    let bytes = text.as_bytes();

    let mut events = Vec::new();
    for &b in bytes {
        events.extend(parser.feed_bytes(&[b]));
    }

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type.as_deref(), Some("message"));
    assert_eq!(events[0].data, "Hello 🌍 世界 café 🦀! \u{1F680}\u{1F389}");
}

#[test]
fn test_stress_utf8_multibyte_all_split_positions() {
    let samples = ["é", "€", "🦀", "𐍈", "🔥"];
    for sample in samples {
        let char_bytes = sample.as_bytes();
        let char_len = char_bytes.len();

        for split_at in 1..char_len {
            let mut parser = SseParser::new();
            let prefix = format!("data: start_{}", sample).into_bytes();
            let char_pos = prefix.len() - char_len;

            let chunk1 = &prefix[..char_pos + split_at];
            let chunk2 = [&prefix[char_pos + split_at..], b"_end\n\n"].concat();

            let ev1 = parser.feed_bytes(chunk1);
            assert!(ev1.is_empty());

            let ev2 = parser.feed_bytes(&chunk2);
            assert_eq!(ev2.len(), 1);
            assert_eq!(ev2[0].data, format!("start_{}_end", sample));
        }
    }
}

#[test]
fn test_stress_utf8_multiple_corrupted_bytes_traps_stream() {
    let mut parser = SseParser::new();
    let _ = parser.feed_bytes(b"data: start");
    let _ = parser.feed_bytes(&[0xFF, 0xFE]);
    let evs = parser.feed_bytes(b"_end\n\n");
    assert_eq!(evs.len(), 1);
    assert_eq!(evs[0].data, "start_end");
}

#[test]
fn test_stress_multi_delimiter_permutations() {
    let delims = ["\r\n\r\n", "\n\n", "\r\r", "\r\n\n", "\n\r\n"];
    for delim in delims {
        let mut parser = SseParser::new();
        let payload = format!("data: item1{}data: item2{}", delim, delim);
        let events = parser.feed_bytes(payload.as_bytes());
        assert_eq!(events.len(), 2, "Failed on delimiter: {:?}", delim);
        assert_eq!(events[0].data, "item1");
        assert_eq!(events[1].data, "item2");
    }
}

#[test]
fn test_stress_delimiters_split_across_chunks() {
    let mut parser = SseParser::new();
    let ev1 = parser.feed_bytes(b"data: chunked_event\r");
    assert!(ev1.is_empty());

    let ev2 = parser.feed_bytes(b"\n\r\n");
    assert_eq!(ev2.len(), 1);
    assert_eq!(ev2[0].data, "chunked_event");

    let ev3 = parser.feed_bytes(b"data: second_chunked\r\n");
    assert!(ev3.is_empty());

    let ev4 = parser.feed_bytes(b"\r\n");
    assert_eq!(ev4.len(), 1);
    assert_eq!(ev4[0].data, "second_chunked");
}

#[test]
fn test_stress_comments_and_empty_lines() {
    let mut parser = SseParser::new();
    let stream = ": initial comment\n\n: ping\r\n\r\ndata: payload\n\n: trailing comment\n\n";
    let events = parser.feed(stream);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].data, "payload");
}

#[test]
fn test_stress_circuit_breaker_complete_state_lifecycle() {
    let cb = CircuitBreaker::with_window(3, Duration::from_millis(50), Duration::from_millis(500));
    assert_eq!(cb.state(), CircuitState::Closed);
    assert!(cb.allow_request());

    cb.record_failure(false, None);
    assert_eq!(cb.state(), CircuitState::Closed);
    cb.record_failure(false, None);
    assert_eq!(cb.state(), CircuitState::Closed);

    cb.record_failure(false, None);
    assert_eq!(cb.state(), CircuitState::Open);
    assert!(!cb.allow_request());

    std::thread::sleep(Duration::from_millis(60));
    assert_eq!(cb.state(), CircuitState::HalfOpen);
    assert!(cb.allow_request());

    cb.record_failure(false, None);
    assert_eq!(cb.state(), CircuitState::Open);
    assert!(!cb.allow_request());

    std::thread::sleep(Duration::from_millis(60));
    assert_eq!(cb.state(), CircuitState::HalfOpen);
    assert!(cb.allow_request());

    cb.record_success();
    assert_eq!(cb.state(), CircuitState::Closed);
    assert!(cb.allow_request());
}

#[test]
fn test_stress_circuit_breaker_429_immediate_trip_and_cooldown() {
    let cb = CircuitBreaker::with_window(5, Duration::from_millis(200), Duration::from_secs(60));
    assert_eq!(cb.state(), CircuitState::Closed);

    cb.record_failure(true, Some(Duration::from_millis(80)));
    assert_eq!(cb.state(), CircuitState::Open);
    assert!(!cb.allow_request());

    std::thread::sleep(Duration::from_millis(30));
    assert_eq!(cb.state(), CircuitState::Open);
    assert!(!cb.allow_request());

    std::thread::sleep(Duration::from_millis(60));
    assert_eq!(cb.state(), CircuitState::HalfOpen);
    assert!(cb.allow_request());
}

#[test]
fn test_stress_circuit_breaker_retry_after_parsing() {
    assert_eq!(
        CircuitBreaker::parse_retry_after_str("HTTP 429: retry-after: 5s"),
        Some(Duration::from_secs(5))
    );
    assert_eq!(
        CircuitBreaker::parse_retry_after_str("Rate limited, retry after: 250ms"),
        Some(Duration::from_millis(250))
    );
    assert_eq!(
        CircuitBreaker::parse_retry_after_str("error retry_after = 12"),
        Some(Duration::from_secs(12))
    );

    let mut headers = HeaderMap::new();
    headers.insert("retry-after", HeaderValue::from_static("3"));
    assert_eq!(
        CircuitBreaker::parse_retry_after_headers(&headers),
        Some(Duration::from_secs(3))
    );

    let mut headers2 = HeaderMap::new();
    headers2.insert("x-ratelimit-reset", HeaderValue::from_static("7"));
    assert_eq!(
        CircuitBreaker::parse_retry_after_headers(&headers2),
        Some(Duration::from_secs(7))
    );

    let mut headers3 = HeaderMap::new();
    headers3.insert("x-ratelimit-reset-requests", HeaderValue::from_static("400ms"));
    assert_eq!(
        CircuitBreaker::parse_retry_after_headers(&headers3),
        Some(Duration::from_millis(400))
    );
}

#[test]
fn test_stress_circuit_breaker_concurrent_threads() {
    let cb = Arc::new(CircuitBreaker::new(5, Duration::from_millis(20)));
    let (done_tx, done_rx) = std::sync::mpsc::channel();
    let cb_clone = cb.clone();
    std::thread::spawn(move || {
        let mut handles = Vec::new();
        for i in 0..4 {
            let cb_c = cb_clone.clone();
            handles.push(std::thread::spawn(move || {
                for j in 0..50 {
                    if (i + j) % 2 == 0 {
                        cb_c.record_failure(false, None);
                    } else {
                        cb_c.record_success();
                    }
                }
            }));
        }
        for h in handles {
            let _ = h.join();
        }
        let _ = done_tx.send(());
    });

    let _ = done_rx.recv_timeout(Duration::from_millis(500));
}

#[tokio::test]
async fn test_stress_retrying_client_failover_on_429() {
    let primary = Box::new(ScriptedLlmClient::new(vec![Err(RyvosError::LlmRequest(
        "HTTP 429 Too Many Requests (retry-after: 10s)".into(),
    ))]));

    let secondary_cfg = test_model_config("gemini", "gemini-1.5-pro");
    let secondary = Box::new(ScriptedLlmClient::new(vec![
        Ok(vec![
            StreamDelta::TextDelta("gemini answer".to_string()),
            StreamDelta::Stop(StopReason::EndTurn),
        ]),
        Ok(vec![
            StreamDelta::TextDelta("gemini answer 2".to_string()),
            StreamDelta::Stop(StopReason::EndTurn),
        ]),
    ]));

    let retry_cfg = RetryConfig {
        max_retries: 0,
        initial_backoff_ms: 1,
        max_backoff_ms: 2,
    };

    let client = RetryingClient::new(
        primary,
        vec![(secondary_cfg, secondary)],
        retry_cfg,
    );

    let cfg = test_model_config("openai", "gpt-4o");

    let stream = client
        .chat_stream(&cfg, vec![ChatMessage::user("hello")], &[])
        .await
        .unwrap();

    let mut stream = stream;
    let mut texts = Vec::new();
    while let Some(Ok(delta)) = stream.next().await {
        if let StreamDelta::TextDelta(t) = delta {
            texts.push(t);
        }
    }
    assert_eq!(texts, vec!["gemini answer"]);

    assert_eq!(client.circuit_breaker().state(), CircuitState::Open);

    let stream2 = client
        .chat_stream(&cfg, vec![ChatMessage::user("hello again")], &[])
        .await
        .unwrap();

    let mut stream2 = stream2;
    let mut texts2 = Vec::new();
    while let Some(Ok(delta)) = stream2.next().await {
        if let StreamDelta::TextDelta(t) = delta {
            texts2.push(t);
        }
    }
    assert_eq!(texts2, vec!["gemini answer 2"]);
}

#[tokio::test]
async fn test_stress_retrying_client_all_fail_zero_panic() {
    let primary = Box::new(ScriptedLlmClient::new(vec![Err(RyvosError::LlmRequest(
        "Primary down".into(),
    ))]));

    let secondary_cfg = test_model_config("gemini", "gemini-1.5-pro");
    let secondary = Box::new(ScriptedLlmClient::new(vec![Err(RyvosError::LlmRequest(
        "Secondary down".into(),
    ))]));

    let retry_cfg = RetryConfig {
        max_retries: 0,
        initial_backoff_ms: 1,
        max_backoff_ms: 2,
    };

    let client = RetryingClient::new(
        primary,
        vec![(secondary_cfg, secondary)],
        retry_cfg,
    );

    let cfg = test_model_config("openai", "gpt-4o");
    let res = client
        .chat_stream(&cfg, vec![ChatMessage::user("test")], &[])
        .await;

    assert!(res.is_err());
}

#[tokio::test]
async fn test_stress_agent_runtime_midstream_drop_discards_tool_call() {
    let tool_calls_count = Arc::new(AtomicUsize::new(0));
    let tool = CountingTool {
        calls: tool_calls_count.clone(),
    };

    let mut registry = ToolRegistry::new();
    registry.register(tool);
    let tools = Arc::new(tokio::sync::RwLock::new(registry));

    let deltas = vec![
        StreamDelta::ToolUseStart {
            index: 0,
            id: "call-1".to_string(),
            name: "counting_tool".to_string(),
        },
        StreamDelta::ToolInputDelta {
            index: 0,
            delta: "{\"count\":".to_string(),
        },
        StreamDelta::ToolInputDelta {
            index: 0,
            delta: " 42".to_string(),
        },
    ];

    let llm: Arc<dyn LlmClient> = Arc::new(ScriptedLlmClient::new(vec![Ok(deltas)]));
    let store = Arc::new(SqliteStore::in_memory().unwrap());
    let event_bus = Arc::new(EventBus::new(32));
    let config = test_app_config();

    let runtime = AgentRuntime::new(config, llm, tools, store, event_bus);
    let session_id = SessionId::new();

    let result = runtime.run(&session_id, "run counting tool").await;
    assert!(result.is_err());

    match result {
        Err(RyvosError::LlmStream(msg)) => {
            assert!(msg.contains("dropped before tool call completion"));
        }
        other => panic!("Expected LlmStream error, got {:?}", other),
    }

    assert_eq!(tool_calls_count.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn test_stress_agent_runtime_null_arguments_discarded() {
    let tool_calls_count = Arc::new(AtomicUsize::new(0));
    let tool = CountingTool {
        calls: tool_calls_count.clone(),
    };

    let mut registry = ToolRegistry::new();
    registry.register(tool);
    let tools = Arc::new(tokio::sync::RwLock::new(registry));

    let deltas = vec![
        StreamDelta::ToolUseStart {
            index: 0,
            id: "call-2".to_string(),
            name: "counting_tool".to_string(),
        },
        StreamDelta::ToolInputDelta {
            index: 0,
            delta: "null".to_string(),
        },
        StreamDelta::Stop(StopReason::ToolUse),
    ];

    let llm: Arc<dyn LlmClient> = Arc::new(ScriptedLlmClient::new(vec![Ok(deltas)]));
    let store = Arc::new(SqliteStore::in_memory().unwrap());
    let event_bus = Arc::new(EventBus::new(32));
    let config = test_app_config();

    let runtime = AgentRuntime::new(config, llm, tools, store, event_bus);
    let session_id = SessionId::new();

    let result = runtime.run(&session_id, "run counting tool").await;
    assert!(result.is_err());

    match result {
        Err(RyvosError::LlmStream(msg)) => {
            assert!(msg.contains("null arguments"));
        }
        other => panic!("Expected LlmStream null error, got {:?}", other),
    }

    assert_eq!(tool_calls_count.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn test_stress_agent_runtime_malformed_json_discarded() {
    let tool_calls_count = Arc::new(AtomicUsize::new(0));
    let tool = CountingTool {
        calls: tool_calls_count.clone(),
    };

    let mut registry = ToolRegistry::new();
    registry.register(tool);
    let tools = Arc::new(tokio::sync::RwLock::new(registry));

    let deltas = vec![
        StreamDelta::ToolUseStart {
            index: 0,
            id: "call-3".to_string(),
            name: "counting_tool".to_string(),
        },
        StreamDelta::ToolInputDelta {
            index: 0,
            delta: "{ invalid_unquoted_token }".to_string(),
        },
        StreamDelta::Stop(StopReason::ToolUse),
    ];

    let llm: Arc<dyn LlmClient> = Arc::new(ScriptedLlmClient::new(vec![Ok(deltas)]));
    let store = Arc::new(SqliteStore::in_memory().unwrap());
    let event_bus = Arc::new(EventBus::new(32));
    let config = test_app_config();

    let runtime = AgentRuntime::new(config, llm, tools, store, event_bus);
    let session_id = SessionId::new();

    let result = runtime.run(&session_id, "run counting tool").await;
    assert!(result.is_err());

    match result {
        Err(RyvosError::LlmStream(msg)) => {
            assert!(msg.contains("failed to parse"));
        }
        other => panic!("Expected LlmStream parse error, got {:?}", other),
    }

    assert_eq!(tool_calls_count.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn test_stress_agent_runtime_unclosed_json_repaired() {
    let tool_calls_count = Arc::new(AtomicUsize::new(0));
    let tool = CountingTool {
        calls: tool_calls_count.clone(),
    };

    let mut registry = ToolRegistry::new();
    registry.register(tool);
    let tools = Arc::new(tokio::sync::RwLock::new(registry));

    let turn1_deltas = vec![
        StreamDelta::ToolUseStart {
            index: 0,
            id: "call-4".to_string(),
            name: "counting_tool".to_string(),
        },
        StreamDelta::ToolInputDelta {
            index: 0,
            delta: "{\"count\": 99".to_string(),
        },
        StreamDelta::Stop(StopReason::ToolUse),
    ];

    let turn2_deltas = vec![
        StreamDelta::TextDelta("Tool execution finished.".to_string()),
        StreamDelta::Stop(StopReason::EndTurn),
    ];

    let llm: Arc<dyn LlmClient> = Arc::new(ScriptedLlmClient::new(vec![
        Ok(turn1_deltas),
        Ok(turn2_deltas),
    ]));
    let store = Arc::new(SqliteStore::in_memory().unwrap());
    let event_bus = Arc::new(EventBus::new(32));
    let config = test_app_config();

    let runtime = AgentRuntime::new(config, llm, tools, store, event_bus);
    let session_id = SessionId::new();

    let result = runtime.run(&session_id, "run unclosed tool").await;
    assert!(result.is_ok(), "Expected success with unclosed JSON repair, got {:?}", result);
    assert_eq!(result.unwrap(), "Tool execution finished.");
    assert_eq!(tool_calls_count.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn test_stress_agent_runtime_inner_trailing_comma_repaired() {
    let tool_calls_count = Arc::new(AtomicUsize::new(0));
    let tool = CountingTool {
        calls: tool_calls_count.clone(),
    };

    let mut registry = ToolRegistry::new();
    registry.register(tool);
    let tools = Arc::new(tokio::sync::RwLock::new(registry));

    let turn1_deltas = vec![
        StreamDelta::ToolUseStart {
            index: 0,
            id: "call-5".to_string(),
            name: "counting_tool".to_string(),
        },
        StreamDelta::ToolInputDelta {
            index: 0,
            delta: "{\"count\": 99,}".to_string(),
        },
        StreamDelta::Stop(StopReason::ToolUse),
    ];

    let turn2_deltas = vec![
        StreamDelta::TextDelta("Tool execution finished.".to_string()),
        StreamDelta::Stop(StopReason::EndTurn),
    ];

    let llm: Arc<dyn LlmClient> = Arc::new(ScriptedLlmClient::new(vec![
        Ok(turn1_deltas),
        Ok(turn2_deltas),
    ]));
    let store = Arc::new(SqliteStore::in_memory().unwrap());
    let event_bus = Arc::new(EventBus::new(32));
    let config = test_app_config();

    let runtime = AgentRuntime::new(config, llm, tools, store, event_bus);
    let session_id = SessionId::new();

    let result = runtime.run(&session_id, "run trailing comma tool").await;
    assert!(result.is_ok(), "Inner trailing comma repaired successfully, got {:?}", result);
    assert_eq!(result.unwrap(), "Tool execution finished.");
    assert_eq!(tool_calls_count.load(Ordering::SeqCst), 1);
}
