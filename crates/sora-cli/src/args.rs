use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "sora")]
#[command(about = "Sora game configuration compiler")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Check(CheckArgs),
    Gen {
        #[command(subcommand)]
        target: GenCommand,
    },
    Export(ExportArgs),
    ExcelTemplate(ExcelTemplateArgs),
}

#[derive(Debug, Args)]
pub struct CheckArgs {
    #[arg(long)]
    pub project: PathBuf,
}

#[derive(Debug, Subcommand)]
pub enum GenCommand {
    Rust(GenArgs),
    Kotlin(GenArgs),
}

#[derive(Debug, Args)]
pub struct GenArgs {
    #[arg(long)]
    pub project: PathBuf,

    #[arg(long)]
    pub out: PathBuf,
}

#[derive(Debug, Args)]
pub struct ExportArgs {
    #[arg(long)]
    pub format: String,

    #[arg(long, value_enum, default_value_t = DataFormat::Xlsx)]
    pub data_format: DataFormat,

    #[arg(long)]
    pub project: PathBuf,

    #[arg(long)]
    pub data_root: PathBuf,

    #[arg(long)]
    pub out: PathBuf,
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum DataFormat {
    Csv,
    Toml,
    Xlsx,
}

#[derive(Debug, Args)]
pub struct ExcelTemplateArgs {
    #[arg(long)]
    pub project: PathBuf,

    #[arg(long)]
    pub out: PathBuf,
}
