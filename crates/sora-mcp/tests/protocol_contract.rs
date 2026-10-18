use std::sync::Arc;

use anyhow::{Ok as anyhow_ok, Result, anyhow};
use rmcp::{
    ClientHandler, ServiceExt,
    model::{
        ArgumentInfo, ClientInfo, CompleteRequestParams, Implementation, ProtocolVersion,
        ReadResourceRequestParams, Reference,
    },
};
use serde_json::Value;
use sora_mcp::{SERVER_NAME, SoraMcpServer, TARGET_PROTOCOL_VERSION};
use sora_workspace::WorkspaceService;
use tokio::{io::duplex, spawn};

#[derive(Debug, Clone)]
struct ContractClient;

impl ClientHandler for ContractClient {
    fn get_info(&self) -> ClientInfo {
        ClientInfo::new(
            Default::default(),
            Implementation::new("sora-contract-test", env!("CARGO_PKG_VERSION")),
        )
        .with_protocol_version(TARGET_PROTOCOL_VERSION)
    }
}

#[tokio::test]
async fn lifecycle_negotiates_the_pinned_protocol_and_lists_strict_tools() -> Result<()> {
    let (server_transport, client_transport) = duplex(16 * 1024);
    let server = SoraMcpServer::new(Arc::new(WorkspaceService::new()));
    let server_handle = spawn(async move {
        server.serve(server_transport).await?.waiting().await?;
        anyhow_ok(())
    });

    let client = ContractClient.serve(client_transport).await?;
    let peer_info = client
        .peer_info()
        .ok_or_else(|| anyhow!("server initialization info is missing"))?;
    assert_eq!(peer_info.protocol_version, ProtocolVersion::V_2025_11_25);
    assert_eq!(peer_info.server_info.name, SERVER_NAME);

    let tools = client.list_tools(Default::default()).await?;
    assert_eq!(tools.tools.len(), 22);
    assert!(
        tools
            .tools
            .iter()
            .any(|tool| tool.name == "sora_server_info")
    );
    for tool in tools.tools {
        assert_eq!(
            tool.input_schema.get("additionalProperties"),
            Some(&Value::Bool(false))
        );
        assert!(tool.output_schema.is_some());
    }
    let prompts = client.list_prompts(None).await?;
    assert_eq!(prompts.prompts.len(), 7);

    let resources = client.list_resources(None).await?;
    assert_eq!(resources.resources.len(), 2);
    assert_eq!(resources.resources[0].uri, "sora://server/info");
    assert_eq!(resources.resources[1].uri, "sora://workspace/projects");
    let templates = client.list_resource_templates(None).await?;
    assert_eq!(templates.resource_templates.len(), 6);
    let server_info = client
        .read_resource(ReadResourceRequestParams::new("sora://server/info"))
        .await?;
    assert_eq!(server_info.contents.len(), 1);
    let completion = client
        .complete(CompleteRequestParams::new(
            Reference::for_resource("sora://project/{project_id}/table/{table}/rows"),
            ArgumentInfo::new("project_id", ""),
        ))
        .await?;
    assert!(completion.completion.values.is_empty());

    client.cancel().await?;
    server_handle.await??;
    Ok(())
}
