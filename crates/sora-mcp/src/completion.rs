use rmcp::{
    ErrorData as McpError,
    model::{CompleteRequestParams, CompleteResult, CompletionInfo},
};
use sora_workspace::{ProjectId, WorkspaceService};

use crate::artifact_store::ArtifactStore;

pub fn complete(
    workspace: &WorkspaceService,
    artifacts: &ArtifactStore,
    authorization_context: &str,
    request: &CompleteRequestParams,
) -> Result<CompleteResult, McpError> {
    let name = request.argument.name.as_str();
    let values = if name == "project_id" {
        workspace
            .project_ids()
            .map_err(|error| McpError::internal_error(error.to_string(), None))?
            .into_iter()
            .map(|id| id.as_str().to_owned())
            .collect()
    } else {
        project_values(workspace, artifacts, authorization_context, request, name)?
    };
    let query = request.argument.value.to_lowercase();
    let mut ranked = values
        .into_iter()
        .filter_map(|value| fuzzy_rank(&value, &query).map(|rank| (rank, value)))
        .collect::<Vec<_>>();
    ranked.sort_by(|(left_rank, left), (right_rank, right)| {
        left_rank
            .cmp(right_rank)
            .then_with(|| left.to_lowercase().cmp(&right.to_lowercase()))
            .then_with(|| left.cmp(right))
    });
    let mut values = ranked
        .into_iter()
        .map(|(_, value)| value)
        .collect::<Vec<_>>();
    values.dedup();
    let total = values.len();
    values.truncate(CompletionInfo::MAX_VALUES);
    let completion = CompletionInfo::with_pagination(values, Some(total as u32), total > 100)
        .map_err(|error| McpError::internal_error(error, None))?;
    Ok(CompleteResult::new(completion))
}

fn project_values(
    workspace: &WorkspaceService,
    artifacts: &ArtifactStore,
    authorization_context: &str,
    request: &CompleteRequestParams,
    name: &str,
) -> Result<Vec<String>, McpError> {
    let project_id = request
        .context
        .as_ref()
        .and_then(|context| context.get_argument("project_id"))
        .ok_or_else(|| {
            McpError::invalid_params(
                format!("completion for `{name}` requires context argument `project_id`"),
                None,
            )
        })?;
    let id = ProjectId::new(project_id.clone())
        .map_err(|error| McpError::invalid_params(error.to_string(), None))?;
    let session = workspace
        .project(&id)
        .map_err(|error| McpError::invalid_params(error.to_string(), None))?;
    let inspection = session
        .inspect()
        .map_err(|error| McpError::internal_error(error.to_string(), None))?;
    let schema = session
        .normalized_schema()
        .map_err(|error| McpError::internal_error(error.to_string(), None))?;
    let values = match name {
        "table" => schema.tables.iter().map(|item| item.name.clone()).collect(),
        "field" => {
            let table = request
                .context
                .as_ref()
                .and_then(|context| context.get_argument("table"));
            schema
                .tables
                .iter()
                .filter(|item| table.is_none_or(|table| item.name == *table))
                .flat_map(|item| item.fields.iter().map(|field| field.name.clone()))
                .collect()
        }
        "enum" => schema.enums.iter().map(|item| item.name.clone()).collect(),
        "entity_name" => match request
            .context
            .as_ref()
            .and_then(|context| {
                context
                    .get_argument("entity_kind")
                    .or_else(|| context.get_argument("kind"))
            })
            .map(String::as_str)
        {
            Some("enum") => schema.enums.iter().map(|item| item.name.clone()).collect(),
            Some("struct") => schema
                .structs
                .iter()
                .map(|item| item.name.clone())
                .collect(),
            Some("union") => schema.unions.iter().map(|item| item.name.clone()).collect(),
            Some("table") => schema.tables.iter().map(|item| item.name.clone()).collect(),
            _ => schema
                .enums
                .iter()
                .map(|item| item.name.clone())
                .chain(schema.structs.iter().map(|item| item.name.clone()))
                .chain(schema.unions.iter().map(|item| item.name.clone()))
                .chain(schema.tables.iter().map(|item| item.name.clone()))
                .collect(),
        },
        "view" => inspection.views,
        "group" => inspection.groups,
        "source" => inspection
            .schema_sources
            .into_iter()
            .chain(inspection.view_sources)
            .chain(
                inspection
                    .data_sources
                    .into_iter()
                    .map(|source| source.file),
            )
            .collect(),
        "target" | "codegen_target" => workspace.supported_codegen_targets(),
        "runtime_format" => request
            .context
            .as_ref()
            .and_then(|context| {
                context
                    .get_argument("codegen_target")
                    .or_else(|| context.get_argument("target"))
            })
            .map(|target| workspace.supported_runtime_formats(target))
            .unwrap_or_else(|| {
                vec![
                    "cbor".to_owned(),
                    "json".to_owned(),
                    "sora".to_owned(),
                    "sora-protobuf".to_owned(),
                ]
            }),
        "format" | "export_format" => inspection.export_formats,
        "locale" => inspection
            .localization
            .map(|localization| localization.locales)
            .unwrap_or_default(),
        "kind" | "entity_kind" => vec![
            "enum".to_owned(),
            "struct".to_owned(),
            "union".to_owned(),
            "table".to_owned(),
        ],
        "mode" => vec!["list".to_owned(), "map".to_owned(), "singleton".to_owned()],
        "artifact" | "artifact_id" => artifacts
            .list_ids(authorization_context, &id)
            .map_err(|error| McpError::internal_error(error.to_string(), None))?,
        _ => Vec::new(),
    };
    Ok(values)
}

fn fuzzy_rank(value: &str, query: &str) -> Option<u8> {
    if query.is_empty() {
        return Some(0);
    }
    let value = value.to_lowercase();
    if value.starts_with(query) {
        return Some(0);
    }
    if value.contains(query) {
        return Some(1);
    }
    let mut query_chars = query.chars();
    let mut current = query_chars.next();
    for character in value.chars() {
        if current == Some(character) {
            current = query_chars.next();
            if current.is_none() {
                return Some(2);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fuzzy_matching_prefers_prefix_then_substring_then_subsequence() {
        assert_eq!(fuzzy_rank("Settings", "set"), Some(0));
        assert_eq!(fuzzy_rank("GameSettings", "set"), Some(1));
        assert_eq!(fuzzy_rank("SchemaEntity", "sme"), Some(2));
        assert_eq!(fuzzy_rank("Table", "xyz"), None);
    }
}
