use std::{
    fs,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::{SystemTime, UNIX_EPOCH},
};

use rmcp::{
    ClientHandler, ServiceExt,
    model::{
        CallToolRequestParams, ClientInfo, Implementation, ResourceUpdatedNotificationParam,
        SubscribeRequestParams,
    },
};
use sora_mcp::{SoraMcpServer, TARGET_PROTOCOL_VERSION};
use sora_workspace::{ProjectId, RuntimeOptions, WorkspaceService};
use tokio::sync::Notify;

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone)]
struct MutationClient {
    resource_updated: Arc<Notify>,
}

impl ClientHandler for MutationClient {
    fn get_info(&self) -> ClientInfo {
        ClientInfo::new(
            Default::default(),
            Implementation::new("sora-schema-mutation-test", env!("CARGO_PKG_VERSION")),
        )
        .with_protocol_version(TARGET_PROTOCOL_VERSION)
    }

    async fn on_resource_updated(
        &self,
        _params: ResourceUpdatedNotificationParam,
        _context: rmcp::service::NotificationContext<rmcp::RoleClient>,
    ) {
        self.resource_updated.notify_one();
    }
}

#[tokio::test]
async fn preview_and_apply_preserve_the_plan_transaction_contract() -> anyhow::Result<()> {
    let root = temp_project();
    let project = root.join("project.toml");
    let schema_path = root.join("schema.toml");
    let original = fs::read_to_string(&schema_path)?;
    let workspace = Arc::new(WorkspaceService::new());
    let id = ProjectId::new("demo")?;
    let session = workspace.open_project(id.clone(), project, RuntimeOptions::default())?;
    let revision = session.revision();
    let (server_transport, client_transport) = tokio::io::duplex(128 * 1024);
    let server = SoraMcpServer::new(Arc::clone(&workspace));
    let server_handle = tokio::spawn(async move {
        server.serve(server_transport).await?.waiting().await?;
        anyhow::Ok(())
    });
    let updated = Arc::new(Notify::new());
    let client = MutationClient {
        resource_updated: Arc::clone(&updated),
    }
    .serve(client_transport)
    .await?;
    client
        .subscribe(SubscribeRequestParams::new("sora://project/demo/revision"))
        .await?;

    let preview = client
        .call_tool(
            CallToolRequestParams::new("sora_schema_preview").with_arguments(
                serde_json::json!({
                    "project_id": "demo",
                    "expected_schema_revision": revision.schema,
                    "expected_manifest_revision": revision.manifest,
                    "operations": [{
                        "op": "add_field",
                        "owner": {
                            "kind": "table",
                            "name": "Item",
                            "variant": null
                        },
                        "field": {
                            "name": "name",
                            "type": "string",
                            "groups": [],
                            "parser": null,
                            "comment": null,
                            "default": null,
                            "range": null,
                            "length": null
                        }
                    }]
                })
                .as_object()
                .unwrap()
                .clone(),
            ),
        )
        .await?;
    assert_eq!(preview.is_error, Some(false));
    assert_eq!(fs::read_to_string(&schema_path)?, original);
    let plan_id = preview.structured_content.as_ref().unwrap()["data"]["plan_id"]
        .as_str()
        .unwrap()
        .to_owned();

    let apply = client
        .call_tool(
            CallToolRequestParams::new("sora_schema_apply").with_arguments(
                serde_json::json!({
                    "project_id": "demo",
                    "plan_id": plan_id,
                    "idempotency_key": "mutation-test-1"
                })
                .as_object()
                .unwrap()
                .clone(),
            ),
        )
        .await?;
    assert_eq!(apply.is_error, Some(false));
    assert!(fs::read_to_string(&schema_path)?.contains("name = \"name\""));
    tokio::time::timeout(std::time::Duration::from_secs(1), updated.notified()).await?;

    let rejected = client
        .call_tool(
            CallToolRequestParams::new("sora_schema_apply").with_arguments(
                serde_json::json!({
                    "project_id": "demo",
                    "plan_id": "plan:missing",
                    "idempotency_key": "mutation-test-2"
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
    let _ = fs::remove_dir_all(root);
    Ok(())
}

fn temp_project() -> PathBuf {
    let nonce = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "sora-mcp-schema-mutation-{}-{time}-{nonce}",
        std::process::id()
    ));
    fs::create_dir_all(&root).unwrap();
    fs::write(
        root.join("project.toml"),
        "project = { id = \"demo\" }\n\
         groups = { common = { default = true } }\n\
         views = { default = { contract = \"demo/default\", groups = [\"common\"] } }\n\
         includes = [\"schema.toml\"]\n",
    )
    .unwrap();
    fs::write(
        root.join("schema.toml"),
        r#"[[tables]]
id = "item"
name = "Item"
mode = "map"
key = "id"

[[tables.fields]]
name = "id"
type = "i32"
"#,
    )
    .unwrap();
    root
}
