//! Shared project application services used by Sora's user-facing adapters.
//!
//! This crate owns project sessions and application-level coordination. The
//! CLI, Studio, and MCP adapters must depend on this crate rather than
//! duplicating project orchestration.

mod build;
mod capabilities;
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
    BuildArtifact, BuildArtifactKind, BuildCodegen, BuildConfig, BuildControl, BuildExport,
    BuildPhase, BuildProgress, BuildReport, BuildRequest, CodeFormatMode, ExportCompression,
    ProjectManifest, ScriptConfig, SourceFormat, build_project, build_project_with_control,
};
pub use diagnostics::{
    Diagnostic, DiagnosticCell, DiagnosticEntity, DiagnosticSeverity, DiagnosticSpan,
    diagnostics_from_anyhow, diagnostics_from_sora_error,
};
pub use inspect::{
    ProjectBuildOutput, ProjectDataSource, ProjectInspection, ProjectLocalization, ValidationReport,
};
pub use mutation::{
    DataApplyReport, DataExecution, DataFileChange, DataMutationError, DataMutationPlan,
    DataOperation, DataPlanError, DataSourceImpact, DerivedFieldSource, EnumValueDefinition,
    ExcelSyncApplyReport, ExcelSyncControl, ExcelSyncPhase, ExcelSyncPlan, ExcelSyncPlanError,
    ExcelSyncProgress, ExcelSyncSheetChange, ExcelSyncWorkbookChange, FieldDefinition, FieldOwner,
    FieldOwnerKind, IndexDefinition, LocalizationChange, MutableTableSource, MutationPlanError,
    MutationSchemaEntityKind, ProjectInitApplyReport, ProjectInitPlan, ProjectInitPlanFile,
    RowChange, RowSelector, SchemaApplyReport, SchemaExecution, SchemaMutationError,
    SchemaMutationPlan, SchemaOperation, SchemaTableMode, StudioSchemaApplyReport,
    StudioSchemaMutationPlan, TableSourceDefinition, TextFileDiff, TransactionError,
    TransactionReceipt, UnionVariantDefinition, execute_data_operations, execute_schema_operations,
};
pub use parser::{ParserRegistries, load_parser_registries};
pub use project::{ProjectId, ProjectRevision, ProjectSession};
pub use query::{
    DataValidationQuery, DataValidationReport, IndexLookup, SchemaEntityKind, SchemaSearchQuery,
    SchemaSearchReport, SchemaSearchResult, TableFilter, TableQuery, TableQueryReport,
    TableQueryRow,
};
pub use runtime::{ProjectRuntime, RuntimeOptions};
pub use service::{
    DiscoveredProjectInspection, ProjectCandidate, ProjectScript, ProjectScriptKind,
    WorkspaceError, WorkspaceRoot, WorkspaceService,
};
pub use type_mapping::load_type_mapping_registry;
