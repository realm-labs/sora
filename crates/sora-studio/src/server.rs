use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::PathBuf,
    sync::Arc,
};

use anyhow::{Context, Result};
use axum::{
    Json, Router,
    body::Body,
    extract::{Path as AxumPath, State},
    http::{StatusCode, Uri, header},
    response::{IntoResponse, Response},
    routing::{get, put},
};
use include_dir::{Dir, include_dir};
use serde::Serialize;
use tower_http::cors::CorsLayer;

use crate::{
    model::{StudioPreviewResponse, StudioSchema, StudioSchemaResponse},
    service::{load_studio_schema, preview_studio_schema, save_studio_schema},
};

static STUDIO_DIST: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../../apps/studio/dist");

#[derive(Debug, Clone)]
pub struct StudioOptions {
    pub project: PathBuf,
    pub host: IpAddr,
    pub port: u16,
}

impl StudioOptions {
    pub fn local(project: PathBuf, port: u16) -> Self {
        Self {
            project,
            host: IpAddr::V4(Ipv4Addr::LOCALHOST),
            port,
        }
    }
}

pub fn run_blocking(options: StudioOptions) -> Result<()> {
    let runtime = tokio::runtime::Runtime::new().context("failed to start async runtime")?;
    runtime.block_on(run(options))
}

pub async fn run(options: StudioOptions) -> Result<()> {
    let addr = SocketAddr::new(options.host, options.port);
    let project = options.project.clone();
    let state = StudioState {
        project: Arc::new(project.clone()),
    };
    let app = Router::new()
        .route("/api/health", get(health))
        .route("/api/schema", get(schema))
        .route("/api/schema", put(save_schema))
        .route("/api/schema/preview", put(preview_schema))
        .route("/", get(studio_index))
        .route("/assets/{*path}", get(studio_asset))
        .fallback(not_found)
        .with_state(state)
        .layer(CorsLayer::permissive());
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("failed to bind studio server at http://{addr}"))?;

    println!("Sora Studio: http://{addr}");
    println!("Project: {}", project.display());

    axum::serve(listener, app)
        .await
        .context("studio server stopped unexpectedly")
}

#[derive(Debug, Clone)]
struct StudioState {
    project: Arc<PathBuf>,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { ok: true })
}

async fn schema(State(state): State<StudioState>) -> Json<StudioSchemaResponse> {
    Json(load_studio_schema(&state.project))
}

async fn save_schema(
    State(state): State<StudioState>,
    Json(schema): Json<StudioSchema>,
) -> Json<StudioSchemaResponse> {
    Json(save_studio_schema(&state.project, &schema))
}

async fn preview_schema(
    State(state): State<StudioState>,
    Json(schema): Json<StudioSchema>,
) -> Json<StudioPreviewResponse> {
    Json(preview_studio_schema(&state.project, &schema))
}

async fn studio_index() -> Response {
    embedded_asset_response("index.html").unwrap_or_else(missing_frontend_response)
}

async fn studio_asset(AxumPath(path): AxumPath<String>) -> Response {
    let path = format!("assets/{path}");
    embedded_asset_response(&path).unwrap_or_else(|| StatusCode::NOT_FOUND.into_response())
}

async fn not_found(uri: Uri) -> Response {
    (StatusCode::NOT_FOUND, format!("not found: {}", uri.path())).into_response()
}

fn embedded_asset_response(path: &str) -> Option<Response> {
    let file = STUDIO_DIST.get_file(path)?;
    Some(
        Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, content_type(file.path()))
            .body(Body::from(file.contents().to_vec()))
            .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response()),
    )
}

fn missing_frontend_response() -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "Sora Studio frontend assets are not embedded. Run `npm run build` in apps/studio before building the CLI.",
    )
        .into_response()
}

fn content_type(path: &std::path::Path) -> &'static str {
    match path.extension().and_then(|value| value.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("js") => "text/javascript; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("json") => "application/json; charset=utf-8",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("webp") => "image/webp",
        Some("ico") => "image/x-icon",
        Some("wasm") => "application/wasm",
        _ => "application/octet-stream",
    }
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    ok: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embeds_studio_frontend_entrypoint() {
        let index = STUDIO_DIST
            .get_file("index.html")
            .expect("apps/studio/dist/index.html must be embedded");

        assert!(
            std::str::from_utf8(index.contents())
                .unwrap()
                .contains("<script")
        );
    }

    #[test]
    fn assigns_frontend_asset_content_types() {
        assert_eq!(
            content_type(std::path::Path::new("index.html")),
            "text/html; charset=utf-8"
        );
        assert_eq!(
            content_type(std::path::Path::new("assets/app.js")),
            "text/javascript; charset=utf-8"
        );
        assert_eq!(
            content_type(std::path::Path::new("assets/app.css")),
            "text/css; charset=utf-8"
        );
    }
}
