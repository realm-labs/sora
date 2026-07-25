use std::{
    fs,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicU64, AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};

use rmcp::{
    ClientHandler, ServiceExt,
    model::{
        CallToolRequest, CallToolRequestParams, CancelTaskParams, CancelTaskRequest,
        ClientCapabilities, ClientInfo, ClientRequest, GetTaskParams, GetTaskPayloadParams,
        GetTaskPayloadRequest, GetTaskRequest, Implementation, ListTasksRequest,
        ReadResourceRequestParams, ServerResult, TaskMetadata, TaskStatus,
        TaskStatusNotificationParam, TasksCapability,
    },
};
use sora_mcp::{SoraMcpServer, TARGET_PROTOCOL_VERSION};
use sora_workspace::{ProjectId, RuntimeOptions, WorkspaceService};

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Default)]
struct TaskClient {
    status_count: Arc<AtomicUsize>,
}

impl ClientHandler for TaskClient {
    fn get_info(&self) -> ClientInfo {
        let mut capabilities = ClientCapabilities::default();
        capabilities.tasks = Some(TasksCapability::client_default());
        ClientInfo::new(
            capabilities,
            Implementation::new("sora-task-test", env!("CARGO_PKG_VERSION")),
        )
        .with_protocol_version(TARGET_PROTOCOL_VERSION)
    }

    async fn on_task_status(
        &self,
        _params: TaskStatusNotificationParam,
        _context: rmcp::service::NotificationContext<rmcp::RoleClient>,
    ) {
        self.status_count.fetch_add(1, Ordering::Relaxed);
    }
}

#[tokio::test]
async fn task_augmented_build_can_be_polled_and_read_as_a_resource() -> anyhow::Result<()> {
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
    let client = TaskClient::default().serve(client_transport).await?;

    let request = CallToolRequestParams::new("sora_schema_lock")
        .with_arguments(
            serde_json::json!({
                "project_id": "demo",
                "expected_project_revision": revision.project,
                "view": null,
                "clean": true
            })
            .as_object()
            .unwrap()
            .clone(),
        )
        .with_task(TaskMetadata::new().with_ttl(60_000));
    let task = match client
        .send_request(ClientRequest::CallToolRequest(CallToolRequest::new(
            request,
        )))
        .await?
    {
        ServerResult::CreateTaskResult(result) => result.task,
        result => anyhow::bail!("expected task result, got {result:?}"),
    };

    let completed = loop {
        let result = client
            .send_request(ClientRequest::GetTaskRequest(GetTaskRequest::new(
                GetTaskParams::new(&task.task_id),
            )))
            .await?;
        let ServerResult::GetTaskResult(result) = result else {
            anyhow::bail!("expected task info");
        };
        if result.task.status != TaskStatus::Working {
            break result.task;
        }
        tokio::task::yield_now().await;
    };
    assert_eq!(completed.status, TaskStatus::Completed);
    assert!(root.join("generated/schema.lock").is_file());

    let payload = client
        .send_request(ClientRequest::GetTaskPayloadRequest(
            GetTaskPayloadRequest::new(GetTaskPayloadParams::new(&task.task_id)),
        ))
        .await?;
    let payload = match payload {
        ServerResult::CallToolResult(payload) => serde_json::to_value(payload)?,
        ServerResult::CustomResult(payload) => payload.0,
        payload => anyhow::bail!("expected task payload, got {payload:?}"),
    };
    assert_eq!(payload["isError"], false);
    assert_eq!(payload["structuredContent"]["ok"], true);

    let list = client
        .send_request(ClientRequest::ListTasksRequest(ListTasksRequest::default()))
        .await?;
    let ServerResult::ListTasksResult(list) = list else {
        anyhow::bail!("expected task list");
    };
    assert_eq!(list.tasks.len(), 1);
    assert_eq!(list.tasks[0].task_id, task.task_id);

    let resource = client
        .read_resource(ReadResourceRequestParams::new(format!(
            "sora://project/demo/task/{}",
            task.task_id
        )))
        .await?;
    assert_eq!(resource.contents.len(), 1);
    assert!(client.service().status_count.load(Ordering::Relaxed) >= 1);

    fs::write(root.join("generated/schema.lock"), "sentinel")?;
    let notifications_before_cancel = client.service().status_count.load(Ordering::Relaxed);
    let request = CallToolRequestParams::new("sora_schema_lock")
        .with_arguments(
            serde_json::json!({
                "project_id": "demo",
                "expected_project_revision": revision.project,
                "view": null,
                "clean": true
            })
            .as_object()
            .unwrap()
            .clone(),
        )
        .with_task(TaskMetadata::new().with_ttl(60_000));
    let cancelled_task = match client
        .send_request(ClientRequest::CallToolRequest(CallToolRequest::new(
            request,
        )))
        .await?
    {
        ServerResult::CreateTaskResult(result) => result.task,
        result => anyhow::bail!("expected task result, got {result:?}"),
    };
    let cancellation = client
        .send_request(ClientRequest::CancelTaskRequest(CancelTaskRequest::new(
            CancelTaskParams::new(&cancelled_task.task_id),
        )))
        .await?;
    let cancellation = match cancellation {
        ServerResult::CancelTaskResult(result) => result.task,
        ServerResult::GetTaskResult(result) => result.task,
        result => anyhow::bail!("expected cancellation result, got {result:?}"),
    };
    assert_eq!(cancellation.status, TaskStatus::Cancelled);
    let deadline = Instant::now() + Duration::from_secs(10);
    while client.service().status_count.load(Ordering::Relaxed) < notifications_before_cancel + 2
        && Instant::now() < deadline
    {
        tokio::task::yield_now().await;
    }
    let cancelled = client
        .send_request(ClientRequest::GetTaskRequest(GetTaskRequest::new(
            GetTaskParams::new(&cancelled_task.task_id),
        )))
        .await?;
    let ServerResult::GetTaskResult(cancelled) = cancelled else {
        anyhow::bail!("expected cancelled task info");
    };
    assert_eq!(cancelled.task.status, TaskStatus::Cancelled);
    assert_eq!(
        fs::read_to_string(root.join("generated/schema.lock"))?,
        "sentinel"
    );

    client.cancel().await?;
    server_handle.await??;
    fs::remove_dir_all(root)?;
    Ok(())
}

fn temp_project() -> PathBuf {
    let nonce = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!("sora-mcp-task-{}-{nonce}", std::process::id()));
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
[[tables]]
id = "settings"
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
