use sora_data::model::{ConfigData, LocalizationData};
use sora_diagnostics::Result;
use sora_execution::ExecutionContext;
use sora_ir::model::ConfigIr;
use sora_schema::model::SchemaFile;

use crate::traits::{DataInput, SchemaInput};

#[derive(Debug, Clone)]
pub struct SplitProjectInput<S, D> {
    schema_input: S,
    data_input: D,
}

impl<S, D> SplitProjectInput<S, D> {
    pub fn new(schema_input: S, data_input: D) -> Self {
        Self {
            schema_input,
            data_input,
        }
    }

    pub fn schema_input(&self) -> &S {
        &self.schema_input
    }

    pub fn data_input(&self) -> &D {
        &self.data_input
    }
}

impl<S: SchemaInput, D> SchemaInput for SplitProjectInput<S, D> {
    fn load_schema(&self) -> Result<SchemaFile> {
        self.schema_input.load_schema()
    }
}

impl<S, D: DataInput> DataInput for SplitProjectInput<S, D> {
    fn load_data(&self, ir: &ConfigIr) -> Result<ConfigData> {
        self.data_input.load_data(ir)
    }

    fn load_localization_data(&self, ir: &ConfigIr) -> Result<LocalizationData> {
        self.data_input.load_localization_data(ir)
    }

    fn load_data_with_context(
        &self,
        ir: &ConfigIr,
        execution: &ExecutionContext,
    ) -> Result<ConfigData> {
        self.data_input.load_data_with_context(ir, execution)
    }

    fn load_localization_data_with_context(
        &self,
        ir: &ConfigIr,
        execution: &ExecutionContext,
    ) -> Result<LocalizationData> {
        self.data_input
            .load_localization_data_with_context(ir, execution)
    }
}
