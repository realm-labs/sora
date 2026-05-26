use std::path::{Path, PathBuf};

use sora_data::model::{ConfigData, LocalizationData};
use sora_diagnostics::Result;
use sora_execution::ExecutionContext;
use sora_input::traits::{DataInput, SchemaInput};
use sora_ir::model::ConfigIr;
use sora_schema::model::SchemaFile;

use crate::reader::{
    load_xlsx_config_data, load_xlsx_config_data_with_context, load_xlsx_localization_source_data,
};

#[derive(Debug, Clone)]
pub struct XlsxProjectInput<S> {
    schema_input: S,
    data_root: PathBuf,
}

impl<S> XlsxProjectInput<S> {
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

impl<S: SchemaInput> SchemaInput for XlsxProjectInput<S> {
    fn load_schema(&self) -> Result<SchemaFile> {
        self.schema_input.load_schema()
    }
}

impl<S: SchemaInput> DataInput for XlsxProjectInput<S> {
    fn load_data(&self, ir: &ConfigIr) -> Result<ConfigData> {
        load_xlsx_config_data(ir, &self.data_root)
    }

    fn load_localization_data(&self, ir: &ConfigIr) -> Result<LocalizationData> {
        load_xlsx_localization_data(ir, &self.data_root)
    }

    fn load_data_with_context(
        &self,
        ir: &ConfigIr,
        execution: &ExecutionContext,
    ) -> Result<ConfigData> {
        load_xlsx_config_data_with_context(ir, &self.data_root, execution)
    }
}

fn load_xlsx_localization_data(ir: &ConfigIr, data_root: &Path) -> Result<LocalizationData> {
    let Some(localization) = &ir.localization else {
        return Ok(LocalizationData::default());
    };
    let mut sources = Vec::with_capacity(localization.sources.len());
    for source in &localization.sources {
        let sheet = source.sheet.as_deref().unwrap_or(&source.name);
        sources.push(load_xlsx_localization_source_data(
            source,
            &data_root.join(&source.file),
            sheet,
        )?);
    }
    Ok(LocalizationData { sources })
}
