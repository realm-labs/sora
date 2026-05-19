use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use clap::{Args, Parser, Subcommand};
use sora_core::{CodegenTarget, ExportOutput, OutputKind};
use sora_input_toml::{TomlProjectInput, TomlSchemaInput};

#[derive(Debug, Parser)]
#[command(name = "sora")]
#[command(about = "Sora game configuration compiler")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Check(CheckArgs),
    Gen {
        #[command(subcommand)]
        target: GenCommand,
    },
    Export(ExportArgs),
    ExcelTemplate(ExcelTemplateArgs),
}

#[derive(Debug, Args)]
struct CheckArgs {
    #[arg(long)]
    schema: PathBuf,
}

#[derive(Debug, Subcommand)]
enum GenCommand {
    Rust(GenArgs),
    Kotlin(GenArgs),
}

#[derive(Debug, Args)]
struct GenArgs {
    #[arg(long)]
    schema: PathBuf,

    #[arg(long)]
    out: PathBuf,
}

#[derive(Debug, Args)]
struct ExportArgs {
    #[arg(long)]
    format: String,

    #[arg(long)]
    schema: PathBuf,

    #[arg(long)]
    data_root: PathBuf,

    #[arg(long)]
    out: PathBuf,
}

#[derive(Debug, Args)]
struct ExcelTemplateArgs {
    #[arg(long)]
    schema: PathBuf,

    #[arg(long)]
    out: PathBuf,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Check(args) => {
            let input = TomlSchemaInput::new(&args.schema);
            sora_core::check_schema(&input)
                .with_context(|| format!("failed to check schema `{}`", args.schema.display()))?;
        }
        Command::Gen { target } => match target {
            GenCommand::Rust(args) => generate(args, CodegenTarget::Rust)?,
            GenCommand::Kotlin(args) => generate(args, CodegenTarget::Kotlin)?,
        },
        Command::Export(args) => export(args)?,
        Command::ExcelTemplate(args) => {
            let input = TomlSchemaInput::new(&args.schema);
            sora_core::generate_excel_template(&input, &args.out).with_context(|| {
                format!(
                    "failed to generate Excel templates from `{}`",
                    args.schema.display()
                )
            })?;
        }
    }

    Ok(())
}

fn generate(args: GenArgs, target: CodegenTarget) -> Result<()> {
    let input = TomlSchemaInput::new(&args.schema);
    sora_core::generate_code(&input, target, &args.out).with_context(|| {
        format!(
            "failed to generate code from `{}` into `{}`",
            args.schema.display(),
            args.out.display()
        )
    })
}

fn export(args: ExportArgs) -> Result<()> {
    let output = match sora_core::export_output_kind(&args.format) {
        Some(OutputKind::File) => ExportOutput::File(args.out.clone()),
        Some(OutputKind::Directory) => ExportOutput::Directory(args.out.clone()),
        None => {
            bail!(
                "unknown export format `{}`; supported formats: {}",
                args.format,
                sora_core::supported_export_formats().join(", ")
            );
        }
    };

    let input = TomlProjectInput::new(&args.schema, &args.data_root);
    sora_core::export_data(&input, &args.format, output).with_context(|| {
        format!(
            "failed to export `{}` data from `{}`",
            args.format,
            args.data_root.display()
        )
    })
}
