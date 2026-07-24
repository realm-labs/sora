use rmcp::{
    handler::server::wrapper::{Json, Parameters},
    tool, tool_router,
};
use schemars::JsonSchema;
use serde::Deserialize;
use sora_workspace::{
    DataValidationQuery, DataValidationReport, ProjectId, TableQuery, TableQueryReport,
};

use crate::{SoraMcpServer, dto::ToolEnvelope};

#[tool_router(router = data_tool_router, vis = "pub(crate)")]
impl SoraMcpServer {
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
