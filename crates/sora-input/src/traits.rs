use sora_data::model::{ConfigData, LocalizationData};
use sora_diagnostics::Result;
use sora_execution::ExecutionContext;
use sora_ir::model::ConfigIr;
use sora_schema::model::SchemaFile;

pub trait SchemaInput {
    fn load_schema(&self) -> Result<SchemaFile>;
}

pub trait DataInput {
    fn load_data(&self, ir: &ConfigIr) -> Result<ConfigData>;

    fn load_localization_data(&self, _ir: &ConfigIr) -> Result<LocalizationData> {
        Ok(LocalizationData::default())
    }

    fn load_data_with_context(
        &self,
        ir: &ConfigIr,
        _execution: &ExecutionContext,
    ) -> Result<ConfigData> {
        self.load_data(ir)
    }

    fn load_localization_data_with_context(
        &self,
        ir: &ConfigIr,
        _execution: &ExecutionContext,
    ) -> Result<LocalizationData> {
        self.load_localization_data(ir)
    }
}

pub trait ProjectInput: SchemaInput + DataInput {}

impl<T> ProjectInput for T where T: SchemaInput + DataInput {}
