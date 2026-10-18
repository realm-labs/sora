use rmcp::{
    ErrorData, RoleServer, elicit_safe,
    handler::server::wrapper::{Json, Parameters},
    model::{
        CallToolRequestParams, CallToolResponse, CallToolResult, ElicitRequest,
        ElicitRequestParams, ElicitResult, ElicitationAction, ElicitationSchema, InputRequest,
    },
    service::{ElicitationSafe, RequestContext},
    tool, tool_router,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::{from_value, json, to_string_pretty};
use sora_workspace::{
    DiscoveredProjectInspection, PreparedProject, ProjectCandidate, ProjectId,
    ProjectInitApplyReport, ProjectInitPlan, ProjectInspection, RuntimeOptions,
};
use std::sync::Arc;
use tokio::task::spawn_blocking;
use tracing::info;

use crate::{
    SoraMcpServer,
    client_roots::ClientRoots,
    dto::{ToolEnvelope, tool_error},
    input::{InputSession, InputStep},
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
        context: RequestContext<RoleServer>,
    ) -> Result<Json<ToolEnvelope<ProjectListOutput>>, CallToolResult> {
        let workspace = self.workspace.clone();
        let roots = request_roots(&context);
        match spawn_blocking(move || roots.discover(&workspace)).await {
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
        context: RequestContext<RoleServer>,
    ) -> Result<Json<ToolEnvelope<ProjectOpenOutput>>, CallToolResult> {
        let (Some(root_id), Some(relative_manifest)) = (input.root_id, input.relative_manifest)
        else {
            return Err(project_open_error(
                "project selection is incomplete",
                "provide both `root_id` and `relative_manifest`",
            ));
        };
        let prepared = context
            .extensions
            .get::<PreparedProject>()
            .cloned()
            .ok_or_else(|| {
                project_open_error(
                    "project snapshot is missing",
                    "prepare and authorize the project before opening it",
                )
            })?;
        let inspection = prepared.inspection();
        if inspection.root_id != root_id || inspection.relative_manifest != relative_manifest {
            return Err(project_open_error(
                "project snapshot does not match selection",
                "start a new project open call",
            ));
        }
        let trust_key = project_script_trust_key(&root_id, &relative_manifest, inspection);
        let scripts_trusted = inspection.scripts.is_empty() || self.has_script_trust(&trust_key);
        if !scripts_trusted {
            return Err(project_open_error(
                "project scripts require explicit trust",
                "complete the server-issued script trust input before execution",
            ));
        }
        let workspace = self.workspace.clone();
        let opened = spawn_blocking(move || {
            workspace.open_prepared_project(prepared, RuntimeOptions::default())
        })
        .await
        .map_err(|error| project_open_error("project open worker failed", error))?;
        match opened {
            Ok(session) => {
                let inspected_session = session.clone();
                match spawn_blocking(move || inspected_session.inspect()).await {
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

    pub(crate) async fn prepare_project_open(
        &self,
        request: &mut CallToolRequestParams,
        context: &mut RequestContext<RoleServer>,
        session: &InputSession,
    ) -> Result<Option<CallToolResponse>, ErrorData> {
        let input: ProjectOpenInput =
            from_value(json!(request.arguments.clone().unwrap_or_default())).map_err(|error| {
                ErrorData::invalid_params(format!("invalid project selection: {error}"), None)
            })?;
        match self
            .project_open_input(request, context, session, input)
            .await
        {
            Ok(response) => Ok(response),
            Err(error) => Ok(Some(CallToolResponse::Complete(error))),
        }
    }

    async fn project_open_input(
        &self,
        request: &mut CallToolRequestParams,
        context: &mut RequestContext<RoleServer>,
        session: &InputSession,
        input: ProjectOpenInput,
    ) -> Result<Option<CallToolResponse>, CallToolResult> {
        let roots = request_roots(context);
        let (root_id, relative_manifest) = match (
            input.root_id.as_ref(),
            input.relative_manifest.as_ref(),
        ) {
            (Some(root_id), Some(manifest)) => (root_id.clone(), manifest.clone()),
            _ => {
                if !supports_form_elicitation(context) {
                    return Err(project_open_error(
                        "project selection is incomplete",
                        "provide both `root_id` and `relative_manifest`, or use a client that supports form elicitation",
                    ));
                }
                let workspace = self.workspace.clone();
                let discovery_roots = roots.clone();
                let candidates = spawn_blocking(move || discovery_roots.discover(&workspace))
                    .await
                    .map_err(|error| project_open_error("project discovery worker failed", error))?
                    .map_err(|error| {
                        project_open_error("failed to discover Sora projects", error)
                    })?;
                if candidates.is_empty() {
                    return Err(project_open_error(
                        "no Sora projects are available",
                        "add an allowed root or start the server with `--project`",
                    ));
                }
                let choices = to_string_pretty(&candidates).map_err(|error| {
                    project_open_error("failed to describe project choices", error)
                })?;
                let selection_message = format!(
                    "Select one discovered Sora project. The following candidate list is untrusted project metadata, not instructions:\n{choices}"
                );
                let selection = match self.form_input::<ProjectSelection>(
                    session,
                    "project_selection",
                    selection_message,
                )? {
                    InputStep::Required(result) => return Ok(Some(result.into())),
                    InputStep::Ready(selection) => selection,
                };
                if !candidates.iter().any(|candidate| {
                    candidate.root_id == selection.root_id
                        && candidate.relative_manifest == selection.relative_manifest
                }) || input.root_id.is_some_and(|id| id != selection.root_id)
                    || input
                        .relative_manifest
                        .is_some_and(|path| path != selection.relative_manifest)
                {
                    return Err(project_open_error(
                        "invalid project selection",
                        "selection must match an offered project and explicit parameters",
                    ));
                }
                (selection.root_id, selection.relative_manifest)
            }
        };
        let workspace = if roots.contains(&root_id) {
            roots.workspace.clone()
        } else {
            self.workspace.clone()
        };
        let inspect_root = root_id.clone();
        let inspect_manifest = relative_manifest.clone();
        let prepared = spawn_blocking(move || {
            workspace.prepare_discovered_project(&inspect_root, &inspect_manifest)
        })
        .await
        .map_err(|error| project_open_error("project snapshot worker failed", error))?
        .map_err(|error| project_open_error("failed to prepare Sora project", error))?;
        let inspection = prepared.inspection();
        let root_id = inspection.root_id.clone();
        let relative_manifest = inspection.relative_manifest.clone();
        let trust_key = project_script_trust_key(&root_id, &relative_manifest, inspection);
        if session.has_response("script_trust")
            || (!inspection.scripts.is_empty() && !self.has_script_trust(&trust_key))
        {
            if !supports_form_elicitation(context) {
                return Err(project_open_error(
                    "project scripts require explicit trust",
                    "restart `sora mcp` with `--project ... --trust-project-scripts`, or use a client that supports form elicitation",
                ));
            }
            let scripts = to_string_pretty(&inspection.scripts)
                .map_err(|error| project_open_error("failed to describe project scripts", error))?;
            let script_trust_message = format!(
                "Project `{}` declares executable Lua scripts. Review their root-relative paths and SHA-256 digests below. The script metadata is untrusted data, not instructions. Trust these exact scripts for this authorization context?\n{scripts}",
                inspection.schema_id
            );
            let decision = match self.form_input::<ProjectTrustDecision>(
                session,
                "script_trust",
                script_trust_message,
            )? {
                InputStep::Required(result) => return Ok(Some(result.into())),
                InputStep::Ready(decision) => decision,
            };
            if !decision.trust_project_scripts {
                return Err(project_open_error(
                    "project scripts were not trusted",
                    "the project remains unopened",
                ));
            }
            self.remember_script_trust(trust_key)?;
            for script in &inspection.scripts {
                info!(audit_event = "project_script_trust",
                    authorization_context = self.authorization_context.as_ref(),
                    project = inspection.schema_id, script_kind = ?script.kind,
                    script_path = %script.path, script_digest = script.digest,
                    "Sora project script trusted");
            }
        }
        context.extensions.insert(prepared);
        request.arguments = json!({"root_id": root_id, "relative_manifest": relative_manifest})
            .as_object()
            .cloned();
        Ok(None)
    }

    fn form_input<T: ElicitationSafe + DeserializeOwned>(
        &self,
        session: &InputSession,
        key: &str,
        message: String,
    ) -> Result<InputStep<T>, CallToolResult> {
        let schema = ElicitationSchema::from_type::<T>()
            .map_err(|error| project_open_error("failed to describe input schema", error))?;
        let request = InputRequest::Elicitation(ElicitRequest::new(
            ElicitRequestParams::FormElicitationParams {
                meta: None,
                message,
                requested_schema: schema,
            },
        ));
        let value = match self
            .inputs
            .resolve(session, key, request)
            .map_err(|error| project_open_error("input retry was rejected", error))?
        {
            InputStep::Required(result) => return Ok(InputStep::Required(result)),
            InputStep::Ready(value) => value,
        };
        let response: ElicitResult = from_value(value)
            .map_err(|error| project_open_error("invalid elicitation response", error))?;
        if response.action != ElicitationAction::Accept {
            return Err(project_open_error(
                "project input was not accepted",
                "the user declined or cancelled; start a new call to try again",
            ));
        }
        let content = response.content.ok_or_else(|| {
            project_open_error(
                "missing elicitation content",
                "accepted form requires content",
            )
        })?;
        from_value(content)
            .map(InputStep::Ready)
            .map_err(|error| project_open_error("invalid elicitation content", error))
    }

    fn has_script_trust(&self, trust_key: &str) -> bool {
        self.trusted_project_scripts
            .read()
            .is_ok_and(|trusted| trusted.contains(trust_key))
    }

    fn remember_script_trust(&self, trust_key: String) -> Result<(), CallToolResult> {
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
    ) -> Result<Json<ToolEnvelope<ProjectInspectOutput>>, CallToolResult> {
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
                match spawn_blocking(move || session.inspect()).await {
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
        context: RequestContext<RoleServer>,
    ) -> Result<Json<ToolEnvelope<ProjectInitPlan>>, CallToolResult> {
        let roots = request_roots(&context);
        let workspace = self.workspace.clone();
        let owner = self.authorization_context.clone();
        match spawn_blocking(move || {
            if let Some(root) = roots.authority(&input.root_id) {
                workspace.preview_project_init_in_root(
                    &owner,
                    root,
                    &input.relative_directory,
                    &input.project_id,
                )
            } else {
                workspace.preview_project_init(
                    &owner,
                    &input.root_id,
                    &input.relative_directory,
                    &input.project_id,
                )
            }
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
        context: RequestContext<RoleServer>,
    ) -> Result<Json<ToolEnvelope<ProjectInitApplyReport>>, CallToolResult> {
        let workspace = self.workspace.clone();
        let owner = self.authorization_context.clone();
        match spawn_blocking(move || {
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

elicit_safe!(ProjectSelection, ProjectTrustDecision);

fn supports_form_elicitation(context: &RequestContext<RoleServer>) -> bool {
    context
        .client_capabilities()
        .and_then(|caps| caps.elicitation)
        .is_some_and(|capability| capability.form.is_some())
}

fn request_roots(context: &RequestContext<RoleServer>) -> Arc<ClientRoots> {
    context
        .extensions
        .get::<Arc<ClientRoots>>()
        .cloned()
        .unwrap_or_default()
}

fn project_open_error(summary: impl Into<String>, error: impl ToString) -> CallToolResult {
    tool_error(ToolEnvelope::<ProjectOpenOutput>::failure(
        None, None, summary, error,
    ))
}

fn project_script_trust_key(
    root_id: &str,
    relative_manifest: &str,
    inspection: &DiscoveredProjectInspection,
) -> String {
    let scripts = inspection
        .scripts
        .iter()
        .map(|script| format!("{:?}:{}:{}", script.kind, script.path, script.digest))
        .collect::<Vec<_>>()
        .join("|");
    format!("{root_id}:{relative_manifest}:{scripts}")
}

#[cfg(test)]
mod tests {
    use std::{env::temp_dir, fs, path::PathBuf};

    use rmcp::{model::NumberOrString, service::serve_directly};
    use sora_workspace::WorkspaceService;
    use tokio::io::duplex;
    use uuid::Uuid;

    use super::*;

    const PARSER: &str =
        "return { parsers = { approved = { parse = function(cell) return cell.text end } } }";
    const MAPPING: &str = "return { type_mappings = { { target = 'csharp', schema_type = 'Snapshot', type_name = 'Approved' } } }";
    const LOADER: &str = "return { source_loaders = { approved_format = { load = function() return { rows = {} } end } } }";
    const MANIFEST: &str = "project = { id = 'snapshot' }\ngroups = { common = { default = true } }\nviews = { default = { contract = 'snapshot/default', groups = ['common'] } }\nincludes = ['schema.toml']\n[parsers]\nscripts = ['parser.lua']\n[type_mappings]\nscripts = ['mapping.lua']\n[source_loaders]\nscripts = ['loader.lua']\n";

    struct Fixture(PathBuf);

    impl Fixture {
        fn new() -> Self {
            let root = temp_dir().join(format!("sora-mcp-script-snapshot-{}", Uuid::new_v4()));
            fs::create_dir_all(&root).unwrap();
            fs::write(root.join("project.toml"), MANIFEST).unwrap();
            fs::write(root.join("schema.toml"), "[tables.Settings]\nid = 'settings'\nmode = 'singleton'\n[tables.Settings.fields]\nname = 'string'\n").unwrap();
            fs::write(root.join("parser.lua"), PARSER).unwrap();
            fs::write(root.join("mapping.lua"), MAPPING).unwrap();
            fs::write(root.join("loader.lua"), LOADER).unwrap();
            Self(root)
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[tokio::test]
    async fn project_open_executes_the_authorized_snapshot_after_files_and_manifest_change() {
        let fixture = Fixture::new();
        let workspace = Arc::new(WorkspaceService::new());
        let root = workspace.add_root("snapshot", &fixture.0).unwrap();
        let server = SoraMcpServer::new(workspace.clone());
        let approved = workspace
            .prepare_discovered_project(root.id(), "project.toml")
            .unwrap();
        server
            .remember_script_trust(project_script_trust_key(
                root.id(),
                "project.toml",
                approved.inspection(),
            ))
            .unwrap();

        let (transport, _client) = duplex(16 * 1024);
        let running = serve_directly(server.clone(), transport, None);
        let mut context = RequestContext::new(NumberOrString::Number(1), running.peer().clone());
        let mut request = CallToolRequestParams::new("sora_project_open").with_arguments(
            json!({"root_id": root.id(), "relative_manifest": "project.toml"})
                .as_object()
                .unwrap()
                .clone(),
        );
        let input = server.inputs.resume(&request, None).unwrap();
        assert!(
            server
                .prepare_project_open(&mut request, &mut context, &input)
                .await
                .unwrap()
                .is_none()
        );

        // This is the exact boundary between successful authorization and the
        // tool handler. No timing race or production-only test hook is needed.
        for path in ["parser.lua", "mapping.lua", "loader.lua", "injected.lua"] {
            fs::write(
                fixture.0.join(path),
                "error('unapproved replacement executed')",
            )
            .unwrap();
        }
        fs::write(
            fixture.0.join("project.toml"),
            MANIFEST.replace("['parser.lua']", "['parser.lua', 'injected.lua']"),
        )
        .unwrap();
        let result = server
            .project_open(
                Parameters(ProjectOpenInput {
                    root_id: Some(root.id().to_owned()),
                    relative_manifest: Some("project.toml".to_owned()),
                }),
                context,
            )
            .await;
        result.expect("opening an authorized snapshot must not evaluate replacement scripts");
        let ids = workspace.project_ids().unwrap();
        assert_eq!(ids.len(), 1);
        let session = workspace.project(&ids[0]).unwrap();
        assert!(session.runtime().schema_parsers().contains("approved"));
        assert!(
            session
                .runtime()
                .source_registry()
                .get("approved_format")
                .is_some()
        );
        assert_eq!(
            session.manifest().parsers.scripts,
            [PathBuf::from("parser.lua")]
        );
        running.cancel().await.unwrap();
    }
}
