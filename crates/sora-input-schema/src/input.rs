use std::path::{Path, PathBuf};

use sora_diagnostics::Result;
use sora_input::traits::SchemaInput;
use sora_schema::model::ProjectSchema;

use crate::schema::load_project_schema;

#[derive(Debug, Clone)]
pub struct ProjectSchemaInput {
    project_path: PathBuf,
}

impl ProjectSchemaInput {
    pub fn new(project_path: impl Into<PathBuf>) -> Self {
        Self {
            project_path: project_path.into(),
        }
    }

    pub fn project_path(&self) -> &Path {
        &self.project_path
    }
}

impl SchemaInput for ProjectSchemaInput {
    fn load_schema(&self) -> Result<ProjectSchema> {
        load_project_schema(&self.project_path)
    }
}
