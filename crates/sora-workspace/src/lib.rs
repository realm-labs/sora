//! Shared project application services used by Sora's user-facing adapters.
//!
//! This crate owns project sessions and application-level coordination. The
//! CLI, Studio, and MCP adapters must depend on this crate rather than
//! duplicating project orchestration.

mod project;
mod service;

pub use project::{ProjectId, ProjectRevision, ProjectSession};
pub use service::{WorkspaceError, WorkspaceService};
