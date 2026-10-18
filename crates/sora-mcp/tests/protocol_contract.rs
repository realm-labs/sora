use rmcp::service::{ClientLifecycleMode, ClientServiceExt};
use std::{sync::Arc, time::Duration};

use anyhow::{Ok as anyhow_ok, Result, anyhow};
use rmcp::{
    ClientHandler, ServiceExt,
    model::{
        ArgumentInfo, ClientConfig, CompleteRequestParams, ErrorCode, Implementation,
        ProtocolVersion, ReadResourceRequestParams, Reference,
    },
};
use serde_json::{Value, from_str, json, to_value};
use sora_mcp::{SERVER_NAME, SoraMcpServer, TARGET_PROTOCOL_VERSION};
use sora_workspace::WorkspaceService;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, duplex},
    spawn,
    time::timeout,
};

#[derive(Debug, Clone)]
struct ContractClient;

impl ClientHandler for ContractClient {
    fn get_info(&self) -> ClientConfig {
        ClientConfig::new(
            Default::default(),
            Implementation::new("sora-contract-test", env!("CARGO_PKG_VERSION")),
        )
        .with_protocol_version(TARGET_PROTOCOL_VERSION)
    }
}

#[tokio::test]
async fn unsupported_protocol_is_rejected_during_discovery() -> Result<()> {
    let server = SoraMcpServer::new(Arc::new(WorkspaceService::new()));
    let (server_transport, mut client_transport) = duplex(16 * 1024);
    let server_handle = spawn(async move { server.serve(server_transport).await });
    let request = json!({
        "jsonrpc": "2.0", "id": 1, "method": "server/discover", "params": {"_meta": {
            "io.modelcontextprotocol/protocolVersion": "2025-11-25",
            "io.modelcontextprotocol/clientCapabilities": {}
        }}
    });
    client_transport
        .write_all(format!("{request}\n").as_bytes())
        .await?;
    let mut line = String::new();
    let mut reader = BufReader::new(client_transport);
    timeout(Duration::from_secs(5), reader.read_line(&mut line)).await??;
    let response: Value = from_str(&line)?;
    assert_eq!(
        response["error"]["code"],
        to_value(ErrorCode::UNSUPPORTED_PROTOCOL_VERSION)?
    );
    assert_eq!(
        response["error"]["data"]["supported"],
        json!(["2026-07-28"])
    );
    drop(reader);
    server_handle.abort();
    Ok(())
}

#[tokio::test]
async fn lifecycle_negotiates_the_pinned_protocol_and_lists_strict_tools() -> Result<()> {
    let (server_transport, client_transport) = duplex(16 * 1024);
    let server = SoraMcpServer::new(Arc::new(WorkspaceService::new()));
    let server_handle = spawn(async move {
        server.serve(server_transport).await?.waiting().await?;
        anyhow_ok(())
    });

    let client = ContractClient
        .serve_with_lifecycle(
            client_transport,
            ClientLifecycleMode::Discover {
                preferred_versions: vec![TARGET_PROTOCOL_VERSION],
            },
        )
        .await?;
    let peer_info = client
        .peer_info()
        .ok_or_else(|| anyhow!("server initialization info is missing"))?;
    assert_eq!(peer_info.protocol_version, ProtocolVersion::V_2026_07_28);
    assert_eq!(
        peer_info
            .server_info
            .as_ref()
            .expect("server identity")
            .name,
        SERVER_NAME
    );
    assert!(peer_info.capabilities.supports_tasks());

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
