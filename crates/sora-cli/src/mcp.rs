use std::{path::Path, sync::Arc};

use anyhow::Result;
use sora_workspace::{ProjectId, RuntimeOptions, WorkspaceService};

use crate::args::McpArgs;

pub fn run(args: McpArgs, runtime_options: RuntimeOptions) -> Result<()> {
    let workspace = Arc::new(WorkspaceService::new());
    if let Some(project) = args.project {
        let root = project.parent().unwrap_or_else(|| Path::new("."));
        workspace.add_root("explicit", root)?;
        workspace.open_project(ProjectId::new("project-1")?, project, runtime_options)?;
    }
    sora_mcp::serve_stdio(workspace)
}
