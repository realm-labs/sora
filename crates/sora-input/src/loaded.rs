use sora_data::model::ConfigData;
use sora_diagnostics::{Result, SoraError};
use sora_ir::model::ConfigIr;
use sora_schema::model::SchemaFile;

use crate::traits::{DataInput, SchemaInput};

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
