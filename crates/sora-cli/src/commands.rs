use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use sora_codegen::target::CodegenTarget;
use sora_export::exporter::{ExportOutput, OutputKind};
use sora_input_toml::input::TomlSchemaInput;

use crate::args::{
    CheckArgs, Command, DiffArgs, ExcelTemplateArgs, ExportArgs, GenArgs, GenCommand,
    SchemaLockArgs, SourceFormatArg,
};
use crate::source::MixedProjectInput;

pub fn run(command: Command) -> Result<()> {
    match command {
        Command::Build(args) => crate::build::run(args),
        Command::Check(args) => check(args),
        Command::Gen { target } => match target {
            GenCommand::Rust(args) => generate(args, CodegenTarget::Rust),
            GenCommand::Kotlin(args) => generate(args, CodegenTarget::Kotlin),
            GenCommand::Csharp(args) => generate(args, CodegenTarget::CSharp),
            GenCommand::Java(args) => generate(args, CodegenTarget::Java),
            GenCommand::Go(args) => generate(args, CodegenTarget::Go),
            GenCommand::Dart(args) => generate(args, CodegenTarget::Dart),
            GenCommand::Godot(args) => generate(args, CodegenTarget::Godot),
            GenCommand::C(args) => generate(args, CodegenTarget::C),
            GenCommand::Cpp(args) => generate(args, CodegenTarget::Cpp),
            GenCommand::Typescript(args) => generate(args, CodegenTarget::TypeScript),
            GenCommand::Javascript(args) => generate(args, CodegenTarget::JavaScript),
            GenCommand::Erlang(args) => generate(args, CodegenTarget::Erlang),
            GenCommand::Lua(args) => generate(args, CodegenTarget::Lua),
            GenCommand::ProtoSchema(args) => generate(args, CodegenTarget::ProtoSchema),
            GenCommand::Python(args) => generate(args, CodegenTarget::Python),
        },
        Command::Export(args) => export(args),
        Command::Diff(args) => diff(args),
        Command::ExcelTemplate(args) => excel_template(args),
        Command::SchemaLock(args) => schema_lock(args),
    }
}

fn check(args: CheckArgs) -> Result<()> {
    let input = TomlSchemaInput::new(&args.project);
    match &args.lock {
        Some(lock) => {
            sora_core::pipeline::check_schema_with_lock(&input, lock).with_context(|| {
                format!(
                    "failed to check project `{}` against lock `{}`",
                    args.project.display(),
                    lock.display()
                )
            })
        }
        None => sora_core::pipeline::check_schema(&input)
            .with_context(|| format!("failed to check project `{}`", args.project.display())),
    }
}

fn generate(args: GenArgs, target: CodegenTarget) -> Result<()> {
    let input = TomlSchemaInput::new(&args.project);
    sora_core::pipeline::generate_code_with_scope_and_format(
        &input,
        target,
        &args.out,
        args.format_code.into(),
        args.scope.as_deref(),
    )
    .with_context(|| {
        format!(
            "failed to generate code from `{}` into `{}`",
            args.project.display(),
            args.out.display()
        )
    })
}

fn excel_template(args: ExcelTemplateArgs) -> Result<()> {
    let input = TomlSchemaInput::new(&args.project);
    sora_core::pipeline::generate_excel_template_with_scope(
        &input,
        &args.out,
        args.scope.as_deref(),
    )
    .with_context(|| {
        format!(
            "failed to generate Excel templates from `{}`",
            args.project.display()
        )
    })
}

fn schema_lock(args: SchemaLockArgs) -> Result<()> {
    let input = TomlSchemaInput::new(&args.project);
    sora_core::pipeline::generate_schema_lock_with_scope(&input, &args.out, args.scope.as_deref())
        .with_context(|| {
            format!(
                "failed to generate schema lock from `{}` into `{}`",
                args.project.display(),
                args.out.display()
            )
        })
}

fn export(args: ExportArgs) -> Result<()> {
    export_project_data(
        &args.project,
        &args.data_root,
        args.default_source_format,
        &args.format,
        args.out,
        args.scope.as_deref(),
    )
    .with_context(|| {
        format!(
            "failed to export `{}` data from `{}`",
            args.format,
            args.data_root.display()
        )
    })
}

pub(crate) fn export_project_data(
    project: &Path,
    data_root: &Path,
    default_source_format: Option<SourceFormatArg>,
    format: &str,
    out: PathBuf,
    scope: Option<&str>,
) -> Result<()> {
    let output = match sora_core::pipeline::export_output_kind(format) {
        Some(OutputKind::File) => ExportOutput::File(out.clone()),
        Some(OutputKind::Directory) => ExportOutput::Directory(out.clone()),
        None => {
            bail!(
                "unknown export format `{}`; supported formats: {}",
                format,
                sora_core::pipeline::supported_export_formats().join(", ")
            );
        }
    };

    let schema_input = TomlSchemaInput::new(project);
    let input = MixedProjectInput::new(
        schema_input,
        data_root,
        default_source_format.map(SourceFormatArg::as_str),
    );
    sora_core::pipeline::export_data_with_scope(&input, format, output, scope)?;
    Ok(())
}

fn diff(args: DiffArgs) -> Result<()> {
    let default_source_format = args.default_source_format.map(SourceFormatArg::as_str);
    let left_schema = TomlSchemaInput::new(&args.project);
    let right_schema = TomlSchemaInput::new(&args.project);
    let left = MixedProjectInput::new(left_schema, &args.left_root, default_source_format);
    let right = MixedProjectInput::new(right_schema, &args.right_root, default_source_format);
    sora_core::pipeline::diff_data_with_scope(&left, &right, &args.out, args.scope.as_deref())
        .with_context(|| {
            format!(
                "failed to diff `{}` against `{}` into `{}`",
                args.left_root.display(),
                args.right_root.display(),
                args.out.display()
            )
        })?;

    Ok(())
}
