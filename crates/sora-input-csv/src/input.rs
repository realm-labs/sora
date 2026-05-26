use std::path::{Path, PathBuf};

use sora_data::model::{ConfigData, LocalizationData};
use sora_diagnostics::Result;
use sora_input::traits::{DataInput, SchemaInput};
use sora_ir::model::ConfigIr;
use sora_schema::model::SchemaFile;

use crate::reader::{load_csv_config_data, load_csv_localization_source_data};

#[derive(Debug, Clone)]
pub struct CsvProjectInput<S> {
    schema_input: S,
    data_root: PathBuf,
}

impl<S> CsvProjectInput<S> {
    pub fn new(schema_input: S, data_root: impl Into<PathBuf>) -> Self {
        Self {
            schema_input,
            data_root: data_root.into(),
        }
    }

    pub fn data_root(&self) -> &Path {
        &self.data_root
    }
}

impl<S: SchemaInput> SchemaInput for CsvProjectInput<S> {
    fn load_schema(&self) -> Result<SchemaFile> {
        self.schema_input.load_schema()
    }
}

impl<S: SchemaInput> DataInput for CsvProjectInput<S> {
    fn load_data(&self, ir: &ConfigIr) -> Result<ConfigData> {
        load_csv_config_data(ir, &self.data_root)
    }

    fn load_localization_data(&self, ir: &ConfigIr) -> Result<LocalizationData> {
        let Some(localization) = &ir.localization else {
            return Ok(LocalizationData::default());
        };
        let mut sources = Vec::with_capacity(localization.sources.len());
        for source in &localization.sources {
            sources.push(load_csv_localization_source_data(
                source,
                &self.data_root.join(&source.file),
            )?);
        }
        Ok(LocalizationData { sources })
    }
}
