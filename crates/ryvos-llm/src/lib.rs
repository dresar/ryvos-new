//! Multi-provider LLM client library for Ryvos.
//!
//! Abstracts 17+ LLM providers behind a single [`LlmClient`] trait with
//! streaming support. Each provider translates between the Ryvos message
//! format and the provider's native wire format.
//!
//! **Providers:**
//! - Anthropic, OpenAI, Google Gemini, Azure OpenAI, Cohere, AWS Bedrock (stub)
//! - Claude Code CLI and GitHub Copilot CLI (subprocess-based, subscription billing)
//! - 10 OpenAI-compatible presets: Ollama, Groq, OpenRouter, Together, Fireworks,
//!   Cerebras, xAI, Mistral, Perplexity, DeepSeek
//!
//! **Key components:**
//! - [`create_client`] / [`create_client_with_security`]: Factory functions
//! - [`RetryingClient`]: Wraps any client with exponential backoff and model fallback
//! - [`streaming::SseParser`]: Server-Sent Events parser for HTTP streaming

pub mod providers;
pub mod retry;
pub mod streaming;

use futures::future::BoxFuture;
use futures::stream::BoxStream;
use ryvos_core::config::ModelConfig;
use ryvos_core::error::Result;
use ryvos_core::traits::LlmClient;
use ryvos_core::types::{ChatMessage, StreamDelta, ToolDefinition};

pub use providers::anthropic::AnthropicClient;
pub use providers::azure::AzureClient;
pub use providers::bedrock::BedrockClient;
pub use providers::claude_code::ClaudeCodeClient;
pub use providers::cohere::CohereClient;
pub use providers::copilot::CopilotClient;
pub use providers::gemini::GeminiClient;
pub use providers::openai::OpenAiClient;
pub use retry::RetryingClient;

pub struct DynamicClient {
    anthropic: AnthropicClient,
    gemini: GeminiClient,
    openai: OpenAiClient,
    azure: AzureClient,
    cohere: CohereClient,
    claude_code: ClaudeCodeClient,
    copilot: CopilotClient,
}

impl DynamicClient {
    pub fn new(dangerous_patterns: &[ryvos_core::security::DangerousPattern]) -> Self {
        Self {
            anthropic: AnthropicClient::new(),
            gemini: GeminiClient::new(),
            openai: OpenAiClient::new(),
            azure: AzureClient::new(),
            cohere: CohereClient::new(),
            claude_code: ClaudeCodeClient::with_patterns(dangerous_patterns),
            copilot: CopilotClient::with_patterns(dangerous_patterns),
        }
    }
}

impl Default for DynamicClient {
    fn default() -> Self {
        Self::new(&[])
    }
}

impl LlmClient for DynamicClient {
    fn chat_stream(
        &self,
        config: &ModelConfig,
        messages: Vec<ChatMessage>,
        tools: &[ToolDefinition],
    ) -> BoxFuture<'_, Result<BoxStream<'_, Result<StreamDelta>>>> {
        let mut cfg = config.clone();
        apply_preset_defaults(&mut cfg);
        let tools = tools.to_vec();
        Box::pin(async move {
            match cfg.provider.as_str() {
                "anthropic" | "claude" => self.anthropic.chat_stream(&cfg, messages, &tools).await,
                "gemini" | "google" => self.gemini.chat_stream(&cfg, messages, &tools).await,
                "azure" | "azure-openai" => self.azure.chat_stream(&cfg, messages, &tools).await,
                "cohere" => self.cohere.chat_stream(&cfg, messages, &tools).await,
                "claude-code" | "claude-cli" | "claude-sub" => {
                    self.claude_code.chat_stream(&cfg, messages, &tools).await
                }
                "copilot" | "github-copilot" | "copilot-cli" => {
                    self.copilot.chat_stream(&cfg, messages, &tools).await
                }
                _ => self.openai.chat_stream(&cfg, messages, &tools).await,
            }
        })
    }
}

pub fn create_dynamic_client_with_security(
    dangerous_patterns: &[ryvos_core::security::DangerousPattern],
) -> Box<dyn LlmClient> {
    Box::new(DynamicClient::new(dangerous_patterns))
}

/// Create an LLM client based on the provider name.
///
/// Supports 17 providers:
/// - `anthropic` / `claude` — Anthropic Messages API
/// - `gemini` — Google Gemini native API
/// - `azure` — Azure OpenAI (api-key header, deployment URL)
/// - `bedrock` — AWS Bedrock (stub, v0.3.0)
/// - `cohere` — Cohere v2 Chat API
/// - `claude-code` — Claude Code CLI subprocess
/// - `copilot` / `github-copilot` — GitHub Copilot CLI subprocess
/// - `openai` — OpenAI (default fallback)
/// - 10 preset providers (OpenAI-compatible): ollama, groq, openrouter,
///   together, fireworks, cerebras, xai, mistral, perplexity, deepseek
pub fn create_client(config: &ModelConfig) -> Box<dyn LlmClient> {
    match config.provider.as_str() {
        "anthropic" | "claude" => Box::new(AnthropicClient::new()),
        "gemini" | "google" => Box::new(GeminiClient::new()),
        "azure" | "azure-openai" => Box::new(AzureClient::new()),
        "bedrock" | "aws-bedrock" | "aws" => Box::new(BedrockClient::new()),
        "cohere" => Box::new(CohereClient::new()),
        "claude-code" | "claude-cli" | "claude-sub" => Box::new(ClaudeCodeClient::new()),
        "copilot" | "github-copilot" | "copilot-cli" => Box::new(CopilotClient::new()),
        // NOTE: For security pattern matching, use create_client_with_security() instead.
        // Everything else uses the OpenAI-compatible client.
        // For known presets, apply default base_url and extra headers
        // via the config's extra_headers and base_url fields (set during
        // config loading or init).
        _ => Box::new(OpenAiClient::new()),
    }
}

/// Create an LLM client with security pattern matching for CLI-based providers.
pub fn create_client_with_security(
    config: &ModelConfig,
    dangerous_patterns: &[ryvos_core::security::DangerousPattern],
) -> Box<dyn LlmClient> {
    match config.provider.as_str() {
        "claude-code" | "claude-cli" | "claude-sub" => {
            Box::new(ClaudeCodeClient::with_patterns(dangerous_patterns))
        }
        "copilot" | "github-copilot" | "copilot-cli" => {
            Box::new(CopilotClient::with_patterns(dangerous_patterns))
        }
        _ => create_client(config),
    }
}

/// Resolve preset defaults into a ModelConfig, filling in base_url and
/// extra_headers if not already set by the user.
pub fn apply_preset_defaults(config: &mut ModelConfig) {
    if let Some(preset) = providers::presets::get_preset(&config.provider) {
        // Fill base_url if not set
        if config.base_url.is_none() {
            config.base_url = Some(preset.default_base_url.to_string());
        }

        // Merge preset headers (user headers take precedence)
        let merged = providers::presets::build_extra_headers(&preset, &config.extra_headers);
        config.extra_headers = merged.into_iter().collect();
    }
}
