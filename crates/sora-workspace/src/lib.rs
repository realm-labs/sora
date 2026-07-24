//! Shared project application services used by Sora's user-facing adapters.
//!
//! This crate owns project sessions and application-level coordination. The
//! CLI, Studio, and MCP adapters must depend on this crate rather than
//! duplicating project orchestration.

mod parser;
mod project;
mod runtime;
mod service;
pub mod source;
mod type_mapping;

pub use parser::{ParserRegistries, load_parser_registries};
pub use project::{ProjectId, ProjectRevision, ProjectSession};
pub use runtime::{ProjectRuntime, RuntimeOptions};
pub use service::{WorkspaceError, WorkspaceService};
pub use type_mapping::load_type_mapping_registry;
