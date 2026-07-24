mod data;
mod data_plan;
mod excel_plan;
mod plan;
mod project_init;
mod schema;
mod transaction;

pub use plan::{
    MutationPlanError, SchemaApplyReport, SchemaMutationPlan, StudioSchemaApplyReport,
    StudioSchemaMutationPlan, TextFileDiff,
};
pub use project_init::{ProjectInitApplyReport, ProjectInitPlan, ProjectInitPlanFile};
pub use schema::{
    DerivedFieldSource, EnumValueDefinition, FieldDefinition, FieldOwner, FieldOwnerKind,
    IndexDefinition, SchemaEntityKind as MutationSchemaEntityKind, SchemaExecution,
    SchemaMutationError, SchemaOperation, SchemaTableMode, TableSourceDefinition,
    UnionVariantDefinition, execute_schema_operations,
};
pub use transaction::{TransactionError, TransactionReceipt};

pub use data::{
    DataExecution, DataMutationError, DataOperation, DataSourceImpact, LocalizationChange,
    MutableTableSource, RowChange, RowSelector, execute_data_operations,
};
pub(crate) use data::{data_row_hash, load_raw_project_data};
pub(crate) use data_plan::DataMutationCoordinator;
pub use data_plan::{DataApplyReport, DataFileChange, DataMutationPlan, DataPlanError};
pub(crate) use excel_plan::ExcelSyncCoordinator;
pub use excel_plan::{
    ExcelSyncApplyReport, ExcelSyncPlan, ExcelSyncPlanError, ExcelSyncSheetChange,
    ExcelSyncWorkbookChange,
};
pub(crate) use plan::MutationCoordinator;
pub(crate) use project_init::ProjectInitCoordinator;
pub(crate) use transaction::{
    FileWrite, commit_file_transaction, commit_text_transaction, recover_transactions,
};
