use rmcp::{
    handler::server::wrapper::{Json, Parameters},
    tool, tool_router,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sora_workspace::{
    ProjectCandidate, ProjectId, ProjectInitApplyReport, ProjectInitPlan, ProjectInspection,
    RuntimeOptions,
};

use crate::{
    SoraMcpServer,
    dto::{ToolEnvelope, tool_error},
};

#[tool_router(router = project_tool_router, vis = "pub(crate)")]
impl SoraMcpServer {
    #[tool(
        name = "sora_project_list",
        description = "List Sora project manifests discovered inside the server's allowed roots"
    )]
    fn project_list(
        &self,
        Parameters(_input): Parameters<ProjectListInput>,
    ) -> Json<ToolEnvelope<ProjectListOutput>> {
        match self.workspace.discover_projects() {
            Ok(projects) => {
                let count = projects.len();
                Json(ToolEnvelope::success(
                    None,
                    None,
                    format!("found {count} Sora project manifest(s)"),
                    ProjectListOutput { projects },
                ))
            }
            Err(error) => Json(ToolEnvelope::failure(
                None,
                None,
                "failed to discover Sora projects",
                error,
            )),
        }
    }

    #[tool(
        name = "sora_project_open",
        description = "Open one discovered Sora project by root id and root-relative manifest path"
    )]
    fn project_open(
        &self,
        Parameters(input): Parameters<ProjectOpenInput>,
    ) -> Json<ToolEnvelope<ProjectOpenOutput>> {
        match self.workspace.open_discovered_project(
            &input.root_id,
            &input.relative_manifest,
            RuntimeOptions::default(),
            false,
        ) {
            Ok(session) => match session.inspect() {
                Ok(project) => Json(ToolEnvelope::success(
                    Some(session.id().clone()),
                    Some(session.revision()),
                    format!("opened Sora project `{}`", project.package),
                    ProjectOpenOutput { project },
                )),
                Err(error) => Json(ToolEnvelope::failure(
                    Some(session.id().clone()),
                    Some(session.revision()),
                    "project opened but inspection failed",
                    error,
                )),
            },
            Err(error) => Json(ToolEnvelope::failure(
                None,
                None,
                "failed to open Sora project",
                error,
            )),
        }
    }

    #[tool(
        name = "sora_project_inspect",
        description = "Inspect a registered project's schema sources, data sources, scopes, and build capabilities"
    )]
    fn project_inspect(
        &self,
        Parameters(input): Parameters<ProjectInput>,
    ) -> Json<ToolEnvelope<ProjectInspectOutput>> {
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
            Ok(session) => match session.inspect() {
                Ok(project) => Json(ToolEnvelope::success(
                    Some(id),
                    Some(session.revision()),
                    format!("inspected Sora project `{}`", project.package),
                    ProjectInspectOutput { project },
                )),
                Err(error) => Json(ToolEnvelope::failure(
                    Some(id),
                    Some(session.revision()),
                    "failed to inspect Sora project",
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
        name = "sora_project_init",
        description = "Preview a new project scaffold inside an allowed root; this operation never writes files"
    )]
    fn project_init(
        &self,
        Parameters(input): Parameters<ProjectInitInput>,
    ) -> Result<Json<ToolEnvelope<ProjectInitPlan>>, rmcp::model::CallToolResult> {
        match self.workspace.preview_project_init(
            &self.authorization_context,
            &input.root_id,
            &input.relative_directory,
            &input.package,
        ) {
            Ok(plan) => Ok(Json(ToolEnvelope::success(
                None,
                None,
                format!("planned {} new project file(s)", plan.files.len()),
                plan,
            ))),
            Err(error) => Err(tool_error(ToolEnvelope::<ProjectInitPlan>::failure(
                None,
                None,
                "project initialization preview failed",
                error,
            ))),
        }
    }

    #[tool(
        name = "sora_project_init_apply",
        description = "Atomically create and open a project from an unexpired initialization plan"
    )]
    async fn project_init_apply(
        &self,
        Parameters(input): Parameters<ProjectInitApplyInput>,
        context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<Json<ToolEnvelope<ProjectInitApplyReport>>, rmcp::model::CallToolResult> {
        match self.workspace.apply_project_init(
            &self.authorization_context,
            &input.plan_id,
            &input.idempotency_key,
        ) {
            Ok(report) => {
                let _ = context.peer.notify_resource_list_changed().await;
                Ok(Json(ToolEnvelope::success(
                    Some(report.project_id.clone()),
                    Some(report.revision.clone()),
                    format!("created Sora project `{}`", report.project_id),
                    report,
                )))
            }
            Err(error) => Err(tool_error(ToolEnvelope::<ProjectInitApplyReport>::failure(
                None,
                None,
                "project initialization apply failed",
                error,
            ))),
        }
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct ProjectListInput {}

#[derive(Debug, Serialize, JsonSchema)]
struct ProjectListOutput {
    projects: Vec<ProjectCandidate>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct ProjectOpenInput {
    root_id: String,
    relative_manifest: String,
}

#[derive(Debug, Serialize, JsonSchema)]
struct ProjectOpenOutput {
    project: ProjectInspection,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct ProjectInput {
    project_id: String,
}

#[derive(Debug, Serialize, JsonSchema)]
struct ProjectInspectOutput {
    project: ProjectInspection,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct ProjectInitInput {
    root_id: String,
    relative_directory: String,
    package: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct ProjectInitApplyInput {
    plan_id: String,
    idempotency_key: String,
}
