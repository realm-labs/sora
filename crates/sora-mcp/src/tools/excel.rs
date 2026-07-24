use rmcp::{
    handler::server::wrapper::{Json, Parameters},
    tool, tool_router,
};
use schemars::JsonSchema;
use serde::Deserialize;
use sora_workspace::{ExcelSyncApplyReport, ExcelSyncPlan, ProjectId};

use crate::{
    SoraMcpServer,
    dto::{ToolEnvelope, tool_error},
};

#[tool_router(router = excel_tool_router, vis = "pub(crate)")]
impl SoraMcpServer {
    #[tool(
        name = "sora_excel_sync_preview",
        description = "Preview schema-to-workbook synchronization without modifying any data source"
    )]
    async fn excel_sync_preview(
        &self,
        Parameters(input): Parameters<ExcelSyncPreviewInput>,
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
        let result = tokio::task::spawn_blocking(move || {
            workspace.preview_excel_sync(
                &worker_id,
                &owner,
                &input.expected_schema_revision,
                &input.expected_data_revision,
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
        description = "Atomically apply an unexpired Excel synchronization plan after authorization and revision checks"
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
        let result = tokio::task::spawn_blocking(move || {
            workspace.apply_excel_sync(&worker_id, &owner, &input.plan_id, &input.idempotency_key)
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
