use rmcp::service::{ClientLifecycleMode, ClientServiceExt};
use std::{
    env::temp_dir as system_temp_dir,
    fs,
    path::PathBuf,
    process::id as process_id,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, AtomicUsize, Ordering},
    },
};

use anyhow::{Ok as anyhow_ok, Result};
#[allow(deprecated)]
use rmcp::model::{ListRootsResult, Root};
use rmcp::{
    ClientHandler, ErrorData, RoleClient, ServiceExt,
    model::{
        CallToolRequestParams, ClientCapabilities, ClientConfig, ElicitRequestParams, ElicitResult,
        ElicitationAction, ElicitationCapability, FormElicitationCapability, Implementation,
        ReadResourceRequestParams,
    },
    service::RequestContext,
};
use serde_json::{Value, from_str, json, to_value};
use sora_mcp::{SoraMcpServer, TARGET_PROTOCOL_VERSION};
use sora_workspace::WorkspaceService;
use tokio::{io::duplex, spawn};
use url::Url;

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone)]
struct ElicitingClient {
    root_id: Arc<str>,
    requests: Arc<AtomicUsize>,
    messages: Arc<Mutex<Vec<String>>>,
}

impl ClientHandler for ElicitingClient {
    fn get_info(&self) -> ClientConfig {
        let mut capabilities = ClientCapabilities::default();
        capabilities.elicitation = Some(
            ElicitationCapability::new()
                .with_form(FormElicitationCapability::new().with_schema_validation(true)),
        );
        ClientConfig::new(
            capabilities,
            Implementation::new("sora-elicitation-test", env!("CARGO_PKG_VERSION")),
        )
        .with_protocol_version(TARGET_PROTOCOL_VERSION)
    }

    async fn create_elicitation(
        &self,
        request: ElicitRequestParams,
        context: RequestContext<RoleClient>,
    ) -> Result<ElicitResult, ErrorData> {
        assert!(matches!(
            context.id.to_string().as_str(),
            "project_selection" | "script_trust"
        ));
        let ElicitRequestParams::FormElicitationParams { message, .. } = request else {
            return Ok(ElicitResult::new(ElicitationAction::Decline));
        };
        self.messages
            .lock()
            .map_err(|_| ErrorData::internal_error("message lock poisoned", None))?
            .push(message);
        let index = self.requests.fetch_add(1, Ordering::Relaxed);
        let content = match index {
            0 => json!({
                "root_id": self.root_id.as_ref(),
                "relative_manifest": "project.toml"
            }),
            1 => json!({
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
    fn get_info(&self) -> ClientConfig {
        ClientConfig::new(
            Default::default(),
            Implementation::new("sora-no-elicitation-test", env!("CARGO_PKG_VERSION")),
        )
        .with_protocol_version(TARGET_PROTOCOL_VERSION)
    }
}

#[tokio::test]
async fn project_selection_and_script_trust_use_capability_negotiated_elicitation() -> Result<()> {
    let root = scripted_project();
    let workspace = Arc::new(WorkspaceService::new());
    let workspace_root = workspace.add_root("test", &root)?;
    let handler = ElicitingClient {
        root_id: Arc::from(workspace_root.id()),
        requests: Arc::new(AtomicUsize::new(0)),
        messages: Arc::new(Mutex::new(Vec::new())),
    };
    let (server_transport, client_transport) = duplex(128 * 1024);
    let server = SoraMcpServer::new(Arc::clone(&workspace));
    let server_handle = spawn(async move {
        server.serve(server_transport).await?.waiting().await?;
        anyhow_ok(())
    });
    let client = handler
        .serve_with_lifecycle(
            client_transport,
            ClientLifecycleMode::Discover {
                preferred_versions: vec![TARGET_PROTOCOL_VERSION],
            },
        )
        .await?;

    let result = client
        .call_tool(
            CallToolRequestParams::new("sora_project_open")
                .with_arguments(json!({}).as_object().expect("object").clone()),
        )
        .await?;
    assert_eq!(result.is_error, Some(false), "{result:?}");
    assert_eq!(
        client.service().requests.load(Ordering::Relaxed),
        2,
        "{result:?}"
    );
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
async fn clients_without_elicitation_receive_an_explicit_parameter_fallback() -> Result<()> {
    let root = plain_project();
    let workspace = Arc::new(WorkspaceService::new());
    let workspace_root = workspace.add_root("test", &root)?;
    let (server_transport, client_transport) = duplex(64 * 1024);
    let server = SoraMcpServer::new(Arc::clone(&workspace));
    let server_handle = spawn(async move {
        server.serve(server_transport).await?.waiting().await?;
        anyhow_ok(())
    });
    let client = NonElicitingClient
        .serve_with_lifecycle(
            client_transport,
            ClientLifecycleMode::Discover {
                preferred_versions: vec![TARGET_PROTOCOL_VERSION],
            },
        )
        .await?;

    let missing = client
        .call_tool(
            CallToolRequestParams::new("sora_project_open")
                .with_arguments(json!({}).as_object().expect("object").clone()),
        )
        .await?;
    assert_eq!(missing.is_error, Some(true));

    let opened = client
        .call_tool(
            CallToolRequestParams::new("sora_project_open").with_arguments(
                json!({
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

#[tokio::test]
async fn explicit_project_selection_does_not_bypass_script_trust() -> Result<()> {
    let root = scripted_project();
    let workspace = Arc::new(WorkspaceService::new());
    let workspace_root = workspace.add_root("test", &root)?;
    let (server_transport, client_transport) = duplex(64 * 1024);
    let server = SoraMcpServer::new(workspace.clone());
    let server_handle = spawn(async move {
        server.serve(server_transport).await?.waiting().await?;
        anyhow_ok(())
    });
    let client = NonElicitingClient
        .serve_with_lifecycle(
            client_transport,
            ClientLifecycleMode::Discover {
                preferred_versions: vec![TARGET_PROTOCOL_VERSION],
            },
        )
        .await?;
    let result = client
        .call_tool(
            CallToolRequestParams::new("sora_project_open").with_arguments(
                json!({
                    "root_id": workspace_root.id(), "relative_manifest": "project.toml"
                })
                .as_object()
                .unwrap()
                .clone(),
            ),
        )
        .await?;
    assert_eq!(result.is_error, Some(true));
    assert_eq!(
        result.structured_content.as_ref().unwrap()["summary"],
        "project scripts require explicit trust"
    );
    assert!(workspace.project_ids()?.is_empty());
    client.cancel().await?;
    server_handle.await??;
    fs::remove_dir_all(root)?;
    Ok(())
}

#[derive(Clone, Debug)]
struct RootsClient {
    uri: String,
}

#[allow(deprecated)]
impl ClientHandler for RootsClient {
    fn get_info(&self) -> ClientConfig {
        ClientConfig::new(
            ClientCapabilities::builder().enable_roots().build(),
            Implementation::new("sora-roots-test", "1"),
        )
        .with_protocol_version(TARGET_PROTOCOL_VERSION)
    }

    async fn list_roots(
        &self,
        context: RequestContext<RoleClient>,
    ) -> Result<ListRootsResult, ErrorData> {
        assert_eq!(context.id.to_string(), "roots");
        Ok(ListRootsResult::new(vec![
            Root::new(self.uri.clone()).with_name("test"),
        ]))
    }
}

#[tokio::test]
async fn roots_are_requested_within_the_modern_tool_request_not_initialization() -> Result<()> {
    let root = plain_project();
    let workspace = Arc::new(WorkspaceService::new());
    let (server_transport, client_transport) = duplex(64 * 1024);
    let server = SoraMcpServer::new(workspace.clone());
    let server_handle = spawn(async move {
        server.serve(server_transport).await?.waiting().await?;
        anyhow_ok(())
    });
    let client = RootsClient {
        uri: Url::from_directory_path(&root).unwrap().to_string(),
    }
    .serve_with_lifecycle(
        client_transport,
        ClientLifecycleMode::Discover {
            preferred_versions: vec![TARGET_PROTOCOL_VERSION],
        },
    )
    .await?;
    let result = client
        .call_tool(
            CallToolRequestParams::new("sora_project_list").with_arguments(Default::default()),
        )
        .await?;
    assert_eq!(result.is_error, Some(false), "{result:?}");
    assert_eq!(
        result.structured_content.as_ref().unwrap()["data"]["projects"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    // Client roots must not grant discovery authority to another authorization server.
    assert!(workspace.discover_projects()?.is_empty());
    let resource = client
        .read_resource(ReadResourceRequestParams::new("sora://workspace/projects"))
        .await?;
    let resource = to_value(resource)?;
    let projects: Value = from_str(resource["contents"][0]["text"].as_str().unwrap())?;
    assert_eq!(projects.as_array().unwrap().len(), 1);
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
    let root = system_temp_dir().join(format!("sora-mcp-elicit-{}-{nonce}", process_id()));
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
[tables.Settings]
id = "settings"
mode = "singleton"
"#,
    )
    .unwrap();
    root
}
