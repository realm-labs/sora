use std::sync::Arc;

use anyhow::Result;
use sora_studio::{StudioOptions, run_blocking};
use sora_workspace::{ProjectId, RuntimeOptions, WorkspaceService};

use crate::args::StudioArgs;

pub fn run(args: StudioArgs, runtime_options: RuntimeOptions) -> Result<()> {
    let workspace = Arc::new(WorkspaceService::new());
    let project_id = ProjectId::new("studio")?;
    workspace.open_project(project_id.clone(), &args.project, runtime_options)?;
    run_blocking(StudioOptions {
        workspace,
        project_id,
        host: args.host,
        port: args.port,
    })
}
