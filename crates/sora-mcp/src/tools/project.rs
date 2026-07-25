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
        description = "List Sora project manifests discovered inside the server's allowed roots",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn project_list(
        &self,
        Parameters(_input): Parameters<ProjectListInput>,
    ) -> Result<Json<ToolEnvelope<ProjectListOutput>>, rmcp::model::CallToolResult> {
        let workspace = self.workspace.clone();
        match tokio::task::spawn_blocking(move || workspace.discover_projects()).await {
            Ok(Ok(projects)) => {
                let count = projects.len();
                Ok(Json(ToolEnvelope::success(
                    None,
                    None,
                    format!("found {count} Sora project manifest(s)"),
                    ProjectListOutput { projects },
                )))
            }
            Ok(Err(error)) => Err(tool_error(ToolEnvelope::<ProjectListOutput>::failure(
                None,
                None,
                "failed to discover Sora projects",
                error,
            ))),
            Err(error) => Err(tool_error(ToolEnvelope::<ProjectListOutput>::failure(
                None,
                None,
                "project discovery worker failed",
                error,
            ))),
        }
    }

    #[tool(
        name = "sora_project_open",
        description = "Open one discovered Sora project by root id and root-relative manifest path",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn project_open(
        &self,
        Parameters(input): Parameters<ProjectOpenInput>,
        context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<Json<ToolEnvelope<ProjectOpenOutput>>, rmcp::model::CallToolResult> {
        let (root_id, relative_manifest) =
            self.resolve_project_selection(input, &context.peer).await?;
        let workspace = self.workspace.clone();
        let inspect_root = root_id.clone();
        let inspect_manifest = relative_manifest.clone();
        let inspection = tokio::task::spawn_blocking(move || {
            workspace.inspect_discovered_project(&inspect_root, &inspect_manifest)
        })
        .await
        .map_err(|error| project_open_error("project inspection worker failed", error))?
        .map_err(|error| project_open_error("failed to inspect Sora project", error))?;
        let trust_key = project_script_trust_key(&root_id, &relative_manifest, &inspection);
        let scripts_trusted = inspection.scripts.is_empty() || self.has_script_trust(&trust_key);
        if !scripts_trusted {
            self.request_project_script_trust(&inspection, &context.peer)
                .await?;
            self.remember_script_trust(trust_key)?;
            for script in &inspection.scripts {
                tracing::info!(
                    audit_event = "project_script_trust",
                    authorization_context = self.authorization_context.as_ref(),
                    project = inspection.schema_id,
                    script_kind = ?script.kind,
                    script_path = %script.path,
                    script_digest = script.digest,
                    "Sora project script trusted"
                );
            }
        }
        let workspace = self.workspace.clone();
        let open_root = root_id.clone();
        let open_manifest = relative_manifest.clone();
        let opened = tokio::task::spawn_blocking(move || {
            workspace.open_discovered_project(
                &open_root,
                &open_manifest,
                RuntimeOptions::default(),
                true,
            )
        })
        .await
        .map_err(|error| project_open_error("project open worker failed", error))?;
        match opened {
            Ok(session) => {
                let inspected_session = session.clone();
                match tokio::task::spawn_blocking(move || inspected_session.inspect()).await {
                    Ok(Ok(project)) => Ok(Json(ToolEnvelope::success(
                        Some(session.id().clone()),
                        Some(session.revision()),
                        format!("opened Sora project `{}`", project.schema_id),
                        ProjectOpenOutput { project },
                    ))),
                    Ok(Err(error)) => Err(project_open_error(
                        "project opened but inspection failed",
                        error,
                    )),
                    Err(error) => Err(project_open_error(
                        "opened project inspection worker failed",
                        error,
                    )),
                }
            }
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
        let workspace = self.workspace.clone();
        let candidates = tokio::task::spawn_blocking(move || workspace.discover_projects())
            .await
            .map_err(|error| project_open_error("project discovery worker failed", error))?
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
                inspection.schema_id
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

    fn has_script_trust(&self, trust_key: &str) -> bool {
        self.trusted_project_scripts
            .read()
            .is_ok_and(|trusted| trusted.contains(trust_key))
    }

    fn remember_script_trust(&self, trust_key: String) -> Result<(), rmcp::model::CallToolResult> {
        self.trusted_project_scripts
            .write()
            .map_err(|_| {
                project_open_error(
                    "failed to record project script trust",
                    "script trust state lock is poisoned",
                )
            })?
            .insert(trust_key);
        Ok(())
    }

    #[tool(
        name = "sora_project_inspect",
        description = "Inspect a registered project's schema sources, data sources, groups, views, and build capabilities",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn project_inspect(
        &self,
        Parameters(input): Parameters<ProjectInput>,
    ) -> Result<Json<ToolEnvelope<ProjectInspectOutput>>, rmcp::model::CallToolResult> {
        let id = ProjectId::new(input.project_id).map_err(|error| {
            tool_error(ToolEnvelope::<ProjectInspectOutput>::failure(
                None,
                None,
                "invalid project id",
                error,
            ))
        })?;
        match self.workspace.project(&id) {
            Ok(session) => {
                let revision = session.revision();
                match tokio::task::spawn_blocking(move || session.inspect()).await {
                    Ok(Ok(project)) => Ok(Json(ToolEnvelope::success(
                        Some(id),
                        Some(revision),
                        format!("inspected Sora project `{}`", project.schema_id),
                        ProjectInspectOutput { project },
                    ))),
                    Ok(Err(error)) => {
                        Err(tool_error(ToolEnvelope::<ProjectInspectOutput>::failure(
                            Some(id),
                            Some(revision),
                            "failed to inspect Sora project",
                            error,
                        )))
                    }
                    Err(error) => Err(tool_error(ToolEnvelope::<ProjectInspectOutput>::failure(
                        Some(id),
                        Some(revision),
                        "project inspection worker failed",
                        error,
                    ))),
                }
            }
            Err(error) => Err(tool_error(ToolEnvelope::<ProjectInspectOutput>::failure(
                Some(id),
                None,
                "unknown Sora project",
                error,
            ))),
        }
    }

    #[tool(
        name = "sora_project_init",
        description = "Preview a new project scaffold inside an allowed root; this operation never writes files",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn project_init(
        &self,
        Parameters(input): Parameters<ProjectInitInput>,
    ) -> Result<Json<ToolEnvelope<ProjectInitPlan>>, rmcp::model::CallToolResult> {
        let workspace = self.workspace.clone();
        let owner = self.authorization_context.clone();
        match tokio::task::spawn_blocking(move || {
            workspace.preview_project_init(
                &owner,
                &input.root_id,
                &input.relative_directory,
                &input.project_id,
            )
        })
        .await
        {
            Ok(Ok(plan)) => Ok(Json(ToolEnvelope::success(
                None,
                None,
                format!("planned {} new project file(s)", plan.files.len()),
                plan,
            ))),
            Ok(Err(error)) => Err(tool_error(ToolEnvelope::<ProjectInitPlan>::failure(
                None,
                None,
                "project initialization preview failed",
                error,
            ))),
            Err(error) => Err(tool_error(ToolEnvelope::<ProjectInitPlan>::failure(
                None,
                None,
                "project initialization preview worker failed",
                error,
            ))),
        }
    }

    #[tool(
        name = "sora_project_init_apply",
        description = "Atomically create and open a project from an unexpired initialization plan",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn project_init_apply(
        &self,
        Parameters(input): Parameters<ProjectInitApplyInput>,
        context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<Json<ToolEnvelope<ProjectInitApplyReport>>, rmcp::model::CallToolResult> {
        let workspace = self.workspace.clone();
        let owner = self.authorization_context.clone();
        match tokio::task::spawn_blocking(move || {
            workspace.apply_project_init(&owner, &input.plan_id, &input.idempotency_key)
        })
        .await
        {
            Ok(Ok(report)) => {
                let _ = context.peer.notify_resource_list_changed().await;
                Ok(Json(ToolEnvelope::success(
                    Some(report.project_id.clone()),
                    Some(report.revision.clone()),
                    format!("created Sora project `{}`", report.project_id),
                    report,
                )))
            }
            Ok(Err(error)) => Err(tool_error(ToolEnvelope::<ProjectInitApplyReport>::failure(
                None,
                None,
                "project initialization apply failed",
                error,
            ))),
            Err(error) => Err(tool_error(ToolEnvelope::<ProjectInitApplyReport>::failure(
                None,
                None,
                "project initialization apply worker failed",
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
    project_id: String,
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

fn project_script_trust_key(
    root_id: &str,
    relative_manifest: &str,
    inspection: &sora_workspace::DiscoveredProjectInspection,
) -> String {
    let scripts = inspection
        .scripts
        .iter()
        .map(|script| format!("{:?}:{}:{}", script.kind, script.path, script.digest))
        .collect::<Vec<_>>()
        .join("|");
    format!("{root_id}:{relative_manifest}:{scripts}")
}
