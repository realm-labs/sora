use rmcp::{
    handler::server::wrapper::{Json, Parameters},
    tool, tool_router,
};
use schemars::JsonSchema;
use serde::Deserialize;
use sora_workspace::{
    ProjectId, SchemaApplyReport, SchemaMutationPlan, SchemaOperation, SchemaSearchQuery,
    SchemaSearchReport, ValidationReport,
};

use crate::{
    SoraMcpServer,
    dto::{ToolEnvelope, tool_error},
};

#[tool_router(router = schema_tool_router, vis = "pub(crate)")]
impl SoraMcpServer {
    #[tool(
        name = "sora_schema_validate",
        description = "Load, normalize, and validate a registered project's schema without writing files",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn schema_validate(
        &self,
        Parameters(input): Parameters<ProjectInput>,
    ) -> Result<Json<ToolEnvelope<ValidationReport>>, rmcp::model::CallToolResult> {
        let id = ProjectId::new(input.project_id).map_err(|error| {
            tool_error(ToolEnvelope::<ValidationReport>::failure(
                None,
                None,
                "invalid project id",
                error,
            ))
        })?;
        match self.workspace.project(&id) {
            Ok(session) => {
                let report = tokio::task::spawn_blocking(move || session.validate_schema())
                    .await
                    .map_err(|error| {
                        tool_error(ToolEnvelope::<ValidationReport>::failure(
                            Some(id.clone()),
                            None,
                            "schema validation worker failed",
                            error,
                        ))
                    })?;
                let summary = if report.ok {
                    "schema is valid"
                } else {
                    "schema validation failed"
                };
                let mut envelope =
                    ToolEnvelope::success(Some(id), Some(report.revision.clone()), summary, report);
                if let Some(report) = envelope.data.as_ref()
                    && !report.ok
                {
                    envelope.ok = false;
                    envelope.diagnostics = report.diagnostics.clone();
                }
                if envelope.ok {
                    Ok(Json(envelope))
                } else {
                    Err(tool_error(envelope))
                }
            }
            Err(error) => Err(tool_error(ToolEnvelope::<ValidationReport>::failure(
                Some(id),
                None,
                "unknown Sora project",
                error,
            ))),
        }
    }

    #[tool(
        name = "sora_schema_search",
        description = "Search normalized schema entities by kind, name, field, type, group, source, or references",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn schema_search(
        &self,
        Parameters(input): Parameters<SchemaSearchInput>,
    ) -> Result<Json<ToolEnvelope<SchemaSearchReport>>, rmcp::model::CallToolResult> {
        let id = ProjectId::new(input.project_id).map_err(|error| {
            tool_error(ToolEnvelope::<SchemaSearchReport>::failure(
                None,
                None,
                "invalid project id",
                error,
            ))
        })?;
        match self.workspace.project(&id) {
            Ok(session) => {
                let revision = session.revision();
                match tokio::task::spawn_blocking(move || session.search_schema(&input.query)).await
                {
                    Ok(Ok(report)) => Ok(Json(ToolEnvelope::success(
                        Some(id),
                        Some(report.revision.clone()),
                        format!("found {} schema entities", report.results.len()),
                        report,
                    ))),
                    Ok(Err(error)) => Err(tool_error(ToolEnvelope::<SchemaSearchReport>::failure(
                        Some(id),
                        Some(revision),
                        "schema search failed",
                        error,
                    ))),
                    Err(error) => Err(tool_error(ToolEnvelope::<SchemaSearchReport>::failure(
                        Some(id),
                        Some(revision),
                        "schema search worker failed",
                        error,
                    ))),
                }
            }
            Err(error) => Err(tool_error(ToolEnvelope::<SchemaSearchReport>::failure(
                Some(id),
                None,
                "unknown Sora project",
                error,
            ))),
        }
    }

    #[tool(
        name = "sora_schema_preview",
        description = "Validate an ordered schema operation batch and return an immutable, revision-bound plan without writing files",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn schema_preview(
        &self,
        Parameters(input): Parameters<SchemaPreviewInput>,
    ) -> Result<Json<ToolEnvelope<SchemaMutationPlan>>, rmcp::model::CallToolResult> {
        let id = ProjectId::new(input.project_id).map_err(|error| {
            tool_error(ToolEnvelope::<SchemaMutationPlan>::failure(
                None,
                None,
                "invalid project id",
                error,
            ))
        })?;
        let workspace = self.workspace.clone();
        let owner = self.authorization_context.clone();
        let operation_id = id.clone();
        let result = tokio::task::spawn_blocking(move || {
            workspace.preview_schema_mutation(
                &operation_id,
                &owner,
                &input.expected_schema_revision,
                &input.expected_manifest_revision,
                input.operations,
            )
        })
        .await;
        match result {
            Ok(Ok(plan)) => {
                let mut envelope = ToolEnvelope::success(
                    Some(id),
                    Some(plan.input_revisions.clone()),
                    format!(
                        "planned {} schema operation(s) affecting {} file(s)",
                        plan.normalized_operations.len(),
                        plan.affected_files.len()
                    ),
                    plan,
                );
                envelope.changes = envelope
                    .data
                    .as_ref()
                    .map(|plan| {
                        plan.text_diffs
                            .iter()
                            .filter_map(|diff| serde_json::to_value(diff).ok())
                            .collect()
                    })
                    .unwrap_or_default();
                Ok(Json(envelope))
            }
            Ok(Err(error)) => Err(tool_error(ToolEnvelope::<SchemaMutationPlan>::failure(
                Some(id.clone()),
                self.workspace
                    .project(&id)
                    .ok()
                    .map(|session| session.revision()),
                "schema preview failed",
                error,
            ))),
            Err(error) => Err(tool_error(ToolEnvelope::<SchemaMutationPlan>::failure(
                Some(id.clone()),
                self.workspace
                    .project(&id)
                    .ok()
                    .map(|session| session.revision()),
                "schema preview worker failed",
                error,
            ))),
        }
    }

    #[tool(
        name = "sora_schema_apply",
        description = "Atomically apply an unexpired schema plan after authorization, revision, and idempotency checks",
        annotations(
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn schema_apply(
        &self,
        Parameters(input): Parameters<SchemaApplyInput>,
        context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<Json<ToolEnvelope<SchemaApplyReport>>, rmcp::model::CallToolResult> {
        let id = ProjectId::new(input.project_id).map_err(|error| {
            tool_error(ToolEnvelope::<SchemaApplyReport>::failure(
                None,
                None,
                "invalid project id",
                error,
            ))
        })?;
        let workspace = self.workspace.clone();
        let owner = self.authorization_context.clone();
        let operation_id = id.clone();
        let result = tokio::task::spawn_blocking(move || {
            workspace.apply_schema_mutation(
                &operation_id,
                &owner,
                &input.plan_id,
                &input.idempotency_key,
            )
        })
        .await;
        match result {
            Ok(Ok(report)) => {
                self.notify_project_resources_updated(&context.peer, id.as_str())
                    .await;
                let envelope = ToolEnvelope::success(
                    Some(id),
                    Some(report.revision.clone()),
                    format!(
                        "applied schema plan to {} file(s)",
                        report.affected_files.len()
                    ),
                    report,
                );
                Ok(Json(envelope))
            }
            Ok(Err(error)) => Err(tool_error(ToolEnvelope::<SchemaApplyReport>::failure(
                Some(id.clone()),
                self.workspace
                    .project(&id)
                    .ok()
                    .map(|session| session.revision()),
                "schema apply failed",
                error,
            ))),
            Err(error) => Err(tool_error(ToolEnvelope::<SchemaApplyReport>::failure(
                Some(id.clone()),
                self.workspace
                    .project(&id)
                    .ok()
                    .map(|session| session.revision()),
                "schema apply worker failed",
                error,
            ))),
        }
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct ProjectInput {
    project_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct SchemaSearchInput {
    project_id: String,
    #[serde(flatten)]
    query: SchemaSearchQuery,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct SchemaPreviewInput {
    project_id: String,
    expected_schema_revision: String,
    expected_manifest_revision: String,
    operations: Vec<SchemaOperation>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct SchemaApplyInput {
    project_id: String,
    plan_id: String,
    idempotency_key: String,
}
