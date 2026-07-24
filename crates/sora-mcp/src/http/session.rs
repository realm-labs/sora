use std::{
    str,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use futures_util::StreamExt;
use hmac::{Hmac, Mac};
use rmcp::{
    model::{ClientJsonRpcMessage, ServerJsonRpcMessage},
    transport::streamable_http_server::{
        RestoreOutcome, SessionId, SessionManager,
        session::ServerSseMessage,
        session::local::{
            LocalSessionManager, LocalSessionManagerError, SessionConfig, SessionTransport,
        },
    },
};
use sha2::Sha256;
use thiserror::Error;
use tokio::sync::Mutex;

type HmacSha256 = Hmac<Sha256>;

/// Configuration for one authorization-bound HTTP session manager.
#[derive(Debug, Clone)]
pub struct SecureSessionConfig {
    /// Maximum number of live MCP sessions for one authorization context.
    pub max_sessions: usize,
    /// How long an inactive session remains live.
    pub idle_timeout: Duration,
    /// How long a completed SSE event remains resumable.
    pub event_ttl: Duration,
    /// Number of pending events buffered by each session channel.
    pub channel_capacity: usize,
    /// Maximum time allowed for the initialize handshake.
    pub initialize_timeout: Duration,
}

impl Default for SecureSessionConfig {
    fn default() -> Self {
        Self {
            max_sessions: 16,
            idle_timeout: Duration::from_secs(300),
            event_ttl: Duration::from_secs(60),
            channel_capacity: 32,
            initialize_timeout: Duration::from_secs(60),
        }
    }
}

/// Errors produced while managing authorization-bound HTTP sessions.
#[derive(Debug, Error)]
pub enum SecureSessionError {
    #[error("authorization context has reached its MCP session limit")]
    SessionLimit,
    #[error("invalid or expired resumable event cursor")]
    InvalidEventCursor,
    #[error("failed to initialize the event cursor signer")]
    CursorSigner,
    #[error(transparent)]
    Local(#[from] LocalSessionManagerError),
}

#[derive(Clone)]
struct EventCursorCodec {
    signer: HmacSha256,
    authorization_context: Arc<str>,
    ttl: Duration,
}

impl EventCursorCodec {
    fn new(
        key: &[u8],
        authorization_context: Arc<str>,
        ttl: Duration,
    ) -> Result<Self, SecureSessionError> {
        let signer =
            HmacSha256::new_from_slice(key).map_err(|_| SecureSessionError::CursorSigner)?;
        Ok(Self {
            signer,
            authorization_context,
            ttl,
        })
    }

    fn encode(&self, session_id: &SessionId, raw_event_id: &str) -> String {
        let expires_at = unix_seconds().saturating_add(self.ttl.as_secs());
        let authorization_context = URL_SAFE_NO_PAD.encode(self.authorization_context.as_bytes());
        let session_id = URL_SAFE_NO_PAD.encode(session_id.as_bytes());
        let raw_event_id = URL_SAFE_NO_PAD.encode(raw_event_id.as_bytes());
        let unsigned =
            format!("v1.{expires_at}.{authorization_context}.{session_id}.{raw_event_id}");
        let mut signer = self.signer.clone();
        signer.update(unsigned.as_bytes());
        let signature = URL_SAFE_NO_PAD.encode(signer.finalize().into_bytes());
        format!("{unsigned}.{signature}")
    }

    fn decode(
        &self,
        expected_session_id: &SessionId,
        cursor: &str,
    ) -> Result<String, SecureSessionError> {
        let mut components = cursor.rsplitn(2, '.');
        let signature = components
            .next()
            .ok_or(SecureSessionError::InvalidEventCursor)?;
        let unsigned = components
            .next()
            .ok_or(SecureSessionError::InvalidEventCursor)?;
        let signature = URL_SAFE_NO_PAD
            .decode(signature)
            .map_err(|_| SecureSessionError::InvalidEventCursor)?;
        let mut signer = self.signer.clone();
        signer.update(unsigned.as_bytes());
        signer
            .verify_slice(&signature)
            .map_err(|_| SecureSessionError::InvalidEventCursor)?;

        let mut fields = unsigned.split('.');
        let version = fields.next();
        let expires_at = fields
            .next()
            .and_then(|value| value.parse::<u64>().ok())
            .ok_or(SecureSessionError::InvalidEventCursor)?;
        let authorization_context = decode_text(
            fields
                .next()
                .ok_or(SecureSessionError::InvalidEventCursor)?,
        )?;
        let session_id = decode_text(
            fields
                .next()
                .ok_or(SecureSessionError::InvalidEventCursor)?,
        )?;
        let raw_event_id = decode_text(
            fields
                .next()
                .ok_or(SecureSessionError::InvalidEventCursor)?,
        )?;
        if version != Some("v1")
            || fields.next().is_some()
            || expires_at < unix_seconds()
            || authorization_context != self.authorization_context.as_ref()
            || session_id != expected_session_id.as_ref()
        {
            return Err(SecureSessionError::InvalidEventCursor);
        }
        Ok(raw_event_id)
    }
}

fn decode_text(encoded: &str) -> Result<String, SecureSessionError> {
    let bytes = URL_SAFE_NO_PAD
        .decode(encoded)
        .map_err(|_| SecureSessionError::InvalidEventCursor)?;
    str::from_utf8(&bytes)
        .map(str::to_owned)
        .map_err(|_| SecureSessionError::InvalidEventCursor)
}

fn unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs())
}

/// In-memory MCP sessions with bounded capacity and opaque resumable cursors.
///
/// One instance must be created per authorization context. Its signed cursors
/// cannot be replayed by another principal or another session, and expire after
/// `event_ttl`.
pub struct SecureSessionManager {
    inner: LocalSessionManager,
    codec: EventCursorCodec,
    max_sessions: usize,
    creation_lock: Mutex<()>,
}

impl std::fmt::Debug for SecureSessionManager {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("SecureSessionManager")
            .field("max_sessions", &self.max_sessions)
            .finish_non_exhaustive()
    }
}

impl SecureSessionManager {
    /// Creates a manager for exactly one authorization context.
    pub fn new(
        authorization_context: Arc<str>,
        signing_key: &[u8],
        config: SecureSessionConfig,
    ) -> Result<Self, SecureSessionError> {
        let mut session_config = SessionConfig::default();
        session_config.channel_capacity = config.channel_capacity;
        session_config.keep_alive = Some(config.idle_timeout);
        session_config.sse_retry = Some(Duration::from_secs(3));
        session_config.completed_cache_ttl = config.event_ttl;
        session_config.init_timeout = Some(config.initialize_timeout);
        let mut inner = LocalSessionManager::default();
        inner.session_config = session_config;
        Ok(Self {
            inner,
            codec: EventCursorCodec::new(signing_key, authorization_context, config.event_ttl)?,
            max_sessions: config.max_sessions,
            creation_lock: Mutex::new(()),
        })
    }

    pub(crate) async fn active_session_count(&self) -> usize {
        self.inner.sessions.read().await.len()
    }

    fn secure_stream(
        &self,
        session_id: SessionId,
        stream: impl futures_util::Stream<Item = ServerSseMessage> + Send + Sync + 'static,
    ) -> impl futures_util::Stream<Item = ServerSseMessage> + Send + Sync + 'static {
        let codec = self.codec.clone();
        stream.map(move |mut message| {
            if let Some(raw_event_id) = message.event_id.take() {
                message.event_id = Some(codec.encode(&session_id, &raw_event_id));
            }
            message
        })
    }
}

impl SessionManager for SecureSessionManager {
    type Error = SecureSessionError;
    type Transport = SessionTransport;

    async fn create_session(&self) -> Result<(SessionId, Self::Transport), Self::Error> {
        let _guard = self.creation_lock.lock().await;
        if self.inner.sessions.read().await.len() >= self.max_sessions {
            return Err(SecureSessionError::SessionLimit);
        }
        Ok(self.inner.create_session().await?)
    }

    async fn initialize_session(
        &self,
        id: &SessionId,
        message: ClientJsonRpcMessage,
    ) -> Result<ServerJsonRpcMessage, Self::Error> {
        Ok(self.inner.initialize_session(id, message).await?)
    }

    async fn has_session(&self, id: &SessionId) -> Result<bool, Self::Error> {
        Ok(self.inner.has_session(id).await?)
    }

    async fn close_session(&self, id: &SessionId) -> Result<(), Self::Error> {
        Ok(self.inner.close_session(id).await?)
    }

    async fn create_stream(
        &self,
        id: &SessionId,
        message: ClientJsonRpcMessage,
    ) -> Result<
        impl futures_util::Stream<Item = ServerSseMessage> + Send + Sync + 'static,
        Self::Error,
    > {
        let stream = self.inner.create_stream(id, message).await?;
        Ok(self.secure_stream(id.clone(), stream))
    }

    async fn accept_message(
        &self,
        id: &SessionId,
        message: ClientJsonRpcMessage,
    ) -> Result<(), Self::Error> {
        Ok(self.inner.accept_message(id, message).await?)
    }

    async fn create_standalone_stream(
        &self,
        id: &SessionId,
    ) -> Result<
        impl futures_util::Stream<Item = ServerSseMessage> + Send + Sync + 'static,
        Self::Error,
    > {
        let stream = self.inner.create_standalone_stream(id).await?;
        Ok(self.secure_stream(id.clone(), stream))
    }

    async fn resume(
        &self,
        id: &SessionId,
        last_event_id: String,
    ) -> Result<
        impl futures_util::Stream<Item = ServerSseMessage> + Send + Sync + 'static,
        Self::Error,
    > {
        let raw_event_id = self.codec.decode(id, &last_event_id)?;
        let stream = self.inner.resume(id, raw_event_id).await?;
        Ok(self.secure_stream(id.clone(), stream))
    }

    async fn restore_session(
        &self,
        id: SessionId,
    ) -> Result<RestoreOutcome<Self::Transport>, Self::Error> {
        let _guard = self.creation_lock.lock().await;
        if !self.inner.has_session(&id).await?
            && self.inner.sessions.read().await.len() >= self.max_sessions
        {
            return Err(SecureSessionError::SessionLimit);
        }
        Ok(self.inner.restore_session(id).await?)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::{EventCursorCodec, SecureSessionError};

    #[test]
    fn cursor_is_opaque_and_bound_to_authorization_and_session() {
        let codec = EventCursorCodec::new(
            b"01234567890123456789012345678901",
            Arc::from("subject-a"),
            std::time::Duration::from_secs(60),
        )
        .expect("valid signer");
        let session: Arc<str> = Arc::from("session-a");
        let cursor = codec.encode(&session, "7/42");

        assert!(!cursor.contains("session-a"));
        assert!(!cursor.contains("7/42"));
        assert_eq!(
            codec.decode(&session, &cursor).expect("valid cursor"),
            "7/42"
        );
        assert!(matches!(
            codec.decode(&Arc::from("session-b"), &cursor),
            Err(SecureSessionError::InvalidEventCursor)
        ));

        let other = EventCursorCodec::new(
            b"01234567890123456789012345678901",
            Arc::from("subject-b"),
            std::time::Duration::from_secs(60),
        )
        .expect("valid signer");
        assert!(matches!(
            other.decode(&session, &cursor),
            Err(SecureSessionError::InvalidEventCursor)
        ));
    }

    #[test]
    fn cursor_rejects_tampering() {
        let codec = EventCursorCodec::new(
            b"01234567890123456789012345678901",
            Arc::from("subject"),
            std::time::Duration::from_secs(60),
        )
        .expect("valid signer");
        let session: Arc<str> = Arc::from("session");
        let mut cursor = codec.encode(&session, "1");
        cursor.push('x');
        assert!(matches!(
            codec.decode(&session, &cursor),
            Err(SecureSessionError::InvalidEventCursor)
        ));
    }
}
