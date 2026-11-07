use std::collections::HashMap;

use axum::extract::Path;
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use rust_embed::Embed;
use tokio::net::TcpListener;

use ryvos_core::config::{McpServerConfig, McpTransport};

#[derive(Embed)]
#[folder = "crates/ryvos-gateway/ui/"]
struct EmbeddedUiAssets;

fn get_js_asset_path() -> String {
    for file in EmbeddedUiAssets::iter() {
        if file.starts_with("assets/index-") && file.ends_with(".js") {
            return file.to_string();
        }
    }
    "assets/index-DZUtNm8l.js".to_string()
}

fn get_css_asset_path() -> String {
    for file in EmbeddedUiAssets::iter() {
        if file.starts_with("assets/index-") && file.ends_with(".css") {
            return file.to_string();
        }
    }
    "assets/index-BTcWGj9i.css".to_string()
}

async fn spa_index() -> impl IntoResponse {
    match EmbeddedUiAssets::get("index.html") {
        Some(content) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
            content.data.to_vec(),
        )
            .into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn spa_static_file(Path(path): Path<String>) -> impl IntoResponse {
    let asset_path = format!("assets/{}", path);
    match EmbeddedUiAssets::get(&asset_path) {
        Some(content) => {
            let mime = if path.ends_with(".js") {
                "application/javascript"
            } else if path.ends_with(".css") {
                "text/css"
            } else if path.ends_with(".png") {
                "image/png"
            } else {
                "application/octet-stream"
            };

            let headers = [
                (header::CONTENT_TYPE, HeaderValue::from_static(mime)),
                (
                    header::CACHE_CONTROL,
                    HeaderValue::from_static("public, max-age=31536000, immutable"),
                ),
            ];

            (StatusCode::OK, headers, content.data.to_vec()).into_response()
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn api_health() -> impl IntoResponse {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        r#"{"status":"ok","version":"0.9.0"}"#,
    )
}

fn build_spa_router() -> Router {
    Router::new()
        .route("/api/health", get(api_health))
        .route("/", get(spa_index))
        .route("/index.html", get(spa_index))
        .route("/assets/{*path}", get(spa_static_file))
        .fallback(spa_index)
}

#[tokio::test]
async fn test_embedded_assets_exist_and_non_empty() {
    let index = EmbeddedUiAssets::get("index.html");
    assert!(index.is_some());
    let index_data = index.unwrap().data;
    assert!(!index_data.is_empty());
    let html_str = std::str::from_utf8(&index_data).unwrap();
    assert!(html_str.contains("<!doctype html>") || html_str.contains("<html"));

    let js_path = get_js_asset_path();
    let js_asset = EmbeddedUiAssets::get(&js_path);
    assert!(js_asset.is_some());
    assert!(js_asset.unwrap().data.len() > 1000);

    let css_path = get_css_asset_path();
    let css_asset = EmbeddedUiAssets::get(&css_path);
    assert!(css_asset.is_some());
    assert!(css_asset.unwrap().data.len() > 100);

    let logo_asset = EmbeddedUiAssets::get("assets/logo-D5wfv2Rf.png");
    assert!(logo_asset.is_some());
    let logo_data = logo_asset.unwrap().data;
    assert_eq!(&logo_data[0..4], b"\x89PNG");
}

#[tokio::test]
async fn test_spa_root_serves_index_html() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let app = build_spa_router();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let client = reqwest::Client::new();
    let resp = client
        .get(format!("http://{}/", addr))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), reqwest::StatusCode::OK);
    assert_eq!(
        resp.headers().get("content-type").unwrap(),
        "text/html; charset=utf-8"
    );
    let body = resp.text().await.unwrap();
    assert!(body.contains("<html") || body.contains("<!doctype html>"));
}

#[tokio::test]
async fn test_spa_static_asset_serves_with_cache_control_headers() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let app = build_spa_router();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let client = reqwest::Client::new();
    let js_rel = get_js_asset_path()
        .strip_prefix("assets/")
        .unwrap()
        .to_string();
    let resp = client
        .get(format!("http://{}/assets/{}", addr, js_rel))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), reqwest::StatusCode::OK);
    assert_eq!(
        resp.headers().get("content-type").unwrap(),
        "application/javascript"
    );
    let cache_control = resp.headers().get("cache-control").unwrap().to_str().unwrap();
    assert!(cache_control.contains("max-age=31536000"));
    assert!(cache_control.contains("immutable"));
}

#[tokio::test]
async fn test_spa_deep_link_routes_fallback_to_index_html() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let app = build_spa_router();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let client = reqwest::Client::new();
    let deep_routes = vec![
        "/chat",
        "/dashboard",
        "/sessions",
        "/settings",
        "/sessions/detail/abc-123",
        "/viking/memory/browse",
    ];

    for route in deep_routes {
        let resp = client
            .get(format!("http://{}{}", addr, route))
            .send()
            .await
            .unwrap();

        assert_eq!(resp.status(), reqwest::StatusCode::OK);
        assert_eq!(
            resp.headers().get("content-type").unwrap(),
            "text/html; charset=utf-8"
        );
        let body = resp.text().await.unwrap();
        assert!(body.contains("<html") || body.contains("<!doctype html>"));
    }
}

#[tokio::test]
async fn test_spa_missing_asset_returns_404() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let app = build_spa_router();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let client = reqwest::Client::new();
    let resp = client
        .get(format!("http://{}/assets/completely_missing_asset.xyz", addr))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), reqwest::StatusCode::NOT_FOUND);
}

#[test]
fn test_supermemory_mcp_config_header_retention() {
    let mut custom_headers = HashMap::new();
    custom_headers.insert(
        "Authorization".to_string(),
        "Bearer supermemory-secret-token-12345".to_string(),
    );
    custom_headers.insert("X-Custom-Profile".to_string(), "agent-v1".to_string());

    let config = McpServerConfig {
        transport: McpTransport::Sse {
            url: "https://api.supermemory.ai/mcp".to_string(),
        },
        auto_connect: true,
        allow_sampling: false,
        timeout_secs: 60,
        tier_override: None,
        headers: custom_headers.clone(),
    };

    assert_eq!(config.headers.len(), 2);
    assert_eq!(
        config.headers.get("Authorization").unwrap(),
        "Bearer supermemory-secret-token-12345"
    );
    assert_eq!(
        config.headers.get("X-Custom-Profile").unwrap(),
        "agent-v1"
    );

    let serialized = serde_json::to_string(&config).unwrap();
    let deserialized: McpServerConfig = serde_json::from_str(&serialized).unwrap();
    assert_eq!(
        deserialized.headers.get("Authorization").unwrap(),
        "Bearer supermemory-secret-token-12345"
    );
}

#[tokio::test]
async fn test_zero_panic_path_traversal_in_asset_route() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let app = build_spa_router();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let client = reqwest::Client::new();
    let traversal_paths = vec![
        "/assets/../../Cargo.toml",
        "/assets/../src/main.rs",
        "/assets/....//....//etc/passwd",
    ];

    for path in traversal_paths {
        let resp = client
            .get(format!("http://{}{}", addr, path))
            .send()
            .await;
        if let Ok(res) = resp {
            assert!(
                res.status() == reqwest::StatusCode::NOT_FOUND
                    || res.status() == reqwest::StatusCode::OK
                    || res.status() == reqwest::StatusCode::BAD_REQUEST
            );
        }
    }
}

#[tokio::test]
async fn test_zero_panic_deeply_nested_url_paths() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let app = build_spa_router();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let client = reqwest::Client::new();
    let deep_nested = "/a".repeat(100);
    let resp = client
        .get(format!("http://{}{}", addr, deep_nested))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), reqwest::StatusCode::OK);
}

#[tokio::test]
async fn test_pairwise_spa_fallback_with_query_params_and_headers() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let app = build_spa_router();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let client = reqwest::Client::new();
    let resp = client
        .get(format!("http://{}/chat?session=abc&filter=all&view=compact", addr))
        .header("User-Agent", "RyvosE2ETest/1.0")
        .header("Accept", "text/html,application/xhtml+xml")
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), reqwest::StatusCode::OK);
    let body = resp.text().await.unwrap();
    assert!(body.contains("<html") || body.contains("<!doctype html>"));
}

#[tokio::test]
async fn test_real_world_single_binary_delivery_lifecycle() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let app = build_spa_router();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let client = reqwest::Client::new();

    let index_res = client
        .get(format!("http://{}/", addr))
        .send()
        .await
        .unwrap();
    assert_eq!(index_res.status(), reqwest::StatusCode::OK);

    let js_rel = get_js_asset_path()
        .strip_prefix("assets/")
        .unwrap()
        .to_string();
    let js_res = client
        .get(format!("http://{}/assets/{}", addr, js_rel))
        .send()
        .await
        .unwrap();
    assert_eq!(js_res.status(), reqwest::StatusCode::OK);
    assert!(js_res.headers().contains_key("cache-control"));

    let css_rel = get_css_asset_path()
        .strip_prefix("assets/")
        .unwrap()
        .to_string();
    let css_res = client
        .get(format!("http://{}/assets/{}", addr, css_rel))
        .send()
        .await
        .unwrap();
    assert_eq!(css_res.status(), reqwest::StatusCode::OK);
    assert!(css_res.headers().contains_key("cache-control"));

    let logo_res = client
        .get(format!("http://{}/assets/logo-D5wfv2Rf.png", addr))
        .send()
        .await
        .unwrap();
    assert_eq!(logo_res.status(), reqwest::StatusCode::OK);

    let health_res = client
        .get(format!("http://{}/api/health", addr))
        .send()
        .await
        .unwrap();
    assert_eq!(health_res.status(), reqwest::StatusCode::OK);

    let spa_nav = client
        .get(format!("http://{}/dashboard", addr))
        .send()
        .await
        .unwrap();
    assert_eq!(spa_nav.status(), reqwest::StatusCode::OK);
}

#[tokio::test]
async fn test_spa_index_html_route_serves_200() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let app = build_spa_router();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let client = reqwest::Client::new();
    let resp = client
        .get(format!("http://{}/index.html", addr))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), reqwest::StatusCode::OK);
    assert_eq!(
        resp.headers().get("content-type").unwrap(),
        "text/html; charset=utf-8"
    );
    let body = resp.text().await.unwrap();
    assert!(body.contains("<html") || body.contains("<!doctype html>"));
}

#[test]
fn test_supermemory_mcp_json_deserialization_with_headers() {
    use ryvos_core::config::McpJsonConfig;

    let json_content = r#"{
        "mcpServers": {
            "supermemory": {
                "url": "https://mcp.supermemory.ai/mcp",
                "headers": {
                    "Authorization": "Bearer sm_test_token_12345"
                }
            }
        }
    }"#;

    let config: McpJsonConfig = serde_json::from_str(json_content).unwrap();
    assert!(config.mcp_servers.contains_key("supermemory"));
    let entry = config.mcp_servers.get("supermemory").unwrap();
    assert_eq!(
        entry.headers.get("Authorization").map(|s| s.as_str()),
        Some("Bearer sm_test_token_12345")
    );

    let server_cfg = entry.to_server_config().unwrap();
    assert_eq!(
        server_cfg.headers.get("Authorization").map(|s| s.as_str()),
        Some("Bearer sm_test_token_12345")
    );
}
