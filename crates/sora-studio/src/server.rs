use std::{
    net::{IpAddr, SocketAddr},
    sync::Arc,
};

use anyhow::{Context, Result};
use axum::{
    Json, Router,
    body::Body,
    extract::{Path as AxumPath, Query, State},
    http::{StatusCode, Uri, header},
    response::{IntoResponse, Response},
    routing::{get, put},
};
use include_dir::{Dir, include_dir};
use serde::{Deserialize, Serialize};
use sora_workspace::{
    ProjectId, ProjectRevision, StudioSchemaApplyReport, WorkspaceService,
    studio::{StudioPreviewResponse, StudioSchema, StudioSchemaResponse},
};
use tower_http::cors::CorsLayer;

static STUDIO_DIST: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/dist");

#[derive(Clone)]
pub struct StudioOptions {
    pub workspace: Arc<WorkspaceService>,
    pub project_id: ProjectId,
    pub host: IpAddr,
    pub port: u16,
}

pub fn run_blocking(options: StudioOptions) -> Result<()> {
    let runtime = tokio::runtime::Runtime::new().context("failed to start async runtime")?;
    runtime.block_on(run(options))
}

pub async fn run(options: StudioOptions) -> Result<()> {
    let addr = SocketAddr::new(options.host, options.port);
    let session = options
        .workspace
        .project(&options.project_id)
        .context("Studio project is not registered")?;
    let project = session.manifest_path().to_path_buf();
    let state = StudioState {
        workspace: options.workspace,
        project_id: options.project_id,
    };
    let app = Router::new()
        .route("/api/health", get(health))
        .route("/api/schema", get(schema))
        .route("/api/schema/preview", put(preview_schema))
        .route("/api/schema/apply", put(apply_schema))
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

#[derive(Clone)]
struct StudioState {
    workspace: Arc<WorkspaceService>,
    project_id: ProjectId,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { ok: true })
}

async fn schema(
    State(state): State<StudioState>,
    Query(query): Query<StudioSchemaQuery>,
) -> Json<StudioLoadResponse> {
    let worker_state = state.clone();
    match tokio::task::spawn_blocking(move || {
        worker_state
            .workspace
            .project(&worker_state.project_id)
            .map(|session| {
                (
                    session.load_studio_schema_view(query.view.as_deref()),
                    session.revision(),
                )
            })
    })
    .await
    {
        Ok(Ok((document, revision))) => Json(StudioLoadResponse {
            document,
            revision: Some(revision),
        }),
        Ok(Err(error)) => Json(StudioLoadResponse {
            document: studio_error("", error),
            revision: None,
        }),
        Err(error) => Json(StudioLoadResponse {
            document: studio_error("", error),
            revision: None,
        }),
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct StudioSchemaQuery {
    view: Option<String>,
}

async fn preview_schema(
    State(state): State<StudioState>,
    Json(request): Json<StudioPreviewRequest>,
) -> Json<StudioPlanResponse> {
    let worker_state = state.clone();
    let result = tokio::task::spawn_blocking(move || {
        worker_state.workspace.preview_studio_schema_mutation(
            &worker_state.project_id,
            STUDIO_AUTHORIZATION_CONTEXT,
            &request.expected_schema_revision,
            &request.expected_manifest_revision,
            request.schema,
        )
    })
    .await;
    match result {
        Ok(Ok(plan)) => Json(StudioPlanResponse {
            preview: plan.preview,
            plan_id: Some(plan.plan_id),
            revision: Some(plan.input_revisions),
        }),
        Ok(Err(error)) => Json(StudioPlanResponse {
            preview: StudioPreviewResponse {
                ok: false,
                project: project_name(&state),
                target: None,
                content: None,
                diff: None,
                diagnostics: vec![sora_workspace::Diagnostic::error(error.to_string())],
            },
            plan_id: None,
            revision: current_revision(&state),
        }),
        Err(error) => Json(StudioPlanResponse {
            preview: StudioPreviewResponse {
                ok: false,
                project: project_name(&state),
                target: None,
                content: None,
                diff: None,
                diagnostics: vec![sora_workspace::Diagnostic::error(error.to_string())],
            },
            plan_id: None,
            revision: current_revision(&state),
        }),
    }
}

async fn apply_schema(
    State(state): State<StudioState>,
    Json(request): Json<StudioApplyRequest>,
) -> Json<StudioApplyResponse> {
    let worker_state = state.clone();
    let result = tokio::task::spawn_blocking(move || {
        let report = worker_state.workspace.apply_studio_schema_mutation(
            &worker_state.project_id,
            STUDIO_AUTHORIZATION_CONTEXT,
            &request.plan_id,
            &request.idempotency_key,
        )?;
        let document = worker_state
            .workspace
            .project(&worker_state.project_id)
            .map(|session| session.load_studio_schema())
            .unwrap_or_else(|error| studio_error("", error));
        Ok::<_, sora_workspace::MutationPlanError>((report, document))
    })
    .await;
    match result {
        Ok(Ok((report, document))) => Json(StudioApplyResponse {
            document,
            revision: Some(report.revision.clone()),
            apply: Some(report),
        }),
        Ok(Err(error)) => Json(StudioApplyResponse {
            document: studio_error(&project_name(&state), error),
            revision: current_revision(&state),
            apply: None,
        }),
        Err(error) => Json(StudioApplyResponse {
            document: studio_error(&project_name(&state), error),
            revision: current_revision(&state),
            apply: None,
        }),
    }
}

const STUDIO_AUTHORIZATION_CONTEXT: &str = "studio-local";

#[derive(Debug, Serialize)]
struct StudioLoadResponse {
    #[serde(flatten)]
    document: StudioSchemaResponse,
    revision: Option<ProjectRevision>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct StudioPreviewRequest {
    schema: StudioSchema,
    expected_schema_revision: String,
    expected_manifest_revision: String,
}

#[derive(Debug, Serialize)]
struct StudioPlanResponse {
    #[serde(flatten)]
    preview: StudioPreviewResponse,
    plan_id: Option<String>,
    revision: Option<ProjectRevision>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct StudioApplyRequest {
    plan_id: String,
    idempotency_key: String,
}

#[derive(Debug, Serialize)]
struct StudioApplyResponse {
    #[serde(flatten)]
    document: StudioSchemaResponse,
    revision: Option<ProjectRevision>,
    apply: Option<StudioSchemaApplyReport>,
}

fn current_revision(state: &StudioState) -> Option<ProjectRevision> {
    state
        .workspace
        .project(&state.project_id)
        .map(|session| session.revision())
        .ok()
}

fn project_name(state: &StudioState) -> String {
    state
        .workspace
        .project(&state.project_id)
        .map(|session| session.manifest_path().display().to_string())
        .unwrap_or_default()
}

fn studio_error(project: &str, error: impl ToString) -> StudioSchemaResponse {
    StudioSchemaResponse {
        ok: false,
        project: project.to_owned(),
        diagnostics: vec![sora_workspace::Diagnostic::error(error.to_string())],
        schema: None,
    }
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
            .expect("crates/sora-studio/dist/index.html must be embedded");

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
