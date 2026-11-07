use std::path::PathBuf;
use std::time::Duration;

use futures::future::BoxFuture;
use serde::Deserialize;

use ryvos_core::error::{Result, RyvosError};
use ryvos_core::security::SecurityTier;
use ryvos_core::traits::Tool;
use ryvos_core::types::{ToolContext, ToolResult};

fn resolve(p: &str, wd: &std::path::Path) -> PathBuf {
    let path = PathBuf::from(p);
    if path.is_absolute() {
        path
    } else {
        wd.join(path)
    }
}

pub struct HttpRequestTool;

#[derive(Deserialize)]
struct HttpRequestInput {
    url: String,
    #[serde(default = "default_method")]
    method: String,
    #[serde(default)]
    headers: std::collections::HashMap<String, String>,
    #[serde(default)]
    body: Option<String>,
}
fn default_method() -> String {
    "GET".into()
}

impl Tool for HttpRequestTool {
    fn name(&self) -> &str {
        "http_request"
    }
    fn tier(&self) -> SecurityTier {
        SecurityTier::T1
    }
    fn timeout_secs(&self) -> u64 {
        60
    }
    fn description(&self) -> &str {
        "Make an HTTP request. Returns status, headers, and body."
    }
    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "url": { "type": "string" },
                "method": { "type": "string", "description": "HTTP method (default: GET)" },
                "headers": { "type": "object", "description": "Request headers" },
                "body": { "type": "string", "description": "Request body" }
            },
            "required": ["url"]
        })
    }
    fn execute(
        &self,
        input: serde_json::Value,
        _ctx: ToolContext,
    ) -> BoxFuture<'_, Result<ToolResult>> {
        Box::pin(async move {
            let p: HttpRequestInput = serde_json::from_value(input)
                .map_err(|e| RyvosError::ToolValidation(e.to_string()))?;

            if let Err(e) = crate::egress::validate_url_for_egress(&p.url).await {
                return Ok(ToolResult::error(format!("Blocked by egress policy: {}", e)));
            }

            let client = reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .redirect(reqwest::redirect::Policy::none())
                .build()
                .map_err(|e| RyvosError::ToolExecution {
                    tool: "http_request".into(),
                    message: e.to_string(),
                })?;

            let method = p
                .method
                .to_uppercase()
                .parse::<reqwest::Method>()
                .map_err(|e| RyvosError::ToolValidation(format!("Invalid method: {}", e)))?;

            let mut req = client.request(method, &p.url);
            for (k, v) in &p.headers {
                req = req.header(k.as_str(), v.as_str());
            }
            if let Some(body) = p.body {
                req = req.body(body);
            }

            let resp = req.send().await.map_err(|e| RyvosError::ToolExecution {
                tool: "http_request".into(),
                message: e.to_string(),
            })?;
            let status = resp.status();
            let headers = resp
                .headers()
                .iter()
                .map(|(k, v)| format!("{}: {}", k, v.to_str().unwrap_or("?")))
                .collect::<Vec<_>>()
                .join("\n");
            let body = resp.text().await.unwrap_or_default();
            let truncated = if body.len() > 10_000 {
                &body[..10_000]
            } else {
                &body
            };

            let payload = format!(
                "HTTP {} {}\n\n{}\n\n{}",
                status.as_u16(),
                status.canonical_reason().unwrap_or(""),
                headers,
                truncated
            );

            Ok(ToolResult {
                content: ryvos_core::types::wrap_untrusted_content(&payload),
                is_error: false,
                trust: ryvos_core::types::TrustLevel::Untrusted,
            })
        })
    }
}

pub struct HttpDownloadTool;

#[derive(Deserialize)]
struct HttpDownloadInput {
    url: String,
    output: String,
}

impl Tool for HttpDownloadTool {
    fn name(&self) -> &str {
        "http_download"
    }
    fn tier(&self) -> SecurityTier {
        SecurityTier::T1
    }
    fn timeout_secs(&self) -> u64 {
        300
    }
    fn description(&self) -> &str {
        "Download a file from a URL."
    }
    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "url": { "type": "string" },
                "output": { "type": "string", "description": "Output file path" }
            },
            "required": ["url", "output"]
        })
    }
    fn execute(
        &self,
        input: serde_json::Value,
        ctx: ToolContext,
    ) -> BoxFuture<'_, Result<ToolResult>> {
        Box::pin(async move {
            let p: HttpDownloadInput = serde_json::from_value(input)
                .map_err(|e| RyvosError::ToolValidation(e.to_string()))?;

            if let Err(e) = crate::egress::validate_url_for_egress(&p.url).await {
                return Ok(ToolResult::error(format!("Blocked by egress policy: {}", e)));
            }
            let out = resolve(&p.output, &ctx.working_dir);
            let client = reqwest::Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .build()
                .map_err(|e| RyvosError::ToolExecution {
                    tool: "http_download".into(),
                    message: e.to_string(),
                })?;
            let resp = client
                .get(&p.url)
                .send()
                .await
                .map_err(|e| RyvosError::ToolExecution {
                    tool: "http_download".into(),
                    message: e.to_string(),
                })?;
            if !resp.status().is_success() {
                return Ok(ToolResult::error(format!("HTTP {}", resp.status())));
            }
            if let Some(parent) = out.parent() {
                tokio::fs::create_dir_all(parent).await.ok();
            }
            let mut file = tokio::fs::File::create(&out)
                .await
                .map_err(|e| RyvosError::ToolExecution {
                    tool: "http_download".into(),
                    message: e.to_string(),
                })?;
            let stream = futures::TryStreamExt::map_err(resp.bytes_stream(), std::io::Error::other);
            let stream_reader = tokio_util::io::StreamReader::new(stream);
            let mut buf_reader = tokio::io::BufReader::with_capacity(65536, stream_reader);
            let bytes_copied = tokio::io::copy(&mut buf_reader, &mut file)
                .await
                .map_err(|e| RyvosError::ToolExecution {
                    tool: "http_download".into(),
                    message: e.to_string(),
                })?;
            Ok(ToolResult::success(format!(
                "Downloaded {} bytes to {}",
                bytes_copied,
                out.display()
            )))
        })
    }
}

pub struct DnsLookupTool;

#[derive(Deserialize)]
struct DnsInput {
    hostname: String,
}

impl Tool for DnsLookupTool {
    fn name(&self) -> &str {
        "dns_lookup"
    }
    fn tier(&self) -> SecurityTier {
        SecurityTier::T0
    }
    fn description(&self) -> &str {
        "Look up DNS records for a hostname."
    }
    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": { "hostname": { "type": "string" } },
            "required": ["hostname"]
        })
    }
    fn execute(
        &self,
        input: serde_json::Value,
        _ctx: ToolContext,
    ) -> BoxFuture<'_, Result<ToolResult>> {
        Box::pin(async move {
            let p: DnsInput = serde_json::from_value(input)
                .map_err(|e| RyvosError::ToolValidation(e.to_string()))?;
            let addr = format!("{}:80", p.hostname);
            let result = tokio::net::lookup_host(addr.as_str()).await;
            match result {
                Ok(addrs) => {
                    let ips: Vec<String> = addrs.map(|a| a.ip().to_string()).collect();
                    Ok(ToolResult::success(format!(
                        "{} resolves to:\n{}",
                        p.hostname,
                        ips.join("\n")
                    )))
                }
                Err(e) => Ok(ToolResult::error(format!("DNS lookup failed: {}", e))),
            }
        })
    }
}

pub struct NetworkCheckTool;

#[derive(Deserialize)]
struct NetworkCheckInput {
    host: String,
    port: u16,
}

impl Tool for NetworkCheckTool {
    fn name(&self) -> &str {
        "network_check"
    }
    fn tier(&self) -> SecurityTier {
        SecurityTier::T0
    }
    fn description(&self) -> &str {
        "Check TCP connectivity to a host:port."
    }
    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "host": { "type": "string" },
                "port": { "type": "integer" }
            },
            "required": ["host", "port"]
        })
    }
    fn execute(
        &self,
        input: serde_json::Value,
        _ctx: ToolContext,
    ) -> BoxFuture<'_, Result<ToolResult>> {
        Box::pin(async move {
            let p: NetworkCheckInput = serde_json::from_value(input)
                .map_err(|e| RyvosError::ToolValidation(e.to_string()))?;
            let addr = format!("{}:{}", p.host, p.port);
            let start = std::time::Instant::now();
            match tokio::time::timeout(
                Duration::from_secs(5),
                tokio::net::TcpStream::connect(&addr),
            )
            .await
            {
                Ok(Ok(_)) => {
                    let ms = start.elapsed().as_millis();
                    Ok(ToolResult::success(format!(
                        "Connected to {} in {}ms",
                        addr, ms
                    )))
                }
                Ok(Err(e)) => Ok(ToolResult::error(format!("Connection refused: {}", e))),
                Err(_) => Ok(ToolResult::error(
                    "Connection timed out after 5s".to_string(),
                )),
            }
        })
    }
}
