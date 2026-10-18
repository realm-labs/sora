use rmcp::service::{ClientLifecycleMode, ClientServiceExt};
use std::{
    env::temp_dir as system_temp_dir,
    fs,
    path::PathBuf,
    process::id as process_id,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use anyhow::{Ok as anyhow_ok, Result};
use rmcp::{
    ClientHandler, ServiceExt,
    model::{
        CallToolRequestParams, ClientConfig, Implementation, ServerNotification, SubscriptionFilter,
    },
};
use serde_json::{Value, json};
use sora_mcp::{SoraMcpServer, TARGET_PROTOCOL_VERSION};
use sora_workspace::{ProjectId, RuntimeOptions, WorkspaceService};
use tokio::{io::duplex, spawn, time::timeout};

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone)]
struct MutationClient;

impl ClientHandler for MutationClient {
    fn get_info(&self) -> ClientConfig {
        ClientConfig::new(
            Default::default(),
            Implementation::new("sora-schema-mutation-test", env!("CARGO_PKG_VERSION")),
        )
        .with_protocol_version(TARGET_PROTOCOL_VERSION)
    }
}

#[tokio::test]
async fn preview_and_apply_preserve_the_plan_transaction_contract() -> Result<()> {
    let root = temp_project();
    let project = root.join("project.toml");
    let schema_path = root.join("schema.toml");
    let original = fs::read_to_string(&schema_path)?;
    let workspace = Arc::new(WorkspaceService::new());
    let id = ProjectId::new("demo")?;
    let session = workspace.open_project(id.clone(), project, RuntimeOptions::default())?;
    let revision = session.revision();
    let (server_transport, client_transport) = duplex(128 * 1024);
    let server = SoraMcpServer::new(Arc::clone(&workspace));
    let server_handle = spawn(async move {
        server.serve(server_transport).await?.waiting().await?;
        anyhow_ok(())
    });
    let client = MutationClient
        .serve_with_lifecycle(
            client_transport,
            ClientLifecycleMode::Discover {
                preferred_versions: vec![TARGET_PROTOCOL_VERSION],
            },
        )
        .await?;
    let mut filter = SubscriptionFilter::new();
    filter.resource_subscriptions = Some(vec!["sora://project/demo/revision".to_owned()]);
    let mut subscription = client.listen(filter).await?;

    let preview = client
        .call_tool(
            CallToolRequestParams::new("sora_schema_preview").with_arguments(
                json!({
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
                json!({
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
    assert!(fs::read_to_string(&schema_path)?.contains("[tables.Item.fields.name]"));
    let notification = timeout(Duration::from_secs(1), subscription.next())
        .await??
        .expect("resource notification");
    assert!(
        matches!(notification, ServerNotification::ResourceUpdatedNotification(notification) if notification.params.uri == "sora://project/demo/revision")
    );

    let rejected = client
        .call_tool(
            CallToolRequestParams::new("sora_schema_apply").with_arguments(
                json!({
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
        Value::Bool(false)
    );

    subscription.cancel().await?;
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
    let root = system_temp_dir().join(format!(
        "sora-mcp-schema-mutation-{}-{time}-{nonce}",
        process_id()
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
        r#"[tables.Item]
id = "item"
mode = "map"
key = "id"

[tables.Item.fields]
id = "i32"
"#,
    )
    .unwrap();
    root
}
