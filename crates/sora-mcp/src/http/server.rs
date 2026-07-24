use std::{
    collections::BTreeMap,
    net::{IpAddr, SocketAddr},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};

use anyhow::{Context, Result};
use axum::{
    Json, Router,
    body::Body,
    extract::State,
    http::{
        HeaderMap, HeaderValue, Request, Response, StatusCode,
        header::{AUTHORIZATION, WWW_AUTHENTICATE},
    },
    response::IntoResponse,
    routing::{any, get},
};
use hmac::{Hmac, Mac};
use rmcp::transport::streamable_http_server::{StreamableHttpServerConfig, StreamableHttpService};
use sha2::Sha256;
use sora_workspace::WorkspaceService;
use thiserror::Error;
use tokio::sync::{Mutex, Semaphore};
use tokio_util::sync::CancellationToken;
use tower_http::sensitive_headers::SetSensitiveRequestHeadersLayer;
use url::Url;
use uuid::Uuid;

use super::{
    AuthorizationPrincipal, OAuthAuthenticator, OAuthError, OAuthResourceServerConfig,
    ProtectedResourceMetadata, SecureSessionConfig, SecureSessionManager,
};
use crate::SoraMcpServer;

const MCP_PATH: &str = "/mcp";
const RESOURCE_METADATA_PATH: &str = "/.well-known/oauth-protected-resource";

type HmacSha256 = Hmac<Sha256>;
type McpTransport = StreamableHttpService<SoraMcpServer, SecureSessionManager>;

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
    /// Per-authorization session limits and timeouts.
    pub sessions: SecureSessionConfig,
    /// Maximum request body size.
    pub max_request_body_bytes: usize,
    /// Maximum requests entering the MCP transport concurrently.
    pub max_concurrent_requests: usize,
    /// Maximum concurrent requests for one authorization context.
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
            sessions: SecureSessionConfig::default(),
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
            || self.sessions.max_sessions == 0
            || self.sessions.channel_capacity == 0
        {
            return Err(HttpConfigError::ZeroLimit);
        }
        Ok(())
    }

    fn allowed_hosts(&self) -> Vec<String> {
        let mut hosts = vec![socket_authority(self.bind)];
        if let Some(host) = self.public_url.host_str() {
            hosts.push(match self.public_url.port() {
                Some(port) => authority(host, port),
                None => host.to_owned(),
            });
        }
        hosts.sort();
        hosts.dedup();
        hosts
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
    if host.contains(':') {
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
    sessions: Arc<SecureSessionManager>,
    rate_limiter: Mutex<FixedWindowRateLimiter>,
    concurrency: Arc<Semaphore>,
    last_seen: AtomicU64,
}

struct HttpState {
    workspace: Arc<WorkspaceService>,
    config: HttpServerConfig,
    authenticator: Option<Arc<OAuthAuthenticator>>,
    transports: Mutex<BTreeMap<Arc<str>, Arc<AuthorizationTransport>>>,
    concurrency: Arc<Semaphore>,
    signing_key: [u8; 32],
    cancellation: CancellationToken,
}

impl HttpState {
    async fn transport_for(
        &self,
        principal: &AuthorizationPrincipal,
    ) -> Result<Arc<AuthorizationTransport>, HttpRequestError> {
        let mut transports = self.transports.lock().await;
        if let Some(transport) = transports.get(&principal.context) {
            transport.last_seen.store(unix_seconds(), Ordering::Relaxed);
            return Ok(transport.clone());
        }
        if transports.len() >= self.config.max_authorization_contexts {
            let stale_before =
                unix_seconds().saturating_sub(self.config.sessions.idle_timeout.as_secs());
            let candidates = transports
                .iter()
                .filter(|(_, transport)| transport.last_seen.load(Ordering::Relaxed) < stale_before)
                .map(|(context, transport)| (context.clone(), transport.clone()))
                .collect::<Vec<_>>();
            for (context, transport) in candidates {
                if transport.sessions.active_session_count().await == 0 {
                    transports.remove(&context);
                }
            }
            if transports.len() >= self.config.max_authorization_contexts {
                return Err(HttpRequestError::AuthorizationCapacity);
            }
        }

        let signing_key = derive_signing_key(&self.signing_key, &principal.context)?;
        let manager = Arc::new(
            SecureSessionManager::new(
                principal.context.clone(),
                &signing_key,
                self.config.sessions.clone(),
            )
            .map_err(|_| HttpRequestError::Internal)?,
        );
        let server = SoraMcpServer::new_with_authorization_context(
            self.workspace.clone(),
            principal.context.clone(),
        );
        let protocol_config = StreamableHttpServerConfig::default()
            .with_allowed_hosts(self.config.allowed_hosts())
            .with_allowed_origins(self.config.allowed_origins.clone())
            .with_sse_keep_alive(Some(Duration::from_secs(15)))
            .with_sse_retry(Some(Duration::from_secs(3)))
            .with_cancellation_token(self.cancellation.child_token());
        let service = StreamableHttpService::new(
            move || Ok(server.clone()),
            manager.clone(),
            protocol_config,
        );
        let transport = Arc::new(AuthorizationTransport {
            service,
            sessions: manager,
            rate_limiter: Mutex::new(FixedWindowRateLimiter::new(self.config.requests_per_minute)),
            concurrency: Arc::new(Semaphore::new(
                self.config.max_concurrent_requests_per_authorization,
            )),
            last_seen: AtomicU64::new(unix_seconds()),
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
    #[error("internal HTTP transport error")]
    Internal,
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
        signing_key: random_signing_key(),
        cancellation,
    });
    Ok(router_with_state(state))
}

fn router_with_state(state: Arc<HttpState>) -> Router {
    Router::new()
        .route(MCP_PATH, any(handle_mcp))
        .route(RESOURCE_METADATA_PATH, get(handle_resource_metadata))
        .with_state(state)
        .layer(SetSensitiveRequestHeadersLayer::new(std::iter::once(
            AUTHORIZATION,
        )))
}

/// Runs the Streamable HTTP listener until Ctrl-C or listener failure.
pub fn serve_http(workspace: Arc<WorkspaceService>, config: HttpServerConfig) -> Result<()> {
    let bind = config.bind;
    let runtime =
        tokio::runtime::Runtime::new().context("failed to start MCP HTTP async runtime")?;
    runtime.block_on(async move {
        let cancellation = CancellationToken::new();
        let router = build_http_router(workspace, config, cancellation.clone()).await?;
        let listener = tokio::net::TcpListener::bind(bind)
            .await
            .with_context(|| format!("failed to bind MCP HTTP listener at {bind}"))?;
        tracing::info!(bind = %bind, "Sora MCP Streamable HTTP listener started");
        axum::serve(listener, router)
            .with_graceful_shutdown(async move {
                if tokio::signal::ctrl_c().await.is_ok() {
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

async fn handle_mcp(State(state): State<Arc<HttpState>>, request: Request<Body>) -> Response<Body> {
    let request_id = Uuid::new_v4();
    let started = Instant::now();
    let method = request.method().clone();
    let session_id = request
        .headers()
        .get("mcp-session-id")
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);
    let principal = match state.authenticate(request.headers()).await {
        Ok(principal) => principal,
        Err(error) => {
            let status = if matches!(error, OAuthError::InsufficientScope) {
                StatusCode::FORBIDDEN
            } else {
                StatusCode::UNAUTHORIZED
            };
            tracing::warn!(
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

    let (parts, body) = request.into_parts();
    let body = match axum::body::to_bytes(body, state.config.max_request_body_bytes).await {
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
    let response = transport
        .service
        .handle(Request::from_parts(parts, Body::from(body)))
        .await;
    let status = response.status();
    tracing::info!(
        %request_id,
        authorization_context = principal.context.as_ref(),
        %method,
        session_id = session_id.as_deref().unwrap_or("new"),
        status = status.as_u16(),
        duration_ms = started.elapsed().as_millis(),
        "MCP HTTP request completed"
    );
    drop(authorization_permit);
    drop(global_permit);
    let (parts, body) = response.into_parts();
    Response::from_parts(parts, Body::new(body))
}

fn limited_response(
    principal: &AuthorizationPrincipal,
    request_id: Uuid,
    method: &str,
    error: HttpRequestError,
) -> Response<Body> {
    let status = match error {
        HttpRequestError::AuthorizationCapacity | HttpRequestError::Internal => {
            StatusCode::SERVICE_UNAVAILABLE
        }
        HttpRequestError::RateLimit | HttpRequestError::ConcurrencyLimit => {
            StatusCode::TOO_MANY_REQUESTS
        }
        HttpRequestError::PayloadTooLarge => StatusCode::PAYLOAD_TOO_LARGE,
    };
    tracing::warn!(
        %request_id,
        authorization_context = principal.context.as_ref(),
        method,
        status = status.as_u16(),
        reason = %error,
        "MCP HTTP request limited"
    );
    status.into_response()
}

fn random_signing_key() -> [u8; 32] {
    let mut key = [0_u8; 32];
    key[..16].copy_from_slice(Uuid::new_v4().as_bytes());
    key[16..].copy_from_slice(Uuid::new_v4().as_bytes());
    key
}

fn derive_signing_key(
    root_key: &[u8; 32],
    authorization_context: &str,
) -> Result<[u8; 32], HttpRequestError> {
    let mut signer =
        HmacSha256::new_from_slice(root_key).map_err(|_| HttpRequestError::Internal)?;
    signer.update(authorization_context.as_bytes());
    Ok(signer.finalize().into_bytes().into())
}

fn unix_seconds() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs())
}

#[cfg(test)]
mod tests {
    use std::{
        net::{IpAddr, Ipv4Addr, SocketAddr},
        sync::Arc,
    };

    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use jsonwebtoken::jwk::JwkSet;
    use sora_workspace::WorkspaceService;
    use tokio_util::sync::CancellationToken;
    use tower::ServiceExt;

    use super::{
        AuthorizationPrincipal, HttpConfigError, HttpServerConfig, HttpState, OAuthAuthenticator,
        OAuthResourceServerConfig, build_http_router, router_with_state,
    };

    const INITIALIZE: &str = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"security-test","version":"1.0"}}}"#;

    #[test]
    fn non_loopback_listener_requires_oauth() {
        let config = HttpServerConfig::new(
            SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 8080),
            url::Url::parse("https://sora.example.com/mcp").expect("valid URL"),
        );
        assert!(matches!(
            config.validate(),
            Err(HttpConfigError::OAuthRequired)
        ));
    }

    #[test]
    fn origin_must_be_an_exact_web_origin() {
        let mut config = HttpServerConfig::new(
            SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8080),
            url::Url::parse("http://127.0.0.1:8080/mcp").expect("valid URL"),
        );
        config.allowed_origins = vec!["https://client.example.com/path".to_owned()];
        assert!(matches!(
            config.validate(),
            Err(HttpConfigError::InvalidOrigin(_))
        ));
    }

    #[test]
    fn local_http_configuration_is_valid() {
        let config = HttpServerConfig::new(
            SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8080),
            url::Url::parse("http://127.0.0.1:8080/mcp").expect("valid URL"),
        );
        assert!(config.validate().is_ok());
    }

    #[tokio::test]
    async fn router_rejects_unlisted_origin_and_dns_rebinding_host() {
        let router = build_http_router(
            Arc::new(WorkspaceService::new()),
            local_config(),
            CancellationToken::new(),
        )
        .await
        .expect("valid router");

        let bad_origin = router
            .clone()
            .oneshot(initialize_request(
                "127.0.0.1:8080",
                "https://evil.example.com",
                INITIALIZE,
            ))
            .await
            .expect("response");
        assert_eq!(bad_origin.status(), StatusCode::FORBIDDEN);

        let bad_host = router
            .oneshot(initialize_request(
                "attacker.example.com",
                "http://127.0.0.1:8080",
                INITIALIZE,
            ))
            .await
            .expect("response");
        assert_eq!(bad_host.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn router_enforces_request_body_limit_without_content_length() {
        let mut config = local_config();
        config.max_request_body_bytes = 64;
        let router = build_http_router(
            Arc::new(WorkspaceService::new()),
            config,
            CancellationToken::new(),
        )
        .await
        .expect("valid router");
        let response = router
            .oneshot(initialize_request(
                "127.0.0.1:8080",
                "http://127.0.0.1:8080",
                INITIALIZE,
            ))
            .await
            .expect("response");
        assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
    }

    #[tokio::test]
    async fn local_router_creates_and_deletes_stateful_session() {
        let router = build_http_router(
            Arc::new(WorkspaceService::new()),
            local_config(),
            CancellationToken::new(),
        )
        .await
        .expect("valid router");
        let initialized = router
            .clone()
            .oneshot(initialize_request(
                "127.0.0.1:8080",
                "http://127.0.0.1:8080",
                INITIALIZE,
            ))
            .await
            .expect("response");
        assert_eq!(initialized.status(), StatusCode::OK);
        let session_id = initialized
            .headers()
            .get("mcp-session-id")
            .expect("session header")
            .clone();

        let deleted = router
            .oneshot(
                Request::delete("/mcp")
                    .header("host", "127.0.0.1:8080")
                    .header("origin", "http://127.0.0.1:8080")
                    .header("mcp-session-id", session_id)
                    .body(Body::empty())
                    .expect("valid request"),
            )
            .await
            .expect("response");
        assert_eq!(deleted.status(), StatusCode::ACCEPTED);
    }

    #[tokio::test]
    async fn sessions_are_bound_to_authorization_context() {
        let state = test_state(local_config(), None);
        let principal_a = principal("oauth:subject-a");
        let principal_b = principal("oauth:subject-b");
        let transport_a = state
            .transport_for(&principal_a)
            .await
            .expect("transport A");
        let transport_b = state
            .transport_for(&principal_b)
            .await
            .expect("transport B");

        let initialized = transport_a
            .service
            .handle(initialize_request(
                "127.0.0.1:8080",
                "http://127.0.0.1:8080",
                INITIALIZE,
            ))
            .await;
        assert_eq!(initialized.status(), StatusCode::OK);
        let session_id = initialized
            .headers()
            .get("mcp-session-id")
            .expect("session header")
            .clone();

        let cross_authorization = transport_b
            .service
            .handle(
                Request::get("/mcp")
                    .header("host", "127.0.0.1:8080")
                    .header("origin", "http://127.0.0.1:8080")
                    .header("accept", "text/event-stream")
                    .header("mcp-session-id", session_id.clone())
                    .body(Body::empty())
                    .expect("valid request"),
            )
            .await;
        assert_eq!(cross_authorization.status(), StatusCode::NOT_FOUND);

        let owner = transport_a
            .service
            .handle(
                Request::delete("/mcp")
                    .header("host", "127.0.0.1:8080")
                    .header("origin", "http://127.0.0.1:8080")
                    .header("mcp-session-id", session_id)
                    .body(Body::empty())
                    .expect("valid request"),
            )
            .await;
        assert_eq!(owner.status(), StatusCode::ACCEPTED);
    }

    #[tokio::test]
    async fn protected_router_advertises_metadata_and_challenges_missing_token() {
        let mut config = local_config();
        let oauth = OAuthResourceServerConfig::new(
            url::Url::parse("https://id.example.com").expect("valid issuer"),
            "http://127.0.0.1:8080/mcp",
        );
        config.oauth = Some(oauth.clone());
        let authenticator = OAuthAuthenticator::for_testing(
            oauth,
            serde_json::from_str::<JwkSet>(r#"{"keys":[]}"#).expect("valid JWKS"),
        );
        let router = router_with_state(test_state(config, Some(Arc::new(authenticator))));

        let metadata = router
            .clone()
            .oneshot(
                Request::get("/.well-known/oauth-protected-resource")
                    .body(Body::empty())
                    .expect("valid request"),
            )
            .await
            .expect("response");
        assert_eq!(metadata.status(), StatusCode::OK);

        let unauthorized = router
            .oneshot(initialize_request(
                "127.0.0.1:8080",
                "http://127.0.0.1:8080",
                INITIALIZE,
            ))
            .await
            .expect("response");
        assert_eq!(unauthorized.status(), StatusCode::UNAUTHORIZED);
        let challenge = unauthorized
            .headers()
            .get("www-authenticate")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default();
        assert!(challenge.contains("resource_metadata="));
        assert!(challenge.contains("scope=\"sora:mcp\""));
    }

    fn local_config() -> HttpServerConfig {
        HttpServerConfig::new(
            SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8080),
            url::Url::parse("http://127.0.0.1:8080/mcp").expect("valid URL"),
        )
    }

    fn principal(context: &str) -> AuthorizationPrincipal {
        AuthorizationPrincipal {
            context: Arc::from(context),
            subject: Arc::from("test-subject"),
            expires_at: u64::MAX,
        }
    }

    fn test_state(
        config: HttpServerConfig,
        authenticator: Option<Arc<OAuthAuthenticator>>,
    ) -> Arc<HttpState> {
        Arc::new(HttpState {
            workspace: Arc::new(WorkspaceService::new()),
            concurrency: Arc::new(tokio::sync::Semaphore::new(config.max_concurrent_requests)),
            config,
            authenticator,
            transports: tokio::sync::Mutex::new(std::collections::BTreeMap::new()),
            signing_key: [7; 32],
            cancellation: CancellationToken::new(),
        })
    }

    fn initialize_request(host: &str, origin: &str, body: &'static str) -> Request<Body> {
        Request::post("/mcp")
            .header("host", host)
            .header("origin", origin)
            .header("content-type", "application/json")
            .header("accept", "application/json, text/event-stream")
            .body(Body::from(body))
            .expect("valid request")
    }
}
