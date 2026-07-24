//! Shared project application services used by Sora's user-facing adapters.
//!
//! This crate owns project sessions and application-level coordination. The
//! CLI, Studio, and MCP adapters must depend on this crate rather than
//! duplicating project orchestration.

mod build;
mod diagnostics;
mod inspect;
mod mutation;
mod parser;
mod project;
mod query;
mod revision;
mod runtime;
mod service;
pub mod source;
pub mod studio;
mod type_mapping;

pub use build::{
    BuildArtifact, BuildArtifactKind, BuildCodegen, BuildConfig, BuildExport, BuildReport,
    BuildRequest, CodeFormatMode, ExportCompression, ProjectManifest, ScriptConfig, SourceFormat,
    build_project,
};
pub use diagnostics::{
    Diagnostic, DiagnosticEntity, DiagnosticLevel, DiagnosticSpan, diagnostics_from_anyhow,
    diagnostics_from_sora_error,
};
pub use inspect::{
    ProjectBuildOutput, ProjectDataSource, ProjectInspection, ProjectLocalization, ValidationReport,
};
pub use mutation::{
    DerivedFieldSource, EnumValueDefinition, FieldDefinition, FieldOwner, FieldOwnerKind,
    IndexDefinition, MutationPlanError, MutationSchemaEntityKind, ProjectInitApplyReport,
    ProjectInitPlan, ProjectInitPlanFile, SchemaApplyReport, SchemaExecution, SchemaMutationError,
    SchemaMutationPlan, SchemaOperation, SchemaTableMode, TableSourceDefinition, TextFileDiff,
    TransactionError, TransactionReceipt, UnionVariantDefinition, execute_schema_operations,
};
pub use parser::{ParserRegistries, load_parser_registries};
pub use project::{ProjectId, ProjectRevision, ProjectSession};
pub use query::{
    DataValidationQuery, DataValidationReport, IndexLookup, SchemaEntityKind, SchemaSearchQuery,
    SchemaSearchReport, SchemaSearchResult, TableFilter, TableQuery, TableQueryReport,
};
pub use runtime::{ProjectRuntime, RuntimeOptions};
pub use service::{ProjectCandidate, WorkspaceError, WorkspaceRoot, WorkspaceService};
pub use type_mapping::load_type_mapping_registry;
