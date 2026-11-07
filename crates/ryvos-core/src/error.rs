use thiserror::Error;

#[derive(Debug, Error)]
pub enum RyvosError {
    #[error("LLM request failed: {0}")]
    LlmRequest(String),

    #[error("LLM streaming error: {0}")]
    LlmStream(String),

    #[error("LLM provider not supported: {0}")]
    UnsupportedProvider(String),

    #[error("LLM response parse error: {0}")]
    LlmParse(String),

    #[error("Tool not found: {0}")]
    ToolNotFound(String),

    #[error("Tool execution failed: {tool}: {message}")]
    ToolExecution { tool: String, message: String },

    #[error("Tool timeout after {timeout_secs}s: {tool}")]
    ToolTimeout { tool: String, timeout_secs: u64 },

    #[error("Tool input validation failed: {0}")]
    ToolValidation(String),

    #[error("Agent exceeded max turns ({0})")]
    MaxTurnsExceeded(usize),

    #[error("Agent exceeded max duration ({0}s)")]
    MaxDurationExceeded(u64),

    #[error("Agent cancelled")]
    Cancelled,

    #[error("Config error: {0}")]
    Config(String),

    #[error("Config file not found: {0}")]
    ConfigNotFound(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Channel error: {channel}: {message}")]
    Channel { channel: String, message: String },

    #[error("Gateway error: {0}")]
    Gateway(String),

    #[error("Tool blocked by security policy: {tool} (tier {tier})")]
    ToolBlocked { tool: String, tier: String },

    #[error("Approval denied for tool {tool}: {reason}")]
    ApprovalDenied { tool: String, reason: String },

    #[error("Approval timeout for tool: {tool}")]
    ApprovalTimeout { tool: String },

    #[error("Security violation: {0}")]
    SecurityViolation(String),

    #[error("Budget exceeded: spent {spent_cents} cents of {budget_cents} cent budget")]
    BudgetExceeded { spent_cents: u64, budget_cents: u64 },

    #[error("MCP error: {0}")]
    Mcp(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

impl RyvosError {
    pub fn is_rate_limited(&self) -> bool {
        match self {
            Self::LlmRequest(msg) => {
                let lower = msg.to_lowercase();
                lower.contains("429") || lower.contains("rate limit")
            }
            _ => false,
        }
    }

    pub fn is_transient(&self) -> bool {
        match self {
            Self::LlmRequest(msg) => {
                let lower = msg.to_lowercase();
                lower.contains("429")
                    || lower.contains("500")
                    || lower.contains("502")
                    || lower.contains("503")
                    || lower.contains("504")
                    || lower.contains("timeout")
                    || lower.contains("connection")
                    || lower.contains("rate limit")
            }
            Self::LlmStream(_) => true,
            _ => false,
        }
    }
}

pub type Result<T> = std::result::Result<T, RyvosError>;
