use rmcp::{
    handler::server::wrapper::{Json, Parameters},
    tool, tool_router,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{SERVER_NAME, SoraMcpServer, TARGET_PROTOCOL_VERSION};

#[tool_router(router = server_tool_router, vis = "pub(crate)")]
impl SoraMcpServer {
    #[tool(
        name = "sora_server_info",
        description = "Return the Sora MCP protocol revision and current workspace project count",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    fn server_info(
        &self,
        Parameters(_input): Parameters<ServerInfoInput>,
    ) -> Result<Json<ServerInfoOutput>, String> {
        let project_count = self
            .workspace
            .project_ids()
            .map_err(|error| error.to_string())?
            .len();
        Ok(Json(ServerInfoOutput {
            protocol_version: TARGET_PROTOCOL_VERSION.as_str(),
            server_name: SERVER_NAME,
            server_version: env!("CARGO_PKG_VERSION"),
            project_count,
        }))
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct ServerInfoInput {}

#[derive(Debug, Serialize, JsonSchema)]
struct ServerInfoOutput {
    protocol_version: &'static str,
    server_name: &'static str,
    server_version: &'static str,
    project_count: usize,
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use sora_workspace::WorkspaceService;

    use super::*;

    #[test]
    fn server_info_reports_workspace_state() {
        let server = SoraMcpServer::new(Arc::new(WorkspaceService::new()));
        let Json(output) = server
            .server_info(Parameters(ServerInfoInput {}))
            .expect("empty workspace should be readable");

        assert_eq!(output.protocol_version, "2025-11-25");
        assert_eq!(output.server_name, "sora");
        assert_eq!(output.project_count, 0);
    }
}
