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
}

#[cfg(test)]
mod tests;
