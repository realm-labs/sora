pub(crate) mod diff;
pub(crate) mod graph;
mod model;
mod render;
pub(crate) mod service;

pub use model::{
    DiagnosticSeverity, StudioDiagnostic, StudioEdge, StudioEdgeKind, StudioEnumAlias, StudioField,
    StudioIndex, StudioNode, StudioNodeKind, StudioPreviewResponse, StudioSchema,
    StudioSchemaResponse, StudioSummary,
};

use crate::ProjectSession;

impl ProjectSession {
    pub fn load_studio_schema(&self) -> StudioSchemaResponse {
        self.load_studio_schema_view(None)
    }

    pub fn load_studio_schema_view(&self, view: Option<&str>) -> StudioSchemaResponse {
        service::load_studio_schema_with_parsers(
            self.manifest_path(),
            self.runtime().schema_parsers(),
            view,
        )
    }
}

#[cfg(test)]
mod tests;
