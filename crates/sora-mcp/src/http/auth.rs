use std::{
    collections::BTreeSet,
    sync::Arc,
    time::{Duration, Instant},
};

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use jsonwebtoken::{
    Algorithm, DecodingKey, Validation, decode, decode_header,
    jwk::{Jwk, JwkSet, PublicKeyUse},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;
use tokio::sync::{Mutex, RwLock};
use url::Url;

const SUPPORTED_ALGORITHMS: [Algorithm; 9] = [
    Algorithm::RS256,
    Algorithm::RS384,
    Algorithm::RS512,
    Algorithm::PS256,
    Algorithm::PS384,
    Algorithm::PS512,
    Algorithm::ES256,
    Algorithm::ES384,
    Algorithm::EdDSA,
];

/// OAuth 2.1 resource-server settings for the Streamable HTTP transport.
#[derive(Debug, Clone)]
pub struct OAuthResourceServerConfig {
    /// Authorization server issuer identifier.
    pub issuer: Url,
    /// Audience required in JWT access tokens.
    pub audience: String,
    /// Scopes required for every MCP request.
    pub required_scopes: BTreeSet<String>,
    /// Explicit JWKS endpoint. When omitted, RFC 8414 discovery is used.
    pub jwks_uri: Option<Url>,
    /// Maximum age of a cached JWKS document.
    pub jwks_cache_ttl: Duration,
}

impl OAuthResourceServerConfig {
    /// Creates resource-server settings with the `sora:mcp` required scope.
    pub fn new(issuer: Url, audience: impl Into<String>) -> Self {
        Self {
            issuer,
            audience: audience.into(),
            required_scopes: BTreeSet::from(["sora:mcp".to_owned()]),
            jwks_uri: None,
            jwks_cache_ttl: Duration::from_secs(300),
        }
    }
}

/// RFC 9728 metadata advertised by the protected Sora MCP resource.
#[derive(Debug, Clone, Serialize)]
pub struct ProtectedResourceMetadata {
    pub resource: String,
    pub authorization_servers: Vec<String>,
    pub scopes_supported: Vec<String>,
    pub bearer_methods_supported: Vec<String>,
}

/// Validated authorization identity used to isolate Sora state.
#[derive(Debug, Clone)]
pub struct AuthorizationPrincipal {
    /// Stable, non-secret fingerprint of issuer and subject.
    pub context: Arc<str>,
    /// Subject claim from the validated access token.
    pub subject: Arc<str>,
    /// Access-token expiry as seconds since the Unix epoch.
    pub expires_at: u64,
}

/// OAuth configuration and validation failures.
#[derive(Debug, Error)]
pub enum OAuthError {
    #[error("authorization header is missing")]
    MissingToken,
    #[error("authorization header is not a bearer token")]
    MalformedToken,
    #[error("access token is invalid")]
    InvalidToken,
    #[error("access token does not grant all required scopes")]
    InsufficientScope,
    #[error("authorization server metadata is invalid: {0}")]
    InvalidServerMetadata(String),
    #[error("authorization server discovery failed: {0}")]
    Discovery(String),
    #[error("authorization server JWKS request failed: {0}")]
    Jwks(String),
}

#[derive(Debug, Deserialize)]
struct AuthorizationServerMetadata {
    issuer: String,
    jwks_uri: String,
}

#[derive(Debug, Deserialize)]
struct AccessTokenClaims {
    sub: String,
    exp: u64,
    #[serde(default)]
    scope: String,
}

#[derive(Debug)]
struct CachedJwks {
    document: JwkSet,
    loaded_at: Instant,
}

/// Validates RFC 9068-style JWT access tokens using an authorization server's
/// rotating JWKS.
pub struct OAuthAuthenticator {
    config: OAuthResourceServerConfig,
    client: reqwest::Client,
    jwks_uri: Url,
    jwks: RwLock<CachedJwks>,
    refresh_lock: Mutex<()>,
}

impl std::fmt::Debug for OAuthAuthenticator {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("OAuthAuthenticator")
            .field("issuer", &self.config.issuer)
            .field("audience", &self.config.audience)
            .field("jwks_uri", &self.jwks_uri)
            .finish_non_exhaustive()
    }
}

impl OAuthAuthenticator {
    /// Discovers the authorization server when needed and loads its initial
    /// JWKS. Network failures are reported during startup, not on first use.
    pub async fn initialize(config: OAuthResourceServerConfig) -> Result<Self, OAuthError> {
        validate_oauth_config(&config)?;
        let client = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(10))
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|error| OAuthError::Discovery(error.to_string()))?;
        let jwks_uri = match config.jwks_uri.clone() {
            Some(uri) => uri,
            None => discover_jwks_uri(&client, &config.issuer).await?,
        };
        ensure_https_or_loopback(&jwks_uri, "JWKS URI")?;
        let document = fetch_jwks(&client, &jwks_uri).await?;
        Ok(Self {
            config,
            client,
            jwks_uri,
            jwks: RwLock::new(CachedJwks {
                document,
                loaded_at: Instant::now(),
            }),
            refresh_lock: Mutex::new(()),
        })
    }

    #[cfg(test)]
    pub(super) fn for_testing(config: OAuthResourceServerConfig, document: JwkSet) -> Self {
        let jwks_uri = config
            .jwks_uri
            .clone()
            .unwrap_or_else(|| config.issuer.clone());
        Self {
            config,
            client: reqwest::Client::new(),
            jwks_uri,
            jwks: RwLock::new(CachedJwks {
                document,
                loaded_at: Instant::now(),
            }),
            refresh_lock: Mutex::new(()),
        }
    }

    /// Returns the RFC 9728 metadata document for this resource server.
    pub fn protected_resource_metadata(&self, resource: &Url) -> ProtectedResourceMetadata {
        ProtectedResourceMetadata {
            resource: resource.as_str().to_owned(),
            authorization_servers: vec![self.config.issuer.as_str().to_owned()],
            scopes_supported: self.config.required_scopes.iter().cloned().collect(),
            bearer_methods_supported: vec!["header".to_owned()],
        }
    }

    /// Returns the required OAuth scopes in deterministic order.
    pub fn required_scopes(&self) -> impl Iterator<Item = &str> {
        self.config.required_scopes.iter().map(String::as_str)
    }

    /// Validates an HTTP Authorization header without retaining the token.
    pub async fn authenticate(
        &self,
        authorization_header: Option<&str>,
    ) -> Result<AuthorizationPrincipal, OAuthError> {
        let token = bearer_token(authorization_header)?;
        let header = decode_header(token).map_err(|_| OAuthError::InvalidToken)?;
        if !SUPPORTED_ALGORITHMS.contains(&header.alg)
            || matches!(
                header.alg,
                Algorithm::HS256 | Algorithm::HS384 | Algorithm::HS512
            )
        {
            return Err(OAuthError::InvalidToken);
        }
        let kid = header.kid.as_deref().ok_or(OAuthError::InvalidToken)?;
        let key = self.decoding_key(kid).await?;

        let mut validation = Validation::new(header.alg);
        validation.set_required_spec_claims(&["exp", "sub", "iss", "aud"]);
        validation.set_issuer(&[self.config.issuer.as_str()]);
        validation.set_audience(&[self.config.audience.as_str()]);
        validation.validate_nbf = true;
        let token = decode::<AccessTokenClaims>(token, &key, &validation)
            .map_err(|_| OAuthError::InvalidToken)?;
        let granted_scopes = token
            .claims
            .scope
            .split_ascii_whitespace()
            .collect::<BTreeSet<_>>();
        if !self
            .config
            .required_scopes
            .iter()
            .all(|scope| granted_scopes.contains(scope.as_str()))
        {
            return Err(OAuthError::InsufficientScope);
        }

        let mut fingerprint = Sha256::new();
        fingerprint.update(self.config.issuer.as_str().as_bytes());
        fingerprint.update([0]);
        fingerprint.update(token.claims.sub.as_bytes());
        let context = format!("oauth:{}", URL_SAFE_NO_PAD.encode(fingerprint.finalize()));
        Ok(AuthorizationPrincipal {
            context: Arc::from(context),
            subject: Arc::from(token.claims.sub),
            expires_at: token.claims.exp,
        })
    }

    async fn decoding_key(&self, kid: &str) -> Result<DecodingKey, OAuthError> {
        let should_refresh = {
            let jwks = self.jwks.read().await;
            let missing_key = find_signing_key(&jwks.document, kid).is_none();
            jwks.loaded_at.elapsed() >= self.config.jwks_cache_ttl
                || (missing_key && jwks.loaded_at.elapsed() >= Duration::from_secs(5))
        };
        if should_refresh {
            self.refresh_jwks().await?;
        }
        let jwks = self.jwks.read().await;
        let jwk = find_signing_key(&jwks.document, kid).ok_or(OAuthError::InvalidToken)?;
        DecodingKey::from_jwk(jwk).map_err(|_| OAuthError::InvalidToken)
    }

    async fn refresh_jwks(&self) -> Result<(), OAuthError> {
        let _guard = self.refresh_lock.lock().await;
        {
            let jwks = self.jwks.read().await;
            if jwks.loaded_at.elapsed() < Duration::from_secs(5) {
                return Ok(());
            }
        }
        let document = fetch_jwks(&self.client, &self.jwks_uri).await?;
        *self.jwks.write().await = CachedJwks {
            document,
            loaded_at: Instant::now(),
        };
        Ok(())
    }
}

fn find_signing_key<'a>(jwks: &'a JwkSet, kid: &str) -> Option<&'a Jwk> {
    jwks.keys.iter().find(|key| {
        key.common.key_id.as_deref() == Some(kid)
            && key.common.public_key_use.as_ref() != Some(&PublicKeyUse::Encryption)
    })
}

fn bearer_token(header: Option<&str>) -> Result<&str, OAuthError> {
    let header = header.ok_or(OAuthError::MissingToken)?;
    let (scheme, token) = header.split_once(' ').ok_or(OAuthError::MalformedToken)?;
    if !scheme.eq_ignore_ascii_case("bearer")
        || token.is_empty()
        || token.chars().any(char::is_whitespace)
    {
        return Err(OAuthError::MalformedToken);
    }
    Ok(token)
}

async fn discover_jwks_uri(client: &reqwest::Client, issuer: &Url) -> Result<Url, OAuthError> {
    let discovery_url = authorization_server_metadata_url(issuer)?;
    let metadata = client
        .get(discovery_url)
        .send()
        .await
        .and_then(reqwest::Response::error_for_status)
        .map_err(|error| OAuthError::Discovery(error.to_string()))?
        .json::<AuthorizationServerMetadata>()
        .await
        .map_err(|error| OAuthError::Discovery(error.to_string()))?;
    let discovered_issuer = Url::parse(&metadata.issuer)
        .map_err(|error| OAuthError::InvalidServerMetadata(error.to_string()))?;
    if &discovered_issuer != issuer {
        return Err(OAuthError::InvalidServerMetadata(
            "discovered issuer does not exactly match configured issuer".to_owned(),
        ));
    }
    let jwks_uri = Url::parse(&metadata.jwks_uri)
        .map_err(|error| OAuthError::InvalidServerMetadata(error.to_string()))?;
    Ok(jwks_uri)
}

fn authorization_server_metadata_url(issuer: &Url) -> Result<Url, OAuthError> {
    let mut discovery = issuer.clone();
    let issuer_path = issuer.path().trim_start_matches('/');
    let path = if issuer_path.is_empty() {
        "/.well-known/oauth-authorization-server".to_owned()
    } else {
        format!("/.well-known/oauth-authorization-server/{issuer_path}")
    };
    discovery.set_path(&path);
    discovery.set_query(None);
    discovery.set_fragment(None);
    Ok(discovery)
}

async fn fetch_jwks(client: &reqwest::Client, uri: &Url) -> Result<JwkSet, OAuthError> {
    let jwks = client
        .get(uri.clone())
        .send()
        .await
        .and_then(reqwest::Response::error_for_status)
        .map_err(|error| OAuthError::Jwks(error.to_string()))?
        .json::<JwkSet>()
        .await
        .map_err(|error| OAuthError::Jwks(error.to_string()))?;
    if jwks.keys.is_empty() {
        return Err(OAuthError::InvalidServerMetadata(
            "JWKS document contains no keys".to_owned(),
        ));
    }
    Ok(jwks)
}

fn validate_oauth_config(config: &OAuthResourceServerConfig) -> Result<(), OAuthError> {
    ensure_https_or_loopback(&config.issuer, "issuer")?;
    if config.audience.trim().is_empty() {
        return Err(OAuthError::InvalidServerMetadata(
            "audience must not be empty".to_owned(),
        ));
    }
    if config.required_scopes.is_empty()
        || config
            .required_scopes
            .iter()
            .any(|scope| scope.is_empty() || scope.chars().any(char::is_whitespace))
    {
        return Err(OAuthError::InvalidServerMetadata(
            "required scopes must be non-empty OAuth scope tokens".to_owned(),
        ));
    }
    Ok(())
}

pub(super) fn ensure_https_or_loopback(url: &Url, label: &str) -> Result<(), OAuthError> {
    if url.scheme() == "https" {
        return Ok(());
    }
    let is_loopback = url
        .host_str()
        .and_then(|host| host.parse::<std::net::IpAddr>().ok())
        .is_some_and(|address| address.is_loopback())
        || matches!(url.host_str(), Some("localhost"));
    if url.scheme() == "http" && is_loopback {
        return Ok(());
    }
    Err(OAuthError::InvalidServerMetadata(format!(
        "{label} must use HTTPS unless it targets loopback"
    )))
}

#[cfg(test)]
mod tests {
    use std::{
        collections::BTreeSet,
        time::{Duration, Instant},
    };

    use base64::{Engine as _, engine::general_purpose::STANDARD};
    use jsonwebtoken::{
        Algorithm, EncodingKey, Header, encode, get_current_timestamp, jwk::JwkSet,
    };
    use serde::Serialize;
    use tokio::sync::{Mutex, RwLock};

    use super::{
        CachedJwks, OAuthAuthenticator, OAuthError, OAuthResourceServerConfig,
        authorization_server_metadata_url, bearer_token,
    };

    const PRIVATE_KEY_DER: &str = "MIIEowIBAAKCAQEAitVCD5i5Vave9S70cxHwgwv//uZFQctIYJ13PWF+/GEiF/UEylqTzDud110VhXWlaFtmHKs9w8F43YAEbfDTtgeKBDt20PLphCcojNweQrVgAMkyBkQ+tN2+FcIc5meVBr1I4nUQz64LS1UCq9Jkd5MHWbCztSEVx56xcOEYIcblxqWoYktI8+JhfGJ0wftW5D3ONVX8+UxqKQN/QJ/V8Wlx4jLgzEK3L8fuLPZtZkdNAMkztvfTUJGgH7uOvk/aRvFUzsZmGLKZp6DFXJ/prEorsT9MtKFw2G0JDX8FV+sqnQ5Xx3QQKZxE1i/KS5KhA1AWNCEMIE+srn/F2R3tZwIDAQABAoIBADPLcAPllG9T2bBbPjOX48TgOzdbrUboki4JUS0DLJ73lgKeFJVxyWYOb8qjHZ5Wz8jpIGVdtYNMsMziL8Tcbf+hPGprzN4TYe89tLEohdmZVSG17gXMfCB6AaqYarPCvSRwhbqGtijk463NH1v++CBsJ/fcJo9Sso9wsv5L2/btcqccY7yb0RvnrPyeYjws/cEEwTmuLnApUn8IVKvIwMYiy17T7ThyPPYrJVWIskpx4kdhr9/jXv1fiFVjBIUspvTfAWbkIRZjmr1L9oQ4pnnsCA30GCs1KOQ6104Ra1XGx8HkdQ6uoVXv0BiYy0+P9AVOP7VqdkLtGemlh37WAlECgYEAv/KLaUrwtwdMm6CSaKiTpKGVZqfUebFvB3nSSA9wGyrUI9d9XaDNBQaAmj40st9Shjah2DME1naIMbbcA8WsX1bzBh1DmmZ+lDKYfSi6zqfsCftyQ7jhAUNOCwkZ2FRubNQnpllsywDbkYZBwJA65aR6f2T7X0A4B9EplFbNVtsCgYEAuSlR+1ESLlVHwmvZcNFwo4SdnhPo1T0wArWcxSXdEiGIfotgQyZwjPy8lHxWIx0u0QmQAYRKyGiNwjRnEm9BXyd3N1MdfB20mzcv5176ip/+XsCgLciUZYQ0Kw0T+VyBRPiJ+orzZbHxs4nM8+MN0MzwDCoR48RvtjdzSfwhy2UCgYA/Ys8a5D59kpF/yxTOLu0irqFxlvaZgTiTFW2VklOJBSms9FpX8uQBRtZtGSG59/l+jMgBZ0evstDi6enZ7QsxKLt0R3GtqS8frV2h5zNZTyapXTmsl37mNd2t00GPqMIWejDlxB4jI5NeiSFpf7eTYi95plVEbSaTnkTG0atZhQKBgAva3Wjd0/ArfaCxDwbuIewWPZE1bR7aAAzsW9Ezj2ftxbN0cQFboLEmLVzEaS9uWuT/W7z9H1yddRYODNtdTC4vdK6SH70mmb2mVubVae0eOUcPxjGoOfgV8tEe0TyR86Ta1Nq5ssO8FOGN3RVwYB4SQPxGZUbT+OMNwc/m2LMRAoGBAK2h92s0cqvxGfExltttTPab0E60n/KHukJfSbXajDL2lX3l33TQ+TFfpI2corrzByXQYZbZVRBGbZ46XEnPYSnhOLXpIVM6RCLTfHUVnwSvd73N0fiqHbVyMXyXd575QXCbVKRML0/i3mUUasMUSrUC6DZFaYOMTZlTTwPSIqvd";
    const JWK: &str = r#"{"keys":[{"kty":"RSA","use":"sig","alg":"RS256","kid":"test-key","n":"itVCD5i5Vave9S70cxHwgwv__uZFQctIYJ13PWF-_GEiF_UEylqTzDud110VhXWlaFtmHKs9w8F43YAEbfDTtgeKBDt20PLphCcojNweQrVgAMkyBkQ-tN2-FcIc5meVBr1I4nUQz64LS1UCq9Jkd5MHWbCztSEVx56xcOEYIcblxqWoYktI8-JhfGJ0wftW5D3ONVX8-UxqKQN_QJ_V8Wlx4jLgzEK3L8fuLPZtZkdNAMkztvfTUJGgH7uOvk_aRvFUzsZmGLKZp6DFXJ_prEorsT9MtKFw2G0JDX8FV-sqnQ5Xx3QQKZxE1i_KS5KhA1AWNCEMIE-srn_F2R3tZw","e":"AQAB"}]}"#;

    #[derive(Serialize)]
    struct TestClaims<'a> {
        iss: &'a str,
        sub: &'a str,
        aud: &'a str,
        exp: u64,
        scope: &'a str,
    }

    #[test]
    fn bearer_scheme_is_case_insensitive_but_token_is_single_value() {
        assert_eq!(bearer_token(Some("bEaReR abc")).expect("valid"), "abc");
        assert!(matches!(
            bearer_token(Some("Bearer abc def")),
            Err(OAuthError::MalformedToken)
        ));
    }

    #[test]
    fn discovery_url_preserves_issuer_path_using_rfc_8414_layout() {
        let issuer = url::Url::parse("https://id.example.com/tenant").expect("valid URL");
        assert_eq!(
            authorization_server_metadata_url(&issuer)
                .expect("valid discovery URL")
                .as_str(),
            "https://id.example.com/.well-known/oauth-authorization-server/tenant"
        );
    }

    #[tokio::test]
    async fn validates_signature_standard_claims_and_scope() {
        let authenticator = test_authenticator();
        let token = test_token("sora:mcp", "https://sora.example.com/mcp");
        let principal = authenticator
            .authenticate(Some(&format!("Bearer {token}")))
            .await
            .expect("valid access token");
        assert_eq!(principal.subject.as_ref(), "user-42");
        assert!(principal.context.starts_with("oauth:"));

        let wrong_scope = test_token("profile", "https://sora.example.com/mcp");
        assert!(matches!(
            authenticator
                .authenticate(Some(&format!("Bearer {wrong_scope}")))
                .await,
            Err(OAuthError::InsufficientScope)
        ));

        let wrong_audience = test_token("sora:mcp", "https://other.example.com");
        assert!(matches!(
            authenticator
                .authenticate(Some(&format!("Bearer {wrong_audience}")))
                .await,
            Err(OAuthError::InvalidToken)
        ));
    }

    fn test_authenticator() -> OAuthAuthenticator {
        let issuer = url::Url::parse("https://id.example.com").expect("valid issuer");
        let config = OAuthResourceServerConfig {
            issuer: issuer.clone(),
            audience: "https://sora.example.com/mcp".to_owned(),
            required_scopes: BTreeSet::from(["sora:mcp".to_owned()]),
            jwks_uri: Some(issuer.clone()),
            jwks_cache_ttl: Duration::from_secs(300),
        };
        OAuthAuthenticator {
            config,
            client: reqwest::Client::new(),
            jwks_uri: issuer,
            jwks: RwLock::new(CachedJwks {
                document: serde_json::from_str::<JwkSet>(JWK).expect("valid JWKS"),
                loaded_at: Instant::now(),
            }),
            refresh_lock: Mutex::new(()),
        }
    }

    fn test_token(scope: &str, audience: &str) -> String {
        let key = STANDARD
            .decode(PRIVATE_KEY_DER)
            .expect("valid private key encoding");
        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some("test-key".to_owned());
        encode(
            &header,
            &TestClaims {
                iss: "https://id.example.com/",
                sub: "user-42",
                aud: audience,
                exp: get_current_timestamp() + 600,
                scope,
            },
            &EncodingKey::from_rsa_der(&key),
        )
        .expect("signed token")
    }
}
