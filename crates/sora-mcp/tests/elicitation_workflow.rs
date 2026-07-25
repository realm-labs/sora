use std::{
    fs,
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, AtomicUsize, Ordering},
    },
};

use rmcp::{
    ClientHandler, ServiceExt,
    model::{
        CallToolRequestParams, ClientCapabilities, ClientInfo, ElicitRequestParams, ElicitResult,
        ElicitationAction, ElicitationCapability, FormElicitationCapability, Implementation,
    },
};
use sora_mcp::{SoraMcpServer, TARGET_PROTOCOL_VERSION};
use sora_workspace::WorkspaceService;

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone)]
struct ElicitingClient {
    root_id: Arc<str>,
    requests: Arc<AtomicUsize>,
    messages: Arc<Mutex<Vec<String>>>,
}

impl ClientHandler for ElicitingClient {
    fn get_info(&self) -> ClientInfo {
        let mut capabilities = ClientCapabilities::default();
        capabilities.elicitation = Some(
            ElicitationCapability::new()
                .with_form(FormElicitationCapability::new().with_schema_validation(true)),
        );
        ClientInfo::new(
            capabilities,
            Implementation::new("sora-elicitation-test", env!("CARGO_PKG_VERSION")),
        )
        .with_protocol_version(TARGET_PROTOCOL_VERSION)
    }

    async fn create_elicitation(
        &self,
        request: ElicitRequestParams,
        _context: rmcp::service::RequestContext<rmcp::RoleClient>,
    ) -> Result<ElicitResult, rmcp::ErrorData> {
        let ElicitRequestParams::FormElicitationParams { message, .. } = request else {
            return Ok(ElicitResult::new(ElicitationAction::Decline));
        };
        self.messages
            .lock()
            .map_err(|_| rmcp::ErrorData::internal_error("message lock poisoned", None))?
            .push(message);
        let index = self.requests.fetch_add(1, Ordering::Relaxed);
        let content = match index {
            0 => serde_json::json!({
                "root_id": self.root_id.as_ref(),
                "relative_manifest": "project.toml"
            }),
            1 => serde_json::json!({
                "trust_project_scripts": true
            }),
            _ => return Ok(ElicitResult::new(ElicitationAction::Decline)),
        };
        Ok(ElicitResult::new(ElicitationAction::Accept).with_content(content))
    }
}

#[derive(Debug, Clone)]
struct NonElicitingClient;

impl ClientHandler for NonElicitingClient {
    fn get_info(&self) -> ClientInfo {
        ClientInfo::new(
            Default::default(),
            Implementation::new("sora-no-elicitation-test", env!("CARGO_PKG_VERSION")),
        )
        .with_protocol_version(TARGET_PROTOCOL_VERSION)
    }
}

#[tokio::test]
async fn project_selection_and_script_trust_use_capability_negotiated_elicitation()
-> anyhow::Result<()> {
    let root = scripted_project();
    let workspace = Arc::new(WorkspaceService::new());
    let workspace_root = workspace.add_root("test", &root)?;
    let handler = ElicitingClient {
        root_id: Arc::from(workspace_root.id()),
        requests: Arc::new(AtomicUsize::new(0)),
        messages: Arc::new(Mutex::new(Vec::new())),
    };
    let (server_transport, client_transport) = tokio::io::duplex(128 * 1024);
    let server = SoraMcpServer::new(Arc::clone(&workspace));
    let server_handle = tokio::spawn(async move {
        server.serve(server_transport).await?.waiting().await?;
        anyhow::Ok(())
    });
    let client = handler.serve(client_transport).await?;

    let result = client
        .call_tool(
            CallToolRequestParams::new("sora_project_open")
                .with_arguments(serde_json::json!({}).as_object().expect("object").clone()),
        )
        .await?;
    assert_eq!(result.is_error, Some(false));
    assert_eq!(client.service().requests.load(Ordering::Relaxed), 2);
    {
        let messages = client.service().messages.lock().expect("messages");
        assert!(messages[0].contains("untrusted project metadata"));
        assert!(messages[1].contains("SHA-256"));
        assert!(messages[1].contains("parser.lua"));
    }
    assert_eq!(workspace.project_ids()?.len(), 1);

    client.cancel().await?;
    server_handle.await??;
    fs::remove_dir_all(root)?;
    Ok(())
}

#[tokio::test]
async fn clients_without_elicitation_receive_an_explicit_parameter_fallback() -> anyhow::Result<()>
{
    let root = plain_project();
    let workspace = Arc::new(WorkspaceService::new());
    let workspace_root = workspace.add_root("test", &root)?;
    let (server_transport, client_transport) = tokio::io::duplex(64 * 1024);
    let server = SoraMcpServer::new(Arc::clone(&workspace));
    let server_handle = tokio::spawn(async move {
        server.serve(server_transport).await?.waiting().await?;
        anyhow::Ok(())
    });
    let client = NonElicitingClient.serve(client_transport).await?;

    let missing = client
        .call_tool(
            CallToolRequestParams::new("sora_project_open")
                .with_arguments(serde_json::json!({}).as_object().expect("object").clone()),
        )
        .await?;
    assert_eq!(missing.is_error, Some(true));

    let opened = client
        .call_tool(
            CallToolRequestParams::new("sora_project_open").with_arguments(
                serde_json::json!({
                    "root_id": workspace_root.id(),
                    "relative_manifest": "project.toml"
                })
                .as_object()
                .expect("object")
                .clone(),
            ),
        )
        .await?;
    assert_eq!(opened.is_error, Some(false));

    client.cancel().await?;
    server_handle.await??;
    fs::remove_dir_all(root)?;
    Ok(())
}

fn scripted_project() -> PathBuf {
    let root = plain_project();
    fs::write(
        root.join("project.toml"),
        r#"
project = { id = "demo" }
groups = { common = { default = true } }
views = { default = { contract = "demo/default", groups = ["common"] } }
includes = ["schema.toml"]

[parsers]
scripts = ["parser.lua"]
"#,
    )
    .unwrap();
    fs::write(
        root.join("parser.lua"),
        r#"
return {
  parsers = {
    identity = {
      parse = function(cell)
        return cell.text
      end,
    },
  },
}
"#,
    )
    .unwrap();
    root
}

fn plain_project() -> PathBuf {
    let nonce = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!("sora-mcp-elicit-{}-{nonce}", std::process::id()));
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
        r#"
[[tables]]
id = "settings"
name = "Settings"
mode = "singleton"
"#,
    )
    .unwrap();
    root
}
