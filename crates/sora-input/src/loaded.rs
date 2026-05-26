use sora_data::model::{ConfigData, LocalizationData};
use sora_diagnostics::{Result, SoraError};
use sora_ir::model::ConfigIr;
use sora_schema::model::SchemaFile;

use crate::traits::{DataInput, SchemaInput};

#[derive(Debug, Clone)]
pub struct LoadedInput {
    schema: SchemaFile,
    data: Option<ConfigData>,
    localization_data: LocalizationData,
}

impl LoadedInput {
    pub fn schema_only(schema: SchemaFile) -> Self {
        Self {
            schema,
            data: None,
            localization_data: LocalizationData::default(),
        }
    }

    pub fn with_data(schema: SchemaFile, data: ConfigData) -> Self {
        Self {
            schema,
            data: Some(data),
            localization_data: LocalizationData::default(),
        }
    }

    pub fn with_data_and_localization(
        schema: SchemaFile,
        data: ConfigData,
        localization_data: LocalizationData,
    ) -> Self {
        Self {
            schema,
            data: Some(data),
            localization_data,
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
        self.data.clone().ok_or(SoraError::MissingInputData)
    }

    fn load_localization_data(&self, _ir: &ConfigIr) -> Result<LocalizationData> {
        Ok(self.localization_data.clone())
    }
}
