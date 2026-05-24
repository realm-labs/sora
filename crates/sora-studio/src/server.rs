use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::PathBuf,
    sync::Arc,
};

use anyhow::{Context, Result};
use axum::{
    Json, Router,
    extract::State,
    routing::{get, put},
};
use serde::Serialize;
use tower_http::cors::CorsLayer;

use crate::{
    model::{StudioPreviewResponse, StudioSchema, StudioSchemaResponse},
    service::{load_studio_schema, preview_studio_schema, save_studio_schema},
};

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
        .with_state(state)
        .layer(CorsLayer::permissive());
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("failed to bind studio server at http://{addr}"))?;

    println!("Sora Studio API: http://{addr}");
    println!("Project: {}", project.display());
    println!("Frontend dev server: cd apps/studio && npm run dev");

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

#[derive(Debug, Serialize)]
struct HealthResponse {
    ok: bool,
}
