use std::{
    fs,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicU64, AtomicUsize, Ordering},
    },
};

use rmcp::{
    ClientHandler, ServiceExt,
    model::{
        CallToolRequestParams, ClientInfo, Implementation, ProgressNotificationParam,
        ReadResourceRequestParams,
    },
};
use sora_mcp::{SoraMcpServer, TARGET_PROTOCOL_VERSION};
use sora_workspace::{ProjectId, RuntimeOptions, WorkspaceService};

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Default)]
struct BuildClient {
    progress_count: Arc<AtomicUsize>,
}

impl ClientHandler for BuildClient {
    fn get_info(&self) -> ClientInfo {
        ClientInfo::new(
            Default::default(),
            Implementation::new("sora-build-test", env!("CARGO_PKG_VERSION")),
        )
        .with_protocol_version(TARGET_PROTOCOL_VERSION)
    }

    async fn on_progress(
        &self,
        _params: ProgressNotificationParam,
        _context: rmcp::service::NotificationContext<rmcp::RoleClient>,
    ) {
        self.progress_count.fetch_add(1, Ordering::Relaxed);
    }
}

#[tokio::test]
async fn schema_lock_build_reports_progress_and_exposes_immutable_artifact() -> anyhow::Result<()> {
    let root = temp_project();
    let workspace = Arc::new(WorkspaceService::new());
    let session = workspace.open_project(
        ProjectId::new("demo")?,
        root.join("project.toml"),
        RuntimeOptions::default(),
    )?;
    let revision = session.revision();
    let (server_transport, client_transport) = tokio::io::duplex(128 * 1024);
    let server = SoraMcpServer::new(Arc::clone(&workspace));
    let server_handle = tokio::spawn(async move {
        server.serve(server_transport).await?.waiting().await?;
        anyhow::Ok(())
    });
    let client = BuildClient::default().serve(client_transport).await?;

    let result = client
        .call_tool(
            CallToolRequestParams::new("sora_schema_lock").with_arguments(
                serde_json::json!({
                    "project_id": "demo",
                    "expected_project_revision": revision.project,
                    "scope": null,
                    "clean": true
                })
                .as_object()
                .unwrap()
                .clone(),
            ),
        )
        .await?;

    assert_eq!(result.is_error, Some(false));
    assert!(root.join("generated/schema.lock").is_file());
    assert!(client.service().progress_count.load(Ordering::Relaxed) >= 8);
    let artifact_uri = result.structured_content.as_ref().unwrap()["artifacts"][0]["uri"]
        .as_str()
        .unwrap();
    let artifact = client
        .read_resource(ReadResourceRequestParams::new(artifact_uri))
        .await?;
    assert_eq!(artifact.contents.len(), 1);

    client.cancel().await?;
    server_handle.await??;
    fs::remove_dir_all(root)?;
    Ok(())
}

fn temp_project() -> PathBuf {
    let nonce = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!("sora-mcp-build-{}-{nonce}", std::process::id()));
    fs::create_dir_all(root.join("schema")).unwrap();
    fs::write(
        root.join("project.toml"),
        r#"
package = "demo"
includes = ["schema/settings.toml"]

[build]
schema_lock = "generated/schema.lock"
"#,
    )
    .unwrap();
    fs::write(
        root.join("schema/settings.toml"),
        r#"
[[tables]]
name = "Settings"
mode = "singleton"

[[tables.fields]]
name = "name"
type = "string"
"#,
    )
    .unwrap();
    root
}
