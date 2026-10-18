use rmcp::service::{ClientLifecycleMode, ClientServiceExt};
use std::{
    collections::HashMap,
    env::temp_dir as system_temp_dir,
    fs,
    path::PathBuf,
    process::id as process_id,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

use anyhow::{Ok as anyhow_ok, Result};
use rmcp::{
    ClientHandler, ServiceExt,
    model::{
        ArgumentInfo, ClientConfig, CompleteRequestParams, CompletionContext,
        GetPromptRequestParams, Implementation, Reference,
    },
};
use serde_json::{json, to_value};
use sora_mcp::{SoraMcpServer, TARGET_PROTOCOL_VERSION};
use sora_workspace::{ProjectId, RuntimeOptions, WorkspaceService};
use tokio::{io::duplex, spawn};

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone)]
struct PromptClient;

impl ClientHandler for PromptClient {
    fn get_info(&self) -> ClientConfig {
        ClientConfig::new(
            Default::default(),
            Implementation::new("sora-prompt-test", env!("CARGO_PKG_VERSION")),
        )
        .with_protocol_version(TARGET_PROTOCOL_VERSION)
    }
}

#[tokio::test]
async fn prompts_embed_bounded_project_context_and_complete_arguments() -> Result<()> {
    let root = temp_project();
    let workspace = Arc::new(WorkspaceService::new());
    workspace.open_project(
        ProjectId::new("demo")?,
        root.join("project.toml"),
        RuntimeOptions::default(),
    )?;
    let (server_transport, client_transport) = duplex(128 * 1024);
    let server = SoraMcpServer::new(Arc::clone(&workspace));
    let server_handle = spawn(async move {
        server.serve(server_transport).await?.waiting().await?;
        anyhow_ok(())
    });
    let client = PromptClient
        .serve_with_lifecycle(
            client_transport,
            ClientLifecycleMode::Discover {
                preferred_versions: vec![TARGET_PROTOCOL_VERSION],
            },
        )
        .await?;

    let prompts = client.list_prompts(None).await?;
    assert_eq!(prompts.prompts.len(), 7);
    assert_eq!(prompts.prompts[0].name, "sora_create_table");
    assert_eq!(prompts.prompts[6].name, "sora_review_schema");

    let prompt = client
        .get_prompt(
            GetPromptRequestParams::new("sora_rename_entity_safely").with_arguments(
                json!({
                    "project_id": "demo",
                    "entity_kind": "table",
                    "entity_name": "Settings",
                    "new_name": "GameSettings"
                })
                .as_object()
                .unwrap()
                .clone(),
            ),
        )
        .await?;
    assert_eq!(prompt.messages.len(), 4);
    let serialized = to_value(&prompt)?;
    let text = serialized.to_string();
    assert!(text.contains("sora_schema_preview"));
    assert!(text.contains("untrusted project data"));
    assert!(text.contains("sora://project/demo/schema"));
    assert!(text.contains("sora://project/demo/diagnostics"));

    let completion = client
        .complete(
            CompleteRequestParams::new(
                Reference::for_prompt("sora_rename_entity_safely"),
                ArgumentInfo::new("entity_name", "St"),
            )
            .with_context(CompletionContext::with_arguments(HashMap::from([
                ("project_id".to_owned(), "demo".to_owned()),
                ("entity_kind".to_owned(), "table".to_owned()),
            ]))),
        )
        .await?;
    assert_eq!(completion.completion.values, ["Settings"]);

    let fuzzy = client
        .complete(
            CompleteRequestParams::new(
                Reference::for_prompt("sora_review_schema"),
                ArgumentInfo::new("entity_name", "stg"),
            )
            .with_context(CompletionContext::with_arguments(HashMap::from([(
                "project_id".to_owned(),
                "demo".to_owned(),
            )]))),
        )
        .await?;
    assert_eq!(fuzzy.completion.values, ["Settings"]);

    client.cancel().await?;
    server_handle.await??;
    fs::remove_dir_all(root)?;
    Ok(())
}

fn temp_project() -> PathBuf {
    let nonce = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let root = system_temp_dir().join(format!("sora-mcp-prompt-{}-{nonce}", process_id()));
    fs::create_dir_all(root.join("schema")).unwrap();
    fs::write(
        root.join("project.toml"),
        r#"
project = { id = "demo" }
groups = { common = { default = true } }
views = { default = { contract = "demo/default", groups = ["common"] } }
includes = ["schema/settings.toml"]
"#,
    )
    .unwrap();
    fs::write(
        root.join("schema/settings.toml"),
        r#"
[tables.Settings]
id = "settings"
mode = "singleton"

[tables.Settings.fields]
name = "string"
"#,
    )
    .unwrap();
    root
}
