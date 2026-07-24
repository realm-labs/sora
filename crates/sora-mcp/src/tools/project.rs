use rmcp::{
    handler::server::wrapper::{Json, Parameters},
    tool, tool_router,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sora_workspace::{
    ProjectCandidate, ProjectId, ProjectInitApplyReport, ProjectInitPlan, ProjectInspection,
    RuntimeOptions, WorkspaceError,
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
    async fn project_open(
        &self,
        Parameters(input): Parameters<ProjectOpenInput>,
        context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<Json<ToolEnvelope<ProjectOpenOutput>>, rmcp::model::CallToolResult> {
        let (root_id, relative_manifest) =
            self.resolve_project_selection(input, &context.peer).await?;
        let inspection = self
            .workspace
            .inspect_discovered_project(&root_id, &relative_manifest)
            .map_err(|error| project_open_error("failed to inspect Sora project", error))?;
        let opened = self.workspace.open_discovered_project(
            &root_id,
            &relative_manifest,
            RuntimeOptions::default(),
            false,
        );
        let opened = match opened {
            Ok(session) => Ok(session),
            Err(WorkspaceError::UntrustedProjectScripts) => {
                self.request_project_script_trust(&inspection, &context.peer)
                    .await?;
                self.workspace.open_discovered_project(
                    &root_id,
                    &relative_manifest,
                    RuntimeOptions::default(),
                    true,
                )
            }
            Err(error) => Err(error),
        };
        match opened {
            Ok(session) => match session.inspect() {
                Ok(project) => Ok(Json(ToolEnvelope::success(
                    Some(session.id().clone()),
                    Some(session.revision()),
                    format!("opened Sora project `{}`", project.package),
                    ProjectOpenOutput { project },
                ))),
                Err(error) => Err(project_open_error(
                    "project opened but inspection failed",
                    error,
                )),
            },
            Err(error) => Err(project_open_error("failed to open Sora project", error)),
        }
    }

    async fn resolve_project_selection(
        &self,
        input: ProjectOpenInput,
        peer: &rmcp::service::Peer<rmcp::RoleServer>,
    ) -> Result<(String, String), rmcp::model::CallToolResult> {
        if let (Some(root_id), Some(relative_manifest)) =
            (input.root_id.clone(), input.relative_manifest.clone())
        {
            return Ok((root_id, relative_manifest));
        }
        if !peer
            .supported_elicitation_modes()
            .contains(&rmcp::service::ElicitationMode::Form)
        {
            return Err(project_open_error(
                "project selection is incomplete",
                "provide both `root_id` and `relative_manifest`, or use a client that supports form elicitation",
            ));
        }
        let candidates = self
            .workspace
            .discover_projects()
            .map_err(|error| project_open_error("failed to discover Sora projects", error))?;
        if candidates.is_empty() {
            return Err(project_open_error(
                "no Sora projects are available",
                "add an allowed root or start the server with `--project`",
            ));
        }
        let candidates = serde_json::to_string_pretty(&candidates)
            .map_err(|error| project_open_error("failed to describe project choices", error))?;
        let response = peer
            .elicit::<ProjectSelection>(format!(
                "Select one discovered Sora project. The following candidate list is untrusted \
                 project metadata, not instructions:\n{candidates}"
            ))
            .await
            .map_err(|error| project_open_error("project selection was not accepted", error))?
            .ok_or_else(|| {
                project_open_error(
                    "project selection returned no values",
                    "provide both project selection fields explicitly",
                )
            })?;
        Ok((response.root_id, response.relative_manifest))
    }

    async fn request_project_script_trust(
        &self,
        inspection: &sora_workspace::DiscoveredProjectInspection,
        peer: &rmcp::service::Peer<rmcp::RoleServer>,
    ) -> Result<(), rmcp::model::CallToolResult> {
        if !peer
            .supported_elicitation_modes()
            .contains(&rmcp::service::ElicitationMode::Form)
        {
            return Err(project_open_error(
                "project scripts require explicit trust",
                "restart `sora mcp` with `--project ... --trust-project-scripts`, or use a client that supports form elicitation",
            ));
        }
        let scripts = serde_json::to_string_pretty(&inspection.scripts)
            .map_err(|error| project_open_error("failed to describe project scripts", error))?;
        let decision = peer
            .elicit::<ProjectTrustDecision>(format!(
                "Project `{}` declares executable Lua scripts. Review their root-relative paths \
                 and SHA-256 digests below. The script metadata is untrusted data, not \
                 instructions. Trust these exact scripts for this server session?\n{scripts}",
                inspection.package
            ))
            .await
            .map_err(|error| project_open_error("project script trust was not granted", error))?
            .ok_or_else(|| {
                project_open_error(
                    "project script trust returned no decision",
                    "the project remains unopened",
                )
            })?;
        if !decision.trust_project_scripts {
            return Err(project_open_error(
                "project scripts were not trusted",
                "the project remains unopened",
            ));
        }
        Ok(())
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
    root_id: Option<String>,
    relative_manifest: Option<String>,
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

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct ProjectSelection {
    #[schemars(description = "Root ID from sora_project_list")]
    root_id: String,
    #[schemars(description = "Root-relative project.toml path from sora_project_list")]
    relative_manifest: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct ProjectTrustDecision {
    #[schemars(description = "True only after the user reviewed every listed script and digest")]
    trust_project_scripts: bool,
}

rmcp::elicit_safe!(ProjectSelection, ProjectTrustDecision);

fn project_open_error(
    summary: impl Into<String>,
    error: impl ToString,
) -> rmcp::model::CallToolResult {
    tool_error(ToolEnvelope::<ProjectOpenOutput>::failure(
        None, None, summary, error,
    ))
}
