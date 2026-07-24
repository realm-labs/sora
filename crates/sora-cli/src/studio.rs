use anyhow::Result;
use sora_workspace::{ProjectId, RuntimeOptions, WorkspaceService};

use crate::args::StudioArgs;

pub fn run(args: StudioArgs, runtime_options: RuntimeOptions) -> Result<()> {
    let workspace = std::sync::Arc::new(WorkspaceService::new());
    let project_id = ProjectId::new("studio")?;
    workspace.open_project(project_id.clone(), &args.project, runtime_options)?;
    sora_studio::run_blocking(sora_studio::StudioOptions {
        workspace,
        project_id,
        host: args.host,
        port: args.port,
    })
}
