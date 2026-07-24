use std::sync::Arc;

use rmcp::{
    ServerHandler,
    handler::server::{
        router::tool::ToolRouter,
        wrapper::{Json, Parameters},
    },
    model::{Implementation, ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sora_workspace::WorkspaceService;

use crate::{SERVER_NAME, TARGET_PROTOCOL_VERSION};

/// MCP protocol adapter backed by the shared Sora workspace service.
#[derive(Debug, Clone)]
pub struct SoraMcpServer {
    workspace: Arc<WorkspaceService>,
    tool_router: ToolRouter<Self>,
}

impl SoraMcpServer {
    /// Creates an MCP server for a workspace service.
    pub fn new(workspace: Arc<WorkspaceService>) -> Self {
        Self {
            workspace,
            tool_router: Self::tool_router(),
        }
    }

    fn capabilities() -> ServerCapabilities {
        let mut capabilities = ServerCapabilities::builder().enable_tools().build();
        if let Some(tools) = capabilities.tools.as_mut() {
            tools.list_changed = Some(false);
        }
        capabilities
    }
}

#[tool_router]
impl SoraMcpServer {
    #[tool(
        name = "sora_server_info",
        description = "Return the Sora MCP protocol revision and current workspace project count"
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

#[tool_handler(router = self.tool_router)]
impl ServerHandler for SoraMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(Self::capabilities())
            .with_protocol_version(TARGET_PROTOCOL_VERSION)
            .with_server_info(
                Implementation::new(SERVER_NAME, env!("CARGO_PKG_VERSION"))
                    .with_title("Sora")
                    .with_description("Sora configuration compiler MCP server"),
            )
            .with_instructions(
                "Use Sora resources and domain tools to inspect, validate, modify, and build \
                 configuration projects. Never edit generated outputs as source files.",
            )
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
    use super::*;

    #[test]
    fn initialization_contract_is_versioned_and_deterministic() {
        let server = SoraMcpServer::new(Arc::new(WorkspaceService::new()));
        let info = server.get_info();

        assert_eq!(info.protocol_version, TARGET_PROTOCOL_VERSION);
        assert_eq!(info.server_info.name, SERVER_NAME);
        assert_eq!(
            info.capabilities
                .tools
                .as_ref()
                .and_then(|tools| tools.list_changed),
            Some(false)
        );
        assert!(info.capabilities.resources.is_none());
        assert!(info.capabilities.prompts.is_none());
        assert!(info.capabilities.completions.is_none());
        assert!(info.capabilities.tasks.is_none());
    }

    #[test]
    fn tools_have_strict_structured_contracts() {
        let server = SoraMcpServer::new(Arc::new(WorkspaceService::new()));
        let tools = server.tool_router.list_all();

        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "sora_server_info");
        assert!(tools[0].output_schema.is_some());
        let input_schema = serde_json::to_value(&tools[0].input_schema)
            .expect("tool input schema should serialize");
        assert_eq!(input_schema["type"], "object");
        assert_eq!(input_schema["additionalProperties"], false);
    }

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
