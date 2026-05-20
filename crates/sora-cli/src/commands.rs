use anyhow::{Context, Result, bail};
use sora_codegen::target::CodegenTarget;
use sora_export::exporter::{ExportOutput, OutputKind};
use sora_input_csv::input::CsvProjectInput;
use sora_input_toml::input::{TomlProjectInput, TomlSchemaInput};
use sora_input_xlsx::input::XlsxProjectInput;

use crate::args::{
    CheckArgs, Command, DataFormat, ExcelTemplateArgs, ExportArgs, GenArgs, GenCommand,
    SchemaLockArgs,
};

pub fn run(command: Command) -> Result<()> {
    match command {
        Command::Check(args) => check(args),
        Command::Gen { target } => match target {
            GenCommand::Rust(args) => generate(args, CodegenTarget::Rust),
            GenCommand::Kotlin(args) => generate(args, CodegenTarget::Kotlin),
        },
        Command::Export(args) => export(args),
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
    let output = match sora_core::pipeline::export_output_kind(&args.format) {
        Some(OutputKind::File) => ExportOutput::File(args.out.clone()),
        Some(OutputKind::Directory) => ExportOutput::Directory(args.out.clone()),
        None => {
            bail!(
                "unknown export format `{}`; supported formats: {}",
                args.format,
                sora_core::pipeline::supported_export_formats().join(", ")
            );
        }
    };

    match args.data_format {
        DataFormat::Csv => {
            let schema_input = TomlSchemaInput::new(&args.project);
            let input = CsvProjectInput::new(schema_input, &args.data_root);
            sora_core::pipeline::export_data(&input, &args.format, output)
        }
        DataFormat::Toml => {
            let input = TomlProjectInput::new(&args.project, &args.data_root);
            sora_core::pipeline::export_data(&input, &args.format, output)
        }
        DataFormat::Xlsx => {
            let schema_input = TomlSchemaInput::new(&args.project);
            let input = XlsxProjectInput::new(schema_input, &args.data_root);
            sora_core::pipeline::export_data(&input, &args.format, output)
        }
    }
    .with_context(|| {
        format!(
            "failed to export `{}` data from `{}`",
            args.format,
            args.data_root.display()
        )
    })
}
