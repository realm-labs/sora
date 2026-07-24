use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::{Context, Result, bail};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sora_data::model::{ConfigData, Value};
use sora_input_schema::input::SchemaFileInput;

use crate::{
    Diagnostic, ProjectRevision, ProjectSession, SourceFormat, diagnostics_from_anyhow,
    source::MixedProjectInput,
};

#[derive(Debug, Clone, Default, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DataValidationQuery {
    pub scope: Option<String>,
    #[serde(default)]
    pub tables: Vec<String>,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct DataValidationReport {
    pub ok: bool,
    pub revision: ProjectRevision,
    pub validated_tables: Vec<String>,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TableFilter {
    pub field: String,
    pub equals: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct IndexLookup {
    pub index: String,
    pub values: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TableQuery {
    pub table: String,
    #[serde(default)]
    pub filters: Vec<TableFilter>,
    pub key: Option<serde_json::Value>,
    pub index: Option<IndexLookup>,
    #[serde(default)]
    pub select: Vec<String>,
    #[serde(default)]
    pub order_by: Vec<String>,
    pub cursor: Option<String>,
    pub limit: Option<usize>,
    pub locale: Option<String>,
    pub include_derived: Option<bool>,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct TableQueryReport {
    pub revision: ProjectRevision,
    pub table: String,
    pub rows: Vec<BTreeMap<String, serde_json::Value>>,
    pub next_cursor: Option<String>,
    pub total_matched: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct QueryCursor {
    revision: String,
    table: String,
    query_hash: String,
    offset: usize,
}

impl ProjectSession {
    pub fn validated_data(&self) -> Result<(sora_ir::model::ConfigIr, ConfigData)> {
        self.load_data_from_root(&self.data_root())
    }

    pub fn validate_data(&self, query: &DataValidationQuery) -> DataValidationReport {
        match self.validated_data().and_then(|(ir, _)| {
            selected_tables(&ir, query).map(|tables| {
                tables
                    .into_iter()
                    .map(|table| table.name.clone())
                    .collect::<Vec<_>>()
            })
        }) {
            Ok(validated_tables) => DataValidationReport {
                ok: true,
                revision: self.revision(),
                validated_tables,
                diagnostics: Vec::new(),
            },
            Err(error) => DataValidationReport {
                ok: false,
                revision: self.revision(),
                validated_tables: Vec::new(),
                diagnostics: diagnostics_from_anyhow(&error),
            },
        }
    }

    pub fn query_table(&self, query: &TableQuery) -> Result<TableQueryReport> {
        if let Some(limit) = query.limit
            && !(1..=500).contains(&limit)
        {
            bail!("table query limit must be between 1 and 500");
        }
        let (ir, data) = self.validated_data()?;
        let table_ir = ir
            .tables
            .iter()
            .find(|table| table.name == query.table)
            .ok_or_else(|| anyhow::anyhow!("unknown table `{}`", query.table))?;
        if let Some(locale) = &query.locale
            && ir
                .localization
                .as_ref()
                .is_none_or(|localization| !localization.locales.contains(locale))
        {
            bail!("unknown project locale `{locale}`");
        }
        validate_query_fields(table_ir, query)?;
        let table_data = data
            .tables
            .iter()
            .find(|table| table.name == query.table)
            .ok_or_else(|| anyhow::anyhow!("validated data omitted table `{}`", query.table))?;
        let revision = self.revision();
        let query_hash = query_hash(query)?;
        let offset = match query.cursor.as_deref() {
            Some(cursor) => decode_cursor(cursor, &revision.project, &query.table, &query_hash)?,
            None => 0,
        };
        let include_derived = query.include_derived.unwrap_or(true);
        let derived = table_ir
            .fields
            .iter()
            .filter(|field| field.derived_from.is_some())
            .map(|field| field.name.as_str())
            .collect::<BTreeSet<_>>();
        let mut rows = table_data
            .rows
            .iter()
            .map(|row| {
                row.values
                    .iter()
                    .filter(|(field, _)| include_derived || !derived.contains(field.as_str()))
                    .map(|(field, value)| (field.clone(), natural_value(value)))
                    .collect::<BTreeMap<_, _>>()
            })
            .filter(|row| row_matches(row, table_ir, query))
            .collect::<Vec<_>>();
        if !query.order_by.is_empty() {
            rows.sort_by(|left, right| {
                query
                    .order_by
                    .iter()
                    .map(|field| stable_json(left.get(field)))
                    .cmp(
                        query
                            .order_by
                            .iter()
                            .map(|field| stable_json(right.get(field))),
                    )
            });
        }
        let total_matched = rows.len();
        let limit = query.limit.unwrap_or(50);
        let end = offset.saturating_add(limit).min(total_matched);
        let page = rows
            .get(offset..end)
            .unwrap_or(&[])
            .iter()
            .map(|row| project_row(row, table_ir, &query.select))
            .collect();
        let next_cursor = (end < total_matched)
            .then(|| encode_cursor(&revision.project, &query.table, &query_hash, end))
            .transpose()?;
        Ok(TableQueryReport {
            revision,
            table: query.table.clone(),
            rows: page,
            next_cursor,
            total_matched,
        })
    }

    pub fn diff_data_root(&self, relative_other_root: &str) -> Result<serde_json::Value> {
        let project_root = self
            .manifest_path()
            .parent()
            .unwrap_or_else(|| Path::new("."));
        let other = Path::new(relative_other_root);
        if other.is_absolute()
            || other
                .components()
                .any(|component| !matches!(component, std::path::Component::Normal(_)))
        {
            bail!("other data root must be a project-relative directory");
        }
        let other = project_root.join(other);
        let canonical = other
            .canonicalize()
            .with_context(|| format!("failed to resolve other data root `{}`", other.display()))?;
        let project_root = project_root.canonicalize()?;
        if !canonical.starts_with(&project_root) || !canonical.is_dir() {
            bail!("other data root must resolve inside the project");
        }
        let (ir, current) = self.validated_data()?;
        let (_, other) = self.load_data_from_root(&canonical)?;
        let diff = sora_core::diff::diff_config_data(&ir, &other, &current)?;
        Ok(serde_json::to_value(diff)?)
    }

    fn data_root(&self) -> PathBuf {
        let project_root = self
            .manifest_path()
            .parent()
            .unwrap_or_else(|| Path::new("."));
        let configured = self
            .manifest()
            .build
            .data_root
            .as_deref()
            .unwrap_or_else(|| Path::new("data"));
        if configured.is_absolute() {
            configured.to_path_buf()
        } else {
            project_root.join(configured)
        }
    }

    fn load_data_from_root(
        &self,
        data_root: &Path,
    ) -> Result<(sora_ir::model::ConfigIr, ConfigData)> {
        let default_format = self
            .manifest()
            .build
            .default_source_format
            .map(SourceFormat::as_str);
        let input = MixedProjectInput::with_parser_registry(
            SchemaFileInput::new(self.manifest_path()),
            data_root,
            default_format,
            Arc::clone(self.runtime().cell_parsers()),
        );
        sora_core::pipeline::load_project_data_with_context_and_parsers(
            &input,
            self.runtime().execution(),
            self.runtime().schema_parsers(),
            self.runtime().cell_parsers(),
        )
        .with_context(|| format!("failed to load data from `{}`", data_root.display()))
    }
}

fn selected_tables<'a>(
    ir: &'a sora_ir::model::ConfigIr,
    query: &DataValidationQuery,
) -> Result<Vec<&'a sora_ir::model::TableIr>> {
    let mut tables = ir.tables.iter().collect::<Vec<_>>();
    if let Some(scope) = &query.scope {
        tables.retain(|table| table.scope.includes(scope));
    }
    if !query.tables.is_empty() {
        for name in &query.tables {
            if !ir.tables.iter().any(|table| &table.name == name) {
                bail!("unknown table `{name}`");
            }
        }
        tables.retain(|table| query.tables.contains(&table.name));
    }
    Ok(tables)
}

fn validate_query_fields(table: &sora_ir::model::TableIr, query: &TableQuery) -> Result<()> {
    let known = table
        .fields
        .iter()
        .map(|field| field.name.as_str())
        .collect::<BTreeSet<_>>();
    for field in query
        .select
        .iter()
        .chain(query.order_by.iter())
        .chain(query.filters.iter().map(|filter| &filter.field))
    {
        if !known.contains(field.as_str()) {
            bail!("unknown field `{field}` in table `{}`", table.name);
        }
    }
    if query.key.is_some() && table.key.is_none() {
        bail!("table `{}` does not support key lookup", table.name);
    }
    if let Some(index) = &query.index {
        let definition = table
            .indexes
            .iter()
            .find(|item| item.name == index.index)
            .ok_or_else(|| anyhow::anyhow!("unknown index `{}`", index.index))?;
        if definition.fields.len() != index.values.len() {
            bail!(
                "index `{}` expects {} values, got {}",
                index.index,
                definition.fields.len(),
                index.values.len()
            );
        }
    }
    Ok(())
}

fn row_matches(
    row: &BTreeMap<String, serde_json::Value>,
    table: &sora_ir::model::TableIr,
    query: &TableQuery,
) -> bool {
    if let Some(key) = &query.key
        && table
            .key
            .as_ref()
            .and_then(|field| row.get(field))
            .is_none_or(|value| value != key)
    {
        return false;
    }
    if query
        .filters
        .iter()
        .any(|filter| row.get(&filter.field) != Some(&filter.equals))
    {
        return false;
    }
    if let Some(index) = &query.index {
        let definition = table
            .indexes
            .iter()
            .find(|item| item.name == index.index)
            .expect("validated index must exist");
        if definition
            .fields
            .iter()
            .zip(&index.values)
            .any(|(field, expected)| row.get(field) != Some(expected))
        {
            return false;
        }
    }
    true
}

fn project_row(
    row: &BTreeMap<String, serde_json::Value>,
    table: &sora_ir::model::TableIr,
    select: &[String],
) -> BTreeMap<String, serde_json::Value> {
    let fields = if select.is_empty() {
        table
            .fields
            .iter()
            .map(|field| field.name.as_str())
            .collect::<Vec<_>>()
    } else {
        select.iter().map(String::as_str).collect()
    };
    fields
        .into_iter()
        .filter_map(|field| {
            row.get(field)
                .cloned()
                .map(|value| (field.to_owned(), value))
        })
        .collect()
}

fn natural_value(value: &Value) -> serde_json::Value {
    match value {
        Value::Bool(value) => (*value).into(),
        Value::Integer(value) => (*value).into(),
        Value::Float(value) => serde_json::Number::from_f64(*value)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null),
        Value::String(value) => value.clone().into(),
        Value::List(values) => values.iter().map(natural_value).collect(),
        Value::Object(values) => values
            .iter()
            .map(|(key, value)| (key.clone(), natural_value(value)))
            .collect(),
        Value::Null => serde_json::Value::Null,
    }
}

fn query_hash(query: &TableQuery) -> Result<String> {
    let mut stable = query.clone();
    stable.cursor = None;
    let bytes = serde_json::to_vec(&stable)?;
    let digest = Sha256::digest(bytes);
    Ok(digest.iter().map(|byte| format!("{byte:02x}")).collect())
}

fn encode_cursor(revision: &str, table: &str, query_hash: &str, offset: usize) -> Result<String> {
    Ok(URL_SAFE_NO_PAD.encode(serde_json::to_vec(&QueryCursor {
        revision: revision.to_owned(),
        table: table.to_owned(),
        query_hash: query_hash.to_owned(),
        offset,
    })?))
}

fn decode_cursor(cursor: &str, revision: &str, table: &str, query_hash: &str) -> Result<usize> {
    let bytes = URL_SAFE_NO_PAD
        .decode(cursor)
        .context("invalid table query cursor encoding")?;
    let cursor: QueryCursor =
        serde_json::from_slice(&bytes).context("invalid table query cursor payload")?;
    if cursor.revision != revision || cursor.table != table || cursor.query_hash != query_hash {
        bail!("table query cursor does not match the current revision or query");
    }
    Ok(cursor.offset)
}

fn stable_json(value: Option<&serde_json::Value>) -> String {
    value
        .and_then(|value| serde_json::to_string(value).ok())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ProjectId, RuntimeOptions};

    #[test]
    fn query_paginates_with_revision_bound_cursor() {
        let project =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/showcase/project.toml");
        let session = ProjectSession::open(
            ProjectId::new("showcase").unwrap(),
            project,
            RuntimeOptions::default(),
        )
        .unwrap();
        let first = session
            .query_table(&TableQuery {
                table: "Item".to_owned(),
                select: vec!["id".to_owned(), "name".to_owned()],
                limit: Some(1),
                ..TableQuery::default()
            })
            .unwrap();
        assert_eq!(first.rows.len(), 1);
        assert_eq!(first.rows[0].len(), 2);
        let second = session
            .query_table(&TableQuery {
                table: "Item".to_owned(),
                select: vec!["id".to_owned(), "name".to_owned()],
                limit: Some(1),
                cursor: first.next_cursor,
                ..TableQuery::default()
            })
            .unwrap();
        assert_eq!(second.rows.len(), 1);
        assert_ne!(first.rows, second.rows);
    }

    #[test]
    fn query_rejects_unknown_projection_fields() {
        let project =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/showcase/project.toml");
        let session = ProjectSession::open(
            ProjectId::new("showcase").unwrap(),
            project,
            RuntimeOptions::default(),
        )
        .unwrap();
        let error = session
            .query_table(&TableQuery {
                table: "Item".to_owned(),
                select: vec!["password".to_owned()],
                ..TableQuery::default()
            })
            .unwrap_err();
        assert!(error.to_string().contains("unknown field `password`"));
    }

    #[test]
    fn query_rejects_limits_outside_the_resource_contract() {
        let project =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/showcase/project.toml");
        let session = ProjectSession::open(
            ProjectId::new("showcase").unwrap(),
            project,
            RuntimeOptions::default(),
        )
        .unwrap();
        for limit in [0, 501] {
            let error = session
                .query_table(&TableQuery {
                    table: "Item".to_owned(),
                    limit: Some(limit),
                    ..TableQuery::default()
                })
                .unwrap_err();
            assert!(
                error
                    .to_string()
                    .contains("table query limit must be between 1 and 500")
            );
        }
    }
}
