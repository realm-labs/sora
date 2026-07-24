use anyhow::Result;
use sora_workspace::{ProjectId, ProjectSession, RuntimeOptions};

use crate::args::StudioArgs;

pub fn run(args: StudioArgs, runtime_options: RuntimeOptions) -> Result<()> {
    let session = std::sync::Arc::new(ProjectSession::open(
        ProjectId::new("studio")?,
        &args.project,
        runtime_options,
    )?);
    sora_studio::run_blocking(sora_studio::StudioOptions {
        session,
        host: args.host,
        port: args.port,
    })
}
