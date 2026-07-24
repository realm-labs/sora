use std::sync::Arc;

use rmcp::{
    ClientHandler, ServiceExt,
    model::{ClientInfo, Implementation, ProtocolVersion},
};
use sora_mcp::{SERVER_NAME, SoraMcpServer, TARGET_PROTOCOL_VERSION};
use sora_workspace::WorkspaceService;

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
async fn lifecycle_negotiates_the_pinned_protocol_and_lists_strict_tools() -> anyhow::Result<()> {
    let (server_transport, client_transport) = tokio::io::duplex(16 * 1024);
    let server = SoraMcpServer::new(Arc::new(WorkspaceService::new()));
    let server_handle = tokio::spawn(async move {
        server.serve(server_transport).await?.waiting().await?;
        anyhow::Ok(())
    });

    let client = ContractClient.serve(client_transport).await?;
    let peer_info = client
        .peer_info()
        .ok_or_else(|| anyhow::anyhow!("server initialization info is missing"))?;
    assert_eq!(peer_info.protocol_version, ProtocolVersion::V_2025_11_25);
    assert_eq!(peer_info.server_info.name, SERVER_NAME);

    let tools = client.list_tools(Default::default()).await?;
    assert_eq!(tools.tools.len(), 1);
    assert_eq!(tools.tools[0].name, "sora_server_info");
    assert_eq!(
        tools.tools[0].input_schema.get("additionalProperties"),
        Some(&serde_json::Value::Bool(false))
    );
    assert!(tools.tools[0].output_schema.is_some());

    client.cancel().await?;
    server_handle.await??;
    Ok(())
}
