use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "sora")]
#[command(about = "Sora game configuration compiler")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Build(BuildArgs),
    Check(CheckArgs),
    Gen {
        #[command(subcommand)]
        target: GenCommand,
    },
    Export(ExportArgs),
    Diff(DiffArgs),
    ExcelTemplate(ExcelTemplateArgs),
    SchemaLock(SchemaLockArgs),
}

#[derive(Debug, Args)]
pub struct CheckArgs {
    #[arg(long)]
    pub project: PathBuf,

    #[arg(long)]
    pub lock: Option<PathBuf>,
}

#[derive(Debug, Subcommand)]
pub enum GenCommand {
    Rust(GenArgs),
    Kotlin(GenArgs),
    Csharp(GenArgs),
    Java(GenArgs),
    Go(GenArgs),
}

#[derive(Debug, Args)]
pub struct GenArgs {
    #[arg(long)]
    pub project: PathBuf,

    #[arg(long)]
    pub out: PathBuf,
}

#[derive(Debug, Args)]
pub struct BuildArgs {
    #[arg(long)]
    pub project: PathBuf,

    #[arg(long, value_enum)]
    pub data_format: Option<DataFormat>,

    #[arg(long)]
    pub data_root: Option<PathBuf>,

    #[arg(long, value_enum)]
    pub target: Vec<BuildTarget>,

    #[arg(long)]
    pub clean: bool,
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

#[derive(Debug, Args)]
pub struct DiffArgs {
    #[arg(long, value_enum, default_value_t = DataFormat::Xlsx)]
    pub data_format: DataFormat,

    #[arg(long)]
    pub project: PathBuf,

    #[arg(long)]
    pub left_root: PathBuf,

    #[arg(long)]
    pub right_root: PathBuf,

    #[arg(long)]
    pub out: PathBuf,
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum DataFormat {
    Csv,
    Toml,
    Xlsx,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum BuildTarget {
    Rust,
    Kotlin,
    Csharp,
    Java,
    Go,
}

#[derive(Debug, Args)]
pub struct ExcelTemplateArgs {
    #[arg(long)]
    pub project: PathBuf,

    #[arg(long)]
    pub out: PathBuf,
}

#[derive(Debug, Args)]
pub struct SchemaLockArgs {
    #[arg(long)]
    pub project: PathBuf,

    #[arg(long)]
    pub out: PathBuf,
}
