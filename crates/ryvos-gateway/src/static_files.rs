use axum::extract::Path;
use axum::http::{header, StatusCode, Uri};
use axum::response::IntoResponse;
use rust_embed::Embed;

#[derive(Embed)]
#[folder = "ui/"]
struct UiAssets;

pub async fn index() -> impl IntoResponse {
    serve_html("index.html")
}

pub async fn next_static(Path(path): Path<String>) -> impl IntoResponse {
    let full_path = format!("_next/{path}");
    match UiAssets::get(&full_path) {
        Some(content) => {
            let mime = mime_guess::from_path(&path)
                .first_or_octet_stream()
                .to_string();
            (
                StatusCode::OK,
                [
                    (header::CONTENT_TYPE, mime),
                    (
                        header::CACHE_CONTROL,
                        "public, max-age=31536000, immutable".to_string(),
                    ),
                ],
                content.data.to_vec(),
            )
                .into_response()
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

pub async fn static_file(Path(path): Path<String>) -> impl IntoResponse {
    match UiAssets::get(&format!("assets/{path}")) {
        Some(content) => {
            let mime = mime_guess::from_path(&path)
                .first_or_octet_stream()
                .to_string();
            (
                StatusCode::OK,
                [
                    (header::CONTENT_TYPE, mime),
                    (
                        header::CACHE_CONTROL,
                        "public, max-age=31536000, immutable".to_string(),
                    ),
                ],
                content.data.to_vec(),
            )
                .into_response()
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

pub async fn spa_fallback(uri: Uri) -> impl IntoResponse {
    let uri_path = uri.path().trim_start_matches('/');
    if uri_path.is_empty() {
        return serve_html("index.html");
    }

    if let Some(content) = UiAssets::get(uri_path) {
        let mime = mime_guess::from_path(uri_path)
            .first_or_octet_stream()
            .to_string();
        return (
            StatusCode::OK,
            [(header::CONTENT_TYPE, mime)],
            content.data.to_vec(),
        )
            .into_response();
    }

    let html_path = format!("{uri_path}.html");
    if UiAssets::get(&html_path).is_some() {
        return serve_html(&html_path);
    }

    let index_nested = format!("{uri_path}/index.html");
    if UiAssets::get(&index_nested).is_some() {
        return serve_html(&index_nested);
    }

    serve_html("index.html")
}

fn serve_html(path: &str) -> axum::response::Response {
    match UiAssets::get(path) {
        Some(content) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "text/html; charset=utf-8".to_string())],
            content.data.to_vec(),
        )
            .into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}
