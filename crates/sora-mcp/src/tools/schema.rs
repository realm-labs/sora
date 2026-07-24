use rmcp::{
    handler::server::wrapper::{Json, Parameters},
    tool, tool_router,
};
use schemars::JsonSchema;
use serde::Deserialize;
use sora_workspace::{ProjectId, SchemaSearchQuery, SchemaSearchReport, ValidationReport};

use crate::{SoraMcpServer, dto::ToolEnvelope};

#[tool_router(router = schema_tool_router, vis = "pub(crate)")]
impl SoraMcpServer {
    #[tool(
        name = "sora_schema_validate",
        description = "Load, normalize, and validate a registered project's schema without writing files"
    )]
    fn schema_validate(
        &self,
        Parameters(input): Parameters<ProjectInput>,
    ) -> Json<ToolEnvelope<ValidationReport>> {
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
                let report = session.validate_schema();
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
        name = "sora_schema_search",
        description = "Search normalized schema entities by kind, name, field, type, scope, source, or references"
    )]
    fn schema_search(
        &self,
        Parameters(input): Parameters<SchemaSearchInput>,
    ) -> Json<ToolEnvelope<SchemaSearchReport>> {
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
            Ok(session) => match session.search_schema(&input.query) {
                Ok(report) => Json(ToolEnvelope::success(
                    Some(id),
                    Some(report.revision.clone()),
                    format!("found {} schema entities", report.results.len()),
                    report,
                )),
                Err(error) => Json(ToolEnvelope::failure(
                    Some(id),
                    Some(session.revision()),
                    "schema search failed",
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
