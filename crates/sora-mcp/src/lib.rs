//! Model Context Protocol adapter for Sora.
//!
//! The MCP adapter contains protocol and transport concerns only. Project
//! behavior is delegated to [`sora_workspace`].

mod artifact_store;
mod completion;
mod dto;
mod http;
mod prompts;
mod protocol;
mod resources;
mod server;
mod task_store;
mod tools;
mod transport;

pub use http::{
    AuthorizationPrincipal, HttpConfigError, HttpServerConfig, OAuthAuthenticator, OAuthError,
    OAuthResourceServerConfig, ProtectedResourceMetadata, SecureSessionConfig, SecureSessionError,
    SecureSessionManager, build_http_router, serve_http,
};
pub use protocol::{SERVER_NAME, TARGET_PROTOCOL_VERSION};
pub use server::SoraMcpServer;
pub use transport::serve_stdio;
