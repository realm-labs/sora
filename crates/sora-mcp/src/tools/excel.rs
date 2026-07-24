use rmcp::{
    handler::server::wrapper::{Json, Parameters},
    tool, tool_router,
};
use schemars::JsonSchema;
use serde::Deserialize;
use sora_workspace::{
    ExcelSyncApplyReport, ExcelSyncControl, ExcelSyncPhase, ExcelSyncPlan, ExcelSyncPlanError,
    ProjectId,
};

use crate::{
    SoraMcpServer,
    dto::{ToolEnvelope, tool_error},
};

#[tool_router(router = excel_tool_router, vis = "pub(crate)")]
impl SoraMcpServer {
    #[tool(
        name = "sora_excel_sync_preview",
        description = "Preview schema-to-workbook synchronization without modifying any data source",
        execution(task_support = "optional")
    )]
    async fn excel_sync_preview(
        &self,
        Parameters(input): Parameters<ExcelSyncPreviewInput>,
        context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<Json<ToolEnvelope<ExcelSyncPlan>>, rmcp::model::CallToolResult> {
        let id = ProjectId::new(input.project_id).map_err(|error| {
            tool_error(ToolEnvelope::<ExcelSyncPlan>::failure(
                None,
                None,
                "invalid project id",
                error,
            ))
        })?;
        let workspace = self.workspace.clone();
        let owner = self.authorization_context.to_string();
        let worker_id = id.clone();
        let result = run_excel_worker(&context, move |control| {
            workspace.preview_excel_sync_with_control(
                &worker_id,
                &owner,
                &input.expected_schema_revision,
                &input.expected_data_revision,
                &control,
            )
        })
        .await;
        match result {
            Ok(Ok(plan)) => Ok(Json(ToolEnvelope::success(
                Some(id),
                Some(plan.input_revisions.clone()),
                format!(
                    "planned synchronization for {} workbook(s)",
                    plan.workbook_changes.len()
                ),
                plan,
            ))),
            Ok(Err(error)) => Err(tool_error(ToolEnvelope::<ExcelSyncPlan>::failure(
                Some(id.clone()),
                self.workspace
                    .project(&id)
                    .ok()
                    .map(|session| session.revision()),
                "Excel sync preview failed",
                error,
            ))),
            Err(error) => Err(tool_error(ToolEnvelope::<ExcelSyncPlan>::failure(
                Some(id),
                None,
                "Excel sync preview worker failed",
                error,
            ))),
        }
    }

    #[tool(
        name = "sora_excel_sync_apply",
        description = "Atomically apply an unexpired Excel synchronization plan after authorization and revision checks",
        execution(task_support = "optional")
    )]
    async fn excel_sync_apply(
        &self,
        Parameters(input): Parameters<ExcelSyncApplyInput>,
        context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<Json<ToolEnvelope<ExcelSyncApplyReport>>, rmcp::model::CallToolResult> {
        let id = ProjectId::new(input.project_id).map_err(|error| {
            tool_error(ToolEnvelope::<ExcelSyncApplyReport>::failure(
                None,
                None,
                "invalid project id",
                error,
            ))
        })?;
        let workspace = self.workspace.clone();
        let owner = self.authorization_context.to_string();
        let worker_id = id.clone();
        let result = run_excel_worker(&context, move |control| {
            workspace.apply_excel_sync_with_control(
                &worker_id,
                &owner,
                &input.plan_id,
                &input.idempotency_key,
                &control,
            )
        })
        .await;
        match result {
            Ok(Ok(report)) => {
                self.notify_project_resources_updated(&context.peer, id.as_str())
                    .await;
                Ok(Json(ToolEnvelope::success(
                    Some(id),
                    Some(report.revision.clone()),
                    format!("synchronized {} workbook(s)", report.workbook_changes.len()),
                    report,
                )))
            }
            Ok(Err(error)) => Err(tool_error(ToolEnvelope::<ExcelSyncApplyReport>::failure(
                Some(id.clone()),
                self.workspace
                    .project(&id)
                    .ok()
                    .map(|session| session.revision()),
                "Excel sync apply failed",
                error,
            ))),
            Err(error) => Err(tool_error(ToolEnvelope::<ExcelSyncApplyReport>::failure(
                Some(id),
                None,
                "Excel sync apply worker failed",
                error,
            ))),
        }
    }
}

async fn run_excel_worker<T, F>(
    context: &rmcp::service::RequestContext<rmcp::RoleServer>,
    worker: F,
) -> Result<Result<T, ExcelSyncPlanError>, tokio::task::JoinError>
where
    T: Send + 'static,
    F: FnOnce(ExcelSyncControl) -> Result<T, ExcelSyncPlanError> + Send + 'static,
{
    let (progress_tx, mut progress_rx) = tokio::sync::mpsc::unbounded_channel();
    let cancellation = ExcelSyncControl::default();
    let cancel_from_request = cancellation.clone();
    let request_cancellation = context.ct.clone();
    let cancellation_task = tokio::spawn(async move {
        request_cancellation.cancelled().await;
        cancel_from_request.cancel();
    });
    let control = cancellation.on_progress(move |progress| {
        let _ = progress_tx.send(progress);
    });
    let progress_token = context.meta.get_progress_token();
    let progress_peer = context.peer.clone();
    let progress_task = tokio::spawn(async move {
        while let Some(progress) = progress_rx.recv().await {
            if let Some(token) = progress_token.clone() {
                let _ = progress_peer
                    .notify_progress(
                        rmcp::model::ProgressNotificationParam::new(
                            token,
                            progress.completed as f64,
                        )
                        .with_total(progress.total as f64)
                        .with_message(excel_phase_name(progress.phase)),
                    )
                    .await;
            }
        }
    });
    let result = tokio::task::spawn_blocking(move || worker(control)).await;
    cancellation_task.abort();
    let _ = progress_task.await;
    result
}

fn excel_phase_name(phase: ExcelSyncPhase) -> &'static str {
    match phase {
        ExcelSyncPhase::ResolveProject => "resolve_project",
        ExcelSyncPhase::ValidateRevision => "validate_revision",
        ExcelSyncPhase::LoadSchema => "load_schema",
        ExcelSyncPhase::InspectWorkbooks => "inspect_workbooks",
        ExcelSyncPhase::StageWorkbooks => "stage_workbooks",
        ExcelSyncPhase::RenderWorkbooks => "render_workbooks",
        ExcelSyncPhase::Diff => "diff",
        ExcelSyncPhase::Commit => "commit",
        ExcelSyncPhase::ValidateData => "validate_data",
        ExcelSyncPhase::RefreshRevision => "refresh_revision",
        ExcelSyncPhase::Complete => "complete",
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct ExcelSyncPreviewInput {
    project_id: String,
    expected_schema_revision: String,
    expected_data_revision: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct ExcelSyncApplyInput {
    project_id: String,
    plan_id: String,
    idempotency_key: String,
}
