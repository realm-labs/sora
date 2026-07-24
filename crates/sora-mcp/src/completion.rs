use rmcp::{
    ErrorData as McpError,
    model::{CompleteRequestParams, CompleteResult, CompletionInfo},
};
use sora_workspace::{ProjectId, WorkspaceService};

pub fn complete(
    workspace: &WorkspaceService,
    request: &CompleteRequestParams,
) -> Result<CompleteResult, McpError> {
    let name = request.argument.name.as_str();
    let mut values = if name == "project_id" {
        workspace
            .project_ids()
            .map_err(|error| McpError::internal_error(error.to_string(), None))?
            .into_iter()
            .map(|id| id.as_str().to_owned())
            .collect()
    } else {
        project_values(workspace, request, name)?
    };
    let query = request.argument.value.to_lowercase();
    values.retain(|value| value.to_lowercase().contains(&query));
    values.sort_by(|left, right| {
        let left_prefix = left.to_lowercase().starts_with(&query);
        let right_prefix = right.to_lowercase().starts_with(&query);
        right_prefix.cmp(&left_prefix).then_with(|| left.cmp(right))
    });
    values.dedup();
    let total = values.len();
    values.truncate(CompletionInfo::MAX_VALUES);
    let completion = CompletionInfo::with_pagination(values, Some(total as u32), total > 100)
        .map_err(|error| McpError::internal_error(error, None))?;
    Ok(CompleteResult::new(completion))
}

fn project_values(
    workspace: &WorkspaceService,
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
        "scope" => inspection.scopes,
        "source" => inspection
            .schema_sources
            .into_iter()
            .chain(
                inspection
                    .data_sources
                    .into_iter()
                    .map(|source| source.file),
            )
            .collect(),
        "target" | "codegen_target" => inspection.codegen_targets,
        "format" | "export_format" => inspection.export_formats,
        "locale" => inspection
            .localization
            .map(|localization| localization.locales)
            .unwrap_or_default(),
        "kind" => vec![
            "enum".to_owned(),
            "struct".to_owned(),
            "union".to_owned(),
            "table".to_owned(),
        ],
        _ => Vec::new(),
    };
    Ok(values)
}
