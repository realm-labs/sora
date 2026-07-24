use std::{
    fs,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

use rmcp::{
    ClientHandler, ServiceExt,
    model::{CallToolRequestParams, ClientInfo, Implementation},
};
use sora_mcp::{SoraMcpServer, TARGET_PROTOCOL_VERSION};
use sora_workspace::{ProjectId, RuntimeOptions, WorkspaceService};

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone)]
struct DataClient;

impl ClientHandler for DataClient {
    fn get_info(&self) -> ClientInfo {
        ClientInfo::new(
            Default::default(),
            Implementation::new("sora-data-mutation-test", env!("CARGO_PKG_VERSION")),
        )
        .with_protocol_version(TARGET_PROTOCOL_VERSION)
    }
}

#[tokio::test]
async fn data_preview_and_apply_are_structured_and_transactional() -> anyhow::Result<()> {
    let root = temp_project();
    let data_path = root.join("data/items.json");
    let original = fs::read_to_string(&data_path)?;
    let workspace = Arc::new(WorkspaceService::new());
    let id = ProjectId::new("demo")?;
    let session =
        workspace.open_project(id, root.join("project.toml"), RuntimeOptions::default())?;
    let revision = session.revision();
    let (server_transport, client_transport) = tokio::io::duplex(128 * 1024);
    let server = SoraMcpServer::new(Arc::clone(&workspace));
    let server_handle = tokio::spawn(async move {
        server.serve(server_transport).await?.waiting().await?;
        anyhow::Ok(())
    });
    let client = DataClient.serve(client_transport).await?;

    let preview = client
        .call_tool(
            CallToolRequestParams::new("sora_data_preview").with_arguments(
                serde_json::json!({
                    "project_id": "demo",
                    "expected_schema_revision": revision.schema,
                    "expected_data_revision": revision.data,
                    "operations": [{
                        "op": "update_fields",
                        "table": "Item",
                        "selector": {"kind": "map", "key": 1},
                        "fields": {"name": "updated"}
                    }]
                })
                .as_object()
                .unwrap()
                .clone(),
            ),
        )
        .await?;
    assert_eq!(preview.is_error, Some(false));
    assert_eq!(fs::read_to_string(&data_path)?, original);
    let plan_id = preview.structured_content.as_ref().unwrap()["data"]["plan_id"]
        .as_str()
        .unwrap()
        .to_owned();

    let apply = client
        .call_tool(
            CallToolRequestParams::new("sora_data_apply").with_arguments(
                serde_json::json!({
                    "project_id": "demo",
                    "plan_id": plan_id,
                    "idempotency_key": "data-mutation-1"
                })
                .as_object()
                .unwrap()
                .clone(),
            ),
        )
        .await?;
    assert_eq!(apply.is_error, Some(false));
    assert!(fs::read_to_string(&data_path)?.contains("\"updated\""));

    let rejected = client
        .call_tool(
            CallToolRequestParams::new("sora_data_apply").with_arguments(
                serde_json::json!({
                    "project_id": "demo",
                    "plan_id": "plan:missing",
                    "idempotency_key": "data-mutation-2"
                })
                .as_object()
                .unwrap()
                .clone(),
            ),
        )
        .await?;
    assert_eq!(rejected.is_error, Some(true));
    assert_eq!(
        rejected.structured_content.as_ref().unwrap()["ok"],
        serde_json::Value::Bool(false)
    );

    client.cancel().await?;
    server_handle.await??;
    fs::remove_dir_all(root)?;
    Ok(())
}

fn temp_project() -> PathBuf {
    let nonce = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!(
        "sora-mcp-data-mutation-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir_all(root.join("data")).unwrap();
    fs::write(
        root.join("project.toml"),
        "package = \"demo\"\nincludes = [\"schema.toml\"]\n\n[build]\ndata_root = \"data\"\n",
    )
    .unwrap();
    fs::write(
        root.join("schema.toml"),
        r#"[[tables]]
name = "Item"
mode = "map"
key = "id"
source = { file = "items.json", format = "json" }

[[tables.fields]]
name = "id"
type = "i32"

[[tables.fields]]
name = "name"
type = "string"
"#,
    )
    .unwrap();
    fs::write(
        root.join("data/items.json"),
        "[{\"id\":1,\"name\":\"old\"}]\n",
    )
    .unwrap();
    root
}
