use anyhow::Result;
use sora_workspace::ProjectRuntime;

use crate::args::StudioArgs;

pub fn run(args: StudioArgs, context: &ProjectRuntime) -> Result<()> {
    sora_studio::run_blocking(sora_studio::StudioOptions {
        project: args.project,
        host: args.host,
        port: args.port,
        schema_parser_registry: std::sync::Arc::clone(context.schema_parsers()),
    })
}
