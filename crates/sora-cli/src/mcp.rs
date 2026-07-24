use std::{path::Path, sync::Arc};

use anyhow::Result;
use sora_workspace::{RuntimeOptions, WorkspaceService};

use crate::args::McpArgs;

pub fn run(args: McpArgs, runtime_options: RuntimeOptions) -> Result<()> {
    let workspace = Arc::new(WorkspaceService::new());
    if let Some(project) = args.project {
        let root = project.parent().unwrap_or_else(|| Path::new("."));
        let root = workspace.add_root("explicit", root)?;
        let relative_manifest = project
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default();
        workspace.open_discovered_project(
            root.id(),
            relative_manifest,
            runtime_options,
            args.trust_project_scripts,
        )?;
    }
    sora_mcp::serve_stdio(workspace)
}
