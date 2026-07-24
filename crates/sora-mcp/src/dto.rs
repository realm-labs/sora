use schemars::JsonSchema;
use serde::Serialize;
use sora_workspace::{Diagnostic, ProjectId, ProjectRevision};

#[derive(Debug, Serialize, JsonSchema)]
pub struct ToolEnvelope<T: JsonSchema> {
    pub ok: bool,
    pub project_id: Option<ProjectId>,
    pub revision: Option<ProjectRevision>,
    pub summary: String,
    pub diagnostics: Vec<Diagnostic>,
    pub changes: Vec<serde_json::Value>,
    pub artifacts: Vec<ArtifactLink>,
    pub next_cursor: Option<String>,
    pub data: Option<T>,
}

impl<T: JsonSchema> ToolEnvelope<T> {
    pub fn success(
        project_id: Option<ProjectId>,
        revision: Option<ProjectRevision>,
        summary: impl Into<String>,
        data: T,
    ) -> Self {
        Self {
            ok: true,
            project_id,
            revision,
            summary: summary.into(),
            diagnostics: Vec::new(),
            changes: Vec::new(),
            artifacts: Vec::new(),
            next_cursor: None,
            data: Some(data),
        }
    }

    pub fn failure(
        project_id: Option<ProjectId>,
        revision: Option<ProjectRevision>,
        summary: impl Into<String>,
        error: impl ToString,
    ) -> Self {
        Self {
            ok: false,
            project_id,
            revision,
            summary: summary.into(),
            diagnostics: vec![Diagnostic::error(error.to_string())],
            changes: Vec::new(),
            artifacts: Vec::new(),
            next_cursor: None,
            data: None,
        }
    }
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ArtifactLink {
    pub artifact_id: String,
    pub uri: String,
    pub mime_type: String,
    pub name: Option<String>,
    pub size: Option<usize>,
}

pub fn tool_error<T>(envelope: ToolEnvelope<T>) -> rmcp::model::CallToolResult
where
    T: JsonSchema + Serialize,
{
    match serde_json::to_value(envelope) {
        Ok(value) => rmcp::model::CallToolResult::structured_error(value),
        Err(error) => rmcp::model::CallToolResult::structured_error(serde_json::json!({
            "ok": false,
            "summary": "failed to encode structured tool error",
            "diagnostics": [{
                "severity": "error",
                "code": null,
                "message": error.to_string()
            }],
            "changes": [],
            "artifacts": [],
            "next_cursor": null,
            "data": null
        })),
    }
}
