use std::{fs, path::PathBuf, sync::Arc};

use rmcp::{
    ClientHandler, ServiceExt,
    model::{CallToolRequestParams, ClientInfo, Implementation},
};
use sora_mcp::{SoraMcpServer, TARGET_PROTOCOL_VERSION};
use sora_workspace::{ProjectId, RuntimeOptions, WorkspaceService};

#[derive(Debug, Clone)]
struct ReadOnlyClient;

impl ClientHandler for ReadOnlyClient {
    fn get_info(&self) -> ClientInfo {
        ClientInfo::new(
            Default::default(),
            Implementation::new("sora-read-only-test", env!("CARGO_PKG_VERSION")),
        )
        .with_protocol_version(TARGET_PROTOCOL_VERSION)
    }
}

#[tokio::test]
async fn inspect_and_query_use_structured_results_without_changing_revision() -> anyhow::Result<()>
{
    let workspace = Arc::new(WorkspaceService::new());
    let project =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/showcase/project.scon");
    let session = workspace.open_project(
        ProjectId::new("showcase")?,
        project,
        RuntimeOptions::default(),
    )?;
    let revision = session.revision();
    let (server_transport, client_transport) = tokio::io::duplex(64 * 1024);
    let server = SoraMcpServer::new(Arc::clone(&workspace));
    let server_handle = tokio::spawn(async move {
        server.serve(server_transport).await?.waiting().await?;
        anyhow::Ok(())
    });
    let client = ReadOnlyClient.serve(client_transport).await?;

    let inspect = client
        .call_tool(
            CallToolRequestParams::new("sora_project_inspect").with_arguments(
                serde_json::json!({"project_id": "showcase"})
                    .as_object()
                    .unwrap()
                    .clone(),
            ),
        )
        .await?;
    assert_eq!(inspect.is_error, Some(false));
    assert_eq!(
        inspect.structured_content.as_ref().unwrap()["ok"],
        serde_json::Value::Bool(true)
    );

    let query = client
        .call_tool(
            CallToolRequestParams::new("sora_table_query").with_arguments(
                serde_json::json!({
                    "project_id": "showcase",
                    "table": "Item",
                    "select": ["id", "name"],
                    "limit": 2
                })
                .as_object()
                .unwrap()
                .clone(),
            ),
        )
        .await?;
    assert_eq!(query.is_error, Some(false));
    assert_eq!(
        query.structured_content.as_ref().unwrap()["data"]["rows"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        workspace.project(&ProjectId::new("showcase")?)?.revision(),
        revision
    );

    client.cancel().await?;
    server_handle.await??;
    Ok(())
}

#[tokio::test]
async fn mcp_query_uses_the_project_lua_source_registry() -> anyhow::Result<()> {
    let root =
        std::env::temp_dir().join(format!("sora-mcp-lua-source-loader-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("data/items"))?;
    fs::create_dir_all(root.join("tools"))?;
    fs::write(
        root.join("project.toml"),
        r#"
project = { id = "mcp_custom" }
groups = { common = { default = true } }
views = { default = { contract = "mcp_custom/default", groups = ["common"] } }
[source_loaders]
scripts = ["tools/source_loaders.lua"]
[build]
data_root = "data"
[tables.Item]
id = "item"
mode = "map"
key = "id"
source = { file = "items", format = "custom_format" }

[tables.Item.fields]
id = "i32"
name = "string"
"#,
    )?;
    fs::write(
        root.join("tools/source_loaders.lua"),
        r#"
return { source_loaders = { custom_format = {
  load = function(source, ctx)
    local rows = {}
    for _, entry in ipairs(ctx.list(".")) do
      table.insert(rows, ctx.json_decode(ctx.read_text(entry.path)))
    end
    return { rows = rows }
  end,
} } }
"#,
    )?;
    fs::write(
        root.join("data/items/item.json"),
        r#"{"id": 7, "name": "from-lua"}"#,
    )?;

    let workspace = Arc::new(WorkspaceService::new());
    workspace.open_project(
        ProjectId::new("mcp-custom")?,
        root.join("project.toml"),
        RuntimeOptions::default(),
    )?;
    let (server_transport, client_transport) = tokio::io::duplex(64 * 1024);
    let server = SoraMcpServer::new(Arc::clone(&workspace));
    let server_handle = tokio::spawn(async move {
        server.serve(server_transport).await?.waiting().await?;
        anyhow::Ok(())
    });
    let client = ReadOnlyClient.serve(client_transport).await?;
    let query = client
        .call_tool(
            CallToolRequestParams::new("sora_table_query").with_arguments(
                serde_json::json!({
                    "project_id": "mcp-custom",
                    "table": "Item",
                    "limit": 10
                })
                .as_object()
                .unwrap()
                .clone(),
            ),
        )
        .await?;
    assert_eq!(query.is_error, Some(false));
    assert_eq!(
        query.structured_content.as_ref().unwrap()["data"]["rows"][0]["values"]["name"],
        serde_json::json!("from-lua")
    );
    client.cancel().await?;
    server_handle.await??;
    let _ = fs::remove_dir_all(root);
    Ok(())
}
