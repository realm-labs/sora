use anyhow::Result;

use crate::args::StudioArgs;

pub fn run(args: StudioArgs) -> Result<()> {
    sora_studio::run_blocking(sora_studio::StudioOptions {
        project: args.project,
        host: args.host,
        port: args.port,
    })
}
