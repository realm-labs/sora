use std::{path::Path, sync::Arc};

use rmcp::{
    handler::server::wrapper::{Json, Parameters},
    tool, tool_router,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sora_workspace::{
    BuildArtifactKind, BuildControl, BuildPhase, BuildRequest, ProjectId, ProjectRevision,
};

use crate::{
    SoraMcpServer,
    artifact_store::ArtifactDescriptor,
    dto::{ArtifactLink, ToolEnvelope, tool_error},
};

#[tool_router(router = build_tool_router, vis = "pub(crate)")]
impl SoraMcpServer {
    #[tool(
        name = "sora_build",
        description = "Run selected outputs from the manifest build graph through an isolated staging area and atomic commit",
        annotations(
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = true,
            open_world_hint = false
        ),
        execution(task_support = "optional")
    )]
    async fn build(
        &self,
        Parameters(input): Parameters<BuildInput>,
        context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<Json<ToolEnvelope<BuildToolReport>>, rmcp::model::CallToolResult> {
        let groups = if input.outputs.is_empty() {
            vec![
                BuildOutputGroup::SchemaLock,
                BuildOutputGroup::ExcelTemplates,
                BuildOutputGroup::Codegen,
                BuildOutputGroup::Exports,
            ]
        } else {
            input.outputs
        };
        self.execute_build(
            BuildInvocation {
                project_id: input.project_id,
                expected_project_revision: input.expected_project_revision,
                view: input.view,
                clean: input.clean,
                include_schema_lock: groups.contains(&BuildOutputGroup::SchemaLock),
                include_excel_templates: groups.contains(&BuildOutputGroup::ExcelTemplates),
                include_codegen: groups.contains(&BuildOutputGroup::Codegen),
                include_exports: groups.contains(&BuildOutputGroup::Exports),
                targets: input.targets,
                export_formats: input.export_formats,
            },
            context,
        )
        .await
    }

    #[tool(
        name = "sora_codegen",
        description = "Generate one manifest-declared language target using its configured output, runtime format, and formatter mode",
        annotations(
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = true,
            open_world_hint = false
        ),
        execution(task_support = "optional")
    )]
    async fn codegen(
        &self,
        Parameters(input): Parameters<CodegenInput>,
        context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<Json<ToolEnvelope<BuildToolReport>>, rmcp::model::CallToolResult> {
        self.execute_build(
            BuildInvocation {
                project_id: input.project_id,
                expected_project_revision: input.expected_project_revision,
                view: input.view,
                clean: input.clean,
                include_schema_lock: false,
                include_excel_templates: false,
                include_codegen: true,
                include_exports: false,
                targets: vec![input.target],
                export_formats: Vec::new(),
            },
            context,
        )
        .await
    }

    #[tool(
        name = "sora_export",
        description = "Export validated data through one manifest-declared exporter format and its declared output kind",
        annotations(
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = true,
            open_world_hint = false
        ),
        execution(task_support = "optional")
    )]
    async fn export(
        &self,
        Parameters(input): Parameters<ExportInput>,
        context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<Json<ToolEnvelope<BuildToolReport>>, rmcp::model::CallToolResult> {
        self.execute_build(
            BuildInvocation {
                project_id: input.project_id,
                expected_project_revision: input.expected_project_revision,
                view: input.view,
                clean: input.clean,
                include_schema_lock: false,
                include_excel_templates: false,
                include_codegen: false,
                include_exports: true,
                targets: Vec::new(),
                export_formats: vec![input.format],
            },
            context,
        )
        .await
    }

    #[tool(
        name = "sora_schema_lock",
        description = "Generate the manifest-declared schema lock through the build transaction",
        annotations(
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = true,
            open_world_hint = false
        ),
        execution(task_support = "optional")
    )]
    async fn schema_lock(
        &self,
        Parameters(input): Parameters<GeneratedOutputInput>,
        context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<Json<ToolEnvelope<BuildToolReport>>, rmcp::model::CallToolResult> {
        self.execute_build(
            BuildInvocation {
                project_id: input.project_id,
                expected_project_revision: input.expected_project_revision,
                view: input.view,
                clean: input.clean,
                include_schema_lock: true,
                include_excel_templates: false,
                include_codegen: false,
                include_exports: false,
                targets: Vec::new(),
                export_formats: Vec::new(),
            },
            context,
        )
        .await
    }

    #[tool(
        name = "sora_excel_template",
        description = "Generate manifest-declared Excel templates through the build transaction",
        annotations(
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = true,
            open_world_hint = false
        ),
        execution(task_support = "optional")
    )]
    async fn excel_template(
        &self,
        Parameters(input): Parameters<GeneratedOutputInput>,
        context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<Json<ToolEnvelope<BuildToolReport>>, rmcp::model::CallToolResult> {
        self.execute_build(
            BuildInvocation {
                project_id: input.project_id,
                expected_project_revision: input.expected_project_revision,
                view: input.view,
                clean: input.clean,
                include_schema_lock: false,
                include_excel_templates: true,
                include_codegen: false,
                include_exports: false,
                targets: Vec::new(),
                export_formats: Vec::new(),
            },
            context,
        )
        .await
    }

    async fn execute_build(
        &self,
        input: BuildInvocation,
        context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<Json<ToolEnvelope<BuildToolReport>>, rmcp::model::CallToolResult> {
        let id = ProjectId::new(input.project_id).map_err(|error| {
            tool_error(ToolEnvelope::<BuildToolReport>::failure(
                None,
                None,
                "invalid project id",
                error,
            ))
        })?;
        let session = self.workspace.project(&id).map_err(|error| {
            tool_error(ToolEnvelope::<BuildToolReport>::failure(
                Some(id.clone()),
                None,
                "unknown Sora project",
                error,
            ))
        })?;
        let revision = session.revision();
        if revision.project != input.expected_project_revision {
            return Err(tool_error(ToolEnvelope::<BuildToolReport>::failure(
                Some(id),
                Some(revision),
                "project revision conflict",
                "expected project revision does not match the current project",
            )));
        }

        let (progress_tx, mut progress_rx) = tokio::sync::mpsc::unbounded_channel();
        let request_cancellation = context.ct.clone();
        let control = BuildControl::default()
            .cancel_when(move || request_cancellation.is_cancelled())
            .on_progress(move |progress| {
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
                            .with_message(phase_name(progress.phase)),
                        )
                        .await;
                }
            }
        });

        let project_root = session
            .manifest_path()
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf();
        let project_path = session.manifest_path().to_path_buf();
        let runtime = session.runtime().clone();
        let artifact_store = Arc::clone(&self.artifacts);
        let owner = self.authorization_context.to_string();
        let worker_id = id.clone();
        let worker_revision = revision.clone();
        let build_result = tokio::task::spawn_blocking(move || {
            let report = sora_workspace::build_project_with_control(
                BuildRequest {
                    project: project_path,
                    default_source_format: None,
                    data_root: None,
                    view: input.view,
                    include_schema_lock: input.include_schema_lock,
                    include_excel_templates: input.include_excel_templates,
                    include_codegen: input.include_codegen,
                    include_exports: input.include_exports,
                    targets: input.targets,
                    export_formats: input.export_formats,
                    clean: input.clean,
                },
                &runtime,
                &control,
            )?;
            let descriptors =
                artifact_store.register_build(&owner, &worker_id, &project_root, &report)?;
            let output = build_tool_report(&project_root, &report, worker_revision);
            Ok::<_, anyhow::Error>((output, descriptors))
        })
        .await;
        let _ = progress_task.await;

        match build_result {
            Ok(Ok((report, descriptors))) => {
                self.notify_project_resources_updated(&context.peer, id.as_str())
                    .await;
                let links: Vec<ArtifactLink> = descriptors
                    .into_iter()
                    .map(|artifact| artifact_link(&id, artifact))
                    .collect();
                let mut envelope = ToolEnvelope::success(
                    Some(id),
                    Some(revision),
                    format!("generated {} build artifact(s)", links.len()),
                    report,
                );
                envelope.artifacts = links;
                Ok(Json(envelope))
            }
            Ok(Err(error)) => Err(tool_error(ToolEnvelope::<BuildToolReport>::failure(
                Some(id),
                Some(revision),
                "build failed",
                error,
            ))),
            Err(error) => Err(tool_error(ToolEnvelope::<BuildToolReport>::failure(
                Some(id),
                Some(revision),
                "build worker failed",
                error,
            ))),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
enum BuildOutputGroup {
    SchemaLock,
    ExcelTemplates,
    Codegen,
    Exports,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct BuildInput {
    project_id: String,
    expected_project_revision: String,
    view: Option<String>,
    #[serde(default)]
    outputs: Vec<BuildOutputGroup>,
    #[serde(default)]
    targets: Vec<String>,
    #[serde(default)]
    export_formats: Vec<String>,
    #[serde(default)]
    clean: bool,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct CodegenInput {
    project_id: String,
    expected_project_revision: String,
    target: String,
    view: Option<String>,
    #[serde(default)]
    clean: bool,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct ExportInput {
    project_id: String,
    expected_project_revision: String,
    format: String,
    view: Option<String>,
    #[serde(default)]
    clean: bool,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct GeneratedOutputInput {
    project_id: String,
    expected_project_revision: String,
    view: Option<String>,
    #[serde(default)]
    clean: bool,
}

struct BuildInvocation {
    project_id: String,
    expected_project_revision: String,
    view: Option<String>,
    clean: bool,
    include_schema_lock: bool,
    include_excel_templates: bool,
    include_codegen: bool,
    include_exports: bool,
    targets: Vec<String>,
    export_formats: Vec<String>,
}

#[derive(Debug, Serialize, JsonSchema)]
struct BuildToolReport {
    revision: ProjectRevision,
    outputs: Vec<BuildOutput>,
}

#[derive(Debug, Serialize, JsonSchema)]
struct BuildOutput {
    kind: BuildArtifactKind,
    path: String,
}

fn build_tool_report(
    project_root: &Path,
    report: &sora_workspace::BuildReport,
    revision: ProjectRevision,
) -> BuildToolReport {
    BuildToolReport {
        revision,
        outputs: report
            .artifacts
            .iter()
            .map(|artifact| BuildOutput {
                kind: artifact.kind.clone(),
                path: artifact
                    .path
                    .strip_prefix(project_root)
                    .unwrap_or(&artifact.path)
                    .to_string_lossy()
                    .replace('\\', "/"),
            })
            .collect(),
    }
}

fn artifact_link(project_id: &ProjectId, artifact: ArtifactDescriptor) -> ArtifactLink {
    ArtifactLink {
        uri: format!(
            "sora://project/{project_id}/artifact/{}",
            artifact.artifact_id
        ),
        artifact_id: artifact.artifact_id,
        mime_type: artifact.mime_type,
        name: Some(artifact.name),
        size: Some(artifact.size),
    }
}

const fn phase_name(phase: BuildPhase) -> &'static str {
    match phase {
        BuildPhase::LoadManifest => "load_manifest",
        BuildPhase::LoadSchema => "load_schema",
        BuildPhase::NormalizeSchema => "normalize_schema",
        BuildPhase::LoadData => "load_data",
        BuildPhase::ValidateData => "validate_data",
        BuildPhase::PlanOutputs => "plan_outputs",
        BuildPhase::Generate => "generate",
        BuildPhase::Format => "format",
        BuildPhase::Export => "export",
        BuildPhase::Commit => "commit",
    }
}
