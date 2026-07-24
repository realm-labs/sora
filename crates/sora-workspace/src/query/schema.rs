use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    ProjectRevision, ProjectSession,
    studio::{StudioNodeKind, StudioSchema},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SchemaEntityKind {
    Enum,
    Struct,
    Union,
    Table,
}

#[derive(Debug, Clone, Default, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SchemaSearchQuery {
    pub text: Option<String>,
    pub kind: Option<SchemaEntityKind>,
    pub field: Option<String>,
    pub type_name: Option<String>,
    pub scope: Option<String>,
    pub source: Option<String>,
    pub references: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct SchemaSearchReport {
    pub revision: ProjectRevision,
    pub results: Vec<SchemaSearchResult>,
    pub truncated: bool,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct SchemaSearchResult {
    pub kind: String,
    pub name: String,
    pub source: String,
    pub scope: String,
    pub matching_fields: Vec<String>,
    pub resource_uri: String,
}

impl ProjectSession {
    pub fn search_schema(&self, query: &SchemaSearchQuery) -> anyhow::Result<SchemaSearchReport> {
        let response = self.load_studio_schema();
        let schema = response
            .schema
            .ok_or_else(|| anyhow::anyhow!("schema graph is unavailable"))?;
        if !response.ok {
            anyhow::bail!("schema must validate before it can be searched");
        }
        let limit = query.limit.unwrap_or(50).clamp(1, 100);
        let mut results = schema
            .nodes
            .iter()
            .filter_map(|node| match_node(self.id().as_str(), &schema, node, query))
            .collect::<Vec<_>>();
        results.sort_by(|left, right| {
            (&left.kind, &left.name, &left.source).cmp(&(&right.kind, &right.name, &right.source))
        });
        let truncated = results.len() > limit;
        results.truncate(limit);
        Ok(SchemaSearchReport {
            revision: self.revision(),
            results,
            truncated,
        })
    }
}

fn match_node(
    project_id: &str,
    schema: &StudioSchema,
    node: &crate::studio::StudioNode,
    query: &SchemaSearchQuery,
) -> Option<SchemaSearchResult> {
    if query.kind.is_some_and(|kind| kind != node.kind.into()) {
        return None;
    }
    if query
        .text
        .as_deref()
        .is_some_and(|text| !contains_case_insensitive(&node.name, text))
    {
        return None;
    }
    if query
        .scope
        .as_deref()
        .is_some_and(|scope| !scope_matches(&node.scope, scope))
    {
        return None;
    }
    if query
        .source
        .as_deref()
        .is_some_and(|source| node.source != source)
    {
        return None;
    }
    let matching_fields = node
        .fields
        .iter()
        .filter(|field| {
            query
                .field
                .as_deref()
                .is_none_or(|name| contains_case_insensitive(&field.name, name))
                && query
                    .type_name
                    .as_deref()
                    .is_none_or(|ty| contains_case_insensitive(&field.ty, ty))
        })
        .map(|field| field.name.clone())
        .collect::<Vec<_>>();
    if (query.field.is_some() || query.type_name.is_some()) && matching_fields.is_empty() {
        return None;
    }
    if let Some(reference) = query.references.as_deref() {
        let matches_reference = schema.edges.iter().any(|edge| {
            (edge.source == node.id || edge.target == node.id)
                && (contains_case_insensitive(&edge.source, reference)
                    || contains_case_insensitive(&edge.target, reference)
                    || contains_case_insensitive(&edge.label, reference))
        });
        if !matches_reference {
            return None;
        }
    }
    let kind = kind_name(node.kind);
    Some(SchemaSearchResult {
        kind: kind.to_owned(),
        name: node.name.clone(),
        source: node.source.clone(),
        scope: node.scope.clone(),
        matching_fields,
        resource_uri: format!("sora://project/{project_id}/schema/{kind}/{}", node.name),
    })
}

fn contains_case_insensitive(value: &str, query: &str) -> bool {
    value.to_lowercase().contains(&query.to_lowercase())
}

fn scope_matches(value: &str, query: &str) -> bool {
    value
        .split(',')
        .map(str::trim)
        .any(|scope| scope == "all" || scope == query)
}

fn kind_name(kind: StudioNodeKind) -> &'static str {
    match kind {
        StudioNodeKind::Enum => "enum",
        StudioNodeKind::Struct => "struct",
        StudioNodeKind::Union => "union",
        StudioNodeKind::Table => "table",
    }
}

impl From<StudioNodeKind> for SchemaEntityKind {
    fn from(value: StudioNodeKind) -> Self {
        match value {
            StudioNodeKind::Enum => Self::Enum,
            StudioNodeKind::Struct => Self::Struct,
            StudioNodeKind::Union => Self::Union,
            StudioNodeKind::Table => Self::Table,
        }
    }
}
