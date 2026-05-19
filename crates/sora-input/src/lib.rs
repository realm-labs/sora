use std::path::{Path, PathBuf};

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
pub struct TomlSchemaInput {
    schema_path: PathBuf,
}

impl TomlSchemaInput {
    pub fn new(schema_path: impl Into<PathBuf>) -> Self {
        Self {
            schema_path: schema_path.into(),
        }
    }

    pub fn schema_path(&self) -> &Path {
        &self.schema_path
    }
}

impl SchemaInput for TomlSchemaInput {
    fn load_schema(&self) -> Result<SchemaFile> {
        sora_schema::load_schema_file(&self.schema_path)
    }
}

#[derive(Debug, Clone)]
pub struct TomlProjectInput {
    schema_path: PathBuf,
    data_root: PathBuf,
}

impl TomlProjectInput {
    pub fn new(schema_path: impl Into<PathBuf>, data_root: impl Into<PathBuf>) -> Self {
        Self {
            schema_path: schema_path.into(),
            data_root: data_root.into(),
        }
    }

    pub fn schema_path(&self) -> &Path {
        &self.schema_path
    }

    pub fn data_root(&self) -> &Path {
        &self.data_root
    }
}

impl SchemaInput for TomlProjectInput {
    fn load_schema(&self) -> Result<SchemaFile> {
        sora_schema::load_schema_file(&self.schema_path)
    }
}

impl DataInput for TomlProjectInput {
    fn load_data(&self, ir: &ConfigIr) -> Result<ConfigData> {
        sora_data::load_config_data(ir, &self.data_root)
    }
}

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
