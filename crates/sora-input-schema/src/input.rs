use std::path::{Path, PathBuf};

use sora_diagnostics::Result;
use sora_input::traits::SchemaInput;
use sora_schema::model::SchemaFile;

use crate::schema::load_project_schema_file;

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
