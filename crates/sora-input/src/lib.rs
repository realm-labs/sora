use sora_data::ConfigData;
use sora_diagnostics::{Result, SoraError};
use sora_ir::ConfigIr;
use sora_schema::SchemaFile;

pub trait SchemaInput {
    fn load_schema(&self) -> Result<SchemaFile>;
}

pub trait DataInput {
    fn load_data(&self, ir: &ConfigIr) -> Result<ConfigData>;
}

pub trait ProjectInput: SchemaInput + DataInput {}

impl<T> ProjectInput for T where T: SchemaInput + DataInput {}

#[derive(Debug, Clone)]
pub struct LoadedInput {
    schema: SchemaFile,
    data: Option<ConfigData>,
}

impl LoadedInput {
    pub fn schema_only(schema: SchemaFile) -> Self {
        Self { schema, data: None }
    }

    pub fn with_data(schema: SchemaFile, data: ConfigData) -> Self {
        Self {
            schema,
            data: Some(data),
        }
    }
}

impl SchemaInput for LoadedInput {
    fn load_schema(&self) -> Result<SchemaFile> {
        Ok(self.schema.clone())
    }
}

impl DataInput for LoadedInput {
    fn load_data(&self, _ir: &ConfigIr) -> Result<ConfigData> {
        self.data.clone().ok_or_else(|| SoraError::MissingInputData)
    }
}
