use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use sora_codegen::target::CodegenTarget;
use sora_export::exporter::{ExportOutput, OutputKind};
use sora_input_csv::input::CsvProjectInput;
use sora_input_toml::input::{TomlProjectInput, TomlSchemaInput};
use sora_input_xlsx::input::XlsxProjectInput;

use crate::args::{
    CheckArgs, Command, DataFormat, DiffArgs, ExcelTemplateArgs, ExportArgs, GenArgs, GenCommand,
    SchemaLockArgs,
};

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
    sora_core::pipeline::generate_code(&input, target, &args.out).with_context(|| {
        format!(
            "failed to generate code from `{}` into `{}`",
            args.project.display(),
            args.out.display()
        )
    })
}

fn excel_template(args: ExcelTemplateArgs) -> Result<()> {
    let input = TomlSchemaInput::new(&args.project);
    sora_core::pipeline::generate_excel_template(&input, &args.out).with_context(|| {
        format!(
            "failed to generate Excel templates from `{}`",
            args.project.display()
        )
    })
}

fn schema_lock(args: SchemaLockArgs) -> Result<()> {
    let input = TomlSchemaInput::new(&args.project);
    sora_core::pipeline::generate_schema_lock(&input, &args.out).with_context(|| {
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
        args.data_format,
        &args.format,
        args.out,
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
    data_format: DataFormat,
    format: &str,
    out: PathBuf,
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

    match data_format {
        DataFormat::Csv => {
            let schema_input = TomlSchemaInput::new(project);
            let input = CsvProjectInput::new(schema_input, data_root);
            sora_core::pipeline::export_data(&input, format, output)
        }
        DataFormat::Toml => {
            let input = TomlProjectInput::new(project, data_root);
            sora_core::pipeline::export_data(&input, format, output)
        }
        DataFormat::Xlsx => {
            let schema_input = TomlSchemaInput::new(project);
            let input = XlsxProjectInput::new(schema_input, data_root);
            sora_core::pipeline::export_data(&input, format, output)
        }
    }?;
    Ok(())
}

fn diff(args: DiffArgs) -> Result<()> {
    match args.data_format {
        DataFormat::Csv => {
            let left_schema = TomlSchemaInput::new(&args.project);
            let right_schema = TomlSchemaInput::new(&args.project);
            let left = CsvProjectInput::new(left_schema, &args.left_root);
            let right = CsvProjectInput::new(right_schema, &args.right_root);
            sora_core::pipeline::diff_data(&left, &right, &args.out)
        }
        DataFormat::Toml => {
            let left = TomlProjectInput::new(&args.project, &args.left_root);
            let right = TomlProjectInput::new(&args.project, &args.right_root);
            sora_core::pipeline::diff_data(&left, &right, &args.out)
        }
        DataFormat::Xlsx => {
            let left_schema = TomlSchemaInput::new(&args.project);
            let right_schema = TomlSchemaInput::new(&args.project);
            let left = XlsxProjectInput::new(left_schema, &args.left_root);
            let right = XlsxProjectInput::new(right_schema, &args.right_root);
            sora_core::pipeline::diff_data(&left, &right, &args.out)
        }
    }
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
