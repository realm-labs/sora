use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use anyhow::{Context, Result, bail};
use schemars::JsonSchema;
use serde::Serialize;
use sora_codegen::{
    format::FormatMode,
    generator::{CodegenRegistry, empty_options, runtime_format_name},
    options::RuntimeFormat,
};
use sora_export::exporter::ExportOptions;
use sora_input::traits::SchemaInput;
use sora_input_schema::input::SchemaFileInput;
use sora_schema::model::CodegenSchema;
use uuid::Uuid;

mod manifest;

use crate::{
    ProjectRuntime,
    mutation::{FileWrite, commit_file_transaction},
    source::MixedProjectInput,
};
pub use manifest::{
    BuildCodegen, BuildConfig, BuildExport, CodeFormatMode, ExportCompression, ProjectManifest,
    ScriptConfig, SourceFormat,
};

#[derive(Debug, Clone)]
pub struct BuildRequest {
    pub project: PathBuf,
    pub default_source_format: Option<SourceFormat>,
    pub data_root: Option<PathBuf>,
    pub scope: Option<String>,
    pub include_schema_lock: bool,
    pub include_excel_templates: bool,
    pub include_codegen: bool,
    pub include_exports: bool,
    pub targets: Vec<String>,
    pub export_formats: Vec<String>,
    pub clean: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct BuildReport {
    pub artifacts: Vec<BuildArtifact>,
}

/// Stable build pipeline phases used by progress reporting and cancellation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BuildPhase {
    LoadManifest,
    LoadSchema,
    NormalizeSchema,
    LoadData,
    ValidateData,
    PlanOutputs,
    Generate,
    Format,
    Export,
    Commit,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct BuildProgress {
    pub phase: BuildPhase,
    pub completed: usize,
    pub total: usize,
}

type ProgressCallback = dyn Fn(BuildProgress) + Send + Sync;

/// Cooperative control shared with an in-flight build.
#[derive(Clone, Default)]
pub struct BuildControl {
    cancelled: Arc<AtomicBool>,
    progress: Option<Arc<ProgressCallback>>,
}

impl BuildControl {
    pub fn with_progress(progress: impl Fn(BuildProgress) + Send + Sync + 'static) -> Self {
        Self::default().on_progress(progress)
    }

    pub fn on_progress(mut self, progress: impl Fn(BuildProgress) + Send + Sync + 'static) -> Self {
        self.progress = Some(Arc::new(progress));
        self
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
    }

    fn checkpoint(&self, phase: BuildPhase) -> Result<()> {
        if self.is_cancelled() {
            return Err(
                sora_diagnostics::SoraError::OperationCancelled { operation: "build" }.into(),
            );
        }
        if let Some(progress) = &self.progress {
            progress(BuildProgress {
                phase,
                completed: phase_index(phase),
                total: 10,
            });
        }
        if self.is_cancelled() {
            return Err(
                sora_diagnostics::SoraError::OperationCancelled { operation: "build" }.into(),
            );
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct BuildArtifact {
    pub kind: BuildArtifactKind,
    pub path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BuildArtifactKind {
    SchemaLock,
    ExcelTemplates,
    Code { target: String },
    Export { format: String },
}

struct StagedOutput {
    final_path: PathBuf,
    staged_path: PathBuf,
    directory: bool,
}

struct SelectedBuild<'a> {
    schema_lock: Option<&'a PathBuf>,
    excel_templates: Option<&'a PathBuf>,
    codegen: Vec<&'a BuildCodegen>,
    exports: Vec<&'a BuildExport>,
}

impl From<CodeFormatMode> for FormatMode {
    fn from(value: CodeFormatMode) -> Self {
        match value {
            CodeFormatMode::Never => Self::Never,
            CodeFormatMode::Auto => Self::Auto,
            CodeFormatMode::Required => Self::Required,
        }
    }
}

pub fn build_project(args: BuildRequest, context: &ProjectRuntime) -> Result<BuildReport> {
    build_project_with_control(args, context, &BuildControl::default())
}

/// Builds through an isolated staging area and commits all outputs together.
pub fn build_project_with_control(
    args: BuildRequest,
    context: &ProjectRuntime,
    control: &BuildControl,
) -> Result<BuildReport> {
    control.checkpoint(BuildPhase::LoadManifest)?;
    let manifest = match context.manifest() {
        Some(manifest) => manifest.clone(),
        None => ProjectManifest::load(&args.project)?,
    };
    let build = manifest.build.clone();
    let project_dir = args.project.parent().unwrap_or_else(|| Path::new("."));
    let schema_input = SchemaFileInput::new(&args.project);
    let mut artifacts = Vec::new();

    let registry = CodegenRegistry::with_builtin_generators();
    let selected = select_build_outputs(&args, &build, &registry)?;

    if selected.schema_lock.is_none()
        && selected.excel_templates.is_none()
        && selected.codegen.is_empty()
        && selected.exports.is_empty()
    {
        bail!(
            "project `{}` does not select any declared build outputs",
            args.project.display()
        );
    }

    validate_export_formats(&selected.exports)?;
    validate_declared_outputs(project_dir, &selected)?;
    let stage_root = project_dir
        .join(".sora")
        .join("build-staging")
        .join(Uuid::new_v4().to_string());
    fs::create_dir_all(&stage_root).with_context(|| {
        format!(
            "failed to create build staging directory `{}`",
            stage_root.display()
        )
    })?;
    let result = build_project_staged(
        &args,
        context,
        control,
        &build,
        &selected,
        project_dir,
        &schema_input,
        &stage_root,
        &mut artifacts,
    );
    let cleanup_error = fs::remove_dir_all(&stage_root)
        .err()
        .filter(|error| error.kind() != std::io::ErrorKind::NotFound);
    match (result, cleanup_error) {
        (Ok(report), None) => Ok(report),
        (Ok(_), Some(error)) => Err(error).with_context(|| {
            format!(
                "failed to remove build staging directory `{}`",
                stage_root.display()
            )
        }),
        (Err(error), _) => Err(error),
    }
}

#[allow(clippy::too_many_arguments)]
fn build_project_staged(
    args: &BuildRequest,
    context: &ProjectRuntime,
    control: &BuildControl,
    build: &BuildConfig,
    selected: &SelectedBuild<'_>,
    project_dir: &Path,
    schema_input: &SchemaFileInput,
    stage_root: &Path,
    artifacts: &mut Vec<BuildArtifact>,
) -> Result<BuildReport> {
    let default_source_format = args.default_source_format.or(build.default_source_format);
    let data_root = args
        .data_root
        .as_ref()
        .or(build.data_root.as_ref())
        .cloned()
        .unwrap_or_else(|| PathBuf::from("data"));
    let data_root = resolve_project_path(project_dir, &data_root);
    let scope = args.scope.as_deref().or(build.scope.as_deref());
    let registry = CodegenRegistry::with_builtin_generators();
    let mut staged_outputs = Vec::new();
    control.checkpoint(BuildPhase::LoadSchema)?;
    let schema = schema_input.load_schema()?;
    let codegen_options = schema.codegen.clone();
    control.checkpoint(BuildPhase::NormalizeSchema)?;
    let ir = sora_ir::normalize::normalize_schema_with_parsers(schema, context.schema_parsers())
        .with_context(|| format!("failed to check project `{}`", args.project.display()))?;
    sora_ir::validate::validate_config_ir(&ir)
        .with_context(|| format!("failed to check project `{}`", args.project.display()))?;
    validate_codegen_runtime_exports(
        &selected.codegen,
        &build.exports,
        scope,
        &registry,
        &codegen_options,
    )?;

    control.checkpoint(BuildPhase::LoadData)?;
    let mut loaded = None;
    if !selected.exports.is_empty() {
        let project_input = MixedProjectInput::with_parser_registry(
            SchemaFileInput::new(&args.project),
            &data_root,
            default_source_format.map(SourceFormat::as_str),
            std::sync::Arc::clone(context.cell_parsers()),
        );
        let values = sora_core::pipeline::load_project_data_and_catalog_with_context_and_parsers(
            &project_input,
            context.execution(),
            context.schema_parsers(),
            context.cell_parsers(),
        )
        .with_context(|| {
            format!(
                "failed to load data from `{}` for project `{}`",
                data_root.display(),
                args.project.display()
            )
        })?;
        loaded = Some(values);
    }
    control.checkpoint(BuildPhase::ValidateData)?;
    control.checkpoint(BuildPhase::PlanOutputs)?;

    control.checkpoint(BuildPhase::Generate)?;
    if let Some(path) = selected.schema_lock {
        let path = resolve_declared_output(project_dir, path)?;
        let staged = staged_output_path(stage_root, staged_outputs.len());
        sora_core::pipeline::generate_schema_lock_with_scope_and_parsers(
            schema_input,
            &staged,
            scope,
            context.schema_parsers(),
        )
        .with_context(|| {
            format!(
                "failed to generate schema lock from `{}` into `{}`",
                args.project.display(),
                path.display()
            )
        })?;
        staged_outputs.push(StagedOutput {
            final_path: path.clone(),
            staged_path: staged,
            directory: false,
        });
        artifacts.push(BuildArtifact {
            kind: BuildArtifactKind::SchemaLock,
            path,
        });
    }

    if let Some(path) = selected.excel_templates {
        let path = resolve_declared_output(project_dir, path)?;
        let staged = staged_output_path(stage_root, staged_outputs.len());
        sora_core::pipeline::generate_excel_template_with_scope_and_parsers(
            schema_input,
            &staged,
            scope,
            context.schema_parsers(),
        )
        .with_context(|| {
            format!(
                "failed to generate Excel templates from `{}` into `{}`",
                args.project.display(),
                path.display()
            )
        })?;
        staged_outputs.push(StagedOutput {
            final_path: path.clone(),
            staged_path: staged,
            directory: true,
        });
        artifacts.push(BuildArtifact {
            kind: BuildArtifactKind::ExcelTemplates,
            path,
        });
    }

    for item in &selected.codegen {
        control.checkpoint(BuildPhase::Generate)?;
        let out = resolve_declared_output(project_dir, &item.out)?;
        let staged = staged_output_path(stage_root, staged_outputs.len());
        let item_scope = item.scope.as_deref().or(scope);
        sora_core::pipeline::generate_code_with_scope_format_parsers_and_cancellation(
            schema_input,
            &item.target,
            &staged,
            FormatMode::from(item.format),
            item_scope,
            context.schema_parsers(),
            context.type_mappings(),
            &|| control.is_cancelled(),
        )
        .with_context(|| {
            format!(
                "failed to generate {} code from `{}` into `{}`",
                item.target,
                args.project.display(),
                out.display()
            )
        })?;
        staged_outputs.push(StagedOutput {
            final_path: out.clone(),
            staged_path: staged,
            directory: true,
        });
        artifacts.push(BuildArtifact {
            kind: BuildArtifactKind::Code {
                target: item.target.clone(),
            },
            path: out,
        });
    }
    control.checkpoint(BuildPhase::Format)?;

    control.checkpoint(BuildPhase::Export)?;
    if let Some((ir, data, locale_catalog)) = loaded {
        for item in &selected.exports {
            control.checkpoint(BuildPhase::Export)?;
            let out = resolve_declared_output(project_dir, &item.out)?;
            let staged = staged_output_path(stage_root, staged_outputs.len());
            let item_scope = item.scope.as_deref().or(scope);
            let output = export_output(&item.format, staged.clone())?;
            sora_core::pipeline::export_loaded_data(sora_core::pipeline::LoadedDataExportRequest {
                ir: &ir,
                data: &data,
                locale_catalog: locale_catalog.as_ref(),
                format: &item.format,
                output,
                scope: item_scope,
                execution: context.execution(),
                options: export_options(item)?,
            })
            .with_context(|| {
                format!(
                    "failed to export `{}` data from `{}`",
                    item.format,
                    data_root.display()
                )
            })?;
            let directory = matches!(
                sora_core::pipeline::export_output_kind(&item.format),
                Some(sora_export::exporter::OutputKind::Directory)
            );
            staged_outputs.push(StagedOutput {
                final_path: out.clone(),
                staged_path: staged,
                directory,
            });
            artifacts.push(BuildArtifact {
                kind: BuildArtifactKind::Export {
                    format: item.format.clone(),
                },
                path: out,
            });
        }
    }

    control.checkpoint(BuildPhase::Commit)?;
    let writes = collect_output_writes(&staged_outputs, args.clean)?;
    if !writes.is_empty() {
        commit_file_transaction(project_dir, &writes, || Ok(()))
            .map_err(|error| anyhow::anyhow!("build output transaction failed: {error}"))?;
    }
    Ok(BuildReport {
        artifacts: std::mem::take(artifacts),
    })
}

fn validate_export_formats(exports: &[&BuildExport]) -> Result<()> {
    for item in exports {
        if sora_core::pipeline::export_output_kind(&item.format).is_none() {
            bail!(
                "unknown export format `{}`; supported formats: {}",
                item.format,
                sora_core::pipeline::supported_export_formats().join(", ")
            );
        }
    }
    Ok(())
}

fn select_build_outputs<'a>(
    args: &BuildRequest,
    build: &'a BuildConfig,
    registry: &CodegenRegistry,
) -> Result<SelectedBuild<'a>> {
    let codegen = if args.include_codegen {
        selected_codegen_targets(&build.codegen, &args.targets, registry)?
    } else {
        if !args.targets.is_empty() {
            bail!("codegen targets were provided while codegen output is disabled");
        }
        Vec::new()
    };
    let exports = if args.include_exports {
        selected_exports(&build.exports, &args.export_formats)?
    } else {
        if !args.export_formats.is_empty() {
            bail!("export formats were provided while export output is disabled");
        }
        Vec::new()
    };
    Ok(SelectedBuild {
        schema_lock: args
            .include_schema_lock
            .then_some(build.schema_lock.as_ref())
            .flatten(),
        excel_templates: args
            .include_excel_templates
            .then_some(build.excel_templates.as_ref())
            .flatten(),
        codegen,
        exports,
    })
}

fn selected_exports<'a>(
    configured: &'a [BuildExport],
    requested: &[String],
) -> Result<Vec<&'a BuildExport>> {
    if requested.is_empty() {
        return Ok(configured.iter().collect());
    }
    let requested = requested
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    for format in &requested {
        if !configured.iter().any(|item| item.format == *format) {
            bail!("export format `{format}` is not declared in [[build.exports]]");
        }
    }
    Ok(configured
        .iter()
        .filter(|item| requested.contains(item.format.as_str()))
        .collect())
}

fn validate_codegen_runtime_exports(
    codegen: &[&BuildCodegen],
    exports: &[BuildExport],
    build_scope: Option<&str>,
    registry: &CodegenRegistry,
    codegen_options: &CodegenSchema,
) -> Result<()> {
    for item in codegen {
        let generator = registry.get(&item.target).ok_or_else(|| {
            anyhow::anyhow!(
                "unknown codegen target `{}`; supported targets: {}",
                item.target,
                registry.supported_targets().join(", ")
            )
        })?;
        let canonical_target = registry.canonical_id(&item.target).unwrap_or(&item.target);
        let empty = empty_options();
        let options = codegen_options
            .target_options(canonical_target)
            .or_else(|| codegen_options.target_options(&item.target))
            .unwrap_or(&empty);
        let Some(runtime_format) = (generator.runtime_format)(canonical_target, options)? else {
            continue;
        };
        if !generator.supports_runtime_format(runtime_format) {
            let supported = generator
                .supported_runtime_formats()
                .iter()
                .map(|format| runtime_format_name(*format))
                .collect::<Vec<_>>()
                .join(", ");
            bail!(
                "{} codegen runtime_format `{}` is not supported; supported runtime_format: {}",
                item.target,
                runtime_format_name(runtime_format),
                supported
            );
        }

        if exports.is_empty() {
            continue;
        }

        let required_format = export_format_for_runtime(runtime_format);
        let item_scope = item.scope.as_deref().or(build_scope);
        let has_matching_export = exports.iter().any(|export| {
            export.format == required_format
                && export.scope.as_deref().or(build_scope) == item_scope
        });
        if !has_matching_export {
            let scope_message = item_scope
                .map(|scope| format!(" with scope `{scope}`"))
                .unwrap_or_else(|| " without scope".to_owned());
            bail!(
                "{} codegen uses runtime_format `{}` and requires a `{}` export{}",
                item.target,
                runtime_format_name(runtime_format),
                required_format,
                scope_message
            );
        }
    }
    Ok(())
}

fn export_format_for_runtime(runtime_format: RuntimeFormat) -> &'static str {
    match runtime_format {
        RuntimeFormat::Sora => "binary",
        RuntimeFormat::Json => "json",
        RuntimeFormat::SoraProtobuf => "sora-protobuf",
        RuntimeFormat::Cbor => "cbor",
    }
}

fn export_output(format: &str, out: PathBuf) -> Result<sora_export::exporter::ExportOutput> {
    match sora_core::pipeline::export_output_kind(format) {
        Some(sora_export::exporter::OutputKind::File) => {
            Ok(sora_export::exporter::ExportOutput::File(out))
        }
        Some(sora_export::exporter::OutputKind::Directory) => {
            Ok(sora_export::exporter::ExportOutput::Directory(out))
        }
        None => {
            bail!(
                "unknown export format `{}`; supported formats: {}",
                format,
                sora_core::pipeline::supported_export_formats().join(", ")
            );
        }
    }
}

fn export_options(item: &BuildExport) -> Result<ExportOptions> {
    let compression = match item.compression {
        ExportCompression::None => sora_export::exporter::ExportCompression::None,
        ExportCompression::Zstd => {
            if item.format != "binary" {
                bail!(
                    "export compression `zstd` is only supported by `binary` exports, got `{}`",
                    item.format
                );
            }
            sora_export::exporter::ExportCompression::Zstd {
                level: item.compression_level.unwrap_or(3),
            }
        }
    };
    Ok(ExportOptions {
        compression,
        locale: item.locale.clone(),
    })
}

fn selected_codegen_targets<'a>(
    configured: &'a [BuildCodegen],
    requested: &[String],
    registry: &CodegenRegistry,
) -> Result<Vec<&'a BuildCodegen>> {
    if requested.is_empty() {
        return Ok(configured.iter().collect());
    }

    let selected = configured
        .iter()
        .filter(|item| {
            requested
                .iter()
                .any(|target| codegen_targets_match(target, &item.target, registry))
        })
        .collect::<Vec<_>>();
    for target in requested {
        if !configured
            .iter()
            .any(|item| codegen_targets_match(target, &item.target, registry))
        {
            bail!(
                "build target `{}` was requested but is not declared in [[build.codegen]]",
                target
            );
        }
    }
    Ok(selected)
}

fn codegen_targets_match(left: &str, right: &str, registry: &CodegenRegistry) -> bool {
    match (registry.canonical_id(left), registry.canonical_id(right)) {
        (Some(left), Some(right)) => left == right,
        _ => left == right,
    }
}

fn validate_declared_outputs(project_dir: &Path, selected: &SelectedBuild<'_>) -> Result<()> {
    let mut outputs = Vec::new();
    if let Some(path) = selected.schema_lock {
        outputs.push(resolve_declared_output(project_dir, path)?);
    }
    if let Some(path) = selected.excel_templates {
        outputs.push(resolve_declared_output(project_dir, path)?);
    }
    for item in &selected.codegen {
        outputs.push(resolve_declared_output(project_dir, &item.out)?);
    }
    for item in &selected.exports {
        outputs.push(resolve_declared_output(project_dir, &item.out)?);
    }
    outputs.sort();
    for pair in outputs.windows(2) {
        if pair[0] == pair[1] || pair[1].starts_with(&pair[0]) {
            bail!(
                "declared build outputs overlap: `{}` and `{}`",
                pair[0].display(),
                pair[1].display()
            );
        }
    }
    Ok(())
}

fn resolve_declared_output(project_dir: &Path, declared: &Path) -> Result<PathBuf> {
    if declared.as_os_str().is_empty()
        || declared.is_absolute()
        || declared
            .components()
            .any(|component| !matches!(component, std::path::Component::Normal(_)))
    {
        bail!(
            "build output must be a non-empty project-relative path without traversal: `{}`",
            declared.display()
        );
    }
    let root = project_dir.canonicalize().with_context(|| {
        format!(
            "failed to resolve project directory `{}`",
            project_dir.display()
        )
    })?;
    let candidate = root.join(declared);
    let mut existing = candidate.as_path();
    while !existing.exists() {
        existing = existing.parent().ok_or_else(|| {
            anyhow::anyhow!(
                "build output has no resolvable parent: `{}`",
                declared.display()
            )
        })?;
    }
    let resolved_existing = existing.canonicalize().with_context(|| {
        format!(
            "failed to resolve build output ancestor `{}`",
            existing.display()
        )
    })?;
    if !resolved_existing.starts_with(&root) {
        bail!(
            "build output resolves outside the project: `{}`",
            declared.display()
        );
    }
    let unresolved = candidate
        .strip_prefix(existing)
        .map_err(|_| anyhow::anyhow!("failed to bound build output `{}`", declared.display()))?;
    Ok(resolved_existing.join(unresolved))
}

fn staged_output_path(stage_root: &Path, index: usize) -> PathBuf {
    stage_root.join(format!("output-{index}"))
}

fn collect_output_writes(outputs: &[StagedOutput], clean: bool) -> Result<Vec<FileWrite>> {
    let mut writes = BTreeMap::<PathBuf, Option<Vec<u8>>>::new();
    for output in outputs {
        if output.directory {
            let staged_files = regular_files(&output.staged_path)?;
            let staged_relative = staged_files
                .iter()
                .map(|path| {
                    path.strip_prefix(&output.staged_path)
                        .map(Path::to_path_buf)
                        .map_err(|_| {
                            anyhow::anyhow!("staged output escaped its root: `{}`", path.display())
                        })
                })
                .collect::<Result<BTreeSet<_>>>()?;
            if clean && output.final_path.exists() {
                for path in regular_files(&output.final_path)? {
                    let relative = path.strip_prefix(&output.final_path).map_err(|_| {
                        anyhow::anyhow!("existing output escaped its root: `{}`", path.display())
                    })?;
                    if !staged_relative.contains(relative) {
                        writes.insert(path, None);
                    }
                }
            }
            for (path, relative) in staged_files.into_iter().zip(staged_relative) {
                writes.insert(
                    output.final_path.join(relative),
                    Some(fs::read(&path).with_context(|| {
                        format!("failed to read staged build output `{}`", path.display())
                    })?),
                );
            }
        } else {
            writes.insert(
                output.final_path.clone(),
                Some(fs::read(&output.staged_path).with_context(|| {
                    format!(
                        "failed to read staged build output `{}`",
                        output.staged_path.display()
                    )
                })?),
            );
        }
    }
    Ok(writes
        .into_iter()
        .map(|(path, content)| FileWrite { path, content })
        .collect())
}

fn regular_files(root: &Path) -> Result<Vec<PathBuf>> {
    if !root.exists() {
        return Ok(Vec::new());
    }
    let metadata = fs::symlink_metadata(root)
        .with_context(|| format!("failed to inspect build output `{}`", root.display()))?;
    if metadata.file_type().is_symlink() {
        bail!(
            "build output contains a symbolic link: `{}`",
            root.display()
        );
    }
    if metadata.is_file() {
        return Ok(vec![root.to_path_buf()]);
    }
    let mut files = Vec::new();
    for entry in fs::read_dir(root)
        .with_context(|| format!("failed to read build output directory `{}`", root.display()))?
    {
        let path = entry
            .with_context(|| format!("failed to read build output entry in `{}`", root.display()))?
            .path();
        files.extend(regular_files(&path)?);
    }
    files.sort();
    Ok(files)
}

const fn phase_index(phase: BuildPhase) -> usize {
    match phase {
        BuildPhase::LoadManifest => 1,
        BuildPhase::LoadSchema => 2,
        BuildPhase::NormalizeSchema => 3,
        BuildPhase::LoadData => 4,
        BuildPhase::ValidateData => 5,
        BuildPhase::PlanOutputs => 6,
        BuildPhase::Generate => 7,
        BuildPhase::Format => 8,
        BuildPhase::Export => 9,
        BuildPhase::Commit => 10,
    }
}

fn resolve_project_path(project_dir: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        project_dir.join(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn declared_outputs_reject_traversal_and_project_root() {
        let root =
            std::env::temp_dir().join(format!("sora-build-output-boundary-{}", Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();

        assert!(resolve_declared_output(&root, Path::new("../outside")).is_err());
        assert!(resolve_declared_output(&root, Path::new(".")).is_err());
        assert_eq!(
            resolve_declared_output(&root, Path::new("generated/code")).unwrap(),
            root.canonicalize().unwrap().join("generated/code")
        );

        let _ = fs::remove_dir_all(root);
    }

    #[cfg(unix)]
    #[test]
    fn declared_outputs_reject_symlink_escape() {
        use std::os::unix::fs::symlink;

        let root =
            std::env::temp_dir().join(format!("sora-build-output-symlink-{}", Uuid::new_v4()));
        let outside =
            std::env::temp_dir().join(format!("sora-build-output-outside-{}", Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        fs::create_dir_all(&outside).unwrap();
        symlink(&outside, root.join("generated")).unwrap();

        assert!(resolve_declared_output(&root, Path::new("generated/code")).is_err());

        let _ = fs::remove_dir_all(root);
        let _ = fs::remove_dir_all(outside);
    }

    #[test]
    fn progress_cancellation_stops_before_the_observed_phase() {
        let cancellation = BuildControl::default();
        let cancel_from_progress = cancellation.clone();
        let control = cancellation.on_progress(move |progress| {
            if progress.phase == BuildPhase::Generate {
                cancel_from_progress.cancel();
            }
        });

        assert!(control.checkpoint(BuildPhase::PlanOutputs).is_ok());
        let error = control.checkpoint(BuildPhase::Generate).unwrap_err();
        assert!(
            error
                .downcast_ref::<sora_diagnostics::SoraError>()
                .is_some_and(|error| matches!(
                    error,
                    sora_diagnostics::SoraError::OperationCancelled { operation: "build" }
                ))
        );
    }

    #[test]
    fn cancelled_build_leaves_existing_outputs_untouched() {
        let root =
            std::env::temp_dir().join(format!("sora-build-cancel-transaction-{}", Uuid::new_v4()));
        fs::create_dir_all(root.join("schema")).unwrap();
        fs::create_dir_all(root.join("generated")).unwrap();
        let project = root.join("project.toml");
        fs::write(
            &project,
            r#"
package = "test"
includes = ["schema/items.toml"]

[build]
schema_lock = "generated/schema.lock"
"#,
        )
        .unwrap();
        fs::write(
            root.join("schema/items.toml"),
            r#"
[[tables]]
name = "Settings"
mode = "singleton"

[[tables.fields]]
name = "name"
type = "string"
"#,
        )
        .unwrap();
        fs::write(root.join("generated/schema.lock"), b"previous").unwrap();
        let runtime = ProjectRuntime::load(Some(&project), Default::default()).unwrap();
        let cancellation = BuildControl::default();
        let cancel_from_progress = cancellation.clone();
        let control = cancellation.on_progress(move |progress| {
            if progress.phase == BuildPhase::Generate {
                cancel_from_progress.cancel();
            }
        });

        let error = build_project_with_control(
            BuildRequest {
                project,
                default_source_format: None,
                data_root: None,
                scope: None,
                include_schema_lock: true,
                include_excel_templates: true,
                include_codegen: true,
                include_exports: true,
                targets: Vec::new(),
                export_formats: Vec::new(),
                clean: true,
            },
            &runtime,
            &control,
        )
        .unwrap_err();

        assert!(error.to_string().contains("cancelled"));
        assert_eq!(
            fs::read(root.join("generated/schema.lock")).unwrap(),
            b"previous"
        );
        let staging = root.join(".sora/build-staging");
        assert!(
            !staging.exists() || fs::read_dir(staging).unwrap().next().is_none(),
            "cancelled build must remove its staging directory"
        );
        let _ = fs::remove_dir_all(root);
    }
}
