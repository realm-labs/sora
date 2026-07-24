pub(crate) mod diff;
pub(crate) mod graph;
mod model;
mod render;
pub(crate) mod service;

pub use model::{
    DiagnosticLevel, StudioDiagnostic, StudioEdge, StudioEdgeKind, StudioEnumAlias, StudioField,
    StudioIndex, StudioNode, StudioNodeKind, StudioPreviewResponse, StudioSchema,
    StudioSchemaResponse, StudioSummary,
};

use crate::ProjectSession;

impl ProjectSession {
    pub fn load_studio_schema(&self) -> StudioSchemaResponse {
        service::load_studio_schema_with_parsers(
            self.manifest_path(),
            self.runtime().schema_parsers(),
        )
    }

    pub fn preview_studio_schema(&self, schema: &StudioSchema) -> StudioPreviewResponse {
        service::preview_studio_schema_with_parsers(
            self.manifest_path(),
            schema,
            self.runtime().schema_parsers(),
        )
    }

    pub fn save_studio_schema(&self, schema: &StudioSchema) -> StudioSchemaResponse {
        let _write_guard = match self.write_lock.lock() {
            Ok(guard) => guard,
            Err(_) => {
                return StudioSchemaResponse {
                    ok: false,
                    project: self.manifest_path().display().to_string(),
                    diagnostics: vec![StudioDiagnostic::error("project write lock is poisoned")],
                    schema: Some(schema.clone()),
                };
            }
        };
        let mut response = service::save_studio_schema_with_parsers(
            self.manifest_path(),
            schema,
            self.runtime().schema_parsers(),
        );
        if response.ok
            && let Err(error) = self.refresh_revision()
        {
            response.ok = false;
            response.diagnostics.push(StudioDiagnostic::error(format!(
                "schema was saved but project revision refresh failed: {error}"
            )));
        }
        response
    }
}

#[cfg(test)]
mod tests;
