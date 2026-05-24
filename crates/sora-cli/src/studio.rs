use anyhow::Result;

use crate::args::StudioArgs;
use crate::commands::CliContext;

pub fn run(args: StudioArgs, context: &CliContext) -> Result<()> {
    sora_studio::run_blocking(sora_studio::StudioOptions {
        project: args.project,
        host: args.host,
        port: args.port,
        schema_parser_registry: std::sync::Arc::clone(&context.schema_parsers),
    })
}
