use sora_data::model::ConfigData;
use sora_diagnostics::Result;
use sora_ir::model::ConfigIr;
use sora_schema::model::SchemaFile;

pub trait SchemaInput {
    fn load_schema(&self) -> Result<SchemaFile>;
}

pub trait DataInput {
    fn load_data(&self, ir: &ConfigIr) -> Result<ConfigData>;
}

pub trait ProjectInput: SchemaInput + DataInput {}

impl<T> ProjectInput for T where T: SchemaInput + DataInput {}
