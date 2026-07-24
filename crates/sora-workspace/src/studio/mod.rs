mod diff;
mod graph;
mod model;
mod render;
mod service;

pub use model::{
    DiagnosticLevel, StudioDiagnostic, StudioEdge, StudioEdgeKind, StudioField, StudioNode,
    StudioNodeKind, StudioPreviewResponse, StudioSchema, StudioSchemaResponse, StudioSummary,
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
