use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};
use sora_codegen::format::FormatMode;

#[derive(Debug, Parser)]
#[command(name = "sora")]
#[command(about = "Sora game configuration compiler")]
#[command(version)]
pub struct Cli {
    #[arg(short = 'j', long, global = true)]
    pub jobs: Option<usize>,

    #[arg(long, global = true)]
    pub serial: bool,

    #[arg(long, global = true, value_name = "PATH")]
    pub parser_script: Vec<PathBuf>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    #[command(visible_alias = "b")]
    Build(BuildArgs),
    #[command(visible_alias = "c")]
    Check(CheckArgs),
    #[command(visible_alias = "i")]
    Init(InitArgs),
    #[command(visible_alias = "g")]
    Gen {
        #[arg(short, long)]
        target: String,

        #[command(flatten)]
        args: GenArgs,
    },
    #[command(visible_alias = "e")]
    Export(ExportArgs),
    #[command(visible_alias = "d")]
    Diff(DiffArgs),
    #[command(visible_aliases = ["template", "et"])]
    ExcelTemplate(ExcelTemplateArgs),
    #[command(visible_aliases = ["sync", "es"])]
    ExcelSync(ExcelSyncArgs),
    #[command(visible_aliases = ["lock", "sl"])]
    SchemaLock(SchemaLockArgs),
    #[command(visible_alias = "st")]
    Studio(StudioArgs),
}

#[derive(Debug, Args)]
pub struct CheckArgs {
    #[arg(short, long)]
    pub project: PathBuf,

    #[arg(short, long)]
    pub lock: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct GenArgs {
    #[arg(short, long)]
    pub project: PathBuf,

    #[arg(short, long)]
    pub out: PathBuf,

    #[arg(long, value_enum, default_value_t = CodeFormatMode::Never)]
    pub format_code: CodeFormatMode,

    #[arg(short, long)]
    pub scope: Option<String>,
}

#[derive(Debug, Args)]
pub struct BuildArgs {
    #[arg(short, long)]
    pub project: PathBuf,

    #[arg(long, value_enum)]
    pub default_source_format: Option<SourceFormatArg>,

    #[arg(short, long)]
    pub data_root: Option<PathBuf>,

    #[arg(short, long)]
    pub scope: Option<String>,

    #[arg(short, long)]
    pub target: Vec<String>,

    #[arg(short, long)]
    pub clean: bool,
}

#[derive(Debug, Args)]
pub struct ExportArgs {
    #[arg(short, long)]
    pub format: String,

    #[arg(long, value_enum)]
    pub default_source_format: Option<SourceFormatArg>,

    #[arg(short, long)]
    pub project: PathBuf,

    #[arg(short, long)]
    pub data_root: PathBuf,

    #[arg(short, long)]
    pub out: PathBuf,

    #[arg(short, long)]
    pub scope: Option<String>,

    #[arg(long, value_enum, default_value = "none")]
    pub compression: ExportCompressionArg,

    #[arg(long)]
    pub compression_level: Option<i32>,
}

#[derive(Debug, Args)]
pub struct DiffArgs {
    #[arg(long, value_enum)]
    pub default_source_format: Option<SourceFormatArg>,

    #[arg(short, long)]
    pub project: PathBuf,

    #[arg(short, long)]
    pub left_root: PathBuf,

    #[arg(short, long)]
    pub right_root: PathBuf,

    #[arg(short, long)]
    pub out: PathBuf,

    #[arg(short, long)]
    pub scope: Option<String>,
}

#[derive(Debug, Args)]
pub struct InitArgs {
    #[arg(short, long)]
    pub out: PathBuf,

    #[arg(long, value_enum, default_value_t = SchemaFormatArg::Toml)]
    pub schema_format: SchemaFormatArg,

    #[arg(long)]
    pub force: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum SchemaFormatArg {
    Toml,
    Yaml,
    Json,
    Lua,
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum SourceFormatArg {
    Csv,
    Json,
    Toml,
    Xlsx,
    Yaml,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum ExportCompressionArg {
    None,
    Zstd,
}

impl SourceFormatArg {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Csv => "csv",
            Self::Json => "json",
            Self::Toml => "toml",
            Self::Xlsx => "xlsx",
            Self::Yaml => "yaml",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, clap::ValueEnum)]
pub enum CodeFormatMode {
    #[default]
    Never,
    Auto,
    Required,
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

#[derive(Debug, Args)]
pub struct ExcelTemplateArgs {
    #[arg(short, long)]
    pub project: PathBuf,

    #[arg(short, long)]
    pub out: PathBuf,

    #[arg(short, long)]
    pub scope: Option<String>,
}

#[derive(Debug, Args)]
pub struct ExcelSyncArgs {
    #[arg(short, long)]
    pub project: PathBuf,

    #[arg(short, long)]
    pub data_root: PathBuf,

    #[arg(short, long)]
    pub scope: Option<String>,

    #[arg(short, long)]
    pub write: bool,
}

#[derive(Debug, Args)]
pub struct SchemaLockArgs {
    #[arg(short, long)]
    pub project: PathBuf,

    #[arg(short, long)]
    pub out: PathBuf,

    #[arg(short, long)]
    pub scope: Option<String>,
}

#[derive(Debug, Args)]
pub struct StudioArgs {
    #[arg(short, long)]
    pub project: PathBuf,

    #[arg(long, default_value = "127.0.0.1")]
    pub host: std::net::IpAddr,

    #[arg(long, default_value_t = 5174)]
    pub port: u16,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_build_alias_and_short_flags() {
        let cli = Cli::parse_from([
            "sora",
            "b",
            "-p",
            "project.toml",
            "-d",
            "data",
            "-s",
            "client",
            "-t",
            "rust",
            "-c",
        ]);

        let Command::Build(args) = cli.command else {
            panic!("expected build command");
        };
        assert_eq!(args.project, PathBuf::from("project.toml"));
        assert_eq!(args.data_root, Some(PathBuf::from("data")));
        assert_eq!(args.scope.as_deref(), Some("client"));
        assert_eq!(args.target, ["rust"]);
        assert!(args.clean);
    }

    #[test]
    fn parses_export_alias_and_short_flags() {
        let cli = Cli::parse_from([
            "sora",
            "e",
            "-p",
            "project.toml",
            "-d",
            "data",
            "-f",
            "json",
            "-o",
            "generated/config.json",
            "-s",
            "server",
        ]);

        let Command::Export(args) = cli.command else {
            panic!("expected export command");
        };
        assert_eq!(args.project, PathBuf::from("project.toml"));
        assert_eq!(args.data_root, PathBuf::from("data"));
        assert_eq!(args.format, "json");
        assert_eq!(args.out, PathBuf::from("generated/config.json"));
        assert_eq!(args.scope.as_deref(), Some("server"));
    }

    #[test]
    fn parses_long_command_aliases() {
        assert!(matches!(
            Cli::parse_from(["sora", "et", "-p", "project.toml", "-o", "generated/excel"]).command,
            Command::ExcelTemplate(_)
        ));
        assert!(matches!(
            Cli::parse_from(["sora", "es", "-p", "project.toml", "-d", "data", "-w"]).command,
            Command::ExcelSync(_)
        ));
        assert!(matches!(
            Cli::parse_from([
                "sora",
                "sl",
                "-p",
                "project.toml",
                "-o",
                "generated/schema.lock"
            ])
            .command,
            Command::SchemaLock(_)
        ));
        assert!(matches!(
            Cli::parse_from(["sora", "st", "-p", "project.toml"]).command,
            Command::Studio(_)
        ));
    }
}
