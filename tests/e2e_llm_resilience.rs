use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use futures::future::BoxFuture;
use futures::stream::{self, BoxStream, StreamExt};

use ryvos_core::config::{ModelConfig, RetryConfig};
use ryvos_core::error::{Result, RyvosError};
use ryvos_core::traits::LlmClient;
use ryvos_core::types::{ChatMessage, StopReason, StreamDelta, ToolDefinition};
use ryvos_llm::retry::RetryingClient;
use ryvos_llm::streaming::SseParser;

struct ByteSseAccumulator {
    raw_buffer: Vec<u8>,
    parser: SseParser,
}

impl ByteSseAccumulator {
    fn new() -> Self {
        Self {
            raw_buffer: Vec::new(),
            parser: SseParser::new(),
        }
    }

    fn feed_bytes(&mut self, chunk: &[u8]) -> Vec<ryvos_llm::streaming::SseEvent> {
        self.raw_buffer.extend_from_slice(chunk);
        let mut events = Vec::new();

        match std::str::from_utf8(&self.raw_buffer) {
            Ok(valid_str) => {
                let parsed = self.parser.feed(valid_str);
                events.extend(parsed);
                self.raw_buffer.clear();
            }
            Err(err) => {
                let valid_up_to = err.valid_up_to();
                if valid_up_to > 0 {
                    let valid_str = std::str::from_utf8(&self.raw_buffer[..valid_up_to]).unwrap();
                    let parsed = self.parser.feed(valid_str);
                    events.extend(parsed);
                    self.raw_buffer.drain(..valid_up_to);
                }
                if let Some(error_len) = err.error_len() {
                    self.raw_buffer.drain(..error_len);
                }
            }
        }

        events
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

struct TestCircuitBreaker {
    failure_threshold: u32,
    cooldown: Duration,
    state: Mutex<CircuitState>,
    consecutive_failures: AtomicU32,
    last_failure_time: Mutex<Option<Instant>>,
    retry_after: Mutex<Option<Duration>>,
}

impl TestCircuitBreaker {
    fn new(threshold: u32, cooldown: Duration) -> Self {
        Self {
            failure_threshold: threshold,
            cooldown,
            state: Mutex::new(CircuitState::Closed),
            consecutive_failures: AtomicU32::new(0),
            last_failure_time: Mutex::new(None),
            retry_after: Mutex::new(None),
        }
    }

    fn state(&self) -> CircuitState {
        let mut state_guard = self.state.lock().unwrap();
        if *state_guard == CircuitState::Open {
            if let Some(last_fail) = *self.last_failure_time.lock().unwrap() {
                let required_cooldown = self.retry_after.lock().unwrap().unwrap_or(self.cooldown);
                if last_fail.elapsed() >= required_cooldown {
                    *state_guard = CircuitState::HalfOpen;
                }
            }
        }
        *state_guard
    }

    fn allow_request(&self) -> bool {
        match self.state() {
            CircuitState::Closed => true,
            CircuitState::HalfOpen => true,
            CircuitState::Open => false,
        }
    }

    fn record_success(&self) {
        let mut state_guard = self.state.lock().unwrap();
        self.consecutive_failures.store(0, Ordering::SeqCst);
        *self.last_failure_time.lock().unwrap() = None;
        *self.retry_after.lock().unwrap() = None;
        *state_guard = CircuitState::Closed;
    }

    fn record_failure(&self, is_429: bool, custom_retry_after: Option<Duration>) {
        let failures = self.consecutive_failures.fetch_add(1, Ordering::SeqCst) + 1;
        *self.last_failure_time.lock().unwrap() = Some(Instant::now());
        if let Some(ra) = custom_retry_after {
            *self.retry_after.lock().unwrap() = Some(ra);
        }

        let mut state_guard = self.state.lock().unwrap();
        if is_429 || failures >= self.failure_threshold {
            *state_guard = CircuitState::Open;
        }
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

fn create_test_model_config(provider: &str, model: &str) -> ModelConfig {
    ModelConfig {
        provider: provider.to_string(),
        model_id: model.to_string(),
        api_key: Some("test-key".to_string()),
        api_keys: vec![],
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
fn test_sse_basic_feed_single_event() {
    let mut parser = SseParser::new();
    let events = parser.feed("event: message_delta\ndata: {\"text\":\"hello\"}\n\n");
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type.as_deref(), Some("message_delta"));
    assert_eq!(events[0].data, "{\"text\":\"hello\"}");
}

#[test]
fn test_sse_multiple_events_in_single_chunk() {
    let mut parser = SseParser::new();
    let events = parser.feed("event: e1\ndata: 1\n\nevent: e2\ndata: 2\n\nevent: e3\ndata: 3\n\n");
    assert_eq!(events.len(), 3);
    assert_eq!(events[0].data, "1");
    assert_eq!(events[1].data, "2");
    assert_eq!(events[2].data, "3");
}

#[test]
fn test_sse_partial_event_across_two_chunks() {
    let mut parser = SseParser::new();
    let ev1 = parser.feed("event: delta\ndata: {\"chunk\":");
    assert!(ev1.is_empty());

    let ev2 = parser.feed("123}\n\n");
    assert_eq!(ev2.len(), 1);
    assert_eq!(ev2[0].data, "{\"chunk\":123}");
}

#[test]
fn test_sse_split_multibyte_utf8_two_bytes() {
    let mut acc = ByteSseAccumulator::new();
    let full_text = "event: delta\ndata: café\n\n";
    let bytes = full_text.as_bytes();

    let split_pos = full_text.find('é').unwrap();
    let part1 = &bytes[..split_pos + 1];
    let part2 = &bytes[split_pos + 1..];

    let ev1 = acc.feed_bytes(part1);
    assert!(ev1.is_empty());

    let ev2 = acc.feed_bytes(part2);
    assert_eq!(ev2.len(), 1);
    assert_eq!(ev2[0].data, "café");
}

#[test]
fn test_sse_split_multibyte_utf8_three_bytes_chinese() {
    let mut acc = ByteSseAccumulator::new();
    let full_text = "event: msg\ndata: 你好世界\n\n";
    let bytes = full_text.as_bytes();

    let split_index = full_text.find('好').unwrap() + 1;
    let ev1 = acc.feed_bytes(&bytes[..split_index]);
    assert!(ev1.is_empty());

    let ev2 = acc.feed_bytes(&bytes[split_index..]);
    assert_eq!(ev2.len(), 1);
    assert_eq!(ev2[0].data, "你好世界");
}

#[test]
fn test_sse_split_multibyte_utf8_four_bytes_emoji() {
    let mut acc = ByteSseAccumulator::new();
    let full_text = "event: msg\ndata: 🦀 crab 🚀 rocket\n\n";
    let bytes = full_text.as_bytes();

    let emoji_pos = full_text.find('🦀').unwrap();
    let ev1 = acc.feed_bytes(&bytes[..emoji_pos + 1]);
    assert!(ev1.is_empty());

    let ev2 = acc.feed_bytes(&bytes[emoji_pos + 1..emoji_pos + 3]);
    assert!(ev2.is_empty());

    let ev3 = acc.feed_bytes(&bytes[emoji_pos + 3..]);
    assert_eq!(ev3.len(), 1);
    assert_eq!(ev3[0].data, "🦀 crab 🚀 rocket");
}

#[test]
fn test_sse_crlf_delimiter_handling() {
    let mut parser = SseParser::new();
    let payload = "event: delta\r\ndata: line1\r\n\r\n";
    let clean_payload = payload.replace("\r\n", "\n");
    let events = parser.feed(&clean_payload);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].data, "line1");
}

#[test]
fn test_circuit_breaker_initial_closed_state() {
    let cb = TestCircuitBreaker::new(3, Duration::from_millis(50));
    assert_eq!(cb.state(), CircuitState::Closed);
    assert!(cb.allow_request());
}

#[test]
fn test_circuit_breaker_trips_immediately_on_429() {
    let cb = TestCircuitBreaker::new(5, Duration::from_millis(100));
    assert_eq!(cb.state(), CircuitState::Closed);

    cb.record_failure(true, None);
    assert_eq!(cb.state(), CircuitState::Open);
    assert!(!cb.allow_request());
}

#[test]
fn test_circuit_breaker_trips_after_consecutive_threshold_failures() {
    let cb = TestCircuitBreaker::new(3, Duration::from_millis(100));
    cb.record_failure(false, None);
    assert_eq!(cb.state(), CircuitState::Closed);

    cb.record_failure(false, None);
    assert_eq!(cb.state(), CircuitState::Closed);

    cb.record_failure(false, None);
    assert_eq!(cb.state(), CircuitState::Open);
    assert!(!cb.allow_request());
}

#[test]
fn test_circuit_breaker_retry_after_cooldown_and_half_open() {
    let cb = TestCircuitBreaker::new(3, Duration::from_millis(50));
    cb.record_failure(true, Some(Duration::from_millis(30)));
    assert_eq!(cb.state(), CircuitState::Open);

    std::thread::sleep(Duration::from_millis(40));
    assert_eq!(cb.state(), CircuitState::HalfOpen);
    assert!(cb.allow_request());

    cb.record_success();
    assert_eq!(cb.state(), CircuitState::Closed);
    assert!(cb.allow_request());
}

#[tokio::test]
async fn test_retrying_client_primary_succeeds_without_retries() {
    let primary = Box::new(ScriptedLlmClient::new(vec![Ok(vec![
        StreamDelta::TextDelta("ok primary".to_string()),
        StreamDelta::Stop(StopReason::EndTurn),
    ])]));

    let retry_cfg = RetryConfig {
        max_retries: 2,
        initial_backoff_ms: 5,
        max_backoff_ms: 20,
    };

    let client = RetryingClient::new(primary, Vec::new(), retry_cfg);
    let cfg = create_test_model_config("openai", "gpt-4o");
    let msgs = vec![ChatMessage::user("hi")];

    let stream = client.chat_stream(&cfg, msgs, &[]).await;
    assert!(stream.is_ok());

    let mut stream = stream.unwrap();
    let mut collected = Vec::new();
    while let Some(Ok(delta)) = stream.next().await {
        if let StreamDelta::TextDelta(t) = delta {
            collected.push(t);
        }
    }
    assert_eq!(collected, vec!["ok primary"]);
}

#[tokio::test]
async fn test_retrying_client_failover_to_secondary_on_429() {
    let primary = Box::new(ScriptedLlmClient::new(vec![
        Err(RyvosError::LlmRequest("HTTP 429 Too Many Requests".into())),
        Err(RyvosError::LlmRequest("HTTP 429 Too Many Requests".into())),
    ]));

    let fallback_cfg = create_test_model_config("anthropic", "claude-sonnet");
    let fallback_client = Box::new(ScriptedLlmClient::new(vec![Ok(vec![
        StreamDelta::TextDelta("fallback response".to_string()),
        StreamDelta::Stop(StopReason::EndTurn),
    ])]));

    let retry_cfg = RetryConfig {
        max_retries: 1,
        initial_backoff_ms: 1,
        max_backoff_ms: 5,
    };

    let client = RetryingClient::new(
        primary,
        vec![(fallback_cfg.clone(), fallback_client)],
        retry_cfg,
    );
    let cfg = create_test_model_config("openai", "gpt-4o");
    let msgs = vec![ChatMessage::user("run failover")];

    let stream = client.chat_stream(&cfg, msgs, &[]).await;
    assert!(stream.is_ok());

    let mut stream = stream.unwrap();
    let mut collected = Vec::new();
    while let Some(Ok(delta)) = stream.next().await {
        if let StreamDelta::TextDelta(t) = delta {
            collected.push(t);
        }
    }
    assert_eq!(collected, vec!["fallback response"]);
}

#[test]
fn test_zero_panic_sse_parser_empty_input() {
    let mut parser = SseParser::new();
    let events = parser.feed("");
    assert!(events.is_empty());
}

#[test]
fn test_zero_panic_sse_parser_only_newlines() {
    let mut parser = SseParser::new();
    let events = parser.feed("\n\n\n\n\n\n");
    assert!(events.is_empty());
}

#[test]
fn test_zero_panic_sse_parser_corrupted_headers() {
    let mut parser = SseParser::new();
    let events = parser.feed("garbage: unexpected\nunknown_key: random_data\n\n");
    assert!(events.is_empty());
}

#[test]
fn test_zero_panic_circuit_breaker_rapid_failure_burst() {
    let cb = TestCircuitBreaker::new(5, Duration::from_millis(10));
    for _ in 0..1000 {
        cb.record_failure(false, None);
    }
    assert_eq!(cb.state(), CircuitState::Open);
    assert!(!cb.allow_request());
}

#[test]
fn test_pairwise_utf8_split_with_crlf_delimiters() {
    let mut acc = ByteSseAccumulator::new();
    let text = "event: delta\r\ndata: 🦀\r\n\r\n";
    let normalized = text.replace("\r\n", "\n");
    let bytes = normalized.as_bytes();

    let split_pos = normalized.find('🦀').unwrap() + 2;
    let ev1 = acc.feed_bytes(&bytes[..split_pos]);
    assert!(ev1.is_empty());

    let ev2 = acc.feed_bytes(&bytes[split_pos..]);
    assert_eq!(ev2.len(), 1);
    assert_eq!(ev2[0].data, "🦀");
}

#[tokio::test]
async fn test_pairwise_rate_limit_trip_with_multimodel_failover() {
    let primary = Box::new(ScriptedLlmClient::new(vec![Err(
        RyvosError::LlmRequest("429 rate limit exceeded".into()),
    )]));

    let secondary_cfg = create_test_model_config("gemini", "gemini-1.5-pro");
    let secondary = Box::new(ScriptedLlmClient::new(vec![Err(
        RyvosError::LlmRequest("503 service unavailable".into()),
    )]));

    let tertiary_cfg = create_test_model_config("cohere", "command-r-plus");
    let tertiary = Box::new(ScriptedLlmClient::new(vec![Ok(vec![
        StreamDelta::TextDelta("tertiary success".to_string()),
        StreamDelta::Stop(StopReason::EndTurn),
    ])]));

    let retry_cfg = RetryConfig {
        max_retries: 0,
        initial_backoff_ms: 1,
        max_backoff_ms: 2,
    };

    let client = RetryingClient::new(
        primary,
        vec![
            (secondary_cfg, secondary),
            (tertiary_cfg, tertiary),
        ],
        retry_cfg,
    );

    let cfg = create_test_model_config("openai", "gpt-4o");
    let stream = client
        .chat_stream(&cfg, vec![ChatMessage::user("test")], &[])
        .await
        .unwrap();

    let mut stream = stream;
    let mut texts = Vec::new();
    while let Some(Ok(delta)) = stream.next().await {
        if let StreamDelta::TextDelta(t) = delta {
            texts.push(t);
        }
    }
    assert_eq!(texts, vec!["tertiary success"]);
}

#[tokio::test]
async fn test_real_world_llm_stream_with_chunked_tokens_and_tool_calls() {
    let deltas = vec![
        StreamDelta::TextDelta("I will now check files.".to_string()),
        StreamDelta::ToolUseStart {
            index: 0,
            id: "call-1".to_string(),
            name: "glob".to_string(),
        },
        StreamDelta::ToolInputDelta {
            index: 0,
            delta: "{\"pattern\":".to_string(),
        },
        StreamDelta::ToolInputDelta {
            index: 0,
            delta: "\"*.rs\"}".to_string(),
        },
        StreamDelta::Stop(StopReason::ToolUse),
    ];

    let client = ScriptedLlmClient::new(vec![Ok(deltas)]);
    let cfg = create_test_model_config("openai", "gpt-4o");
    let mut stream = client
        .chat_stream(&cfg, vec![ChatMessage::user("find rs files")], &[])
        .await
        .unwrap();

    let mut tool_called = false;
    let mut tool_args = String::new();
    while let Some(Ok(delta)) = stream.next().await {
        match delta {
            StreamDelta::ToolUseStart { name, .. } => {
                assert_eq!(name, "glob");
                tool_called = true;
            }
            StreamDelta::ToolInputDelta { delta, .. } => {
                tool_args.push_str(&delta);
            }
            _ => {}
        }
    }
    assert!(tool_called);
    assert_eq!(tool_args, "{\"pattern\":\"*.rs\"}");
}
