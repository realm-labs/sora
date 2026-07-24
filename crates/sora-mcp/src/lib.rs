//! Model Context Protocol adapter for Sora.
//!
//! The MCP adapter contains protocol and transport concerns only. Project
//! behavior is delegated to [`sora_workspace`].

mod artifact_store;
mod completion;
mod dto;
mod protocol;
mod resources;
mod server;
mod task_store;
mod tools;
mod transport;

pub use protocol::{SERVER_NAME, TARGET_PROTOCOL_VERSION};
pub use server::SoraMcpServer;
pub use transport::serve_stdio;
