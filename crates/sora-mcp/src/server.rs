use std::{
    collections::BTreeSet,
    future::{Future, ready},
    sync::atomic::{AtomicU8, Ordering},
    sync::{Arc, RwLock},
};

use rmcp::{
    ServerHandler,
    handler::server::router::tool::ToolRouter,
    model::{
        CompleteRequestParams, CompleteResult, Implementation, ListResourceTemplatesResult,
        ListResourcesResult, PaginatedRequestParams, ReadResourceRequestParams, ReadResourceResult,
        ServerCapabilities, ServerInfo, SubscribeRequestParams, UnsubscribeRequestParams,
    },
    tool_handler,
};
use sora_workspace::WorkspaceService;

use crate::{SERVER_NAME, TARGET_PROTOCOL_VERSION, artifact_store::ArtifactStore};

/// MCP protocol adapter backed by the shared Sora workspace service.
#[derive(Debug, Clone)]
pub struct SoraMcpServer {
    pub(crate) workspace: Arc<WorkspaceService>,
    pub(crate) authorization_context: Arc<str>,
    tool_router: ToolRouter<Self>,
    pub(crate) subscriptions: Arc<RwLock<BTreeSet<String>>>,
    pub(crate) artifacts: Arc<ArtifactStore>,
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
                + Self::build_tool_router(),
            subscriptions: Arc::new(RwLock::new(BTreeSet::new())),
            artifacts: Arc::new(ArtifactStore::default()),
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
            .enable_resources()
            .enable_resources_list_changed()
            .enable_resources_subscribe()
            .enable_tools()
            .build();
        if let Some(tools) = capabilities.tools.as_mut() {
            tools.list_changed = Some(false);
        }
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
        ready(crate::completion::complete(&self.workspace, &request))
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
        assert!(info.capabilities.prompts.is_none());
        assert!(info.capabilities.completions.is_some());
        assert!(info.capabilities.tasks.is_none());
    }

    #[test]
    fn tools_have_strict_structured_contracts() {
        let server = SoraMcpServer::new(Arc::new(WorkspaceService::new()));
        let tools = server.tool_router.list_all();

        assert_eq!(tools.len(), 20);
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
