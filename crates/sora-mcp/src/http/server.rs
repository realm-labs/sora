use std::{
    collections::BTreeMap,
    iter::once,
    net::{IpAddr, SocketAddr},
    sync::{Arc, Mutex as StdMutex},
    time::{Duration, Instant},
};

use anyhow::{Context, Result};
use axum::{
    Json, Router,
    body::{Body, to_bytes},
    extract::State,
    http::{
        HeaderMap, HeaderValue, Request, Response, StatusCode,
        header::{AUTHORIZATION, WWW_AUTHENTICATE},
    },
    response::IntoResponse,
    routing::{any, get},
    serve,
};
use futures_util::{StreamExt, stream};
use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
};
use serde_json::{Value, from_slice, json};
use sora_workspace::WorkspaceService;
use thiserror::Error;
use tokio::{
    net::TcpListener,
    runtime::Runtime,
    signal::ctrl_c,
    sync::{Mutex, OwnedSemaphorePermit, Semaphore},
};
use tokio_util::sync::CancellationToken;
use tower_http::sensitive_headers::SetSensitiveRequestHeadersLayer;
use tracing::{info, warn};
use url::Url;
use uuid::Uuid;

use crate::SoraMcpServer;

use super::{
    AuthorizationPrincipal, OAuthAuthenticator, OAuthError, OAuthResourceServerConfig,
    ProtectedResourceMetadata,
};

const MCP_PATH: &str = "/mcp";
const RESOURCE_METADATA_PATH: &str = "/.well-known/oauth-protected-resource";

type McpTransport = StreamableHttpService<SoraMcpServer, LocalSessionManager>;

/// Streamable HTTP resource and abuse-control settings.
#[derive(Debug, Clone)]
pub struct HttpServerConfig {
    /// Explicit socket address to bind.
    pub bind: SocketAddr,
    /// Externally visible MCP resource URL. Its path must be `/mcp`.
    pub public_url: Url,
    /// Browser origins permitted to call the MCP endpoint.
    pub allowed_origins: Vec<String>,
    /// OAuth resource-server validation. Required for non-loopback binds.
    pub oauth: Option<OAuthResourceServerConfig>,
    /// Idle time before an inactive authorization context without retained tasks may be evicted.
    pub authorization_idle_timeout: Duration,
    /// Maximum request body size.
    pub max_request_body_bytes: usize,
    /// Maximum concurrent requests, including response body and SSE lifetimes.
    pub max_concurrent_requests: usize,
    /// Maximum concurrent requests/response streams for one authorization context.
    pub max_concurrent_requests_per_authorization: usize,
    /// Maximum number of requests per minute for one authorization context.
    pub requests_per_minute: u32,
    /// Maximum number of authorization contexts retained by the process.
    pub max_authorization_contexts: usize,
}

impl HttpServerConfig {
    /// Creates secure defaults for an explicit bind address and public URL.
    pub fn new(bind: SocketAddr, public_url: Url) -> Self {
        let public_origin = public_url.origin().ascii_serialization();
        Self {
            bind,
            public_url,
            allowed_origins: vec![public_origin],
            oauth: None,
            authorization_idle_timeout: Duration::from_secs(300),
            max_request_body_bytes: 1024 * 1024,
            max_concurrent_requests: 64,
            max_concurrent_requests_per_authorization: 8,
            requests_per_minute: 600,
            max_authorization_contexts: 256,
        }
    }

    /// Validates transport security invariants before the listener starts.
    pub fn validate(&self) -> Result<(), HttpConfigError> {
        if self.public_url.path() != MCP_PATH
            || self.public_url.query().is_some()
            || self.public_url.fragment().is_some()
        {
            return Err(HttpConfigError::PublicUrlPath);
        }
        ensure_public_url_security(&self.public_url)?;
        if !self.bind.ip().is_loopback() && self.oauth.is_none() {
            return Err(HttpConfigError::OAuthRequired);
        }
        if self.allowed_origins.is_empty() {
            return Err(HttpConfigError::OriginsRequired);
        }
        for origin in &self.allowed_origins {
            validate_origin(origin)?;
        }
        if self.max_request_body_bytes == 0
            || self.max_concurrent_requests == 0
            || self.max_concurrent_requests_per_authorization == 0
            || self.requests_per_minute == 0
            || self.max_authorization_contexts == 0
            || self.authorization_idle_timeout.is_zero()
        {
            return Err(HttpConfigError::ZeroLimit);
        }
        Ok(())
    }

    fn allowed_hosts(&self) -> Vec<String> {
        let mut hosts = vec![socket_authority(self.bind)];
        if let Some(host) = self.public_url.host_str()
            && let Some(port) = self.public_url.port_or_known_default()
        {
            hosts.push(authority(host, port));
        }
        hosts.sort();
        hosts.dedup();
        hosts
    }

    fn allowed_sdk_origins(&self) -> Vec<String> {
        self.allowed_origins
            .iter()
            .map(|origin| {
                if origin == "null" {
                    return origin.clone();
                }
                let url = Url::parse(origin).expect("validated origin");
                // rmcp 3.5.1 treats portless allowlist entries as any-port origins.
                format!(
                    "{}://{}",
                    url.scheme(),
                    authority(
                        url.host_str().expect("validated origin host"),
                        url.port_or_known_default().expect("validated web scheme"),
                    )
                )
            })
            .collect()
    }

    fn normalize_default_host(&self, request: &mut Request<Body>) {
        // Normalize only the exact public authority, preserving strict port checks
        // when HTTP/2 supplies :authority instead of a Host header.
        if self.public_url.port().is_some() {
            return;
        }
        let Some(host) = self.public_url.host_str() else {
            return;
        };
        let supplied = match request.headers().get_all("host").iter().count() {
            0 => request.uri().authority().map(|value| value.as_str()),
            1 => request
                .headers()
                .get("host")
                .and_then(|value| value.to_str().ok()),
            _ => None,
        };
        if supplied.is_some_and(|value| value.eq_ignore_ascii_case(host))
            && let Some(port) = self.public_url.port_or_known_default()
            && let Ok(value) = HeaderValue::from_str(&authority(host, port))
        {
            request.headers_mut().insert("host", value);
        }
    }

    fn resource_metadata_url(&self) -> Url {
        let mut url = self.public_url.clone();
        url.set_path(RESOURCE_METADATA_PATH);
        url
    }
}

/// Invalid Streamable HTTP server configuration.
#[derive(Debug, Error)]
pub enum HttpConfigError {
    #[error("public URL must use the exact /mcp path without query or fragment")]
    PublicUrlPath,
    #[error("public URL must use HTTPS unless it targets loopback")]
    InsecurePublicUrl,
    #[error("OAuth resource-server configuration is required for non-loopback listeners")]
    OAuthRequired,
    #[error("at least one explicit allowed Origin is required")]
    OriginsRequired,
    #[error("allowed Origin is invalid: {0}")]
    InvalidOrigin(String),
    #[error("transport limits must all be greater than zero")]
    ZeroLimit,
}

fn ensure_public_url_security(url: &Url) -> Result<(), HttpConfigError> {
    let loopback = url
        .host_str()
        .and_then(|host| host.parse::<IpAddr>().ok())
        .is_some_and(|address| address.is_loopback())
        || url.host_str() == Some("localhost");
    if url.scheme() == "https" || (url.scheme() == "http" && loopback) {
        Ok(())
    } else {
        Err(HttpConfigError::InsecurePublicUrl)
    }
}

fn validate_origin(origin: &str) -> Result<(), HttpConfigError> {
    if origin == "null" {
        return Ok(());
    }
    let url =
        Url::parse(origin).map_err(|error| HttpConfigError::InvalidOrigin(error.to_string()))?;
    if !matches!(url.scheme(), "http" | "https")
        || url.cannot_be_a_base()
        || url.path() != "/"
        || url.query().is_some()
        || url.fragment().is_some()
        || url.origin().ascii_serialization() != origin.trim_end_matches('/')
    {
        return Err(HttpConfigError::InvalidOrigin(origin.to_owned()));
    }
    Ok(())
}

fn authority(host: &str, port: u16) -> String {
    if host.contains(':') && !host.starts_with('[') {
        format!("[{host}]:{port}")
    } else {
        format!("{host}:{port}")
    }
}

fn socket_authority(address: SocketAddr) -> String {
    authority(&address.ip().to_string(), address.port())
}

#[derive(Debug)]
struct FixedWindowRateLimiter {
    window_started: Instant,
    count: u32,
    limit: u32,
}

impl FixedWindowRateLimiter {
    fn new(limit: u32) -> Self {
        Self {
            window_started: Instant::now(),
            count: 0,
            limit,
        }
    }

    fn allow(&mut self) -> bool {
        if self.window_started.elapsed() >= Duration::from_secs(60) {
            self.window_started = Instant::now();
            self.count = 0;
        }
        if self.count >= self.limit {
            return false;
        }
        self.count += 1;
        true
    }
}

struct AuthorizationTransport {
    service: McpTransport,
    server: SoraMcpServer,
    rate_limiter: Mutex<FixedWindowRateLimiter>,
    concurrency: Arc<Semaphore>,
    last_seen: StdMutex<Instant>,
}

struct HttpState {
    workspace: Arc<WorkspaceService>,
    config: HttpServerConfig,
    authenticator: Option<Arc<OAuthAuthenticator>>,
    transports: Mutex<BTreeMap<Arc<str>, Arc<AuthorizationTransport>>>,
    concurrency: Arc<Semaphore>,
    cancellation: CancellationToken,
}

impl HttpState {
    async fn transport_for(
        &self,
        principal: &AuthorizationPrincipal,
    ) -> Result<Arc<AuthorizationTransport>, HttpRequestError> {
        let mut transports = self.transports.lock().await;
        if let Some(transport) = transports.get(&principal.context) {
            *transport.last_seen.lock().expect("last-seen lock") = Instant::now();
            return Ok(transport.clone());
        }
        if transports.len() >= self.config.max_authorization_contexts {
            transports.retain(|_, transport| {
                // All request/body references are acquired while holding this map lock.
                // A sole map reference therefore excludes in-flight dispatch and SSE.
                Arc::strong_count(transport) != 1
                    || transport
                        .last_seen
                        .lock()
                        .expect("last-seen lock")
                        .elapsed()
                        < self.config.authorization_idle_timeout
                    || transport.server.has_retained_tasks()
                    || transport.server.has_pending_inputs()
            });
            if transports.len() >= self.config.max_authorization_contexts {
                return Err(HttpRequestError::AuthorizationCapacity);
            }
        }

        let server = SoraMcpServer::new_with_authorization_context(
            self.workspace.clone(),
            principal.context.clone(),
        );
        let protocol_config = StreamableHttpServerConfig::default()
            .with_legacy_session_mode(false)
            .with_stateless_protocol_metadata_required(true)
            .with_json_response(true)
            .with_max_request_body_bytes(self.config.max_request_body_bytes)
            .with_allowed_hosts(self.config.allowed_hosts())
            .with_allowed_origins(self.config.allowed_sdk_origins())
            .with_sse_keep_alive(Some(Duration::from_secs(15)))
            .with_cancellation_token(self.cancellation.child_token());
        let service = StreamableHttpService::new(
            {
                let server = server.clone();
                move || Ok(server.clone())
            },
            Default::default(),
            protocol_config,
        );
        let transport = Arc::new(AuthorizationTransport {
            service,
            server,
            rate_limiter: Mutex::new(FixedWindowRateLimiter::new(self.config.requests_per_minute)),
            concurrency: Arc::new(Semaphore::new(
                self.config.max_concurrent_requests_per_authorization,
            )),
            last_seen: StdMutex::new(Instant::now()),
        });
        transports.insert(principal.context.clone(), transport.clone());
        Ok(transport)
    }

    async fn authenticate(
        &self,
        headers: &HeaderMap,
    ) -> Result<AuthorizationPrincipal, OAuthError> {
        match &self.authenticator {
            Some(authenticator) => {
                if headers.get_all(AUTHORIZATION).iter().count() > 1 {
                    return Err(OAuthError::MalformedToken);
                }
                let header = headers
                    .get(AUTHORIZATION)
                    .and_then(|value| value.to_str().ok());
                authenticator.authenticate(header).await
            }
            None => Ok(AuthorizationPrincipal {
                context: Arc::from("local-http"),
                subject: Arc::from("local"),
                expires_at: u64::MAX,
            }),
        }
    }

    fn resource_metadata(&self) -> Option<ProtectedResourceMetadata> {
        self.authenticator
            .as_ref()
            .map(|authenticator| authenticator.protected_resource_metadata(&self.config.public_url))
    }

    fn authenticate_header(&self, error: &OAuthError) -> HeaderValue {
        let metadata = self.config.resource_metadata_url();
        let scopes = self
            .authenticator
            .as_ref()
            .map(|authenticator| {
                authenticator
                    .required_scopes()
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .unwrap_or_default();
        let error_parameter = match error {
            OAuthError::InsufficientScope => "insufficient_scope",
            OAuthError::MissingToken => "",
            OAuthError::MalformedToken
            | OAuthError::InvalidToken
            | OAuthError::InvalidServerMetadata(_)
            | OAuthError::Discovery(_)
            | OAuthError::Jwks(_) => "invalid_token",
        };
        let mut value = format!(
            "Bearer resource_metadata=\"{}\", scope=\"{scopes}\"",
            metadata.as_str()
        );
        if !error_parameter.is_empty() {
            value.push_str(&format!(", error=\"{error_parameter}\""));
        }
        HeaderValue::from_str(&value)
            .unwrap_or_else(|_| HeaderValue::from_static("Bearer error=\"invalid_token\""))
    }
}

#[derive(Debug, Error)]
enum HttpRequestError {
    #[error("authorization context capacity reached")]
    AuthorizationCapacity,
    #[error("request rate limit exceeded")]
    RateLimit,
    #[error("request concurrency limit exceeded")]
    ConcurrencyLimit,
    #[error("request body size limit exceeded")]
    PayloadTooLarge,
}

/// Builds the Sora Streamable HTTP application after validating OAuth and
/// transport configuration.
pub async fn build_http_router(
    workspace: Arc<WorkspaceService>,
    config: HttpServerConfig,
    cancellation: CancellationToken,
) -> Result<Router> {
    config
        .validate()
        .context("invalid HTTP MCP configuration")?;
    let authenticator = match config.oauth.clone() {
        Some(oauth) => Some(Arc::new(
            OAuthAuthenticator::initialize(oauth)
                .await
                .context("failed to initialize OAuth resource server")?,
        )),
        None => None,
    };
    let state = Arc::new(HttpState {
        workspace,
        concurrency: Arc::new(Semaphore::new(config.max_concurrent_requests)),
        config: config.clone(),
        authenticator,
        transports: Mutex::new(BTreeMap::new()),

        cancellation,
    });
    Ok(router_with_state(state))
}

fn router_with_state(state: Arc<HttpState>) -> Router {
    Router::new()
        .route(MCP_PATH, any(handle_mcp))
        .route(RESOURCE_METADATA_PATH, get(handle_resource_metadata))
        .with_state(state)
        .layer(SetSensitiveRequestHeadersLayer::new(once(AUTHORIZATION)))
}

/// Runs the Streamable HTTP listener until Ctrl-C or listener failure.
pub fn serve_http(workspace: Arc<WorkspaceService>, config: HttpServerConfig) -> Result<()> {
    let bind = config.bind;
    let runtime = Runtime::new().context("failed to start MCP HTTP async runtime")?;
    runtime.block_on(async move {
        let cancellation = CancellationToken::new();
        let router = build_http_router(workspace, config, cancellation.clone()).await?;
        let listener = TcpListener::bind(bind)
            .await
            .with_context(|| format!("failed to bind MCP HTTP listener at {bind}"))?;
        info!(bind = %bind, "Sora MCP Streamable HTTP listener started");
        serve(listener, router)
            .with_graceful_shutdown(async move {
                if ctrl_c().await.is_ok() {
                    cancellation.cancel();
                }
            })
            .await
            .context("MCP HTTP listener failed")
    })
}

async fn handle_resource_metadata(State(state): State<Arc<HttpState>>) -> Response<Body> {
    match state.resource_metadata() {
        Some(metadata) => Json(metadata).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn handle_mcp(
    State(state): State<Arc<HttpState>>,
    mut request: Request<Body>,
) -> Response<Body> {
    let request_id = Uuid::new_v4();
    let started = Instant::now();
    let method = request.method().clone();

    let principal = match state.authenticate(request.headers()).await {
        Ok(principal) => principal,
        Err(error) => {
            let status = if matches!(error, OAuthError::InsufficientScope) {
                StatusCode::FORBIDDEN
            } else {
                StatusCode::UNAUTHORIZED
            };
            warn!(
                %request_id,
                %method,
                status = status.as_u16(),
                reason = %error,
                duration_ms = started.elapsed().as_millis(),
                "MCP HTTP request rejected"
            );
            let mut response = status.into_response();
            response
                .headers_mut()
                .insert(WWW_AUTHENTICATE, state.authenticate_header(&error));
            return response;
        }
    };
    let transport = match state.transport_for(&principal).await {
        Ok(transport) => transport,
        Err(error) => return limited_response(&principal, request_id, method.as_str(), error),
    };
    if !transport.rate_limiter.lock().await.allow() {
        return limited_response(
            &principal,
            request_id,
            method.as_str(),
            HttpRequestError::RateLimit,
        );
    }
    let global_permit = match state.concurrency.clone().try_acquire_owned() {
        Ok(permit) => permit,
        Err(_) => {
            return limited_response(
                &principal,
                request_id,
                method.as_str(),
                HttpRequestError::ConcurrencyLimit,
            );
        }
    };
    let authorization_permit = match transport.concurrency.clone().try_acquire_owned() {
        Ok(permit) => permit,
        Err(_) => {
            return limited_response(
                &principal,
                request_id,
                method.as_str(),
                HttpRequestError::ConcurrencyLimit,
            );
        }
    };

    let lifetime = ResponseLifetime {
        transport: transport.clone(),
        _global_permit: global_permit,
        _authorization_permit: authorization_permit,
    };
    state.config.normalize_default_host(&mut request);
    let (parts, body) = request.into_parts();
    let body = match to_bytes(body, state.config.max_request_body_bytes).await {
        Ok(body) => body,
        Err(_) => {
            return limited_response(
                &principal,
                request_id,
                method.as_str(),
                HttpRequestError::PayloadTooLarge,
            );
        }
    };
    // rmcp validates duplicate standard method/name headers, but 3.5.1 only
    // reads the first protocol-version value. Reject ambiguity before dispatch.
    if parts.headers.get_all("mcp-protocol-version").iter().count() > 1 {
        let id = from_slice::<Value>(&body)
            .ok()
            .and_then(|value| value.get("id").cloned());
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "jsonrpc": "2.0", "id": id,
                "error": {"code": -32020, "message": "Duplicate MCP-Protocol-Version header"}
            })),
        )
            .into_response();
    }
    let response = transport
        .service
        .handle(Request::from_parts(parts, Body::from(body)))
        .await;
    let status = response.status();
    info!(
        %request_id,
        authorization_context = principal.context.as_ref(),
        %method,
        status = status.as_u16(),
        duration_ms = started.elapsed().as_millis(),
        "MCP HTTP request completed"
    );
    let (parts, body) = response.into_parts();
    retain_response_lifetime(Response::from_parts(parts, Body::new(body)), lifetime)
}

fn limited_response(
    principal: &AuthorizationPrincipal,
    request_id: Uuid,
    method: &str,
    error: HttpRequestError,
) -> Response<Body> {
    let status = match error {
        HttpRequestError::AuthorizationCapacity => StatusCode::SERVICE_UNAVAILABLE,
        HttpRequestError::RateLimit | HttpRequestError::ConcurrencyLimit => {
            StatusCode::TOO_MANY_REQUESTS
        }
        HttpRequestError::PayloadTooLarge => StatusCode::PAYLOAD_TOO_LARGE,
    };
    warn!(
        %request_id,
        authorization_context = principal.context.as_ref(),
        method,
        status = status.as_u16(),
        reason = %error,
        "MCP HTTP request limited"
    );
    status.into_response()
}

struct ResponseLifetime {
    transport: Arc<AuthorizationTransport>,
    _global_permit: OwnedSemaphorePermit,
    _authorization_permit: OwnedSemaphorePermit,
}

impl Drop for ResponseLifetime {
    fn drop(&mut self) {
        *self.transport.last_seen.lock().expect("last-seen lock") = Instant::now();
    }
}

fn retain_response_lifetime(
    response: Response<Body>,
    lifetime: ResponseLifetime,
) -> Response<Body> {
    let (parts, body) = response.into_parts();
    let stream = stream::unfold(
        (body.into_data_stream(), lifetime),
        |(mut body, lifetime)| async move { body.next().await.map(|frame| (frame, (body, lifetime))) },
    );
    Response::from_parts(parts, Body::from_stream(stream))
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
