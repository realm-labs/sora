use std::{
    collections::BTreeMap,
    env::temp_dir,
    fs,
    io::Error as IoError,
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

use axum::{
    body::{Body, to_bytes},
    http::{HeaderValue, Request, Response, StatusCode},
};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use jsonwebtoken::{Algorithm, EncodingKey, Header, encode, get_current_timestamp};
use reqwest::Client;
use rmcp::model::{ClientCapabilities, InputRequiredResult};
use serde_json::{Value, from_slice, from_str, from_value, json, to_value};
use sora_workspace::{ProjectId, RuntimeOptions, WorkspaceService};
use tokio::{
    spawn,
    sync::{Mutex, Semaphore},
    task::JoinHandle,
    time::{sleep, timeout},
};
use tokio_util::sync::CancellationToken;
use tower::ServiceExt;
use url::Url;

use super::*;

const VERSION: &str = "2026-07-28";
const PROTOCOL: &str = "io.modelcontextprotocol/protocolVersion";
const CAPS: &str = "io.modelcontextprotocol/clientCapabilities";

fn local_config() -> HttpServerConfig {
    HttpServerConfig::new(
        "127.0.0.1:8080".parse().unwrap(),
        Url::parse("http://127.0.0.1:8080/mcp").unwrap(),
    )
}

fn test_state(config: HttpServerConfig) -> Arc<HttpState> {
    Arc::new(HttpState {
        workspace: Arc::new(WorkspaceService::new()),
        concurrency: Arc::new(Semaphore::new(config.max_concurrent_requests)),
        config,
        authenticator: None,
        transports: Mutex::new(BTreeMap::new()),
        cancellation: CancellationToken::new(),
    })
}

fn principal(context: &str) -> AuthorizationPrincipal {
    AuthorizationPrincipal {
        context: Arc::from(context),
        subject: Arc::from(context),
        expires_at: u64::MAX,
    }
}

fn message(method: &str, mut params: Value, tasks: bool) -> Value {
    let caps = if tasks {
        to_value(ClientCapabilities::builder().enable_tasks().build()).unwrap()
    } else {
        json!({})
    };
    params["_meta"] = json!({
        PROTOCOL: VERSION,
        CAPS: caps,
        "io.modelcontextprotocol/clientInfo": {"name": "http-contract-test", "version": "1"}
    });
    json!({"jsonrpc": "2.0", "id": 1, "method": method, "params": params})
}

fn request(body: Value) -> Request<Body> {
    let mut request = Request::post("/mcp")
        .header("host", "127.0.0.1:8080")
        .header("origin", "http://127.0.0.1:8080")
        .header("content-type", "application/json")
        .header("accept", "application/json, text/event-stream")
        .header("mcp-protocol-version", VERSION)
        .header("mcp-method", body["method"].as_str().unwrap());
    if let Some(name) = body["params"]["name"]
        .as_str()
        .or_else(|| body["params"]["taskId"].as_str())
        .or_else(|| body["params"]["uri"].as_str())
    {
        request = request.header("mcp-name", name);
    }
    request.body(Body::from(body.to_string())).unwrap()
}

fn discover() -> Request<Body> {
    request(message("server/discover", json!({}), false))
}

async fn json_response(response: Response<Body>) -> Value {
    let bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    from_slice(&bytes).unwrap_or_else(|_| panic!("not JSON: {}", String::from_utf8_lossy(&bytes)))
}

async fn send(router: &Router, body: Value) -> Value {
    let response = router.clone().oneshot(request(body)).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert!(response.headers().get("mcp-session-id").is_none());
    json_response(response).await
}

#[test]
fn configuration_requires_explicit_security_and_nonzero_limits() {
    let mut config = local_config();
    assert!(config.validate().is_ok());
    assert_eq!(config.authorization_idle_timeout, Duration::from_secs(300));
    config.bind = "0.0.0.0:8080".parse().unwrap();
    assert!(matches!(
        config.validate(),
        Err(HttpConfigError::OAuthRequired)
    ));
    config = local_config();
    config.authorization_idle_timeout = Duration::ZERO;
    assert!(matches!(config.validate(), Err(HttpConfigError::ZeroLimit)));
    for origin in [
        "https://client.example/path",
        "https://client.example:*",
        "https://user@client.example",
        "https://client.example?x=1",
    ] {
        config = local_config();
        config.allowed_origins = vec![origin.into()];
        assert!(
            matches!(config.validate(), Err(HttpConfigError::InvalidOrigin(_))),
            "{origin}"
        );
    }
}

#[tokio::test]
async fn discover_and_ordinary_tools_need_no_initialize_or_session() {
    let router = build_http_router(
        Arc::new(WorkspaceService::new()),
        local_config(),
        CancellationToken::new(),
    )
    .await
    .unwrap();
    let response = router.clone().oneshot(discover()).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()["content-type"], "application/json");
    assert!(response.headers().get("mcp-session-id").is_none());
    let body = json_response(response).await;
    assert_eq!(body["result"]["supportedVersions"], json!([VERSION]));
    let listed = send(&router, message("tools/list", json!({}), false)).await;
    assert!(
        listed["result"]["tools"]
            .as_array()
            .unwrap()
            .iter()
            .any(|tool| tool["name"] == "sora_server_info")
    );
    let called = send(
        &router,
        message(
            "tools/call",
            json!({"name": "sora_server_info", "arguments": {}}),
            false,
        ),
    )
    .await;
    assert_eq!(called["result"]["resultType"], "complete");
    assert!(called.get("error").is_none(), "{called}");
}

#[tokio::test]
async fn heavy_tool_without_tasks_extension_is_synchronous_and_retains_no_tasks() {
    let state = test_state(local_config());
    let router = router_with_state(state.clone());
    let result = send(
        &router,
        message(
            "tools/call",
            json!({
                "name": "sora_schema_lock", "arguments": {
                    "project_id": "missing", "expected_project_revision": "missing",
                    "view": null, "clean": true
                }
            }),
            false,
        ),
    )
    .await;
    assert_eq!(result["result"]["resultType"], "complete", "{result}");
    assert_eq!(result["result"]["isError"], true);
    assert!(
        !state
            .transport_for(&principal("local-http"))
            .await
            .unwrap()
            .server
            .has_retained_tasks()
    );
}

#[tokio::test]
async fn missing_metadata_and_standard_headers_never_dispatch() {
    let state = test_state(local_config());
    let router = router_with_state(state.clone());
    let heavy = || {
        message(
            "tools/call",
            json!({"name": "sora_schema_lock", "arguments": {"project_id": "demo", "expected_project_revision": "missing", "view": null, "clean": true}}),
            true,
        )
    };
    for missing in [PROTOCOL, CAPS] {
        let mut body = heavy();
        body["params"]["_meta"]
            .as_object_mut()
            .unwrap()
            .remove(missing);
        let response = router.clone().oneshot(request(body)).await.unwrap();
        assert_eq!(
            response.status(),
            StatusCode::BAD_REQUEST,
            "missing {missing}"
        );
        assert_eq!(json_response(response).await["error"]["code"], -32602);
    }
    for header in ["mcp-protocol-version", "mcp-method", "mcp-name"] {
        let mut req = request(heavy());
        req.headers_mut().remove(header);
        let response = router.clone().oneshot(req).await.unwrap();
        assert_eq!(
            response.status(),
            StatusCode::BAD_REQUEST,
            "missing {header}"
        );
        assert_eq!(json_response(response).await["error"]["code"], -32020);
    }
    let transport = state.transport_for(&principal("local-http")).await.unwrap();
    assert!(!transport.server.has_retained_tasks());
}

#[tokio::test]
async fn rejects_header_body_mismatches_and_duplicate_standard_headers() {
    let state = test_state(local_config());
    let router = router_with_state(state.clone());
    for (header, wrong) in [
        ("mcp-method", "tools/list"),
        ("mcp-name", "other"),
        ("mcp-protocol-version", "2025-11-25"),
    ] {
        for duplicate in [false, true] {
            let mut req = request(message(
                "tools/call",
                json!({"name": "sora_schema_lock", "arguments": {}}),
                true,
            ));
            if duplicate {
                req.headers_mut()
                    .append(header, HeaderValue::from_static(wrong));
            } else {
                req.headers_mut()
                    .insert(header, HeaderValue::from_static(wrong));
            }
            let response = router.clone().oneshot(req).await.unwrap();
            assert_eq!(
                response.status(),
                StatusCode::BAD_REQUEST,
                "{header}, duplicate={duplicate}"
            );
            assert_eq!(json_response(response).await["error"]["code"], -32020);
        }
    }
    assert!(
        !state
            .transport_for(&principal("local-http"))
            .await
            .unwrap()
            .server
            .has_retained_tasks()
    );
}

#[tokio::test]
async fn rejects_legacy_protocol_and_has_no_session_or_resume_routes() {
    let router = router_with_state(test_state(local_config()));
    for method in ["server/discover", "tools/list"] {
        let mut body = message(method, json!({}), false);
        body["params"]["_meta"][PROTOCOL] = json!("2025-11-25");
        let mut req = request(body);
        req.headers_mut().insert(
            "mcp-protocol-version",
            HeaderValue::from_static("2025-11-25"),
        );
        let response = router.clone().oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(json_response(response).await["error"]["code"], -32022);
    }
    let legacy_initialize = json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2025-11-25", "capabilities": {}, "clientInfo": {"name": "obsolete", "version": "1"}}});
    let mut req = request(legacy_initialize);
    req.headers_mut().insert(
        "mcp-protocol-version",
        HeaderValue::from_static("2025-11-25"),
    );
    let response = router.clone().oneshot(req).await.unwrap();
    assert!(response.headers().get("mcp-session-id").is_none());
    let rejected = json_response(response).await;
    assert!(
        rejected.get("error").is_some(),
        "legacy initialization must not negotiate: {rejected}"
    );
    for method in ["GET", "DELETE"] {
        let response = router
            .clone()
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri("/mcp")
                    .header("host", "127.0.0.1:8080")
                    .header("mcp-protocol-version", VERSION)
                    .header("mcp-session-id", "obsolete")
                    .header("last-event-id", "untrusted-replay")
                    .header("accept", "text/event-stream")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    }
}

#[tokio::test]
async fn origin_scheme_host_and_port_are_exact_including_default_ports_and_null() {
    for allowed in [
        "https://client.example",
        "http://client.example",
        "http://[::1]:8080",
        "null",
    ] {
        let mut config = local_config();
        config.allowed_origins = vec![allowed.into()];
        config.validate().unwrap();
        let router = router_with_state(test_state(config));
        let good: Vec<&str> = match allowed {
            "https://client.example" => vec![allowed, "https://client.example:443"],
            "http://client.example" => vec![allowed, "http://client.example:80"],
            _ => vec![allowed],
        };
        for origin in good {
            let mut req = discover();
            req.headers_mut()
                .insert("origin", HeaderValue::from_str(origin).unwrap());
            let response = router.clone().oneshot(req).await.unwrap();
            assert_eq!(
                response.status(),
                StatusCode::OK,
                "allowed={allowed}, origin={origin}"
            );
            json_response(response).await;
        }
        for origin in [
            "https://client.example:444",
            "http://client.example:81",
            "http://[::1]:8081",
            "https://evil.example",
        ] {
            let mut req = discover();
            req.headers_mut()
                .insert("origin", HeaderValue::from_static(origin));
            assert_eq!(
                router.clone().oneshot(req).await.unwrap().status(),
                StatusCode::FORBIDDEN,
                "allowed={allowed}, origin={origin}"
            );
        }
    }
}

#[tokio::test]
async fn host_rebinding_and_public_nondefault_ports_are_rejected() {
    let mut config = local_config();
    config.public_url = Url::parse("https://sora.example/mcp").unwrap();
    let router = router_with_state(test_state(config));
    for host in ["sora.example", "sora.example:443", "127.0.0.1:8080"] {
        let mut req = discover();
        req.headers_mut().remove("origin");
        req.headers_mut()
            .insert("host", HeaderValue::from_static(host));
        let response = router.clone().oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK, "{host}");
        json_response(response).await;
    }
    for host in ["attacker.example", "sora.example:444", "127.0.0.1:8081"] {
        let mut req = discover();
        req.headers_mut().remove("origin");
        req.headers_mut()
            .insert("host", HeaderValue::from_static(host));
        assert_eq!(
            router.clone().oneshot(req).await.unwrap().status(),
            StatusCode::FORBIDDEN,
            "{host}"
        );
    }
}

#[tokio::test]
async fn uri_authority_without_host_preserves_exact_public_default_port_checks() {
    for public_url in [
        "https://sora.example/mcp",
        "http://127.0.0.1/mcp",
        "https://[::1]/mcp",
    ] {
        let mut config = local_config();
        config.public_url = Url::parse(public_url).unwrap();
        let router = router_with_state(test_state(config));
        let mut req = discover();
        req.headers_mut().remove("host");
        req.headers_mut().remove("origin");
        *req.uri_mut() = public_url.parse().unwrap();
        let response = router.clone().oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK, "{public_url}");
        json_response(response).await;

        let mut wrong_port = Url::parse(public_url).unwrap();
        wrong_port.set_port(Some(444)).unwrap();
        let mut req = discover();
        req.headers_mut().remove("host");
        req.headers_mut().remove("origin");
        *req.uri_mut() = wrong_port.as_str().parse().unwrap();
        assert_eq!(
            router.oneshot(req).await.unwrap().status(),
            StatusCode::FORBIDDEN,
            "{wrong_port}"
        );
    }
}

#[tokio::test]
async fn streaming_request_body_limit_does_not_trust_content_length() {
    let mut config = local_config();
    config.max_request_body_bytes = 64;
    let router = router_with_state(test_state(config));
    let mut req = discover();
    assert!(!req.headers().contains_key("content-length"));
    *req.body_mut() = Body::from_stream(stream::iter([
        Ok::<_, IoError>(vec![b' '; 40]),
        Ok(vec![b' '; 40]),
    ]));
    assert_eq!(
        router.oneshot(req).await.unwrap().status(),
        StatusCode::PAYLOAD_TOO_LARGE
    );
}

#[tokio::test]
async fn rate_limit_is_per_authorization_and_survives_responses() {
    let mut config = local_config();
    config.requests_per_minute = 1;
    let state = test_state(config);
    let router = router_with_state(state.clone());
    json_response(router.clone().oneshot(discover()).await.unwrap()).await;
    assert_eq!(
        router.oneshot(discover()).await.unwrap().status(),
        StatusCode::TOO_MANY_REQUESTS
    );
    let other = state.transport_for(&principal("other")).await.unwrap();
    assert!(other.rate_limiter.lock().await.allow());
}

#[tokio::test]
async fn response_body_holds_global_and_authorization_concurrency_until_consumed_or_dropped() {
    for global in [true, false] {
        let mut config = local_config();
        if global {
            config.max_concurrent_requests = 1;
        } else {
            config.max_concurrent_requests_per_authorization = 1;
        }
        let state = test_state(config);
        let router = router_with_state(state.clone());
        let response = router.clone().oneshot(discover()).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            router.clone().oneshot(discover()).await.unwrap().status(),
            StatusCode::TOO_MANY_REQUESTS
        );
        json_response(response).await;
        let response = router.clone().oneshot(discover()).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        drop(response);
        assert_eq!(
            router.oneshot(discover()).await.unwrap().status(),
            StatusCode::OK
        );
    }
}

#[tokio::test]
async fn idle_eviction_excludes_inflight_references_and_sse_response_lifetime() {
    let mut config = local_config();
    config.max_authorization_contexts = 1;
    config.authorization_idle_timeout = Duration::from_millis(1);
    let state = test_state(config);
    let owner = principal("owner");
    let transport = state.transport_for(&owner).await.unwrap();
    *transport.last_seen.lock().unwrap() = Instant::now() - Duration::from_secs(60);
    assert!(matches!(
        state.transport_for(&principal("other")).await,
        Err(HttpRequestError::AuthorizationCapacity)
    ));
    let lifetime = ResponseLifetime {
        transport: transport.clone(),
        _global_permit: state.concurrency.clone().try_acquire_owned().unwrap(),
        _authorization_permit: transport.concurrency.clone().try_acquire_owned().unwrap(),
    };
    let sse = Response::builder()
        .header("content-type", "text/event-stream")
        .body(Body::from_stream(
            stream::pending::<Result<String, IoError>>(),
        ))
        .unwrap();
    let response = retain_response_lifetime(sse, lifetime);
    drop(transport);
    assert!(matches!(
        state.transport_for(&principal("other")).await,
        Err(HttpRequestError::AuthorizationCapacity)
    ));
    drop(response);
    assert_eq!(
        state.concurrency.available_permits(),
        state.config.max_concurrent_requests
    );
    // Idle time starts at stream completion, not at the initial HTTP request.
    assert!(
        state.transports.lock().await[&owner.context]
            .last_seen
            .lock()
            .unwrap()
            .elapsed()
            < Duration::from_secs(1)
    );
    sleep(Duration::from_millis(5)).await;
    assert!(state.transport_for(&principal("other")).await.is_ok());
    assert!(!state.transports.lock().await.contains_key(&owner.context));
}

#[tokio::test]
async fn protected_router_advertises_metadata_and_authenticates_every_request() {
    let (state, token_a, _) = protected_state(local_config());
    let router = router_with_state(state);
    let metadata = router
        .clone()
        .oneshot(
            Request::get(RESOURCE_METADATA_PATH)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(metadata.status(), StatusCode::OK);
    assert_eq!(
        json_response(metadata).await["resource"],
        "http://127.0.0.1:8080/mcp"
    );
    let mut req = discover();
    req.headers_mut()
        .insert("authorization", HeaderValue::from_str(&token_a).unwrap());
    let response = router.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    json_response(response).await;
    let mut duplicated = discover();
    duplicated
        .headers_mut()
        .append("authorization", HeaderValue::from_str(&token_a).unwrap());
    duplicated
        .headers_mut()
        .append("authorization", HeaderValue::from_str(&token_a).unwrap());
    assert_eq!(
        router.clone().oneshot(duplicated).await.unwrap().status(),
        StatusCode::UNAUTHORIZED
    );
    let mut insufficient = discover();
    insufficient.headers_mut().insert(
        "authorization",
        HeaderValue::from_str(&token("subject-a", "profile")).unwrap(),
    );
    let response = router.clone().oneshot(insufficient).await.unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert!(
        response.headers()["www-authenticate"]
            .to_str()
            .unwrap()
            .contains("insufficient_scope")
    );
    for token in [None, Some("Bearer malformed")] {
        let mut req = discover();
        if let Some(token) = token {
            req.headers_mut()
                .insert("authorization", HeaderValue::from_static(token));
        }
        let response = router.clone().oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        let challenge = response.headers()["www-authenticate"].to_str().unwrap();
        assert!(challenge.contains("resource_metadata="));
        assert!(challenge.contains("scope=\"sora:mcp\""));
    }
}

#[tokio::test]
async fn actual_tasks_survive_http_response_and_idle_capacity_pressure_and_are_authorization_isolated()
 {
    let mut config = local_config();
    config.max_authorization_contexts = 2;
    config.authorization_idle_timeout = Duration::from_millis(1);
    let (state, token_a, token_b) = protected_state(config);
    let router = router_with_state(state.clone());
    let root = temp_dir().join(format!("sora-http-task-{}", Uuid::new_v4()));
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
    fs::write(root.join("schema/settings.toml"), "[tables.Settings]\nid = \"settings\"\nmode = \"singleton\"\n[tables.Settings.fields]\nname = \"string\"\n").unwrap();
    let session = state
        .workspace
        .open_project(
            ProjectId::new("demo").unwrap(),
            root.join("project.toml"),
            RuntimeOptions::default(),
        )
        .unwrap();
    let args = json!({"project_id": "demo", "expected_project_revision": session.revision().project, "view": null, "clean": true});
    let created = authorized_send(
        &router,
        &token_a,
        message(
            "tools/call",
            json!({"name": "sora_schema_lock", "arguments": args}),
            true,
        ),
    )
    .await;
    assert_eq!(created["result"]["resultType"], "task", "{created}");
    let task_id = created["result"]["taskId"].as_str().unwrap();
    let owner = state
        .authenticator
        .as_ref()
        .unwrap()
        .authenticate(Some(&token_a))
        .await
        .unwrap();
    let observer = state
        .authenticator
        .as_ref()
        .unwrap()
        .authenticate(Some(&token_b))
        .await
        .unwrap();
    let completed = timeout(Duration::from_secs(10), async {
        loop {
            let body = authorized_send(
                &router,
                &token_a,
                message("tasks/get", json!({"taskId": task_id}), true),
            )
            .await;
            if body["result"]["status"] == "completed" {
                break body;
            }
            assert!(body.get("error").is_none(), "{body}");
            sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    assert_eq!(
        completed["result"]["result"]["isError"], false,
        "{completed}"
    );
    assert!(root.join("generated/schema.lock").is_file());
    let denied = authorized_send(
        &router,
        &token_b,
        message("tasks/get", json!({"taskId": task_id}), true),
    )
    .await;
    assert!(denied.get("error").is_some(), "{denied}");
    {
        let transports = state.transports.lock().await;
        assert!(transports[&owner.context].server.has_retained_tasks());
        assert!(!transports[&observer.context].server.has_retained_tasks());
        for transport in transports.values() {
            *transport.last_seen.lock().unwrap() = Instant::now() - Duration::from_secs(60);
        }
    }
    drop(
        state
            .transport_for(&principal("capacity-pressure"))
            .await
            .unwrap(),
    );
    assert!(state.transports.lock().await.contains_key(&owner.context));
    assert!(
        !state
            .transports
            .lock()
            .await
            .contains_key(&observer.context)
    );
    let retained = authorized_send(
        &router,
        &token_a,
        message("tasks/get", json!({"taskId": task_id}), true),
    )
    .await;
    assert_eq!(retained["result"]["status"], "completed");
    // A different client without the extension still executes the heavy tool synchronously.
    let fallback = authorized_send(
        &router,
        &token_a,
        message(
            "tools/call",
            json!({"name": "sora_schema_lock", "arguments": args}),
            false,
        ),
    )
    .await;
    assert_eq!(fallback["result"]["resultType"], "complete", "{fallback}");
    assert_eq!(fallback["result"]["isError"], false, "{fallback}");
    state.cancellation.cancel();
    drop(session);
    fs::remove_dir_all(root).unwrap();
}

struct InputRetryHttpServer {
    client: Client,
    url: String,
    cancellation: CancellationToken,
    listener: JoinHandle<()>,
}

impl InputRetryHttpServer {
    async fn start(workspace: Arc<WorkspaceService>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let bind = listener.local_addr().unwrap();
        let url = format!("http://{bind}/mcp");
        let cancellation = CancellationToken::new();
        let router = build_http_router(
            workspace,
            HttpServerConfig::new(bind, Url::parse(&url).unwrap()),
            cancellation.clone(),
        )
        .await
        .unwrap();
        let shutdown = cancellation.clone();
        let listener = spawn(async move {
            serve(listener, router)
                .with_graceful_shutdown(shutdown.cancelled_owned())
                .await
                .unwrap();
        });
        Self {
            client: Client::builder()
                .no_proxy()
                .timeout(Duration::from_secs(5))
                .build()
                .unwrap(),
            url,
            cancellation,
            listener,
        }
    }

    async fn call(&self, body: &Value) -> Value {
        // A reverse request would switch this response to SSE and never finish:
        // every MRTR round must instead close with one ordinary JSON result.
        let response = self
            .client
            .post(&self.url)
            .header("accept", "application/json, text/event-stream")
            .header("mcp-protocol-version", VERSION)
            .header("mcp-method", "tools/call")
            .header("mcp-name", body["params"]["name"].as_str().unwrap())
            .json(body)
            .send()
            .await
            .expect("HTTP tool call must return within five seconds");
        assert_eq!(response.status(), StatusCode::OK);
        assert!(response.headers().get("mcp-session-id").is_none());
        assert_eq!(
            response.headers()["content-type"],
            "application/json",
            "sessionless client input must be returned as JSON InputRequired, never reverse-RPC SSE"
        );
        let response: Value = response
            .json()
            .await
            .expect("HTTP JSON body must terminate");
        assert_eq!(response["id"], body["id"]);
        assert!(response.get("error").is_none(), "{response}");
        assert!(
            matches!(
                response["result"]["resultType"].as_str(),
                Some("input_required" | "complete")
            ),
            "{response}"
        );
        response
    }
}

impl Drop for InputRetryHttpServer {
    fn drop(&mut self) {
        self.cancellation.cancel();
        self.listener.abort();
    }
}

struct InputRetryProject {
    root: PathBuf,
}

impl InputRetryProject {
    fn new(scripted: bool) -> Self {
        let root = temp_dir().join(format!("sora-http-input-{}", Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        let scripts = if scripted {
            "\n[parsers]\nscripts = [\"parser.lua\"]\n"
        } else {
            ""
        };
        fs::write(root.join("project.toml"), format!(
            "project = {{ id = \"demo\" }}\ngroups = {{ common = {{ default = true }} }}\nviews = {{ default = {{ contract = \"demo/default\", groups = [\"common\"] }} }}\nincludes = [\"schema.toml\"]\n{scripts}"
        )).unwrap();
        fs::write(
            root.join("schema.toml"),
            "[tables.Settings]\nid = \"settings\"\nmode = \"singleton\"\n",
        )
        .unwrap();
        if scripted {
            fs::write(root.join("parser.lua"), "return { parsers = { identity = { parse = function(cell) return cell.text end } } }\n").unwrap();
        }
        Self { root }
    }

    fn roots_response(&self) -> Value {
        json!({"resultType": "complete", "roots": [{
            "uri": Url::from_directory_path(&self.root).unwrap().as_str(),
            "name": "http-project"
        }]})
    }
}

impl Drop for InputRetryProject {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn input_retry_call(tool: &str, arguments: Value, capabilities: Value) -> Value {
    let mut call = message(
        "tools/call",
        json!({"name": tool, "arguments": arguments}),
        false,
    );
    call["params"]["_meta"][CAPS] = capabilities;
    call
}

fn required_input<'a>(response: &'a Value, method: &str) -> (&'a str, &'a Value) {
    // Deserialize the official SDK DTO as well as checking the raw HTTP wire.
    from_value::<InputRequiredResult>(response["result"].clone())
        .expect("official InputRequired result");
    let requests = response["result"]["inputRequests"]
        .as_object()
        .expect("input requests");
    assert_eq!(requests.len(), 1, "{response}");
    let (key, input) = requests.iter().next().unwrap();
    assert_eq!(input["method"], method, "{response}");
    (key, input)
}

fn retry_with_input(call: &Value, response: &Value, method: &str, answer: Value) -> Value {
    let (key, _) = required_input(response, method);
    let mut retry = call.clone();
    let params = retry["params"].as_object_mut().unwrap();
    params.insert("inputResponses".into(), json!({key: answer}));
    if let Some(state) = response["result"].get("requestState") {
        params.insert("requestState".into(), state.clone());
    } else {
        params.remove("requestState");
    }
    retry["id"] = json!(call["id"].as_u64().unwrap() + 1);
    retry
}

fn assert_input_retry_complete(response: &Value, is_error: bool) {
    assert_eq!(response["result"]["resultType"], "complete", "{response}");
    assert_eq!(response["result"]["isError"], is_error, "{response}");
}

#[tokio::test]
async fn http_roots_input_required_retry_completes_without_reverse_rpc() {
    let project = InputRetryProject::new(false);
    let workspace = Arc::new(WorkspaceService::new());
    let server = InputRetryHttpServer::start(workspace.clone()).await;
    let call = input_retry_call("sora_project_list", json!({}), json!({"roots": {}}));
    let pending = server.call(&call).await;
    required_input(&pending, "roots/list");
    assert!(workspace.discover_projects().unwrap().is_empty());
    let retry = retry_with_input(&call, &pending, "roots/list", project.roots_response());
    let completed = server.call(&retry).await;
    assert_input_retry_complete(&completed, false);
    let projects = completed["result"]["structuredContent"]["data"]["projects"]
        .as_array()
        .unwrap();
    assert_eq!(projects.len(), 1, "{completed}");
    assert_eq!(projects[0]["relative_manifest"], "project.toml");
    // Client-provided roots must not become shared filesystem authority.
    assert!(workspace.discover_projects().unwrap().is_empty());
}

#[tokio::test]
async fn http_roots_selection_and_script_trust_converge_through_three_input_retries() {
    let project = InputRetryProject::new(true);
    let workspace = Arc::new(WorkspaceService::new());
    let server = InputRetryHttpServer::start(workspace.clone()).await;
    let call = input_retry_call(
        "sora_project_open",
        json!({}),
        json!({"roots": {}, "elicitation": {"form": {}}}),
    );
    let roots = server.call(&call).await;
    required_input(&roots, "roots/list");
    let call = retry_with_input(&call, &roots, "roots/list", project.roots_response());
    let selection = server.call(&call).await;
    let (_, input) = required_input(&selection, "elicitation/create");
    assert!(
        input["params"]["requestedSchema"]["properties"]
            .get("root_id")
            .is_some(),
        "{input}"
    );
    assert!(workspace.project_ids().unwrap().is_empty());
    assert!(workspace.discover_projects().unwrap().is_empty());
    let prompt = input["params"]["message"].as_str().unwrap();
    assert!(prompt.contains("untrusted project metadata"), "{prompt}");
    let (_, choices) = prompt
        .split_once('\n')
        .expect("project choices in the input prompt");
    let candidates: Value = from_str(choices).expect("JSON project choices");
    let candidates = candidates.as_array().unwrap();
    assert_eq!(candidates.len(), 1);
    let call = retry_with_input(
        &call,
        &selection,
        "elicitation/create",
        json!({
            "resultType": "complete", "action": "accept", "content": {
                "root_id": candidates[0]["root_id"], "relative_manifest": "project.toml"
            }
        }),
    );
    let trust = server.call(&call).await;
    let (_, input) = required_input(&trust, "elicitation/create");
    let prompt = input["params"]["message"].as_str().unwrap();
    assert!(prompt.contains("SHA-256"), "{prompt}");
    assert!(prompt.contains("parser.lua"), "{prompt}");
    assert!(
        input["params"]["requestedSchema"]["properties"]
            .get("trust_project_scripts")
            .is_some(),
        "{input}"
    );
    assert!(workspace.project_ids().unwrap().is_empty());
    let call = retry_with_input(
        &call,
        &trust,
        "elicitation/create",
        json!({
            "resultType": "complete", "action": "accept", "content": {"trust_project_scripts": true}
        }),
    );
    let completed = server.call(&call).await;
    assert_input_retry_complete(&completed, false);
    assert_eq!(workspace.project_ids().unwrap().len(), 1);
}

#[tokio::test]
async fn http_missing_roots_or_form_capability_returns_json_without_requesting_input() {
    let project = InputRetryProject::new(true);
    let workspace = Arc::new(WorkspaceService::new());
    let server = InputRetryHttpServer::start(workspace.clone()).await;
    let result = server
        .call(&input_retry_call("sora_project_list", json!({}), json!({})))
        .await;
    assert_input_retry_complete(&result, false);
    assert!(workspace.discover_projects().unwrap().is_empty());
    let root = workspace.add_root("configured", &project.root).unwrap();
    for capabilities in [json!({}), json!({"elicitation": {"url": {}}})] {
        let missing_selection = server
            .call(&input_retry_call(
                "sora_project_open",
                json!({}),
                capabilities.clone(),
            ))
            .await;
        assert_input_retry_complete(&missing_selection, true);
        let untrusted = server
            .call(&input_retry_call(
                "sora_project_open",
                json!({
                    "root_id": root.id(), "relative_manifest": "project.toml"
                }),
                capabilities,
            ))
            .await;
        assert_input_retry_complete(&untrusted, true);
        assert!(workspace.project_ids().unwrap().is_empty());
    }
}

#[tokio::test]
async fn http_selection_and_script_trust_decline_cancel_and_false_decisions_terminate() {
    let project = InputRetryProject::new(true);
    let workspace = Arc::new(WorkspaceService::new());
    let root = workspace.add_root("configured", &project.root).unwrap();
    let server = InputRetryHttpServer::start(workspace.clone()).await;
    for arguments in [
        json!({}),
        json!({"root_id": root.id(), "relative_manifest": "project.toml"}),
    ] {
        for action in ["decline", "cancel"] {
            let call = input_retry_call(
                "sora_project_open",
                arguments.clone(),
                json!({"elicitation": {"form": {}}}),
            );
            let pending = server.call(&call).await;
            let retry = retry_with_input(
                &call,
                &pending,
                "elicitation/create",
                json!({"resultType": "complete", "action": action}),
            );
            let rejected = server.call(&retry).await;
            assert_input_retry_complete(&rejected, true);
            assert!(workspace.project_ids().unwrap().is_empty());
        }
    }
    let call = input_retry_call(
        "sora_project_open",
        json!({"root_id": root.id(), "relative_manifest": "project.toml"}),
        json!({"elicitation": {"form": {}}}),
    );
    let pending = server.call(&call).await;
    let retry = retry_with_input(
        &call,
        &pending,
        "elicitation/create",
        json!({
            "resultType": "complete", "action": "accept", "content": {"trust_project_scripts": false}
        }),
    );
    assert_input_retry_complete(&server.call(&retry).await, true);
    assert!(workspace.project_ids().unwrap().is_empty());
}

#[tokio::test]
async fn http_pending_input_survives_idle_authorization_capacity_pressure_until_resumed() {
    for roots_input in [true, false] {
        let project = InputRetryProject::new(true);
        let mut config = local_config();
        config.max_authorization_contexts = 1;
        config.authorization_idle_timeout = Duration::from_millis(1);
        let state = test_state(config);
        let (call, method, answer) = if roots_input {
            (
                input_retry_call("sora_project_list", json!({}), json!({"roots": {}})),
                "roots/list",
                project.roots_response(),
            )
        } else {
            let root = state
                .workspace
                .add_root("configured", &project.root)
                .unwrap();
            (
                input_retry_call(
                    "sora_project_open",
                    json!({
                        "root_id": root.id(), "relative_manifest": "project.toml"
                    }),
                    json!({"elicitation": {"form": {}}}),
                ),
                "elicitation/create",
                json!({"resultType": "complete", "action": "accept", "content": {"trust_project_scripts": true}}),
            )
        };
        let router = router_with_state(state.clone());
        let response = timeout(
            Duration::from_secs(5),
            router.clone().oneshot(request(call.clone())),
        )
        .await
        .expect("pending input must not wait for a reverse reply")
        .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers()["content-type"], "application/json");
        assert!(response.headers().get("mcp-session-id").is_none());
        let pending = timeout(Duration::from_secs(5), json_response(response))
            .await
            .expect("InputRequired response body must terminate");
        required_input(&pending, method);
        let owner = principal("local-http");
        let contender = principal("capacity-pressure");
        {
            let transports = state.transports.lock().await;
            let transport = &transports[&owner.context];
            assert_eq!(
                Arc::strong_count(transport),
                1,
                "no in-flight body should protect this context"
            );
            assert_eq!(
                transport.concurrency.available_permits(),
                state.config.max_concurrent_requests_per_authorization
            );
            assert!(!transport.server.has_retained_tasks());
            assert!(transport.server.has_pending_inputs());
            *transport.last_seen.lock().unwrap() = Instant::now() - Duration::from_secs(60);
        }
        assert!(
            matches!(
                state.transport_for(&contender).await,
                Err(HttpRequestError::AuthorizationCapacity)
            ),
            "idle pending input must prevent eviction even after its HTTP response has ended"
        );
        assert!(state.transports.lock().await.contains_key(&owner.context));
        let retry = retry_with_input(&call, &pending, method, answer);
        let completed = timeout(Duration::from_secs(5), send(&router, retry))
            .await
            .expect("the retained input must still resume to completion");
        assert_input_retry_complete(&completed, false);
        {
            let transports = state.transports.lock().await;
            let transport = &transports[&owner.context];
            assert_eq!(Arc::strong_count(transport), 1);
            assert!(!transport.server.has_pending_inputs());
            assert!(!transport.server.has_retained_tasks());
            *transport.last_seen.lock().unwrap() = Instant::now() - Duration::from_secs(60);
        }
        drop(
            state
                .transport_for(&contender)
                .await
                .expect("completed input must no longer pin the idle context"),
        );
        assert!(!state.transports.lock().await.contains_key(&owner.context));
        state.cancellation.cancel();
    }
}

async fn authorized_send(router: &Router, token: &str, body: Value) -> Value {
    let mut req = request(body);
    req.headers_mut()
        .insert("authorization", HeaderValue::from_str(token).unwrap());
    let response = router.clone().oneshot(req).await.unwrap();
    let status = response.status();
    assert!(response.headers().get("mcp-session-id").is_none());
    let body = json_response(response).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body
}

fn protected_state(mut config: HttpServerConfig) -> (Arc<HttpState>, String, String) {
    let oauth = OAuthResourceServerConfig::new(
        Url::parse("https://id.example.com").unwrap(),
        config.public_url.as_str(),
    );
    config.oauth = Some(oauth.clone());
    let mut state = test_state(config);
    Arc::get_mut(&mut state).unwrap().authenticator = Some(Arc::new(
        OAuthAuthenticator::for_testing(oauth, from_str(JWK).unwrap()),
    ));
    (
        state,
        token("subject-a", "sora:mcp"),
        token("subject-b", "sora:mcp"),
    )
}

fn token(subject: &str, scope: &str) -> String {
    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some("test-key".into());
    let key = STANDARD.decode(PRIVATE_KEY_DER).unwrap();
    let claims = json!({"iss": "https://id.example.com/", "sub": subject, "aud": "http://127.0.0.1:8080/mcp", "exp": get_current_timestamp() + 600, "scope": scope});
    format!(
        "Bearer {}",
        encode(&header, &claims, &EncodingKey::from_rsa_der(&key)).unwrap()
    )
}

// Public test-only RSA fixture, shared with the OAuth validation contract.
const PRIVATE_KEY_DER: &str = "MIIEowIBAAKCAQEAitVCD5i5Vave9S70cxHwgwv//uZFQctIYJ13PWF+/GEiF/UEylqTzDud110VhXWlaFtmHKs9w8F43YAEbfDTtgeKBDt20PLphCcojNweQrVgAMkyBkQ+tN2+FcIc5meVBr1I4nUQz64LS1UCq9Jkd5MHWbCztSEVx56xcOEYIcblxqWoYktI8+JhfGJ0wftW5D3ONVX8+UxqKQN/QJ/V8Wlx4jLgzEK3L8fuLPZtZkdNAMkztvfTUJGgH7uOvk/aRvFUzsZmGLKZp6DFXJ/prEorsT9MtKFw2G0JDX8FV+sqnQ5Xx3QQKZxE1i/KS5KhA1AWNCEMIE+srn/F2R3tZwIDAQABAoIBADPLcAPllG9T2bBbPjOX48TgOzdbrUboki4JUS0DLJ73lgKeFJVxyWYOb8qjHZ5Wz8jpIGVdtYNMsMziL8Tcbf+hPGprzN4TYe89tLEohdmZVSG17gXMfCB6AaqYarPCvSRwhbqGtijk463NH1v++CBsJ/fcJo9Sso9wsv5L2/btcqccY7yb0RvnrPyeYjws/cEEwTmuLnApUn8IVKvIwMYiy17T7ThyPPYrJVWIskpx4kdhr9/jXv1fiFVjBIUspvTfAWbkIRZjmr1L9oQ4pnnsCA30GCs1KOQ6104Ra1XGx8HkdQ6uoVXv0BiYy0+P9AVOP7VqdkLtGemlh37WAlECgYEAv/KLaUrwtwdMm6CSaKiTpKGVZqfUebFvB3nSSA9wGyrUI9d9XaDNBQaAmj40st9Shjah2DME1naIMbbcA8WsX1bzBh1DmmZ+lDKYfSi6zqfsCftyQ7jhAUNOCwkZ2FRubNQnpllsywDbkYZBwJA65aR6f2T7X0A4B9EplFbNVtsCgYEAuSlR+1ESLlVHwmvZcNFwo4SdnhPo1T0wArWcxSXdEiGIfotgQyZwjPy8lHxWIx0u0QmQAYRKyGiNwjRnEm9BXyd3N1MdfB20mzcv5176ip/+XsCgLciUZYQ0Kw0T+VyBRPiJ+orzZbHxs4nM8+MN0MzwDCoR48RvtjdzSfwhy2UCgYA/Ys8a5D59kpF/yxTOLu0irqFxlvaZgTiTFW2VklOJBSms9FpX8uQBRtZtGSG59/l+jMgBZ0evstDi6enZ7QsxKLt0R3GtqS8frV2h5zNZTyapXTmsl37mNd2t00GPqMIWejDlxB4jI5NeiSFpf7eTYi95plVEbSaTnkTG0atZhQKBgAva3Wjd0/ArfaCxDwbuIewWPZE1bR7aAAzsW9Ezj2ftxbN0cQFboLEmLVzEaS9uWuT/W7z9H1yddRYODNtdTC4vdK6SH70mmb2mVubVae0eOUcPxjGoOfgV8tEe0TyR86Ta1Nq5ssO8FOGN3RVwYB4SQPxGZUbT+OMNwc/m2LMRAoGBAK2h92s0cqvxGfExltttTPab0E60n/KHukJfSbXajDL2lX3l33TQ+TFfpI2corrzByXQYZbZVRBGbZ46XEnPYSnhOLXpIVM6RCLTfHUVnwSvd73N0fiqHbVyMXyXd575QXCbVKRML0/i3mUUasMUSrUC6DZFaYOMTZlTTwPSIqvd";
const JWK: &str = r#"{"keys":[{"kty":"RSA","use":"sig","alg":"RS256","kid":"test-key","n":"itVCD5i5Vave9S70cxHwgwv__uZFQctIYJ13PWF-_GEiF_UEylqTzDud110VhXWlaFtmHKs9w8F43YAEbfDTtgeKBDt20PLphCcojNweQrVgAMkyBkQ-tN2-FcIc5meVBr1I4nUQz64LS1UCq9Jkd5MHWbCztSEVx56xcOEYIcblxqWoYktI8-JhfGJ0wftW5D3ONVX8-UxqKQN_QJ_V8Wlx4jLgzEK3L8fuLPZtZkdNAMkztvfTUJGgH7uOvk_aRvFUzsZmGLKZp6DFXJ_prEorsT9MtKFw2G0JDX8FV-sqnQ5Xx3QQKZxE1i_KS5KhA1AWNCEMIE-srn_F2R3tZw","e":"AQAB"}]}"#;
