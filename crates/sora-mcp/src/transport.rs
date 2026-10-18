use std::sync::Arc;

use anyhow::{Context, Result};
use rmcp::{ServiceExt, transport::stdio};
use sora_workspace::WorkspaceService;
use tokio::runtime::Runtime;

use crate::SoraMcpServer;

/// Serves Sora MCP over stdin/stdout until the client disconnects.
///
/// The transport owns stdout, so all diagnostics and logging must use stderr.
pub fn serve_stdio(workspace: Arc<WorkspaceService>) -> Result<()> {
    let runtime = Runtime::new().context("failed to start MCP async runtime")?;
    runtime.block_on(async move {
        let service = SoraMcpServer::new(workspace)
            .serve(stdio())
            .await
            .context("failed to initialize Sora MCP stdio transport")?;
        service
            .waiting()
            .await
            .context("Sora MCP stdio service task failed")?;
        Ok(())
    })
}
