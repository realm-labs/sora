use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use sora_codegen::{
    format::FormatMode,
    generator::{CodegenRegistry, empty_options, runtime_format_name},
    options::RuntimeFormat,
};
use sora_execution::ExecutionContext;
use sora_export::exporter::{ExportCompression, ExportOptions};
use sora_input::traits::SchemaInput;
use sora_input_schema::input::SchemaFileInput;
use sora_schema::model::CodegenSchema;

use crate::args::BuildArgs;

mod manifest;

use manifest::{BuildCodegen, BuildConfig, BuildExport, BuildManifest, ExportCompressionArg};

pub fn run(args: BuildArgs, execution: &ExecutionContext) -> Result<()> {
    let manifest = BuildManifest::load(&args.project)?;
    let build = manifest.build;
    let project_dir = args.project.parent().unwrap_or_else(|| Path::new("."));
    let schema_input = SchemaFileInput::new(&args.project);

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
    let requested_targets = args.target;
    let codegen = selected_codegen_targets(&build.codegen, &requested_targets, &registry)?;

    if build.is_empty() {
        bail!(
            "project `{}` does not declare any build outputs; add [build], [[build.codegen]], or [[build.exports]]",
            args.project.display()
        );
    }

    validate_export_formats(&build.exports)?;
    if args.clean {
        clean_build_outputs(project_dir, &build, &codegen)?;
    }

    let schema = schema_input.load_schema()?;
    let codegen_options = schema.codegen.clone();
    let ir = sora_ir::normalize::normalize_schema(schema)
        .with_context(|| format!("failed to check project `{}`", args.project.display()))?;
    sora_ir::validate::validate_config_ir(&ir)
        .with_context(|| format!("failed to check project `{}`", args.project.display()))?;
    validate_codegen_runtime_exports(&codegen, &build.exports, scope, &registry, &codegen_options)?;

    if let Some(path) = build.schema_lock.as_ref() {
        let path = resolve_project_path(project_dir, path);
        sora_core::pipeline::generate_schema_lock_with_scope(&schema_input, &path, scope)
            .with_context(|| {
                format!(
                    "failed to generate schema lock from `{}` into `{}`",
                    args.project.display(),
                    path.display()
                )
            })?;
    }

    if let Some(path) = build.excel_templates.as_ref() {
        let path = resolve_project_path(project_dir, path);
        sora_core::pipeline::generate_excel_template_with_scope(&schema_input, &path, scope)
            .with_context(|| {
                format!(
                    "failed to generate Excel templates from `{}` into `{}`",
                    args.project.display(),
                    path.display()
                )
            })?;
    }

    for item in codegen {
        let out = resolve_project_path(project_dir, &item.out);
        let item_scope = item.scope.as_deref().or(scope);
        sora_core::pipeline::generate_code_with_scope_and_format(
            &schema_input,
            &item.target,
            &out,
            FormatMode::from(item.format),
            item_scope,
        )
        .with_context(|| {
            format!(
                "failed to generate {} code from `{}` into `{}`",
                item.target,
                args.project.display(),
                out.display()
            )
        })?;
    }

    if !build.exports.is_empty() {
        let project_input = crate::source::MixedProjectInput::new(
            SchemaFileInput::new(&args.project),
            &data_root,
            default_source_format.map(crate::args::SourceFormatArg::as_str),
        );
        let (ir, data) =
            sora_core::pipeline::load_project_data_with_context(&project_input, execution)
                .with_context(|| {
                    format!(
                        "failed to load data from `{}` for project `{}`",
                        data_root.display(),
                        args.project.display()
                    )
                })?;

        for item in &build.exports {
            let out = resolve_project_path(project_dir, &item.out);
            let item_scope = item.scope.as_deref().or(scope);
            let output = export_output(&item.format, out)?;
            sora_core::pipeline::export_loaded_data_with_scope_context_and_options(
                &ir,
                &data,
                &item.format,
                output,
                item_scope,
                execution,
                export_options(item)?,
            )
            .with_context(|| {
                format!(
                    "failed to export `{}` data from `{}`",
                    item.format,
                    data_root.display()
                )
            })?;
        }
    }

    Ok(())
}

fn validate_export_formats(exports: &[BuildExport]) -> Result<()> {
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
        ExportCompressionArg::None => ExportCompression::None,
        ExportCompressionArg::Zstd => {
            if item.format != "binary" {
                bail!(
                    "export compression `zstd` is only supported by `binary` exports, got `{}`",
                    item.format
                );
            }
            ExportCompression::Zstd {
                level: item.compression_level.unwrap_or(3),
            }
        }
    };
    Ok(ExportOptions { compression })
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

fn clean_build_outputs(
    project_dir: &Path,
    build: &BuildConfig,
    codegen: &[&BuildCodegen],
) -> Result<()> {
    if let Some(path) = build.schema_lock.as_ref() {
        clean_output(project_dir, &resolve_project_path(project_dir, path))?;
    }
    if let Some(path) = build.excel_templates.as_ref() {
        clean_output(project_dir, &resolve_project_path(project_dir, path))?;
    }
    for item in codegen {
        clean_output(project_dir, &resolve_project_path(project_dir, &item.out))?;
    }
    for item in &build.exports {
        clean_output(project_dir, &resolve_project_path(project_dir, &item.out))?;
    }
    Ok(())
}

fn clean_output(project_dir: &Path, path: &Path) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }

    let project_dir = project_dir.canonicalize().with_context(|| {
        format!(
            "failed to resolve project directory `{}`",
            project_dir.display()
        )
    })?;
    let path = path
        .canonicalize()
        .with_context(|| format!("failed to resolve output path `{}`", path.display()))?;
    if path == project_dir || !path.starts_with(&project_dir) {
        bail!(
            "refusing to clean output `{}` because it is not safely inside project directory `{}`",
            path.display(),
            project_dir.display()
        );
    }

    if path.is_dir() {
        fs::remove_dir_all(&path)
            .with_context(|| format!("failed to clean directory `{}`", path.display()))?;
    } else {
        fs::remove_file(&path)
            .with_context(|| format!("failed to clean file `{}`", path.display()))?;
    }
    Ok(())
}

fn resolve_project_path(project_dir: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        project_dir.join(path)
    }
}

#[cfg(test)]
mod tests;
