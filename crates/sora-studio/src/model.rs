use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct StudioSchemaResponse {
    pub ok: bool,
    pub project: String,
    pub diagnostics: Vec<StudioDiagnostic>,
    pub schema: Option<StudioSchema>,
}

#[derive(Debug, Serialize)]
pub struct StudioPreviewResponse {
    pub ok: bool,
    pub project: String,
    pub target: Option<String>,
    pub content: Option<String>,
    pub diff: Option<String>,
    pub diagnostics: Vec<StudioDiagnostic>,
}

#[derive(Debug, Serialize)]
pub struct StudioDiagnostic {
    pub level: DiagnosticLevel,
    pub message: String,
    #[serde(rename = "targetId", skip_serializing_if = "Option::is_none")]
    pub target_id: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticLevel {
    Error,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudioSchema {
    pub package: String,
    pub sources: Vec<String>,
    pub summary: StudioSummary,
    pub nodes: Vec<StudioNode>,
    pub edges: Vec<StudioEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudioSummary {
    pub enums: usize,
    pub structs: usize,
    pub unions: usize,
    pub tables: usize,
    pub edges: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudioNode {
    pub id: String,
    pub name: String,
    pub kind: StudioNodeKind,
    pub source: String,
    pub scope: String,
    pub subtitle: String,
    pub fields: Vec<StudioField>,
    pub metadata: BTreeMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum StudioNodeKind {
    Enum,
    Struct,
    Union,
    Table,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudioField {
    pub name: String,
    pub ty: String,
    pub scope: String,
    pub parser: Option<String>,
    pub comment: Option<String>,
    pub default: Option<String>,
    pub range: Option<[i64; 2]>,
    pub length: Option<[usize; 2]>,
    pub source: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct StudioEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    pub kind: StudioEdgeKind,
    pub label: String,
    #[serde(default, rename = "targetLabel")]
    pub target_label: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum StudioEdgeKind {
    Type,
    Ref,
    Derived,
}
