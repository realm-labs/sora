use anyhow::Result;
use sora_workspace::{BuildRequest, ProjectRuntime, SourceFormat, build_project};

use crate::args::{BuildArgs, SourceFormatArg};

pub fn run(args: BuildArgs, context: &ProjectRuntime) -> Result<()> {
    build_project(
        BuildRequest {
            project: args.project,
            default_source_format: args.default_source_format.map(SourceFormat::from),
            data_root: args.data_root,
            view: args.view,
            include_schema_lock: true,
            include_excel_templates: true,
            include_codegen: true,
            include_exports: true,
            targets: args.target,
            export_formats: Vec::new(),
            clean: args.clean,
        },
        context,
    )?;
    Ok(())
}

impl From<SourceFormatArg> for SourceFormat {
    fn from(value: SourceFormatArg) -> Self {
        match value {
            SourceFormatArg::Csv => Self::Csv,
            SourceFormatArg::Json => Self::Json,
            SourceFormatArg::Toml => Self::Toml,
            SourceFormatArg::Xlsx => Self::Xlsx,
            SourceFormatArg::Yaml => Self::Yaml,
        }
    }
}

#[cfg(test)]
mod tests;
