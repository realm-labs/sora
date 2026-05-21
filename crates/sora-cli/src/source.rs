use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use sora_data::model::{ConfigData, TableData};
use sora_diagnostics::{Result, SoraError};
use sora_input::{
    source::{SourceFormat, resolve_table_source_format},
    traits::{DataInput, SchemaInput},
};
use sora_input_csv::reader::load_csv_table_data;
use sora_input_toml::data::load_table_data_file;
use sora_input_xlsx::reader::load_xlsx_table_data_with_ir;
use sora_ir::model::{ConfigIr, TableIr};
use sora_schema::model::SchemaFile;

#[derive(Clone)]
pub struct MixedProjectInput<S> {
    schema_input: S,
    data_root: PathBuf,
    default_source_format: Option<Arc<str>>,
}

impl<S> MixedProjectInput<S> {
    pub fn new(
        schema_input: S,
        data_root: impl Into<PathBuf>,
        default_source_format: Option<&str>,
    ) -> Self {
        Self {
            schema_input,
            data_root: data_root.into(),
            default_source_format: default_source_format.map(Arc::from),
        }
    }
}

impl<S: SchemaInput> SchemaInput for MixedProjectInput<S> {
    fn load_schema(&self) -> Result<SchemaFile> {
        self.schema_input.load_schema()
    }
}

impl<S: SchemaInput> DataInput for MixedProjectInput<S> {
    fn load_data(&self, ir: &ConfigIr) -> Result<ConfigData> {
        load_mixed_config_data(ir, &self.data_root, self.default_source_format.as_deref())
    }
}

pub fn load_mixed_config_data(
    ir: &ConfigIr,
    data_root: &Path,
    default_source_format: Option<&str>,
) -> Result<ConfigData> {
    let mut tables = Vec::with_capacity(ir.tables.len());
    for table in &ir.tables {
        tables.push(load_mixed_table_data(
            ir,
            table,
            data_root,
            default_source_format,
        )?);
    }
    Ok(ConfigData { tables })
}

fn load_mixed_table_data(
    ir: &ConfigIr,
    table: &TableIr,
    data_root: &Path,
    default_source_format: Option<&str>,
) -> Result<TableData> {
    let source = table
        .source
        .as_ref()
        .ok_or_else(|| SoraError::MissingTableSource {
            table: table.name.clone(),
        })?;
    let path = data_root.join(&source.file);

    match resolve_table_source_format(table, default_source_format)? {
        SourceFormat::Csv => load_csv_table_data(ir, table, &path),
        SourceFormat::Toml => load_table_data_file(&table.name, &path),
        SourceFormat::Xlsx => {
            let sheet = source.sheet.as_deref().unwrap_or(&table.name);
            load_xlsx_table_data_with_ir(ir, table, &path, sheet)
        }
    }
}
