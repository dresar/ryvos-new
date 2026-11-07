use std::sync::Mutex;
use std::time::{Duration, Instant};

use futures::future::BoxFuture;
use futures::stream::BoxStream;
use reqwest::header::HeaderMap;
use tracing::{info, warn};

use ryvos_core::config::{ModelConfig, RetryConfig};
use ryvos_core::error::{Result, RyvosError};
use ryvos_core::traits::LlmClient;
use ryvos_core::types::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

struct CircuitBreakerInner {
    state: CircuitState,
    failure_timestamps: Vec<Instant>,
    last_failure_time: Option<Instant>,
    retry_after: Option<Duration>,
}

pub struct CircuitBreaker {
    failure_threshold: u32,
    window_duration: Duration,
    default_cooldown: Duration,
    inner: Mutex<CircuitBreakerInner>,
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self::new(3, Duration::from_secs(30))
    }
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u32, cooldown: Duration) -> Self {
        Self::with_window(failure_threshold, cooldown, Duration::from_secs(60))
    }

    pub fn with_window(failure_threshold: u32, cooldown: Duration, window: Duration) -> Self {
        Self {
            failure_threshold,
            window_duration: window,
            default_cooldown: cooldown,
            inner: Mutex::new(CircuitBreakerInner {
                state: CircuitState::Closed,
                failure_timestamps: Vec::new(),
                last_failure_time: None,
                retry_after: None,
            }),
        }
    }

    pub fn state(&self) -> CircuitState {
        let mut inner = self.inner.lock().unwrap();
        if inner.state == CircuitState::Open {
            if let Some(last_fail) = inner.last_failure_time {
                let required_cooldown = inner.retry_after.unwrap_or(self.default_cooldown);
                if last_fail.elapsed() >= required_cooldown {
                    inner.state = CircuitState::HalfOpen;
                }
            }
        }
        inner.state
    }

    pub fn allow_request(&self) -> bool {
        match self.state() {
            CircuitState::Closed => true,
            CircuitState::HalfOpen => true,
            CircuitState::Open => false,
        }
    }

    pub fn record_success(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.failure_timestamps.clear();
        inner.last_failure_time = None;
        inner.retry_after = None;
        inner.state = CircuitState::Closed;
    }

    pub fn record_failure(&self, is_429: bool, custom_retry_after: Option<Duration>) {
        let now = Instant::now();
        let mut inner = self.inner.lock().unwrap();
        inner.last_failure_time = Some(now);
        if let Some(ra) = custom_retry_after {
            inner.retry_after = Some(ra);
        }

        let window = self.window_duration;
        inner.failure_timestamps.retain(|t| now.duration_since(*t) <= window);
        inner.failure_timestamps.push(now);
        let failure_count = inner.failure_timestamps.len() as u32;

        if is_429 || failure_count >= self.failure_threshold {
            inner.state = CircuitState::Open;
        }
    }

    pub fn failure_count(&self) -> usize {
        let now = Instant::now();
        let mut inner = self.inner.lock().unwrap();
        let window = self.window_duration;
        inner.failure_timestamps.retain(|t| now.duration_since(*t) <= window);
        inner.failure_timestamps.len()
    }

    pub fn parse_retry_after_headers(headers: &HeaderMap) -> Option<Duration> {
        if let Some(val) = headers.get("retry-after") {
            if let Ok(val_str) = val.to_str() {
                if let Some(d) = parse_duration_value(val_str) {
                    return Some(d);
                }
                if let Ok(date) = chrono::DateTime::parse_from_rfc2822(val_str) {
                    let now = chrono::Utc::now();
                    let date_utc = date.with_timezone(&chrono::Utc);
                    if date_utc > now {
                        if let Ok(d) = (date_utc - now).to_std() {
                            return Some(d);
                        }
                    }
                }
            }
        }
        if let Some(val) = headers.get("x-ratelimit-reset") {
            if let Ok(val_str) = val.to_str() {
                if let Some(d) = parse_duration_value(val_str) {
                    return Some(d);
                }
            }
        }
        if let Some(val) = headers.get("x-ratelimit-reset-requests").or_else(|| headers.get("x-ratelimit-reset-tokens")) {
            if let Ok(val_str) = val.to_str() {
                if let Some(d) = parse_duration_value(val_str) {
                    return Some(d);
                }
            }
        }
        None
    }

    pub fn parse_retry_after_str(msg: &str) -> Option<Duration> {
        let lower = msg.to_lowercase();
        let marker = lower
            .find("retry-after")
            .or_else(|| lower.find("retry after"))
            .or_else(|| lower.find("retry_after"))
            .map(|idx| idx + 11);

        if let Some(start) = marker {
            let slice = &lower[start..];
            let trimmed = slice.trim_start_matches(|c: char| c == ':' || c == '=' || c.is_whitespace());
            return parse_duration_value(trimmed);
        }
        None
    }
}

fn parse_duration_value(s: &str) -> Option<Duration> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }

    let end_of_num = s.find(|c: char| !c.is_ascii_digit() && c != '.').unwrap_or(s.len());
    let num_part = &s[..end_of_num];
    let unit_part = s[end_of_num..].trim().to_lowercase();

    if let Ok(secs_f) = num_part.parse::<f64>() {
        if unit_part.starts_with("ms") {
            return Some(Duration::from_millis(secs_f as u64));
        }
        if secs_f > 1_000_000_000.0 {
            if let Ok(now_unix) = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
                let target = secs_f as u64;
                let current = now_unix.as_secs();
                if target > current {
                    return Some(Duration::from_secs(target - current));
                }
            }
        }
        return Some(Duration::from_secs_f64(secs_f.max(0.0)));
    }
    None
}

pub struct KeyRotator {
    counter: std::sync::atomic::AtomicUsize,
    cooldowns: Mutex<std::collections::HashMap<String, Instant>>,
}

impl Default for KeyRotator {
    fn default() -> Self {
        Self {
            counter: std::sync::atomic::AtomicUsize::new(0),
            cooldowns: Mutex::new(std::collections::HashMap::new()),
        }
    }
}

pub struct RetryingClient {
    primary: Box<dyn LlmClient>,
    fallbacks: Vec<(ModelConfig, Box<dyn LlmClient>)>,
    retry_config: RetryConfig,
    circuit_breaker: CircuitBreaker,
    key_rotator: KeyRotator,
}

impl RetryingClient {
    pub fn new(
        primary: Box<dyn LlmClient>,
        fallbacks: Vec<(ModelConfig, Box<dyn LlmClient>)>,
        retry_config: RetryConfig,
    ) -> Self {
        Self {
            primary,
            fallbacks,
            retry_config,
            circuit_breaker: CircuitBreaker::new(3, Duration::from_secs(30)),
            key_rotator: KeyRotator::default(),
        }
    }

    pub fn with_circuit_breaker(mut self, circuit_breaker: CircuitBreaker) -> Self {
        self.circuit_breaker = circuit_breaker;
        self
    }

    pub fn circuit_breaker(&self) -> &CircuitBreaker {
        &self.circuit_breaker
    }

    async fn execute_fallbacks<'a>(
        &'a self,
        messages: Vec<ChatMessage>,
        tools: Vec<ToolDefinition>,
    ) -> Result<BoxStream<'a, Result<StreamDelta>>> {
        let mut last_fb_err = None;
        for (fb_config, fb_client) in &self.fallbacks {
            match fb_client
                .chat_stream(fb_config, messages.clone(), &tools)
                .await
            {
                Ok(stream) => {
                    info!(
                        model = %fb_config.model_id,
                        provider = %fb_config.provider,
                        "Fell back to alternative model"
                    );
                    return Ok(stream);
                }
                Err(e) => {
                    warn!(
                        model = %fb_config.model_id,
                        error = %e,
                        "Fallback model failed"
                    );
                    last_fb_err = Some(e);
                    continue;
                }
            }
        }
        Err(last_fb_err.unwrap_or_else(|| {
            RyvosError::LlmRequest("Primary exhausted and all fallback models failed".into())
        }))
    }
}

fn is_retryable(e: &RyvosError) -> bool {
    match e {
        RyvosError::LlmRequest(msg) => {
            let lower = msg.to_lowercase();
            lower.contains("429")
                || lower.contains("500")
                || lower.contains("502")
                || lower.contains("503")
                || lower.contains("timeout")
                || lower.contains("connection")
        }
        RyvosError::LlmStream(_) => true,
        _ => false,
    }
}

fn calculate_backoff(attempt: u32, config: &RetryConfig) -> Duration {
    let ms = (config.initial_backoff_ms.saturating_mul(2u64.pow(attempt.min(10)))).min(config.max_backoff_ms);
    let jitter = 0.8 + rand::random::<f64>() * 0.4;
    Duration::from_millis(((ms as f64) * jitter) as u64)
}

fn is_quota_or_auth_error(e: &RyvosError) -> bool {
    match e {
        RyvosError::LlmRequest(msg) => {
            let lower = msg.to_lowercase();
            lower.contains("429")
                || lower.contains("rate limit")
                || lower.contains("resource_exhausted")
                || lower.contains("quota")
                || lower.contains("credit")
                || lower.contains("401")
                || lower.contains("unauthorized")
                || lower.contains("403")
                || lower.contains("forbidden")
                || lower.contains("api_key_invalid")
                || lower.contains("api key not valid")
                || lower.contains("invalid api key")
        }
        _ => false,
    }
}

impl LlmClient for RetryingClient {
    fn chat_stream(
        &self,
        config: &ModelConfig,
        messages: Vec<ChatMessage>,
        tools: &[ToolDefinition],
    ) -> BoxFuture<'_, Result<BoxStream<'_, Result<StreamDelta>>>> {
        let config = config.clone();
        let messages = messages.clone();
        let tools = tools.to_vec();

        Box::pin(async move {
            let max_retries = self.retry_config.max_retries;

            if !self.circuit_breaker.allow_request() {
                warn!("Circuit breaker is Open; fast-failing over immediately");
                return self.execute_fallbacks(messages, tools).await;
            }

            let key_pool = config.get_api_key_pool();
            let num_keys = key_pool.len();

            if num_keys > 1 {
                let start_idx = self
                    .key_rotator
                    .counter
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
                    % num_keys;

                let now = Instant::now();
                let mut candidate_indices: Vec<usize> =
                    (0..num_keys).map(|i| (start_idx + i) % num_keys).collect();

                {
                    let cds = self.key_rotator.cooldowns.lock().unwrap();
                    candidate_indices.sort_by_key(|&idx| {
                        let k = &key_pool[idx];
                        if let Some(exp) = cds.get(k) {
                            if *exp > now {
                                return 1;
                            }
                        }
                        0
                    });
                }

                let mut last_key_err = None;
                for &idx in &candidate_indices {
                    let candidate_key = &key_pool[idx];
                    let mut active_config = config.clone();
                    active_config.api_key = Some(candidate_key.clone());

                    match self
                        .primary
                        .chat_stream(&active_config, messages.clone(), &tools)
                        .await
                    {
                        Ok(stream) => {
                            self.circuit_breaker.record_success();
                            let mut cds = self.key_rotator.cooldowns.lock().unwrap();
                            cds.remove(candidate_key);
                            return Ok(stream);
                        }
                        Err(e) => {
                            let is_quota_or_auth = is_quota_or_auth_error(&e);
                            if is_quota_or_auth {
                                let mut cds = self.key_rotator.cooldowns.lock().unwrap();
                                let retry_after = CircuitBreaker::parse_retry_after_str(&e.to_string())
                                    .unwrap_or(Duration::from_secs(60));
                                cds.insert(candidate_key.clone(), Instant::now() + retry_after);
                                warn!(
                                    key_index = idx,
                                    total_keys = num_keys,
                                    error = %e,
                                    "API key hit quota or rate limit, rotating to next key in pool"
                                );
                                last_key_err = Some(e);
                                continue;
                            }
                            last_key_err = Some(e);
                            break;
                        }
                    }
                }

                if let Some(e) = last_key_err {
                    if !self.fallbacks.is_empty() {
                        warn!(error = %e, "All keys in pool failed, falling back to alternative models");
                        return self.execute_fallbacks(messages, tools).await;
                    }
                    return Err(e);
                }
            }

            let mut last_err = None;
            for attempt in 0..=max_retries {
                match self
                    .primary
                    .chat_stream(&config, messages.clone(), &tools)
                    .await
                {
                    Ok(stream) => {
                        self.circuit_breaker.record_success();
                        return Ok(stream);
                    }
                    Err(e) => {
                        let is_429 = match &e {
                            RyvosError::LlmRequest(msg) => {
                                let lower = msg.to_lowercase();
                                lower.contains("429") || lower.contains("rate limit")
                            }
                            _ => false,
                        };
                        let retry_after = match &e {
                            RyvosError::LlmRequest(msg) => CircuitBreaker::parse_retry_after_str(msg),
                            _ => None,
                        };

                        self.circuit_breaker.record_failure(is_429, retry_after);

                        if is_429 {
                            warn!(error = %e, "Rate limit hit; fast-failing over immediately");
                            if !self.fallbacks.is_empty() {
                                return self.execute_fallbacks(messages, tools).await;
                            }
                            return Err(e);
                        }

                        if is_retryable(&e) && attempt < max_retries && self.circuit_breaker.allow_request() {
                            let backoff = calculate_backoff(attempt, &self.retry_config);
                            warn!(
                                attempt = attempt + 1,
                                max_retries,
                                backoff_ms = backoff.as_millis() as u64,
                                error = %e,
                                "Retrying LLM request"
                            );
                            tokio::time::sleep(backoff).await;
                            last_err = Some(e);
                            continue;
                        }

                        last_err = Some(e);
                        break;
                    }
                }
            }

            if !self.fallbacks.is_empty() {
                info!("Primary LLM exhausted, trying fallback models");
                return self.execute_fallbacks(messages, tools).await;
            }

            Err(last_err.unwrap_or_else(|| RyvosError::LlmRequest("All providers failed".into())))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_breaker_transitions() {
        let cb = CircuitBreaker::new(3, Duration::from_millis(30));
        assert_eq!(cb.state(), CircuitState::Closed);
        assert!(cb.allow_request());

        cb.record_failure(false, None);
        cb.record_failure(false, None);
        assert_eq!(cb.state(), CircuitState::Closed);

        cb.record_failure(false, None);
        assert_eq!(cb.state(), CircuitState::Open);
        assert!(!cb.allow_request());

        std::thread::sleep(Duration::from_millis(35));
        assert_eq!(cb.state(), CircuitState::HalfOpen);
        assert!(cb.allow_request());

        cb.record_success();
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[test]
    fn test_circuit_breaker_immediate_429_trip() {
        let cb = CircuitBreaker::new(5, Duration::from_millis(50));
        cb.record_failure(true, Some(Duration::from_millis(20)));
        assert_eq!(cb.state(), CircuitState::Open);
        assert!(!cb.allow_request());
    }

    #[test]
    fn test_parse_retry_after_str() {
        let s1 = "HTTP 429 Too Many Requests (retry-after: 5s)";
        assert_eq!(CircuitBreaker::parse_retry_after_str(s1), Some(Duration::from_secs(5)));

        let s2 = "Rate limited: retry after 15";
        assert_eq!(CircuitBreaker::parse_retry_after_str(s2), Some(Duration::from_secs(15)));

        let s3 = "Retry_after=500ms";
        assert_eq!(CircuitBreaker::parse_retry_after_str(s3), Some(Duration::from_millis(500)));
    }

    #[test]
    fn test_is_quota_or_auth_error() {
        let e1 = RyvosError::LlmRequest("HTTP 429: Resource has been exhausted (e.g. check quota)".into());
        assert!(is_quota_or_auth_error(&e1));

        let e2 = RyvosError::LlmRequest("HTTP 403: Forbidden quota exceeded".into());
        assert!(is_quota_or_auth_error(&e2));

        let e3 = RyvosError::LlmRequest("HTTP 401: Unauthorized invalid API key".into());
        assert!(is_quota_or_auth_error(&e3));

        let e_gemini_auth = RyvosError::LlmRequest("HTTP 400 Bad Request: API key not valid. Reason: API_KEY_INVALID".into());
        assert!(is_quota_or_auth_error(&e_gemini_auth));

        let e4 = RyvosError::LlmRequest("HTTP 500: Internal Server Error".into());
        assert!(!is_quota_or_auth_error(&e4));
    }

    #[test]
    fn test_key_pool_parsing() {
        let mut config = ModelConfig::default();
        config.api_key = Some("key1, key2\nkey3; key4".into());
        config.api_keys = vec!["key5".into(), "key2".into()];
        let pool = config.get_api_key_pool();
        assert_eq!(pool.len(), 5);
        assert_eq!(pool[0], "key1");
        assert_eq!(pool[1], "key2");
        assert_eq!(pool[2], "key3");
        assert_eq!(pool[3], "key4");
        assert_eq!(pool[4], "key5");
    }
}
