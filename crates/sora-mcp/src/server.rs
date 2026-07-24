use std::{
    collections::BTreeSet,
    future::{Future, ready},
    sync::atomic::{AtomicU8, Ordering},
    sync::{Arc, RwLock},
    time::Instant,
};

use rmcp::{
    ServerHandler,
    handler::server::router::tool::ToolRouter,
    model::{
        CallToolRequestParams, CancelTaskParams, CancelTaskResult, CompleteRequestParams,
        CompleteResult, CreateTaskResult, GetPromptRequestParams, GetPromptResult, GetTaskParams,
        GetTaskPayloadParams, GetTaskPayloadResult, GetTaskResult, Implementation,
        ListPromptsResult, ListResourceTemplatesResult, ListResourcesResult, ListTasksResult,
        PaginatedRequestParams, ReadResourceRequestParams, ReadResourceResult, ServerCapabilities,
        ServerInfo, SubscribeRequestParams, TaskStatusNotification, TaskStatusNotificationParam,
        TasksCapability, UnsubscribeRequestParams,
    },
    tool_handler,
};
use sora_workspace::{ProjectId, WorkspaceService};

use crate::{
    SERVER_NAME, TARGET_PROTOCOL_VERSION, artifact_store::ArtifactStore, task_store::TaskStore,
};

/// MCP protocol adapter backed by the shared Sora workspace service.
#[derive(Debug, Clone)]
pub struct SoraMcpServer {
    pub(crate) workspace: Arc<WorkspaceService>,
    pub(crate) authorization_context: Arc<str>,
    tool_router: ToolRouter<Self>,
    pub(crate) subscriptions: Arc<RwLock<BTreeSet<String>>>,
    pub(crate) artifacts: Arc<ArtifactStore>,
    pub(crate) tasks: Arc<TaskStore>,
    logging_level: Arc<AtomicU8>,
}

impl SoraMcpServer {
    /// Creates an MCP server for a workspace service.
    pub fn new(workspace: Arc<WorkspaceService>) -> Self {
        Self::new_with_authorization_context(workspace, "local")
    }

    pub(crate) fn new_with_authorization_context(
        workspace: Arc<WorkspaceService>,
        authorization_context: impl Into<Arc<str>>,
    ) -> Self {
        Self {
            workspace,
            authorization_context: authorization_context.into(),
            tool_router: Self::server_tool_router()
                + Self::project_tool_router()
                + Self::schema_tool_router()
                + Self::data_tool_router()
                + Self::build_tool_router()
                + Self::excel_tool_router(),
            subscriptions: Arc::new(RwLock::new(BTreeSet::new())),
            artifacts: Arc::new(ArtifactStore::default()),
            tasks: Arc::new(TaskStore::default()),
            logging_level: Arc::new(AtomicU8::new(1)),
        }
    }

    pub(crate) async fn notify_project_resources_updated(
        &self,
        peer: &rmcp::service::Peer<rmcp::RoleServer>,
        project_id: &str,
    ) {
        let prefix = format!("sora://project/{project_id}/");
        let subscriptions = match self.subscriptions.read() {
            Ok(subscriptions) => subscriptions
                .iter()
                .filter(|uri| uri.starts_with(&prefix))
                .cloned()
                .collect::<Vec<_>>(),
            Err(_) => return,
        };
        for uri in subscriptions {
            let _ = peer
                .notify_resource_updated(rmcp::model::ResourceUpdatedNotificationParam::new(uri))
                .await;
        }
    }

    #[allow(deprecated)]
    fn capabilities() -> ServerCapabilities {
        let mut capabilities = ServerCapabilities::builder()
            .enable_logging()
            .enable_completions()
            .enable_prompts()
            .enable_resources()
            .enable_resources_list_changed()
            .enable_resources_subscribe()
            .enable_tools()
            .build();
        if let Some(tools) = capabilities.tools.as_mut() {
            tools.list_changed = Some(false);
        }
        if let Some(prompts) = capabilities.prompts.as_mut() {
            prompts.list_changed = Some(false);
        }
        capabilities.tasks = Some(TasksCapability::server_default());
        capabilities
    }

    #[allow(deprecated)]
    async fn sync_client_roots(&self, peer: rmcp::service::Peer<rmcp::RoleServer>) {
        let supports_roots = peer
            .peer_info()
            .and_then(|info| info.capabilities.roots.as_ref().cloned())
            .is_some();
        if !supports_roots {
            return;
        }
        let Ok(result) = peer.list_roots().await else {
            return;
        };
        if self
            .workspace
            .remove_roots_with_name_prefix("mcp:")
            .is_err()
        {
            return;
        }
        for (index, root) in result.roots.into_iter().enumerate() {
            let Ok(url) = url::Url::parse(&root.uri) else {
                continue;
            };
            let Ok(path) = url.to_file_path() else {
                continue;
            };
            let name = root.name.unwrap_or_else(|| format!("root-{}", index + 1));
            let _ = self.workspace.add_root(format!("mcp:{name}"), path);
        }
        let _ = peer.notify_resource_list_changed().await;
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for SoraMcpServer {
    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<rmcp::model::CallToolResult, rmcp::ErrorData> {
        let started = Instant::now();
        let tool = request.name.to_string();
        let project_id = request
            .arguments
            .as_ref()
            .and_then(|arguments| arguments.get("project_id"))
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned);
        let revision_before = project_id
            .as_deref()
            .and_then(|project_id| ProjectId::new(project_id).ok())
            .and_then(|project_id| self.workspace.project(&project_id).ok())
            .map(|project| project.revision().project);
        let result = self
            .tool_router
            .call(rmcp::handler::server::tool::ToolCallContext::new(
                self, request, context,
            ))
            .await;
        let revision_after = project_id
            .as_deref()
            .and_then(|project_id| ProjectId::new(project_id).ok())
            .and_then(|project_id| self.workspace.project(&project_id).ok())
            .map(|project| project.revision().project);
        let (outcome, change_summary) = match &result {
            Ok(result) if result.is_error == Some(true) => {
                ("business_error", audit_change_summary(result))
            }
            Ok(result) => ("success", audit_change_summary(result)),
            Err(_) => ("protocol_error", "none".to_owned()),
        };
        tracing::info!(
            audit_event = "tool_call",
            tool,
            project = project_id.as_deref().unwrap_or("none"),
            authorization_context = self.authorization_context.as_ref(),
            revision_before = revision_before.as_deref().unwrap_or("none"),
            revision_after = revision_after.as_deref().unwrap_or("none"),
            outcome,
            duration_ms = started.elapsed().as_millis(),
            change_summary,
            "Sora MCP tool call completed"
        );
        result
    }

    async fn enqueue_task(
        &self,
        mut request: CallToolRequestParams,
        context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<CreateTaskResult, rmcp::ErrorData> {
        let ttl = request.task.take().and_then(|metadata| metadata.ttl);
        let project_id = request
            .arguments
            .as_ref()
            .and_then(|arguments| arguments.get("project_id"))
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned);
        let created = self
            .tasks
            .create(&self.authorization_context, project_id, ttl)?;
        let mut task_context = context.clone();
        task_context.ct = created.cancellation;
        let task_id = created.task.task_id.clone();
        let owner = self.authorization_context.clone();
        let server = self.clone();
        tokio::spawn(async move {
            let peer = task_context.peer.clone();
            let result = server.call_tool(request, task_context).await;
            let serialized = result.and_then(|result| {
                serde_json::to_value(result).map_err(|error| {
                    rmcp::ErrorData::internal_error(
                        format!("failed to serialize task result: {error}"),
                        None,
                    )
                })
            });
            if let Ok(task) = server.tasks.finish(&owner, &task_id, serialized) {
                let _ = peer
                    .send_notification(rmcp::model::ServerNotification::TaskStatusNotification(
                        TaskStatusNotification::new(TaskStatusNotificationParam::new(task)),
                    ))
                    .await;
            }
        });
        Ok(CreateTaskResult::new(created.task))
    }

    fn list_tasks(
        &self,
        request: Option<PaginatedRequestParams>,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> impl Future<Output = Result<ListTasksResult, rmcp::ErrorData>>
    + rmcp::service::MaybeSendFuture
    + '_ {
        ready(
            self.tasks.list(
                &self.authorization_context,
                request
                    .as_ref()
                    .and_then(|request| request.cursor.as_deref()),
            ),
        )
    }

    fn get_task_info(
        &self,
        request: GetTaskParams,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> impl Future<Output = Result<GetTaskResult, rmcp::ErrorData>> + rmcp::service::MaybeSendFuture + '_
    {
        ready(
            self.tasks
                .get(&self.authorization_context, &request.task_id)
                .map(GetTaskResult::new),
        )
    }

    fn get_task_result(
        &self,
        request: GetTaskPayloadParams,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> impl Future<Output = Result<GetTaskPayloadResult, rmcp::ErrorData>>
    + rmcp::service::MaybeSendFuture
    + '_ {
        ready(
            self.tasks
                .result(&self.authorization_context, &request.task_id)
                .and_then(|result| result)
                .map(GetTaskPayloadResult::new),
        )
    }

    async fn cancel_task(
        &self,
        request: CancelTaskParams,
        context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<CancelTaskResult, rmcp::ErrorData> {
        let task = self
            .tasks
            .cancel(&self.authorization_context, &request.task_id)?;
        let _ = context
            .peer
            .send_notification(rmcp::model::ServerNotification::TaskStatusNotification(
                TaskStatusNotification::new(TaskStatusNotificationParam::new(task.clone())),
            ))
            .await;
        Ok(CancelTaskResult::new(task))
    }

    #[allow(deprecated)]
    async fn on_initialized(&self, context: rmcp::service::NotificationContext<rmcp::RoleServer>) {
        self.sync_client_roots(context.peer.clone()).await;
        if self.logging_level.load(Ordering::Relaxed) <= 1 {
            let _ = context
                .peer
                .notify_logging_message(
                    rmcp::model::LoggingMessageNotificationParam::new(
                        rmcp::model::LoggingLevel::Info,
                        serde_json::json!({
                            "event": "server_initialized",
                            "protocol_version": TARGET_PROTOCOL_VERSION.as_str(),
                        }),
                    )
                    .with_logger(SERVER_NAME),
                )
                .await;
        }
    }

    async fn on_roots_list_changed(
        &self,
        context: rmcp::service::NotificationContext<rmcp::RoleServer>,
    ) {
        self.sync_client_roots(context.peer).await;
    }

    fn complete(
        &self,
        request: CompleteRequestParams,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> impl Future<Output = Result<CompleteResult, rmcp::ErrorData>>
    + rmcp::service::MaybeSendFuture
    + '_ {
        ready(crate::completion::complete(
            &self.workspace,
            &self.artifacts,
            &self.authorization_context,
            &request,
        ))
    }

    fn list_prompts(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> impl Future<Output = Result<ListPromptsResult, rmcp::ErrorData>>
    + rmcp::service::MaybeSendFuture
    + '_ {
        ready(Ok(crate::prompts::list()))
    }

    fn get_prompt(
        &self,
        request: GetPromptRequestParams,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> impl Future<Output = Result<GetPromptResult, rmcp::ErrorData>>
    + rmcp::service::MaybeSendFuture
    + '_ {
        ready(crate::prompts::get(&self.workspace, request))
    }

    #[allow(deprecated)]
    fn set_level(
        &self,
        request: rmcp::model::SetLevelRequestParams,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> impl Future<Output = Result<(), rmcp::ErrorData>> + rmcp::service::MaybeSendFuture + '_
    {
        let level = match request.level {
            rmcp::model::LoggingLevel::Debug => 0,
            rmcp::model::LoggingLevel::Info | rmcp::model::LoggingLevel::Notice => 1,
            rmcp::model::LoggingLevel::Warning => 2,
            rmcp::model::LoggingLevel::Error => 3,
            rmcp::model::LoggingLevel::Critical
            | rmcp::model::LoggingLevel::Alert
            | rmcp::model::LoggingLevel::Emergency => 4,
        };
        self.logging_level.store(level, Ordering::Relaxed);
        ready(Ok(()))
    }

    fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> impl Future<Output = Result<ListResourcesResult, rmcp::ErrorData>>
    + rmcp::service::MaybeSendFuture
    + '_ {
        ready(crate::resources::list(&self.workspace))
    }

    fn list_resource_templates(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> impl Future<Output = Result<ListResourceTemplatesResult, rmcp::ErrorData>>
    + rmcp::service::MaybeSendFuture
    + '_ {
        ready(Ok(crate::resources::templates()))
    }

    fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> impl Future<Output = Result<ReadResourceResult, rmcp::ErrorData>>
    + rmcp::service::MaybeSendFuture
    + '_ {
        ready(crate::resources::read(
            &self.workspace,
            &self.artifacts,
            &self.tasks,
            &self.authorization_context,
            &request.uri,
        ))
    }

    fn subscribe(
        &self,
        request: SubscribeRequestParams,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> impl Future<Output = Result<(), rmcp::ErrorData>> + rmcp::service::MaybeSendFuture + '_
    {
        let result = if crate::resources::exists(
            &self.workspace,
            &self.artifacts,
            &self.tasks,
            &self.authorization_context,
            &request.uri,
        ) {
            self.subscriptions
                .write()
                .map_err(|_| rmcp::ErrorData::internal_error("subscription lock poisoned", None))
                .map(|mut subscriptions| {
                    subscriptions.insert(request.uri);
                })
        } else {
            Err(rmcp::ErrorData::resource_not_found(
                "resource is not available for subscription",
                None,
            ))
        };
        ready(result)
    }

    fn unsubscribe(
        &self,
        request: UnsubscribeRequestParams,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> impl Future<Output = Result<(), rmcp::ErrorData>> + rmcp::service::MaybeSendFuture + '_
    {
        let result = self
            .subscriptions
            .write()
            .map_err(|_| rmcp::ErrorData::internal_error("subscription lock poisoned", None))
            .map(|mut subscriptions| {
                subscriptions.remove(&request.uri);
            });
        ready(result)
    }

    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(Self::capabilities())
            .with_protocol_version(TARGET_PROTOCOL_VERSION)
            .with_server_info(
                Implementation::new(SERVER_NAME, env!("CARGO_PKG_VERSION"))
                    .with_title("Sora")
                    .with_description("Sora configuration compiler MCP server"),
            )
            .with_instructions(
                "Use Sora resources and domain tools to inspect, validate, modify, and build \
                 configuration projects. Never edit generated outputs as source files.",
            )
    }
}

fn audit_change_summary(result: &rmcp::model::CallToolResult) -> String {
    const COUNTED_FIELDS: [&str; 7] = [
        "affected_files",
        "affected_entities",
        "changes",
        "created",
        "deleted",
        "updated",
        "warnings",
    ];
    let Some(structured) = result.structured_content.as_ref() else {
        return format!("content_blocks={}", result.content.len());
    };
    let Some(object) = structured.as_object() else {
        return "structured_result=non_object".to_owned();
    };
    let counts = COUNTED_FIELDS
        .iter()
        .filter_map(|field| {
            object
                .get(*field)
                .and_then(serde_json::Value::as_array)
                .map(|values| format!("{field}={}", values.len()))
        })
        .collect::<Vec<_>>();
    if counts.is_empty() {
        "structured_result=object".to_owned()
    } else {
        counts.join(",")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initialization_contract_is_versioned_and_deterministic() {
        let server = SoraMcpServer::new(Arc::new(WorkspaceService::new()));
        let info = server.get_info();

        assert_eq!(info.protocol_version, TARGET_PROTOCOL_VERSION);
        assert_eq!(info.server_info.name, SERVER_NAME);
        assert_eq!(
            info.capabilities
                .tools
                .as_ref()
                .and_then(|tools| tools.list_changed),
            Some(false)
        );
        assert_eq!(
            info.capabilities
                .resources
                .as_ref()
                .and_then(|resources| resources.list_changed),
            Some(true)
        );
        assert_eq!(
            info.capabilities
                .resources
                .as_ref()
                .and_then(|resources| resources.subscribe),
            Some(true)
        );
        assert_eq!(
            info.capabilities
                .prompts
                .as_ref()
                .and_then(|prompts| prompts.list_changed),
            Some(false)
        );
        assert!(info.capabilities.completions.is_some());
        let tasks = info.capabilities.tasks.expect("tasks capability");
        assert!(tasks.supports_list());
        assert!(tasks.supports_cancel());
        assert!(tasks.supports_tools_call());
    }

    #[test]
    fn tools_have_strict_structured_contracts() {
        let server = SoraMcpServer::new(Arc::new(WorkspaceService::new()));
        let tools = server.tool_router.list_all();

        assert_eq!(tools.len(), 22);
        let mut names = tools
            .iter()
            .map(|tool| tool.name.as_ref())
            .collect::<Vec<_>>();
        names.sort_unstable();
        assert_eq!(
            names,
            [
                "sora_build",
                "sora_codegen",
                "sora_data_apply",
                "sora_data_diff",
                "sora_data_preview",
                "sora_data_validate",
                "sora_excel_sync_apply",
                "sora_excel_sync_preview",
                "sora_excel_template",
                "sora_export",
                "sora_project_init",
                "sora_project_init_apply",
                "sora_project_inspect",
                "sora_project_list",
                "sora_project_open",
                "sora_schema_apply",
                "sora_schema_lock",
                "sora_schema_preview",
                "sora_schema_search",
                "sora_schema_validate",
                "sora_server_info",
                "sora_table_query",
            ]
        );
        for tool in tools {
            assert!(tool.output_schema.is_some());
            let input_schema = serde_json::to_value(&tool.input_schema)
                .expect("tool input schema should serialize");
            assert_eq!(input_schema["type"], "object");
            assert_eq!(input_schema["additionalProperties"], false);
        }
    }
}
