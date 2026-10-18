use crate::{
    SERVER_NAME, TARGET_PROTOCOL_VERSION,
    artifact_store::ArtifactStore,
    client_roots::ClientRoots,
    completion::complete as complete_arguments,
    input::{InputStep, InputStore},
    prompts::{get as get_prompt, list as list_prompts},
    resources::{
        exists as resource_exists, list as list_resources, read as read_resource,
        templates as resource_templates,
    },
    task_store::{TaskExecutionState, TaskStore},
};

#[allow(deprecated)]
use rmcp::model::{ListRootsRequest, ListRootsResult};
use rmcp::{
    ErrorData, RoleServer, ServerHandler,
    handler::server::{router::tool::ToolRouter, tool::ToolCallContext},
    model::{
        CallToolRequestParams, CallToolResponse, CallToolResult, CancelTaskParams,
        CompleteRequestParams, CompleteResult, CreateTaskResult, GetPromptRequestParams,
        GetPromptResponse, GetTaskParams, GetTaskResult, Implementation, InputRequest,
        ListPromptsResult, ListResourceTemplatesResult, ListResourcesResult,
        PaginatedRequestParams, ProtocolVersion, ReadResourceRequestParams, ReadResourceResponse,
        ServerCapabilities, ServerConfig, SubscriptionFilter, UpdateTaskParams,
    },
    service::{MaybeSendFuture, RequestContext, SubscriptionContext, SubscriptionSink},
    tool_handler,
};
use serde_json::{Value, from_value};
use sora_workspace::{ProjectId, WorkspaceService};
use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet},
    future::{Future, ready},
    sync::{Arc, RwLock},
    time::Instant,
};

use tokio::spawn;
use tracing::{error, info};

use uuid::Uuid;

/// MCP protocol adapter backed by the shared Sora workspace service.
#[derive(Debug, Clone)]
pub struct SoraMcpServer {
    pub(crate) workspace: Arc<WorkspaceService>,
    pub(crate) authorization_context: Arc<str>,
    tool_router: ToolRouter<Self>,
    subscriptions: Arc<RwLock<BTreeMap<Uuid, SubscriptionSink>>>,
    pub(crate) artifacts: Arc<ArtifactStore>,
    pub(crate) tasks: Arc<TaskStore>,
    pub(crate) trusted_project_scripts: Arc<RwLock<BTreeSet<String>>>,
    pub(crate) inputs: Arc<InputStore>,
    pub(crate) client_roots: Arc<RwLock<Arc<ClientRoots>>>,
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
            subscriptions: Arc::new(RwLock::new(BTreeMap::new())),
            artifacts: Arc::new(ArtifactStore::default()),
            tasks: Arc::new(TaskStore::default()),
            trusted_project_scripts: Arc::new(RwLock::new(BTreeSet::new())),
            inputs: Arc::new(InputStore::default()),
            client_roots: Arc::new(RwLock::new(Arc::new(ClientRoots::default()))),
        }
    }

    pub(crate) async fn notify_project_resources_updated(&self, project_id: &str) {
        let prefix = format!("sora://project/{project_id}/");
        let sinks = match self.subscriptions.read() {
            Ok(subscriptions) => subscriptions.values().cloned().collect::<Vec<_>>(),
            Err(_) => return,
        };
        for sink in sinks {
            for uri in sink
                .accepted()
                .resource_subscriptions
                .as_deref()
                .unwrap_or_default()
            {
                if uri.starts_with(&prefix) {
                    let _ = sink.notify_resource_updated(uri).await;
                }
            }
        }
    }

    pub(crate) fn has_retained_tasks(&self) -> bool {
        self.tasks.has_retained_tasks()
    }

    /// Protect in-flight MRTR continuations from authorization-cache eviction.
    pub(crate) fn has_pending_inputs(&self) -> bool {
        self.inputs.has_pending_inputs()
    }

    fn input_response(
        &self,
        request: &CallToolRequestParams,
        response: CallToolResponse,
    ) -> CallToolResponse {
        let outcome = match &response {
            CallToolResponse::InputRequired(_) => "input_required",
            _ => "business_error",
        };
        info!(
            audit_event = "tool_input",
            tool = request.name.as_ref(),
            authorization_context = self.authorization_context.as_ref(),
            outcome,
            "Sora tool input round returned without executing the tool"
        );
        response
    }

    fn capabilities() -> ServerCapabilities {
        let mut capabilities = ServerCapabilities::builder()
            .enable_completions()
            .enable_prompts()
            .enable_resources()
            .enable_resources_list_changed()
            .enable_resources_subscribe()
            .enable_tools()
            .enable_tasks()
            .build();
        if let Some(tools) = capabilities.tools.as_mut() {
            tools.list_changed = Some(false);
        }
        if let Some(prompts) = capabilities.prompts.as_mut() {
            prompts.list_changed = Some(false);
        }
        capabilities
    }

    async fn execute_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, ErrorData> {
        let started = Instant::now();
        let tool = request.name.to_string();
        let project_id = request
            .arguments
            .as_ref()
            .and_then(|arguments| arguments.get("project_id"))
            .and_then(Value::as_str)
            .map(str::to_owned);
        let revision_before = project_id
            .as_deref()
            .and_then(|project_id| ProjectId::new(project_id).ok())
            .and_then(|project_id| self.workspace.project(&project_id).ok())
            .map(|project| project.revision().project);
        let result = self
            .tool_router
            .call(ToolCallContext::new(self, request, context))
            .await;
        let revision_after = project_id
            .as_deref()
            .and_then(|project_id| ProjectId::new(project_id).ok())
            .and_then(|project_id| self.workspace.project(&project_id).ok())
            .map(|project| project.revision().project);
        let (outcome, change_summary) = match &result {
            Ok(CallToolResponse::Complete(result)) if result.is_error == Some(true) => {
                ("business_error", audit_change_summary(result))
            }
            Ok(CallToolResponse::Complete(result)) => ("success", audit_change_summary(result)),
            Ok(CallToolResponse::InputRequired(_)) => ("input_required", "none".to_owned()),
            Ok(CallToolResponse::Task(_)) => ("task_accepted", "none".to_owned()),
            Ok(_) => ("intermediate", "none".to_owned()),
            Err(_) => ("protocol_error", "none".to_owned()),
        };
        info!(
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

    fn enqueue_task(
        &self,
        request: CallToolRequestParams,
        mut context: RequestContext<RoleServer>,
    ) -> Result<CreateTaskResult, ErrorData> {
        let project_id = request
            .arguments
            .as_ref()
            .and_then(|arguments| arguments.get("project_id"))
            .and_then(Value::as_str)
            .map(str::to_owned);
        let created = self.tasks.create(&self.authorization_context, project_id)?;
        context.ct = created.cancellation.clone();
        let execution = TaskExecutionState::default();
        context.extensions.insert(execution.clone());
        let task = created.task.clone();
        let task_id = task.task_id.clone();
        let server = self.clone();
        info!(
            audit_event = "task_accepted",
            tool = request.name.as_ref(),
            task_id,
            authorization_context = self.authorization_context.as_ref(),
            "Sora task accepted"
        );
        spawn(async move {
            let result = created
                .run(async {
                    match server.execute_tool(request, context).await? {
                        CallToolResponse::Complete(result) => Ok(result),
                        _ => Err(ErrorData::internal_error(
                            "task execution must not request input or create another task",
                            None,
                        )),
                    }
                })
                .await;
            match server
                .tasks
                .finish(&server.authorization_context, &task_id, result, &execution)
            {
                Ok(task) => info!(audit_event = "task_finished", task_id,
                    authorization_context = server.authorization_context.as_ref(),
                    status = ?task.status(), "Sora task finished"),
                Err(error) => error!(task_id, %error, "Failed to finalize Sora task"),
            }
        });
        Ok(CreateTaskResult::new(task))
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for SoraMcpServer {
    fn supported_protocol_versions(&self) -> Cow<'static, [ProtocolVersion]> {
        Cow::Borrowed(&[TARGET_PROTOCOL_VERSION])
    }

    async fn call_tool(
        &self,
        mut request: CallToolRequestParams,
        mut context: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, ErrorData> {
        let session = self
            .inputs
            .resume(&request, context.client_capabilities())?;
        let supports_roots = context
            .client_capabilities()
            .and_then(|capabilities| capabilities.roots)
            .is_some();
        let roots =
            if supports_roots {
                #[allow(deprecated)]
                let input = InputRequest::ListRoots(ListRootsRequest::default());
                match self.inputs.resolve(&session, "roots", input)? {
                    InputStep::Required(result) => {
                        return Ok(self.input_response(&request, result.into()));
                    }
                    InputStep::Ready(value) => {
                        #[allow(deprecated)]
                        let result: ListRootsResult = from_value(value)
                            .map_err(|error| ErrorData::invalid_params(error.to_string(), None))?;
                        let roots =
                            Arc::new(ClientRoots::new(result).map_err(|error| {
                                ErrorData::invalid_params(error.to_string(), None)
                            })?);
                        *self.client_roots.write().map_err(|error| {
                            ErrorData::internal_error(error.to_string(), None)
                        })? = roots.clone();
                        roots
                    }
                }
            } else {
                Arc::new(ClientRoots::default())
            };
        context.extensions.insert(roots);
        if request.name == "sora_project_open"
            && let Some(response) = self
                .prepare_project_open(&mut request, &context, &session)
                .await?
        {
            return Ok(self.input_response(&request, response));
        }
        // All interaction is complete before acceptance. A worker never owns an input round.
        request.input_responses = None;
        request.request_state = None;
        if is_task_tool(&request.name)
            && context
                .client_capabilities()
                .is_some_and(|caps| caps.supports_tasks())
        {
            return self
                .enqueue_task(request, context)
                .map(CallToolResponse::Task);
        }
        self.execute_tool(request, context).await
    }

    async fn get_task(
        &self,
        request: GetTaskParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<GetTaskResult, ErrorData> {
        self.tasks
            .get(&self.authorization_context, &request.task_id)
            .map(GetTaskResult::new)
    }

    async fn cancel_task(
        &self,
        request: CancelTaskParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<(), ErrorData> {
        self.tasks
            .cancel(&self.authorization_context, &request.task_id)
    }

    async fn update_task(
        &self,
        request: UpdateTaskParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<(), ErrorData> {
        self.tasks
            .get(&self.authorization_context, &request.task_id)?;
        Err(ErrorData::invalid_request(
            "Sora tasks do not request client input; tasks/update is not applicable",
            None,
        ))
    }

    fn complete(
        &self,
        request: CompleteRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<CompleteResult, ErrorData>> + MaybeSendFuture + '_ {
        ready(complete_arguments(
            &self.workspace,
            &self.artifacts,
            &self.authorization_context,
            &request,
        ))
    }

    fn list_prompts(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ListPromptsResult, ErrorData>> + MaybeSendFuture + '_ {
        ready(Ok(list_prompts()))
    }

    fn get_prompt(
        &self,
        request: GetPromptRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<GetPromptResponse, ErrorData>> + MaybeSendFuture + '_ {
        ready(get_prompt(&self.workspace, request).map(Into::into))
    }

    fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ListResourcesResult, ErrorData>> + MaybeSendFuture + '_ {
        ready(list_resources(&self.workspace))
    }

    fn list_resource_templates(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ListResourceTemplatesResult, ErrorData>> + MaybeSendFuture + '_
    {
        ready(Ok(resource_templates()))
    }

    fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ReadResourceResponse, ErrorData>> + MaybeSendFuture + '_ {
        ready(
            read_resource(
                &self.workspace,
                &self.artifacts,
                &self.tasks,
                &self.authorization_context,
                &self
                    .client_roots
                    .read()
                    .map(|roots| roots.clone())
                    .unwrap_or_default(),
                &request.uri,
            )
            .map(Into::into),
        )
    }

    fn accepted_subscription_filter(
        &self,
        requested: &SubscriptionFilter,
    ) -> Option<SubscriptionFilter> {
        let mut accepted = SubscriptionFilter::new();
        accepted.resources_list_changed = requested.resources_list_changed;
        accepted.resource_subscriptions = requested.resource_subscriptions.as_ref().map(|uris| {
            uris.iter()
                .filter(|uri| {
                    resource_exists(
                        &self.workspace,
                        &self.artifacts,
                        &self.tasks,
                        &self.authorization_context,
                        &self
                            .client_roots
                            .read()
                            .map(|roots| roots.clone())
                            .unwrap_or_default(),
                        uri,
                    )
                })
                .cloned()
                .collect()
        });
        Some(accepted)
    }

    async fn listen(&self, context: SubscriptionContext) -> Result<(), ErrorData> {
        let id = Uuid::new_v4();
        self.subscriptions
            .write()
            .map_err(|_| ErrorData::internal_error("subscription lock poisoned", None))?
            .insert(id, context.sink().clone());
        let _registration = SubscriptionRegistration {
            id,
            subscriptions: self.subscriptions.clone(),
        };
        context.cancelled().await;
        Ok(())
    }

    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(Self::capabilities())
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

fn is_task_tool(name: &str) -> bool {
    matches!(
        name,
        "sora_build"
            | "sora_codegen"
            | "sora_export"
            | "sora_schema_lock"
            | "sora_excel_template"
            | "sora_excel_sync_preview"
            | "sora_excel_sync_apply"
    )
}

struct SubscriptionRegistration {
    id: Uuid,
    subscriptions: Arc<RwLock<BTreeMap<Uuid, SubscriptionSink>>>,
}

impl Drop for SubscriptionRegistration {
    fn drop(&mut self) {
        if let Ok(mut subscriptions) = self.subscriptions.write() {
            subscriptions.remove(&self.id);
        }
    }
}

fn audit_change_summary(result: &CallToolResult) -> String {
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
                .and_then(Value::as_array)
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
    use rmcp::{
        ServiceExt,
        model::{ClientCapabilities, ClientConfig, ErrorCode, TaskPayload, TaskStatus},
        service::{ClientLifecycleMode, ClientServiceExt},
    };
    use serde_json::{json, to_value};
    use std::time::Duration;
    use tokio::{io::duplex, task::yield_now, time::timeout};

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
        assert!(info.capabilities.supports_tasks());
        assert_eq!(
            server.supported_protocol_versions().as_ref(),
            &[TARGET_PROTOCOL_VERSION]
        );
    }

    #[tokio::test]
    async fn synchronous_fallback_never_creates_a_task_record() {
        let server = SoraMcpServer::new(Arc::new(WorkspaceService::new()));
        let observer = server.clone();
        let (server_transport, client_transport) = duplex(32 * 1024);
        let handle = spawn(async move {
            server
                .serve(server_transport)
                .await
                .unwrap()
                .waiting()
                .await
                .unwrap();
        });
        let config = ClientConfig::new(Default::default(), Implementation::new("sync-test", "1"))
            .with_protocol_version(TARGET_PROTOCOL_VERSION);
        let client = config
            .serve_with_lifecycle(
                client_transport,
                ClientLifecycleMode::Discover {
                    preferred_versions: vec![TARGET_PROTOCOL_VERSION],
                },
            )
            .await
            .unwrap();
        let response = client.call_tool_once(CallToolRequestParams::new("sora_schema_lock").with_arguments(
            json!({"project_id":"missing", "expected_project_revision":"missing", "view":null, "clean":true})
                .as_object().unwrap().clone()
        )).await.unwrap();
        assert!(
            matches!(response, CallToolResponse::Complete(result) if result.is_error == Some(true))
        );
        assert!(!observer.has_retained_tasks());
        client.cancel().await.unwrap();
        handle.await.unwrap();
    }

    #[tokio::test]
    #[allow(deprecated)]
    async fn task_acceptance_waits_for_required_input_without_starting_a_worker() {
        let server = SoraMcpServer::new(Arc::new(WorkspaceService::new()));
        let observer = server.clone();
        let (server_transport, client_transport) = duplex(32 * 1024);
        let handle = spawn(async move {
            server
                .serve(server_transport)
                .await
                .unwrap()
                .waiting()
                .await
                .unwrap()
        });
        let config = ClientConfig::new(
            ClientCapabilities::builder()
                .enable_roots()
                .enable_tasks()
                .build(),
            Implementation::new("input-task-test", "1"),
        )
        .with_protocol_version(TARGET_PROTOCOL_VERSION);
        let client = config
            .serve_with_lifecycle(
                client_transport,
                ClientLifecycleMode::Discover {
                    preferred_versions: vec![TARGET_PROTOCOL_VERSION],
                },
            )
            .await
            .unwrap();
        let mut request = CallToolRequestParams::new("sora_schema_lock").with_arguments(
            json!({"project_id":"missing", "expected_project_revision":"missing", "view":null, "clean":true})
                .as_object().unwrap().clone());
        let CallToolResponse::InputRequired(input) =
            client.call_tool_once(request.clone()).await.unwrap()
        else {
            panic!("roots must be resolved before creating a task")
        };
        assert!(!observer.has_retained_tasks());
        assert!(observer.has_pending_inputs());
        request.request_state = input.request_state;
        request.input_responses = Some(BTreeMap::from([("roots".to_owned(), json!({"roots":[]}))]));
        assert!(matches!(
            client.call_tool_once(request).await.unwrap(),
            CallToolResponse::Task(_)
        ));
        assert!(observer.has_retained_tasks());
        assert!(!observer.has_pending_inputs());
        client.cancel().await.unwrap();
        handle.await.unwrap();
    }

    #[tokio::test]
    async fn task_protocol_error_is_polled_as_failed_with_its_json_rpc_error() {
        let mut server = SoraMcpServer::new(Arc::new(WorkspaceService::new()));
        // Force the router error path, without introducing a production-only
        // test tool or turning an ordinary business failure into a protocol error.
        server.tool_router.remove_route("sora_schema_lock");
        let (server_transport, client_transport) = duplex(32 * 1024);
        let handle = spawn(async move {
            server
                .serve(server_transport)
                .await
                .unwrap()
                .waiting()
                .await
                .unwrap();
        });
        let config = ClientConfig::new(
            ClientCapabilities::builder().enable_tasks().build(),
            Implementation::new("protocol-error-test", "1"),
        )
        .with_protocol_version(TARGET_PROTOCOL_VERSION);
        let client = config
            .serve_with_lifecycle(
                client_transport,
                ClientLifecycleMode::Discover {
                    preferred_versions: vec![TARGET_PROTOCOL_VERSION],
                },
            )
            .await
            .unwrap();
        let CallToolResponse::Task(created) = client
            .call_tool_once(CallToolRequestParams::new("sora_schema_lock"))
            .await
            .unwrap()
        else {
            panic!("task result")
        };
        let task = timeout(Duration::from_secs(2), async {
            loop {
                let task = client
                    .get_task(GetTaskParams::new(&created.task.task_id))
                    .await
                    .unwrap()
                    .task;
                if task.status().is_terminal() {
                    break task;
                }
                yield_now().await;
            }
        })
        .await
        .unwrap();
        assert_eq!(task.status(), TaskStatus::Failed);
        let TaskPayload::Failed { error } = task.payload else {
            panic!("protocol error payload")
        };
        assert_eq!(error["code"], to_value(ErrorCode::INVALID_PARAMS).unwrap());
        client.cancel().await.unwrap();
        handle.await.unwrap();
    }

    #[test]
    fn exactly_the_seven_heavy_tools_are_task_eligible() {
        let server = SoraMcpServer::new(Arc::new(WorkspaceService::new()));
        let task_tools = server
            .tool_router
            .list_all()
            .into_iter()
            .filter(|tool| is_task_tool(&tool.name))
            .map(|tool| tool.name.to_string())
            .collect::<BTreeSet<_>>();
        assert_eq!(
            task_tools,
            [
                "sora_build",
                "sora_codegen",
                "sora_export",
                "sora_schema_lock",
                "sora_excel_template",
                "sora_excel_sync_preview",
                "sora_excel_sync_apply"
            ]
            .into_iter()
            .map(str::to_owned)
            .collect()
        );
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
            let annotations = tool.annotations.as_ref().expect("tool safety annotations");
            assert_eq!(annotations.open_world_hint, Some(false));
            assert!(annotations.read_only_hint.is_some());
            assert!(annotations.destructive_hint.is_some());
            assert!(annotations.idempotent_hint.is_some());
            if matches!(
                tool.name.as_ref(),
                "sora_schema_apply"
                    | "sora_data_apply"
                    | "sora_excel_sync_apply"
                    | "sora_build"
                    | "sora_codegen"
                    | "sora_export"
                    | "sora_schema_lock"
                    | "sora_excel_template"
            ) {
                assert_eq!(annotations.destructive_hint, Some(true));
                assert_eq!(annotations.read_only_hint, Some(false));
            }
            let input_schema =
                to_value(&tool.input_schema).expect("tool input schema should serialize");
            assert_eq!(input_schema["type"], "object");
            assert_eq!(input_schema["additionalProperties"], false);
        }
    }

    #[test]
    fn project_script_trust_is_shared_within_one_authorization_server_only() {
        let workspace = Arc::new(WorkspaceService::new());
        let server_a =
            SoraMcpServer::new_with_authorization_context(workspace.clone(), "subject-a");
        server_a
            .trusted_project_scripts
            .write()
            .expect("trust state")
            .insert("project:digest".to_owned());
        let same_authorization_session = server_a.clone();
        let server_b = SoraMcpServer::new_with_authorization_context(workspace, "subject-b");

        assert!(
            same_authorization_session
                .trusted_project_scripts
                .read()
                .expect("trust state")
                .contains("project:digest")
        );
        assert!(
            !server_b
                .trusted_project_scripts
                .read()
                .expect("trust state")
                .contains("project:digest")
        );
    }
}
