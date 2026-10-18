use std::{
    env::temp_dir as system_temp_dir,
    fs,
    path::PathBuf,
    process::id as process_id,
    sync::{
        Arc,
        atomic::{AtomicU64, AtomicUsize, Ordering},
    },
    time::Duration,
};

use anyhow::{Ok as anyhow_ok, Result, bail};
use rmcp::{
    ClientHandler, RoleClient, ServiceError, ServiceExt,
    model::{
        CallToolRequestParams, CallToolResponse, CancelTaskParams, ClientCapabilities,
        ClientConfig, DetailedTask, ErrorCode, GetTaskParams, Implementation, NumberOrString,
        ProgressNotificationParam, ProgressToken, ReadResourceRequestParams, RequestMetaObject,
        TaskPayload, TaskStatus, TaskStatusNotificationParams, UpdateTaskParams,
    },
    service::{ClientLifecycleMode, ClientServiceExt, NotificationContext},
};
use serde_json::{from_str, json, to_value};
use sora_mcp::{SoraMcpServer, TARGET_PROTOCOL_VERSION};
use sora_workspace::{ProjectId, RuntimeOptions, WorkspaceService};
use tokio::{
    io::duplex,
    spawn,
    time::{sleep, timeout},
};

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Default)]
struct TaskClient {
    supports_tasks: bool,
    progress_count: Arc<AtomicUsize>,
    status_count: Arc<AtomicUsize>,
}

impl ClientHandler for TaskClient {
    fn get_info(&self) -> ClientConfig {
        let capabilities = if self.supports_tasks {
            ClientCapabilities::builder().enable_tasks().build()
        } else {
            ClientCapabilities::default()
        };
        ClientConfig::new(
            capabilities,
            Implementation::new("sora-task-test", env!("CARGO_PKG_VERSION")),
        )
        .with_protocol_version(TARGET_PROTOCOL_VERSION)
    }

    async fn on_progress(
        &self,
        _params: ProgressNotificationParam,
        _context: NotificationContext<RoleClient>,
    ) {
        self.progress_count.fetch_add(1, Ordering::Relaxed);
    }

    async fn on_task_status(
        &self,
        _params: TaskStatusNotificationParams,
        _context: NotificationContext<RoleClient>,
    ) {
        self.status_count.fetch_add(1, Ordering::Relaxed);
    }
}

#[tokio::test]
async fn server_directed_task_is_polled_with_inline_result_and_project_resource() -> Result<()> {
    let root = temp_project();
    let workspace = Arc::new(WorkspaceService::new());
    let session = workspace.open_project(
        ProjectId::new("demo")?,
        root.join("project.toml"),
        RuntimeOptions::default(),
    )?;
    let revision = session.revision();
    let (server_transport, client_transport) = duplex(128 * 1024);
    let server = SoraMcpServer::new(workspace);
    let server_handle = spawn(async move {
        server.serve(server_transport).await?.waiting().await?;
        anyhow_ok(())
    });
    let client = TaskClient {
        supports_tasks: true,
        ..Default::default()
    }
    .serve_with_lifecycle(
        client_transport,
        ClientLifecycleMode::Discover {
            preferred_versions: vec![TARGET_PROTOCOL_VERSION],
        },
    )
    .await?;

    let response = client
        .call_tool_once(build_request(&revision.project))
        .await?;
    let CallToolResponse::Task(created) = response else {
        bail!("expected server-directed task")
    };
    assert_eq!(created.task.ttl_ms, Some(300_000));
    assert_eq!(created.task.poll_interval_ms, Some(250));
    let task_id = created.task.task_id;
    let completed = timeout(Duration::from_secs(10), async {
        loop {
            let task = client.get_task(GetTaskParams::new(&task_id)).await?.task;
            if task.status().is_terminal() {
                break anyhow_ok(task);
            }
            sleep(Duration::from_millis(
                task.task.poll_interval_ms.unwrap_or(250),
            ))
            .await;
        }
    })
    .await??;
    let TaskPayload::Completed { result } = completed.payload else {
        bail!("expected completed result")
    };
    assert_eq!(result["isError"], false);
    assert_eq!(result["structuredContent"]["ok"], true);
    assert!(root.join("generated/schema.lock").is_file());

    let resource = client
        .read_resource(ReadResourceRequestParams::new(format!(
            "sora://project/demo/task/{task_id}"
        )))
        .await?;
    let serialized = to_value(&resource)?;
    let text = serialized["contents"][0]["text"]
        .as_str()
        .expect("resource JSON");
    let resource_task: DetailedTask = from_str(text)?;
    assert_eq!(resource_task.status(), TaskStatus::Completed);
    assert_eq!(resource_task.task.task_id, task_id);
    assert!(
        client
            .read_resource(ReadResourceRequestParams::new(format!(
                "sora://project/other/task/{task_id}"
            )))
            .await
            .is_err()
    );
    assert!(
        client
            .update_task(UpdateTaskParams::new(&task_id, Default::default()))
            .await
            .is_err()
    );
    // Materialized work is observed through polling, not a stale request's
    // progress token or unsolicited task status notifications.
    assert_eq!(client.service().progress_count.load(Ordering::Relaxed), 0);
    assert_eq!(client.service().status_count.load(Ordering::Relaxed), 0);

    // Non-heavy tools stay synchronous even when the client declares tasks.
    assert!(matches!(
        client
            .call_tool_once(
                CallToolRequestParams::new("sora_server_info").with_arguments(Default::default())
            )
            .await?,
        CallToolResponse::Complete(_)
    ));

    fs::write(root.join("generated/schema.lock"), "sentinel")?;
    let CallToolResponse::Task(created) = client
        .call_tool_once(build_request(&revision.project))
        .await?
    else {
        bail!("expected task")
    };
    client
        .cancel_task(CancelTaskParams::new(&created.task.task_id))
        .await?;
    let cancelled = timeout(Duration::from_secs(10), async {
        loop {
            let task = client
                .get_task(GetTaskParams::new(&created.task.task_id))
                .await?
                .task;
            if task.status().is_terminal() {
                break anyhow_ok(task);
            }
            sleep(Duration::from_millis(10)).await;
        }
    })
    .await??;
    let contents = fs::read_to_string(root.join("generated/schema.lock"))?;
    match cancelled.status() {
        TaskStatus::Cancelled => assert_eq!(contents, "sentinel"),
        TaskStatus::Completed => assert_ne!(contents, "sentinel"),
        other => bail!("unexpected cancellation outcome: {other:?}"),
    }
    client.cancel().await?;
    server_handle.await??;
    fs::remove_dir_all(root)?;
    Ok(())
}

#[tokio::test]
async fn task_business_error_is_completed_with_the_tool_error_payload() -> Result<()> {
    let (server_transport, client_transport) = duplex(32 * 1024);
    let server_handle = spawn(async move {
        SoraMcpServer::new(Arc::new(WorkspaceService::new()))
            .serve(server_transport)
            .await?
            .waiting()
            .await?;
        anyhow_ok(())
    });
    let client = TaskClient {
        supports_tasks: true,
        ..Default::default()
    }
    .serve_with_lifecycle(
        client_transport,
        ClientLifecycleMode::Discover {
            preferred_versions: vec![TARGET_PROTOCOL_VERSION],
        },
    )
    .await?;
    let CallToolResponse::Task(created) = client
        .call_tool_once(build_request("missing-revision"))
        .await?
    else {
        bail!("expected task")
    };
    let completed = timeout(Duration::from_secs(10), async {
        loop {
            let task = client
                .get_task(GetTaskParams::new(&created.task.task_id))
                .await?
                .task;
            if task.status().is_terminal() {
                break anyhow_ok(task);
            }
            sleep(Duration::from_millis(10)).await;
        }
    })
    .await??;
    assert_eq!(completed.status(), TaskStatus::Completed);
    let TaskPayload::Completed { result } = completed.payload else {
        bail!("expected tool error result")
    };
    assert_eq!(result["isError"], true);
    assert_eq!(result["structuredContent"]["ok"], false);
    client.cancel().await?;
    server_handle.await??;
    Ok(())
}

#[tokio::test]
async fn non_declaring_client_gets_synchronous_tools_and_cannot_poll_tasks() -> Result<()> {
    let root = temp_project();
    let workspace = Arc::new(WorkspaceService::new());
    let session = workspace.open_project(
        ProjectId::new("demo")?,
        root.join("project.toml"),
        RuntimeOptions::default(),
    )?;
    let (server_transport, client_transport) = duplex(32 * 1024);
    let server = SoraMcpServer::new(workspace);
    let server_handle = spawn(async move {
        server.serve(server_transport).await?.waiting().await?;
        anyhow_ok(())
    });
    let client = TaskClient::default()
        .serve_with_lifecycle(
            client_transport,
            ClientLifecycleMode::Discover {
                preferred_versions: vec![TARGET_PROTOCOL_VERSION],
            },
        )
        .await?;
    let response = client
        .call_tool_once(build_request(&session.revision().project))
        .await?;
    let CallToolResponse::Complete(result) = response else {
        bail!("no task capability must mean synchronous execution")
    };
    assert_eq!(result.is_error, Some(false));
    assert!(root.join("generated/schema.lock").is_file());
    let error = client
        .get_task(GetTaskParams::new("unknown"))
        .await
        .unwrap_err();
    assert!(
        matches!(error, ServiceError::McpError(error) if error.code == ErrorCode::MISSING_REQUIRED_CLIENT_CAPABILITY)
    );
    client.cancel().await?;
    server_handle.await??;
    fs::remove_dir_all(root)?;
    Ok(())
}

fn build_request(revision: &str) -> CallToolRequestParams {
    let mut request = CallToolRequestParams::new("sora_schema_lock").with_arguments(
        json!({
            "project_id": "demo", "expected_project_revision": revision, "view": null, "clean": true
        })
        .as_object()
        .unwrap()
        .clone(),
    );
    request.meta = Some(RequestMetaObject::with_progress_token(ProgressToken(
        NumberOrString::Number(1),
    )));
    request
}

fn temp_project() -> PathBuf {
    let nonce = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let root = system_temp_dir().join(format!("sora-mcp-task-{}-{nonce}", process_id()));
    fs::create_dir_all(root.join("schema")).unwrap();
    fs::write(
        root.join("project.toml"),
        r#"
project = { id = "demo" }
groups = { common = { default = true } }
views = { default = { contract = "demo/default", groups = ["common"] } }
includes = ["schema/settings.toml"]
[build]
schema_lock = "generated/schema.lock"
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
