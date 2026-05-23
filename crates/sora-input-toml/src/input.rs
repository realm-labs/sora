use std::path::{Path, PathBuf};

use sora_data::model::ConfigData;
use sora_diagnostics::Result;
use sora_input::traits::{DataInput, SchemaInput};
use sora_ir::model::ConfigIr;
use sora_schema::model::SchemaFile;

use crate::{data::load_config_data, schema::load_project_schema_file};

#[derive(Debug, Clone)]
pub struct SchemaFileInput {
    project_path: PathBuf,
}

impl SchemaFileInput {
    pub fn new(project_path: impl Into<PathBuf>) -> Self {
        Self {
            project_path: project_path.into(),
        }
    }

    pub fn project_path(&self) -> &Path {
        &self.project_path
    }
}

impl SchemaInput for SchemaFileInput {
    fn load_schema(&self) -> Result<SchemaFile> {
        load_project_schema_file(&self.project_path)
    }
}

#[derive(Debug, Clone)]
pub struct ProjectFileInput {
    project_path: PathBuf,
    data_root: PathBuf,
}

impl ProjectFileInput {
    pub fn new(project_path: impl Into<PathBuf>, data_root: impl Into<PathBuf>) -> Self {
        Self {
            project_path: project_path.into(),
            data_root: data_root.into(),
        }
    }

    pub fn project_path(&self) -> &Path {
        &self.project_path
    }

    pub fn data_root(&self) -> &Path {
        &self.data_root
    }
}

impl SchemaInput for ProjectFileInput {
    fn load_schema(&self) -> Result<SchemaFile> {
        load_project_schema_file(&self.project_path)
    }
}

impl DataInput for ProjectFileInput {
    fn load_data(&self, ir: &ConfigIr) -> Result<ConfigData> {
        load_config_data(ir, &self.data_root)
    }
}
