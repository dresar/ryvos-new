use futures::future::BoxFuture;
use futures::stream::{BoxStream, StreamExt};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::warn;

use ryvos_core::config::ModelConfig;
use ryvos_core::error::{Result, RyvosError};
use ryvos_core::traits::LlmClient;
use ryvos_core::types::*;

use crate::streaming::{SseEvent, SseStream};

pub struct GeminiClient {
    http: Client,
}

impl GeminiClient {
    pub fn new() -> Self {
        Self {
            http: Client::new(),
        }
    }
}

impl Default for GeminiClient {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Serialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system_instruction: Option<GeminiContent>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tools: Vec<GeminiToolDecl>,
    #[serde(skip_serializing_if = "Option::is_none")]
    generation_config: Option<GenerationConfig>,
}

#[derive(Serialize, Deserialize, Debug)]
struct GeminiContent {
    #[serde(skip_serializing_if = "Option::is_none")]
    role: Option<String>,
    parts: Vec<GeminiPart>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
enum GeminiPart {
    Text {
        text: String,
    },
    FunctionCall {
        #[serde(rename = "functionCall")]
        function_call: GeminiFnCall,
    },
    FunctionResponse {
        #[serde(rename = "functionResponse")]
        function_response: GeminiFnResp,
    },
}

#[derive(Serialize, Deserialize, Debug)]
struct GeminiFnCall {
    name: String,
    args: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug)]
struct GeminiFnResp {
    name: String,
    response: serde_json::Value,
}

#[derive(Serialize)]
struct GeminiToolDecl {
    function_declarations: Vec<GeminiFnDecl>,
}

#[derive(Serialize)]
struct GeminiFnDecl {
    name: String,
    description: String,
    parameters: serde_json::Value,
}

#[derive(Serialize)]
struct GenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    max_output_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
}

#[derive(Deserialize, Debug)]
struct GeminiStreamChunk {
    #[serde(default)]
    candidates: Vec<GeminiCandidate>,
    #[serde(default, rename = "usageMetadata")]
    usage_metadata: Option<GeminiUsage>,
}

#[derive(Deserialize, Debug)]
struct GeminiCandidate {
    content: Option<GeminiContent>,
    #[serde(default, rename = "finishReason")]
    finish_reason: Option<String>,
}

#[derive(Deserialize, Debug)]
struct GeminiUsage {
    #[serde(default, rename = "promptTokenCount")]
    prompt_token_count: u64,
    #[serde(default, rename = "candidatesTokenCount")]
    candidates_token_count: u64,
}

fn convert_messages(messages: Vec<ChatMessage>) -> (Option<GeminiContent>, Vec<GeminiContent>) {
    let mut system = None;
    let mut contents = Vec::new();

    for msg in messages {
        match msg.role {
            Role::System => {
                system = Some(GeminiContent {
                    role: None,
                    parts: vec![GeminiPart::Text { text: msg.text() }],
                });
            }
            Role::User => {
                let mut parts = Vec::new();
                for block in &msg.content {
                    match block {
                        ContentBlock::Text { text } => {
                            parts.push(GeminiPart::Text { text: text.clone() });
                        }
                        ContentBlock::ToolResult {
                            tool_use_id,
                            content,
                            ..
                        } => {
                            parts.push(GeminiPart::FunctionResponse {
                                function_response: GeminiFnResp {
                                    name: tool_use_id.clone(),
                                    response: serde_json::json!({ "result": content }),
                                },
                            });
                        }
                        _ => {}
                    }
                }
                if !parts.is_empty() {
                    contents.push(GeminiContent {
                        role: Some("user".to_string()),
                        parts,
                    });
                }
            }
            Role::Assistant => {
                let mut parts = Vec::new();
                for block in &msg.content {
                    match block {
                        ContentBlock::Text { text } => {
                            if !text.is_empty() {
                                parts.push(GeminiPart::Text { text: text.clone() });
                            }
                        }
                        ContentBlock::ToolUse { name, input, .. } => {
                            parts.push(GeminiPart::FunctionCall {
                                function_call: GeminiFnCall {
                                    name: name.clone(),
                                    args: input.clone(),
                                },
                            });
                        }
                        _ => {}
                    }
                }
                if !parts.is_empty() {
                    contents.push(GeminiContent {
                        role: Some("model".to_string()),
                        parts,
                    });
                }
            }
            Role::Tool => {
                let mut parts = Vec::new();
                for block in &msg.content {
                    if let ContentBlock::ToolResult {
                        tool_use_id,
                        content,
                        ..
                    } = block
                    {
                        parts.push(GeminiPart::FunctionResponse {
                            function_response: GeminiFnResp {
                                name: tool_use_id.clone(),
                                response: serde_json::json!({ "result": content }),
                            },
                        });
                    }
                }
                if !parts.is_empty() {
                    contents.push(GeminiContent {
                        role: Some("user".to_string()),
                        parts,
                    });
                }
            }
        }
    }

    (system, contents)
}

fn parse_gemini_chunk(event: SseEvent) -> Vec<Result<StreamDelta>> {
    if event.data.trim() == "[DONE]" {
        return Vec::new();
    }

    let parsed: std::result::Result<GeminiStreamChunk, _> = serde_json::from_str(&event.data);
    match parsed {
        Ok(chunk) => {
            let mut deltas = Vec::new();

            if let Some(usage) = chunk.usage_metadata {
                deltas.push(Ok(StreamDelta::Usage {
                    input_tokens: usage.prompt_token_count,
                    output_tokens: usage.candidates_token_count,
                }));
            }

            if let Some(candidate) = chunk.candidates.into_iter().next() {
                if let Some(reason) = candidate.finish_reason {
                    let stop = match reason.as_str() {
                        "STOP" => StopReason::EndTurn,
                        "MAX_TOKENS" => StopReason::MaxTokens,
                        _ => StopReason::EndTurn,
                    };
                    deltas.push(Ok(StreamDelta::Stop(stop)));
                }

                if let Some(content) = candidate.content {
                    for (i, part) in content.parts.into_iter().enumerate() {
                        match part {
                            GeminiPart::Text { text } => {
                                if !text.is_empty() {
                                    deltas.push(Ok(StreamDelta::TextDelta(text)));
                                }
                            }
                            GeminiPart::FunctionCall { function_call } => {
                                let call_id = format!("call_{}", function_call.name);
                                deltas.push(Ok(StreamDelta::ToolUseStart {
                                    index: i,
                                    id: call_id,
                                    name: function_call.name,
                                }));
                                let args_str = if function_call.args.is_string() {
                                    function_call.args.as_str().unwrap_or("").to_string()
                                } else {
                                    function_call.args.to_string()
                                };
                                if !args_str.is_empty() {
                                    deltas.push(Ok(StreamDelta::ToolInputDelta {
                                        index: i,
                                        delta: args_str,
                                    }));
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }

            deltas
        }
        Err(e) => {
            warn!(data = %event.data, error = %e, "Failed to parse Gemini SSE chunk");
            Vec::new()
        }
    }
}

impl LlmClient for GeminiClient {
    fn chat_stream(
        &self,
        config: &ModelConfig,
        messages: Vec<ChatMessage>,
        tools: &[ToolDefinition],
    ) -> BoxFuture<'_, Result<BoxStream<'_, Result<StreamDelta>>>> {
        let config = config.clone();
        let tools = tools.to_vec();

        Box::pin(async move {
            let effective_key = config
                .get_effective_api_key()
                .or_else(|| config.api_key.clone())
                .ok_or_else(|| RyvosError::Config("Gemini: api_key is required".into()))?;
            let api_key = effective_key.as_str();

            let url = format!(
                "https://generativelanguage.googleapis.com/v1beta/models/{}:streamGenerateContent?alt=sse&key={}",
                config.model_id, api_key
            );

            let (system_instruction, contents) = convert_messages(messages);

            let gemini_tools = if tools.is_empty() {
                vec![]
            } else {
                vec![GeminiToolDecl {
                    function_declarations: tools
                        .iter()
                        .map(|t| GeminiFnDecl {
                            name: t.name.clone(),
                            description: t.description.clone(),
                            parameters: t.input_schema.clone(),
                        })
                        .collect(),
                }]
            };

            let body = GeminiRequest {
                contents,
                system_instruction,
                tools: gemini_tools,
                generation_config: Some(GenerationConfig {
                    max_output_tokens: Some(config.max_tokens),
                    temperature: if config.temperature > 0.0 {
                        Some(config.temperature)
                    } else {
                        None
                    },
                }),
            };

            let response = self
                .http
                .post(&url)
                .header("content-type", "application/json")
                .json(&body)
                .send()
                .await
                .map_err(|e| RyvosError::LlmRequest(e.to_string()))?;

            if !response.status().is_success() {
                let status = response.status();
                let headers = response.headers().clone();
                let body = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "unknown".to_string());
                let retry_info = if status.as_u16() == 429 {
                    crate::retry::CircuitBreaker::parse_retry_after_headers(&headers)
                        .map(|d| format!(" (retry-after: {}s)", d.as_secs()))
                        .unwrap_or_default()
                } else {
                    String::new()
                };
                return Err(RyvosError::LlmRequest(format!("HTTP {}: {}{}", status, body, retry_info)));
            }

            let byte_stream = response.bytes_stream();
            let sse_stream = SseStream::new(byte_stream);

            let delta_stream = sse_stream
                .map(|event| futures::stream::iter(parse_gemini_chunk(event)))
                .flatten();

            Ok(Box::pin(delta_stream) as BoxStream<'_, Result<StreamDelta>>)
        })
    }
}
