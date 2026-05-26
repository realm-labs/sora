use std::path::{Path, PathBuf};

use sora_data::model::{ConfigData, LocalizationData, LocalizationRowData, LocalizationSourceData};
use sora_diagnostics::Result;
use sora_input::traits::DataInput;
use sora_ir::model::ConfigIr;

use crate::data::{load_config_data, load_table_data_file};

#[derive(Debug, Clone)]
pub struct TomlDataInput {
    data_root: PathBuf,
}

impl TomlDataInput {
    pub fn new(data_root: impl Into<PathBuf>) -> Self {
        Self {
            data_root: data_root.into(),
        }
    }

    pub fn data_root(&self) -> &Path {
        &self.data_root
    }
}

impl DataInput for TomlDataInput {
    fn load_data(&self, ir: &ConfigIr) -> Result<ConfigData> {
        load_config_data(ir, &self.data_root)
    }

    fn load_localization_data(&self, ir: &ConfigIr) -> Result<LocalizationData> {
        let Some(localization) = &ir.localization else {
            return Ok(LocalizationData::default());
        };
        let mut sources = Vec::with_capacity(localization.sources.len());
        for source in &localization.sources {
            let table = load_table_data_file(&source.name, &self.data_root.join(&source.file))?;
            let mut columns = Vec::new();
            let mut rows = Vec::with_capacity(table.rows.len());
            for row in table.rows {
                let mut values = std::collections::BTreeMap::new();
                for (field, value) in row.values {
                    if let sora_data::model::Value::String(value) = value {
                        if !columns.iter().any(|column| column == &field) {
                            columns.push(field.clone());
                        }
                        values.insert(field, value);
                    }
                }
                rows.push(LocalizationRowData { values });
            }
            sources.push(LocalizationSourceData {
                name: source.name.clone(),
                columns,
                rows,
            });
        }
        Ok(LocalizationData { sources })
    }
}
