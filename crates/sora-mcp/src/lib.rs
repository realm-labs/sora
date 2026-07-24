//! Model Context Protocol adapter for Sora.
//!
//! The MCP adapter contains protocol and transport concerns only. Project
//! behavior is delegated to [`sora_workspace`].

mod protocol;
mod server;
mod transport;

pub use protocol::{SERVER_NAME, TARGET_PROTOCOL_VERSION};
pub use server::SoraMcpServer;
pub use transport::serve_stdio;
