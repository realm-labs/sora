use rmcp::{
    handler::server::wrapper::{Json, Parameters},
    tool, tool_router,
};
use schemars::JsonSchema;
use serde::Deserialize;
use sora_workspace::{
    DataApplyReport, DataMutationPlan, DataOperation, DataValidationQuery, DataValidationReport,
    ProjectId, TableQuery, TableQueryReport,
};

use crate::{
    SoraMcpServer,
    dto::{ToolEnvelope, tool_error},
};

#[tool_router(router = data_tool_router, vis = "pub(crate)")]
impl SoraMcpServer {
    #[tool(
        name = "sora_data_preview",
        description = "Validate an ordered typed row mutation batch and return a revision-bound plan without writing data sources"
    )]
    fn data_preview(
        &self,
        Parameters(input): Parameters<DataPreviewInput>,
    ) -> Result<Json<ToolEnvelope<DataMutationPlan>>, rmcp::model::CallToolResult> {
        let id = ProjectId::new(input.project_id).map_err(|error| {
            tool_error(ToolEnvelope::<DataMutationPlan>::failure(
                None,
                None,
                "invalid project id",
                error,
            ))
        })?;
        match self.workspace.preview_data_mutation(
            &id,
            &self.authorization_context,
            &input.expected_schema_revision,
            &input.expected_data_revision,
            input.operations,
        ) {
            Ok(plan) => {
                let changes = encode_data_changes(&plan.row_changes, &plan.localization_changes)
                    .map_err(|error| {
                        tool_error(ToolEnvelope::<DataMutationPlan>::failure(
                            Some(id.clone()),
                            Some(plan.input_revisions.clone()),
                            "failed to encode data changes",
                            error,
                        ))
                    })?;
                let mut envelope = ToolEnvelope::success(
                    Some(id),
                    Some(plan.input_revisions.clone()),
                    format!(
                        "planned {} data operation(s) across {} source file(s)",
                        plan.normalized_operations.len(),
                        plan.file_changes.len()
                    ),
                    plan,
                );
                envelope.changes = changes;
                Ok(Json(envelope))
            }
            Err(error) => Err(tool_error(ToolEnvelope::<DataMutationPlan>::failure(
                Some(id.clone()),
                self.workspace
                    .project(&id)
                    .ok()
                    .map(|session| session.revision()),
                "data preview failed",
                error,
            ))),
        }
    }

    #[tool(
        name = "sora_data_apply",
        description = "Atomically apply an unexpired data plan after authorization, schema/data revision, and idempotency checks"
    )]
    async fn data_apply(
        &self,
        Parameters(input): Parameters<DataApplyInput>,
        context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<Json<ToolEnvelope<DataApplyReport>>, rmcp::model::CallToolResult> {
        let id = ProjectId::new(input.project_id).map_err(|error| {
            tool_error(ToolEnvelope::<DataApplyReport>::failure(
                None,
                None,
                "invalid project id",
                error,
            ))
        })?;
        match self.workspace.apply_data_mutation(
            &id,
            &self.authorization_context,
            &input.plan_id,
            &input.idempotency_key,
        ) {
            Ok(report) => {
                self.notify_project_resources_updated(&context.peer, id.as_str())
                    .await;
                let changes =
                    encode_data_changes(&report.row_changes, &report.localization_changes)
                        .map_err(|error| {
                            tool_error(ToolEnvelope::<DataApplyReport>::failure(
                                Some(id.clone()),
                                Some(report.revision.clone()),
                                "failed to encode applied data changes",
                                error,
                            ))
                        })?;
                let mut envelope = ToolEnvelope::success(
                    Some(id),
                    Some(report.revision.clone()),
                    format!(
                        "applied data plan to {} source file(s)",
                        report.transaction.affected_files.len()
                    ),
                    report,
                );
                envelope.changes = changes;
                envelope.artifacts = envelope
                    .data
                    .as_ref()
                    .map(|report| {
                        vec![crate::dto::ArtifactLink {
                            artifact_id: report.transaction.backup_id.clone(),
                            uri: format!(
                                "sora://project/{}/artifact/{}",
                                report.project_id, report.transaction.backup_id
                            ),
                            mime_type: "application/x-sora-backup".to_owned(),
                            name: None,
                            size: None,
                        }]
                    })
                    .unwrap_or_default();
                Ok(Json(envelope))
            }
            Err(error) => Err(tool_error(ToolEnvelope::<DataApplyReport>::failure(
                Some(id.clone()),
                self.workspace
                    .project(&id)
                    .ok()
                    .map(|session| session.revision()),
                "data apply failed",
                error,
            ))),
        }
    }

    #[tool(
        name = "sora_data_validate",
        description = "Load and fully validate project data, optionally selecting a scope or table subset"
    )]
    fn data_validate(
        &self,
        Parameters(input): Parameters<DataValidateInput>,
    ) -> Json<ToolEnvelope<DataValidationReport>> {
        let id = match ProjectId::new(input.project_id) {
            Ok(id) => id,
            Err(error) => {
                return Json(ToolEnvelope::failure(
                    None,
                    None,
                    "invalid project id",
                    error,
                ));
            }
        };
        match self.workspace.project(&id) {
            Ok(session) => {
                let report = session.validate_data(&input.query);
                let mut envelope = ToolEnvelope::success(
                    Some(id),
                    Some(report.revision.clone()),
                    if report.ok {
                        "data is valid"
                    } else {
                        "data validation failed"
                    },
                    report,
                );
                if let Some(report) = envelope.data.as_ref()
                    && !report.ok
                {
                    envelope.ok = false;
                    envelope.diagnostics = report.diagnostics.clone();
                }
                Json(envelope)
            }
            Err(error) => Json(ToolEnvelope::failure(
                Some(id),
                None,
                "unknown Sora project",
                error,
            )),
        }
    }

    #[tool(
        name = "sora_table_query",
        description = "Query validated table rows with typed equality, key or index lookup, projection, ordering, and revision-bound pagination"
    )]
    fn table_query(
        &self,
        Parameters(input): Parameters<TableQueryInput>,
    ) -> Json<ToolEnvelope<TableQueryReport>> {
        let id = match ProjectId::new(input.project_id) {
            Ok(id) => id,
            Err(error) => {
                return Json(ToolEnvelope::failure(
                    None,
                    None,
                    "invalid project id",
                    error,
                ));
            }
        };
        match self.workspace.project(&id) {
            Ok(session) => match session.query_table(&input.query) {
                Ok(report) => {
                    let count = report.rows.len();
                    let mut envelope = ToolEnvelope::success(
                        Some(id),
                        Some(report.revision.clone()),
                        format!("returned {count} validated row(s)"),
                        report,
                    );
                    envelope.next_cursor = envelope
                        .data
                        .as_ref()
                        .and_then(|report| report.next_cursor.clone());
                    Json(envelope)
                }
                Err(error) => Json(ToolEnvelope::failure(
                    Some(id),
                    Some(session.revision()),
                    "table query failed",
                    error,
                )),
            },
            Err(error) => Json(ToolEnvelope::failure(
                Some(id),
                None,
                "unknown Sora project",
                error,
            )),
        }
    }

    #[tool(
        name = "sora_data_diff",
        description = "Compare a project-relative baseline data root with the project's current validated data"
    )]
    fn data_diff(
        &self,
        Parameters(input): Parameters<DataDiffInput>,
    ) -> Json<ToolEnvelope<serde_json::Value>> {
        let id = match ProjectId::new(input.project_id) {
            Ok(id) => id,
            Err(error) => {
                return Json(ToolEnvelope::failure(
                    None,
                    None,
                    "invalid project id",
                    error,
                ));
            }
        };
        match self.workspace.project(&id) {
            Ok(session) => match session.diff_data_root(&input.other_data_root) {
                Ok(diff) => Json(ToolEnvelope::success(
                    Some(id),
                    Some(session.revision()),
                    "compared validated data roots",
                    diff,
                )),
                Err(error) => Json(ToolEnvelope::failure(
                    Some(id),
                    Some(session.revision()),
                    "data diff failed",
                    error,
                )),
            },
            Err(error) => Json(ToolEnvelope::failure(
                Some(id),
                None,
                "unknown Sora project",
                error,
            )),
        }
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct DataValidateInput {
    project_id: String,
    #[serde(flatten)]
    query: DataValidationQuery,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct TableQueryInput {
    project_id: String,
    #[serde(flatten)]
    query: TableQuery,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct DataDiffInput {
    project_id: String,
    other_data_root: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct DataPreviewInput {
    project_id: String,
    expected_schema_revision: String,
    expected_data_revision: String,
    operations: Vec<DataOperation>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct DataApplyInput {
    project_id: String,
    plan_id: String,
    idempotency_key: String,
}

fn encode_data_changes(
    rows: &[sora_workspace::RowChange],
    localization: &[sora_workspace::LocalizationChange],
) -> Result<Vec<serde_json::Value>, serde_json::Error> {
    let mut changes = rows
        .iter()
        .map(serde_json::to_value)
        .collect::<Result<Vec<_>, _>>()?;
    changes.extend(
        localization
            .iter()
            .map(serde_json::to_value)
            .collect::<Result<Vec<_>, _>>()?,
    );
    Ok(changes)
}
