use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use sora_codegen::format::FormatMode;
use sora_execution::ExecutionContext;
use sora_input_toml::input::TomlSchemaInput;

use crate::{
    args::{BuildArgs, BuildTarget},
    commands::export_project_data,
};

mod manifest;

use manifest::{BuildCodegen, BuildConfig, BuildExport, BuildManifest};

pub fn run(args: BuildArgs, execution: &ExecutionContext) -> Result<()> {
    let manifest = BuildManifest::load(&args.project)?;
    let build = manifest.build;
    let project_dir = args.project.parent().unwrap_or_else(|| Path::new("."));
    let schema_input = TomlSchemaInput::new(&args.project);

    let default_source_format = args.default_source_format.or(build.default_source_format);
    let data_root = args
        .data_root
        .as_ref()
        .or(build.data_root.as_ref())
        .cloned()
        .unwrap_or_else(|| PathBuf::from("data"));
    let data_root = resolve_project_path(project_dir, &data_root);
    let scope = args.scope.as_deref().or(build.scope.as_deref());

    let requested_targets = args.target;
    let codegen = selected_codegen_targets(&build.codegen, &requested_targets)?;

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

    sora_core::pipeline::check_schema(&schema_input)
        .with_context(|| format!("failed to check project `{}`", args.project.display()))?;

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
            item.target.into(),
            &out,
            FormatMode::from(item.format),
            item_scope,
        )
        .with_context(|| {
            format!(
                "failed to generate {} code from `{}` into `{}`",
                item.target.as_str(),
                args.project.display(),
                out.display()
            )
        })?;
    }

    for item in &build.exports {
        let out = resolve_project_path(project_dir, &item.out);
        let item_scope = item.scope.as_deref().or(scope);
        export_project_data(
            &args.project,
            &data_root,
            default_source_format,
            &item.format,
            out,
            item_scope,
            execution,
        )
        .with_context(|| {
            format!(
                "failed to export `{}` data from `{}`",
                item.format,
                data_root.display()
            )
        })?;
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

fn selected_codegen_targets<'a>(
    configured: &'a [BuildCodegen],
    requested: &[BuildTarget],
) -> Result<Vec<&'a BuildCodegen>> {
    if requested.is_empty() {
        return Ok(configured.iter().collect());
    }

    let selected = configured
        .iter()
        .filter(|item| requested.contains(&item.target))
        .collect::<Vec<_>>();
    for target in requested {
        if !configured.iter().any(|item| item.target == *target) {
            bail!(
                "build target `{}` was requested but is not declared in [[build.codegen]]",
                target.as_str()
            );
        }
    }
    Ok(selected)
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
