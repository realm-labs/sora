use std::sync::Arc;

use anyhow::Result;
use sora_workspace::WorkspaceService;

pub fn run() -> Result<()> {
    sora_mcp::serve_stdio(Arc::new(WorkspaceService::new()))
}
