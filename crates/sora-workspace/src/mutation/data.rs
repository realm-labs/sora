use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    sync::Arc,
};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sora_data::model::{
    ConfigData, LocalizationData, LocalizationRowData, RowData, TableData, Value,
};
use sora_input::{
    source::{
        SourceFormat, resolve_localization_source_format_with_registry,
        resolve_table_source_format_with_registry,
    },
    traits::DataInput,
};
use sora_input_schema::input::ProjectSchemaInput;
use sora_ir::{
    input_projection::{TaggedColumnKind, struct_columns, tagged_columns, tagged_columns_union},
    model::{ConfigIr, FieldIr, TableIr, TableModeIr, TypeIr},
};

use super::FileWrite;
use crate::{ProjectSession, source::MixedProjectInput};

/// Closed set of supported row selectors.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum RowSelector {
    Map {
        key: serde_json::Value,
    },
    Singleton,
    List {
        index: usize,
        expected_row_hash: String,
    },
}

/// Closed set of data mutation operations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum DataOperation {
    InsertRow {
        table: String,
        row: BTreeMap<String, serde_json::Value>,
        index: Option<usize>,
    },
    UpsertRow {
        table: String,
        selector: RowSelector,
        row: BTreeMap<String, serde_json::Value>,
    },
    UpdateFields {
        table: String,
        selector: RowSelector,
        fields: BTreeMap<String, serde_json::Value>,
    },
    DeleteRow {
        table: String,
        selector: RowSelector,
    },
    MoveListRow {
        table: String,
        selector: RowSelector,
        to_index: usize,
    },
    UpsertLocalization {
        source: String,
        key: String,
        values: BTreeMap<String, String>,
    },
    UpdateLocalization {
        source: String,
        key: String,
        locale: String,
        value: String,
    },
    DeleteLocalization {
        source: String,
        key: String,
    },
}

/// One logical row change produced by the pure data executor.
#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
pub struct RowChange {
    pub table: String,
    pub kind: String,
    pub before_index: Option<usize>,
    pub after_index: Option<usize>,
    pub before: Option<BTreeMap<String, serde_json::Value>>,
    pub after: Option<BTreeMap<String, serde_json::Value>>,
}

/// One logical localization entry change.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct LocalizationChange {
    pub source: String,
    pub key: String,
    pub locale: Option<String>,
    pub before: Option<String>,
    pub after: Option<String>,
}

/// Physical source coordinates affected by a logical row change.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct DataSourceImpact {
    pub path: String,
    pub sheet: Option<String>,
    pub first_row: Option<usize>,
    pub fields: Vec<String>,
}

/// Result of applying an operation batch entirely in memory.
#[derive(Debug, Clone)]
pub struct DataExecution {
    pub data: ConfigData,
    pub localization: LocalizationData,
    pub changes: Vec<RowChange>,
    pub localization_changes: Vec<LocalizationChange>,
    pub affected_tables: BTreeSet<String>,
    pub affected_localization_sources: BTreeSet<String>,
}

/// Data mutation execution or source-rendering failure.
#[derive(Debug, thiserror::Error)]
pub enum DataMutationError {
    #[error("unknown table `{0}`")]
    UnknownTable(String),
    #[error("table `{table}` does not support selector `{selector}`")]
    SelectorMode { table: String, selector: String },
    #[error("row selector did not match table `{0}`")]
    RowNotFound(String),
    #[error("row selector is ambiguous for table `{0}`")]
    AmbiguousSelector(String),
    #[error("list selector hash does not match the current row in table `{0}`")]
    RowHashConflict(String),
    #[error("row index {index} is outside table `{table}` with {rows} rows")]
    IndexOutOfBounds {
        table: String,
        index: usize,
        rows: usize,
    },
    #[error("field `{field}` is unknown in table `{table}`")]
    UnknownField { table: String, field: String },
    #[error("derived field `{field}` in table `{table}` cannot be written")]
    DerivedField { table: String, field: String },
    #[error("invalid typed JSON value for `{table}.{field}`: {message}")]
    InvalidValue {
        table: String,
        field: String,
        message: String,
    },
    #[error("table `{0}` does not declare a mutable source")]
    MissingSource(String),
    #[error("unknown localization source `{0}`")]
    UnknownLocalizationSource(String),
    #[error("unknown localization locale `{0}`")]
    UnknownLocale(String),
    #[error("localization key `{key}` was not found in source `{localization_source}`")]
    LocalizationKeyNotFound {
        localization_source: String,
        key: String,
    },
    #[error("data source format `{0}` is not mutable")]
    UnsupportedFormat(String),
    #[error("directory data sources are not mutable yet: `{0}`")]
    DirectorySource(PathBuf),
    #[error("failed to render data source: {0}")]
    Render(String),
}

/// Writer contract for one validated mutable table source.
pub trait MutableTableSource {
    /// Renders the complete next source without modifying the filesystem.
    fn render(
        &self,
        ir: &ConfigIr,
        table: &TableIr,
        rows: &[RowData],
        source_path: &Path,
    ) -> Result<Vec<u8>, DataMutationError>;
}

/// Applies a data operation batch without performing filesystem I/O.
pub fn execute_data_operations(
    ir: &ConfigIr,
    base: &ConfigData,
    base_localization: &LocalizationData,
    operations: &[DataOperation],
) -> Result<DataExecution, DataMutationError> {
    let mut data = strip_materialized_fields(ir, base);
    let mut localization = base_localization.clone();
    let mut changes = Vec::with_capacity(operations.len());
    let mut localization_changes = Vec::new();
    let mut affected_tables = BTreeSet::new();
    let mut affected_localization_sources = BTreeSet::new();
    for operation in operations {
        if let Some(table_name) = operation_table(operation) {
            let table = ir
                .tables
                .iter()
                .find(|candidate| candidate.name == table_name)
                .ok_or_else(|| DataMutationError::UnknownTable(table_name.to_owned()))?;
            let table_data = data
                .tables
                .iter_mut()
                .find(|candidate| candidate.name == table_name)
                .ok_or_else(|| DataMutationError::UnknownTable(table_name.to_owned()))?;
            changes.push(execute_operation(table, table_data, operation)?);
            affected_tables.insert(table_name.to_owned());
        } else {
            localization_changes.extend(execute_localization_operation(
                ir,
                &mut localization,
                operation,
            )?);
            if let Some(source) = operation_localization_source(operation) {
                affected_localization_sources.insert(source.to_owned());
            }
        }
    }
    Ok(DataExecution {
        data,
        localization,
        changes,
        localization_changes,
        affected_tables,
        affected_localization_sources,
    })
}

pub(crate) fn load_raw_project_data(
    session: &ProjectSession,
) -> anyhow::Result<(ConfigIr, ConfigData, LocalizationData)> {
    let schema_input = ProjectSchemaInput::new(session.manifest_path());
    let ir = sora_core::pipeline::load_schema_ir_with_parsers(
        &schema_input,
        session.runtime().schema_parsers(),
    )?;
    let input = MixedProjectInput::with_source_registry(
        schema_input,
        session.data_root(),
        session
            .manifest()
            .build
            .default_source_format
            .map(crate::SourceFormat::as_str),
        Arc::clone(session.runtime().source_registry()),
        Arc::clone(session.runtime().cell_parsers()),
    );
    let data = input.load_data_with_context(&ir, session.runtime().execution())?;
    let localization =
        input.load_localization_data_with_context(&ir, session.runtime().execution())?;
    Ok((ir, data, localization))
}

pub(crate) fn validate_mutated_data(
    session: &ProjectSession,
    ir: &ConfigIr,
    data: &ConfigData,
    localization: &LocalizationData,
) -> anyhow::Result<ConfigData> {
    let materialized = sora_input::defaults::materialize_defaults_with_parsers(
        ir,
        data,
        session.runtime().cell_parsers(),
    )?;
    let materialized = sora_data::derived::materialize_derived_fields(ir, &materialized)?;
    sora_data::validate::validate_config_data(ir, &materialized)?;
    sora_data::localization::build_locale_catalog(ir, &materialized, localization)?;
    Ok(materialized)
}

pub(crate) fn render_data_writes(
    session: &ProjectSession,
    ir: &ConfigIr,
    data: &ConfigData,
    localization: &LocalizationData,
    affected_tables: &BTreeSet<String>,
    affected_localization_sources: &BTreeSet<String>,
) -> Result<(Vec<FileWrite>, Vec<DataSourceImpact>), DataMutationError> {
    let mut writes = Vec::new();
    let mut impacts = Vec::new();
    let data_root = session.data_root();
    let mut xlsx_tables = BTreeMap::<PathBuf, Vec<(&TableIr, &[RowData])>>::new();
    let mut xlsx_localization = BTreeMap::<
        PathBuf,
        Vec<(
            &sora_ir::model::LocalizationSourceIr,
            &[LocalizationRowData],
        )>,
    >::new();
    for table in &ir.tables {
        if !affected_tables.contains(&table.name) {
            continue;
        }
        let table_data = data
            .tables
            .iter()
            .find(|candidate| candidate.name == table.name)
            .ok_or_else(|| DataMutationError::UnknownTable(table.name.clone()))?;
        let source = table
            .source
            .as_ref()
            .ok_or_else(|| DataMutationError::MissingSource(table.name.clone()))?;
        let path = data_root.join(&source.file);
        let format = resolve_table_source_format_with_registry(
            table,
            session
                .manifest()
                .build
                .default_source_format
                .map(crate::SourceFormat::as_str),
            session.runtime().source_registry(),
        )
        .map_err(|error| DataMutationError::Render(error.to_string()))?;
        let format = SourceFormat::parse(format)
            .map_err(|_| DataMutationError::UnsupportedFormat(format.to_owned()))?;
        if path.is_dir() {
            writes.extend(render_directory_source(format, &path, &table_data.rows)?);
        } else if format == SourceFormat::Xlsx {
            xlsx_tables
                .entry(path.clone())
                .or_default()
                .push((table, &table_data.rows));
        } else {
            let writer = BuiltinTableWriter { format };
            writes.push(FileWrite {
                path: path.clone(),
                content: Some(writer.render(ir, table, &table_data.rows, &path)?),
            });
        }
        impacts.push(DataSourceImpact {
            path: relative_source_path(&data_root, &path),
            sheet: source.sheet.clone(),
            first_row: Some(source_data_start_row(format)),
            fields: table
                .fields
                .iter()
                .filter(|field| field.derived_from.is_none())
                .map(|field| field.name.clone())
                .collect(),
        });
    }
    if let Some(localization_ir) = &ir.localization {
        for source in &localization_ir.sources {
            if !affected_localization_sources.contains(&source.name) {
                continue;
            }
            let source_data = localization
                .sources
                .iter()
                .find(|candidate| candidate.name == source.name)
                .ok_or_else(|| {
                    DataMutationError::Render(format!(
                        "missing localization source data `{}`",
                        source.name
                    ))
                })?;
            let path = data_root.join(&source.file);
            let format = resolve_localization_source_format_with_registry(
                source,
                session
                    .manifest()
                    .build
                    .default_source_format
                    .map(crate::SourceFormat::as_str),
                session.runtime().source_registry(),
            )
            .map_err(|error| DataMutationError::Render(error.to_string()))?;
            let format = SourceFormat::parse(format)
                .map_err(|_| DataMutationError::UnsupportedFormat(format.to_owned()))?;
            if format == SourceFormat::Xlsx {
                xlsx_localization
                    .entry(path.clone())
                    .or_default()
                    .push((source, &source_data.rows));
            } else {
                writes.push(FileWrite {
                    path: path.clone(),
                    content: Some(render_localization_text(
                        format,
                        &source_data.columns,
                        &source_data.rows,
                    )?),
                });
            }
            impacts.push(DataSourceImpact {
                path: relative_source_path(&data_root, &path),
                sheet: source.sheet.clone(),
                first_row: Some(source_data_start_row(format)),
                fields: source_data.columns.clone(),
            });
        }
    }
    let xlsx_paths = xlsx_tables
        .keys()
        .chain(xlsx_localization.keys())
        .cloned()
        .collect::<BTreeSet<_>>();
    for path in xlsx_paths {
        let tables = xlsx_tables.remove(&path).unwrap_or_default();
        let localization = xlsx_localization.remove(&path).unwrap_or_default();
        writes.push(FileWrite {
            content: Some(render_xlsx(&path, ir, &tables, &localization)?),
            path,
        });
    }
    writes.sort_by(|left, right| left.path.cmp(&right.path));
    impacts.sort_by(|left, right| (&left.path, &left.sheet).cmp(&(&right.path, &right.sheet)));
    Ok((writes, impacts))
}

fn render_directory_source(
    format: SourceFormat,
    directory: &Path,
    rows: &[RowData],
) -> Result<Vec<FileWrite>, DataMutationError> {
    let extension = match format {
        SourceFormat::Json => "json",
        SourceFormat::Yaml => "yaml",
        _ => {
            return Err(DataMutationError::DirectorySource(directory.to_path_buf()));
        }
    };
    let mut files = Vec::new();
    collect_directory_files(directory, extension, &mut files)?;
    files.sort();
    let mut writes = Vec::new();
    for (index, row) in rows.iter().enumerate() {
        let path = files
            .get(index)
            .cloned()
            .unwrap_or_else(|| directory.join(format!("sora-{index:06}.{extension}")));
        let content = match format {
            SourceFormat::Json => {
                let mut content = serde_json::to_vec_pretty(&natural_row(row))
                    .map_err(|error| DataMutationError::Render(error.to_string()))?;
                content.push(b'\n');
                content
            }
            SourceFormat::Yaml => serde_yaml::to_string(&natural_row(row))
                .map(String::into_bytes)
                .map_err(|error| DataMutationError::Render(error.to_string()))?,
            _ => {
                return Err(DataMutationError::DirectorySource(directory.to_path_buf()));
            }
        };
        writes.push(FileWrite {
            path,
            content: Some(content),
        });
    }
    writes.extend(files.into_iter().skip(rows.len()).map(|path| FileWrite {
        path,
        content: None,
    }));
    Ok(writes)
}

fn collect_directory_files(
    directory: &Path,
    extension: &str,
    files: &mut Vec<PathBuf>,
) -> Result<(), DataMutationError> {
    for entry in std::fs::read_dir(directory)
        .map_err(|error| DataMutationError::Render(error.to_string()))?
    {
        let entry = entry.map_err(|error| DataMutationError::Render(error.to_string()))?;
        let path = entry.path();
        if path.is_dir() {
            collect_directory_files(&path, extension, files)?;
        } else if path.extension().and_then(|value| value.to_str()) == Some(extension) {
            files.push(path);
        }
    }
    Ok(())
}

struct BuiltinTableWriter {
    format: SourceFormat,
}

impl MutableTableSource for BuiltinTableWriter {
    fn render(
        &self,
        ir: &ConfigIr,
        table: &TableIr,
        rows: &[RowData],
        source_path: &Path,
    ) -> Result<Vec<u8>, DataMutationError> {
        match self.format {
            SourceFormat::Json => {
                let mut bytes =
                    serde_json::to_vec_pretty(&rows.iter().map(natural_row).collect::<Vec<_>>())
                        .map_err(|error| DataMutationError::Render(error.to_string()))?;
                bytes.push(b'\n');
                Ok(bytes)
            }
            SourceFormat::Yaml => {
                serde_yaml::to_string(&rows.iter().map(natural_row).collect::<Vec<_>>())
                    .map(String::into_bytes)
                    .map_err(|error| DataMutationError::Render(error.to_string()))
            }
            SourceFormat::Toml => {
                let value =
                    BTreeMap::from([("rows", rows.iter().map(toml_row).collect::<Vec<_>>())]);
                toml::to_string_pretty(&value)
                    .map(String::into_bytes)
                    .map_err(|error| DataMutationError::Render(error.to_string()))
            }
            SourceFormat::Csv => render_csv(ir, table, rows),
            SourceFormat::Xlsx => Err(DataMutationError::Render(format!(
                "XLSX source `{}` must be rendered as a workbook group",
                source_path.display()
            ))),
        }
    }
}

fn execute_operation(
    table: &TableIr,
    data: &mut TableData,
    operation: &DataOperation,
) -> Result<RowChange, DataMutationError> {
    match operation {
        DataOperation::InsertRow { row, index, .. } => {
            let row = input_row(table, row)?;
            let index = match table.mode {
                TableModeIr::List => index.unwrap_or(data.rows.len()),
                TableModeIr::Map => {
                    if index.is_some() {
                        return Err(DataMutationError::SelectorMode {
                            table: table.name.clone(),
                            selector: "insert index".to_owned(),
                        });
                    }
                    data.rows.len()
                }
                TableModeIr::Singleton => {
                    if !data.rows.is_empty() || index.is_some() {
                        return Err(DataMutationError::SelectorMode {
                            table: table.name.clone(),
                            selector: "insert".to_owned(),
                        });
                    }
                    0
                }
            };
            if index > data.rows.len() {
                return Err(DataMutationError::IndexOutOfBounds {
                    table: table.name.clone(),
                    index,
                    rows: data.rows.len(),
                });
            }
            data.rows.insert(index, row.clone());
            Ok(row_change(
                table,
                "insert",
                None,
                Some(index),
                None,
                Some(&row),
            ))
        }
        DataOperation::UpsertRow { selector, row, .. } => {
            let next = input_row(table, row)?;
            match resolve_selector(table, data, selector) {
                Ok(index) => {
                    let before = std::mem::replace(&mut data.rows[index], next.clone());
                    Ok(row_change(
                        table,
                        "upsert_update",
                        Some(index),
                        Some(index),
                        Some(&before),
                        Some(&next),
                    ))
                }
                Err(DataMutationError::RowNotFound(_)) if table.mode == TableModeIr::Map => {
                    let index = data.rows.len();
                    data.rows.push(next.clone());
                    Ok(row_change(
                        table,
                        "upsert_insert",
                        None,
                        Some(index),
                        None,
                        Some(&next),
                    ))
                }
                Err(error) => Err(error),
            }
        }
        DataOperation::UpdateFields {
            selector, fields, ..
        } => {
            validate_input_fields(table, fields)?;
            let index = resolve_selector(table, data, selector)?;
            let before = data.rows[index].clone();
            for (field, value) in fields {
                data.rows[index]
                    .values
                    .insert(field.clone(), json_to_value(table, field, value)?);
            }
            Ok(row_change(
                table,
                "update",
                Some(index),
                Some(index),
                Some(&before),
                Some(&data.rows[index]),
            ))
        }
        DataOperation::DeleteRow { selector, .. } => {
            let index = resolve_selector(table, data, selector)?;
            let before = data.rows.remove(index);
            Ok(row_change(
                table,
                "delete",
                Some(index),
                None,
                Some(&before),
                None,
            ))
        }
        DataOperation::MoveListRow {
            selector, to_index, ..
        } => {
            if table.mode != TableModeIr::List {
                return Err(DataMutationError::SelectorMode {
                    table: table.name.clone(),
                    selector: "move_list_row".to_owned(),
                });
            }
            let index = resolve_selector(table, data, selector)?;
            if *to_index >= data.rows.len() {
                return Err(DataMutationError::IndexOutOfBounds {
                    table: table.name.clone(),
                    index: *to_index,
                    rows: data.rows.len(),
                });
            }
            let row = data.rows.remove(index);
            data.rows.insert(*to_index, row.clone());
            Ok(row_change(
                table,
                "move",
                Some(index),
                Some(*to_index),
                Some(&row),
                Some(&row),
            ))
        }
        DataOperation::UpsertLocalization { .. }
        | DataOperation::UpdateLocalization { .. }
        | DataOperation::DeleteLocalization { .. } => Err(DataMutationError::Render(
            "localization operation reached the table executor".to_owned(),
        )),
    }
}

fn execute_localization_operation(
    ir: &ConfigIr,
    data: &mut LocalizationData,
    operation: &DataOperation,
) -> Result<Vec<LocalizationChange>, DataMutationError> {
    let source_name = operation_localization_source(operation).ok_or_else(|| {
        DataMutationError::Render("row operation reached localization executor".to_owned())
    })?;
    let localization = ir
        .localization
        .as_ref()
        .ok_or_else(|| DataMutationError::UnknownLocalizationSource(source_name.to_owned()))?;
    let source_ir = localization
        .sources
        .iter()
        .find(|source| source.name == source_name)
        .ok_or_else(|| DataMutationError::UnknownLocalizationSource(source_name.to_owned()))?;
    let source = data
        .sources
        .iter_mut()
        .find(|source| source.name == source_name)
        .ok_or_else(|| DataMutationError::UnknownLocalizationSource(source_name.to_owned()))?;
    let changes = match operation {
        DataOperation::UpsertLocalization { key, values, .. } => {
            let before = source
                .rows
                .iter()
                .find(|row| row.values.get(&source_ir.key) == Some(key))
                .map(|row| row.values.clone());
            for locale in values.keys() {
                if !localization.locales.contains(locale) {
                    return Err(DataMutationError::UnknownLocale(locale.clone()));
                }
            }
            if let Some(row) = source
                .rows
                .iter_mut()
                .find(|row| row.values.get(&source_ir.key) == Some(key))
            {
                row.values.extend(values.clone());
            } else {
                let mut row = values.clone();
                row.insert(source_ir.key.clone(), key.clone());
                source.rows.push(LocalizationRowData { values: row });
            }
            values
                .iter()
                .map(|(locale, value)| LocalizationChange {
                    source: source_name.to_owned(),
                    key: key.clone(),
                    locale: Some(locale.clone()),
                    before: before.as_ref().and_then(|row| row.get(locale)).cloned(),
                    after: Some(value.clone()),
                })
                .collect()
        }
        DataOperation::UpdateLocalization {
            key, locale, value, ..
        } => {
            if !localization.locales.contains(locale) {
                return Err(DataMutationError::UnknownLocale(locale.clone()));
            }
            let row = source
                .rows
                .iter_mut()
                .find(|row| row.values.get(&source_ir.key) == Some(key))
                .ok_or_else(|| DataMutationError::LocalizationKeyNotFound {
                    localization_source: source_name.to_owned(),
                    key: key.clone(),
                })?;
            let before = row.values.insert(locale.clone(), value.clone());
            vec![LocalizationChange {
                source: source_name.to_owned(),
                key: key.clone(),
                locale: Some(locale.clone()),
                before,
                after: Some(value.clone()),
            }]
        }
        DataOperation::DeleteLocalization { key, .. } => {
            let before = source
                .rows
                .iter()
                .find(|row| row.values.get(&source_ir.key) == Some(key))
                .map(|row| row.values.clone());
            source
                .rows
                .retain(|row| row.values.get(&source_ir.key) != Some(key));
            let Some(before) = before else {
                return Err(DataMutationError::LocalizationKeyNotFound {
                    localization_source: source_name.to_owned(),
                    key: key.clone(),
                });
            };
            vec![LocalizationChange {
                source: source_name.to_owned(),
                key: key.clone(),
                locale: None,
                before: Some(
                    serde_json::to_string(&before)
                        .map_err(|error| DataMutationError::Render(error.to_string()))?,
                ),
                after: None,
            }]
        }
        DataOperation::InsertRow { .. }
        | DataOperation::UpsertRow { .. }
        | DataOperation::UpdateFields { .. }
        | DataOperation::DeleteRow { .. }
        | DataOperation::MoveListRow { .. } => {
            return Err(DataMutationError::Render(
                "row operation reached localization executor".to_owned(),
            ));
        }
    };
    Ok(changes)
}

fn resolve_selector(
    table: &TableIr,
    data: &TableData,
    selector: &RowSelector,
) -> Result<usize, DataMutationError> {
    match (table.mode, selector) {
        (TableModeIr::Map, RowSelector::Map { key }) => {
            let key_field = table
                .key
                .as_ref()
                .ok_or_else(|| DataMutationError::SelectorMode {
                    table: table.name.clone(),
                    selector: "map key".to_owned(),
                })?;
            let key = json_to_value(table, key_field, key)?;
            unique_match(
                table,
                data.rows
                    .iter()
                    .enumerate()
                    .filter(|(_, row)| row.values.get(key_field) == Some(&key))
                    .map(|(index, _)| index),
            )
        }
        (TableModeIr::Singleton, RowSelector::Singleton) => {
            if data.rows.len() == 1 {
                Ok(0)
            } else {
                Err(DataMutationError::RowNotFound(table.name.clone()))
            }
        }
        (
            TableModeIr::List,
            RowSelector::List {
                index,
                expected_row_hash,
            },
        ) => {
            let row = data
                .rows
                .get(*index)
                .ok_or_else(|| DataMutationError::IndexOutOfBounds {
                    table: table.name.clone(),
                    index: *index,
                    rows: data.rows.len(),
                })?;
            if data_row_hash(row) == *expected_row_hash {
                Ok(*index)
            } else {
                Err(DataMutationError::RowHashConflict(table.name.clone()))
            }
        }
        (_, selector) => Err(DataMutationError::SelectorMode {
            table: table.name.clone(),
            selector: selector_name(selector).to_owned(),
        }),
    }
}

fn unique_match(
    table: &TableIr,
    mut matches: impl Iterator<Item = usize>,
) -> Result<usize, DataMutationError> {
    let Some(first) = matches.next() else {
        return Err(DataMutationError::RowNotFound(table.name.clone()));
    };
    if matches.next().is_some() {
        Err(DataMutationError::AmbiguousSelector(table.name.clone()))
    } else {
        Ok(first)
    }
}

fn input_row(
    table: &TableIr,
    row: &BTreeMap<String, serde_json::Value>,
) -> Result<RowData, DataMutationError> {
    validate_input_fields(table, row)?;
    Ok(RowData {
        values: row
            .iter()
            .map(|(field, value)| Ok((field.clone(), json_to_value(table, field, value)?)))
            .collect::<Result<_, DataMutationError>>()?,
    })
}

fn validate_input_fields<T>(
    table: &TableIr,
    values: &BTreeMap<String, T>,
) -> Result<(), DataMutationError> {
    for field in values.keys() {
        let definition = table
            .fields
            .iter()
            .find(|candidate| candidate.name == *field)
            .ok_or_else(|| DataMutationError::UnknownField {
                table: table.name.clone(),
                field: field.clone(),
            })?;
        if definition.derived_from.is_some() {
            return Err(DataMutationError::DerivedField {
                table: table.name.clone(),
                field: field.clone(),
            });
        }
    }
    Ok(())
}

fn json_to_value(
    table: &TableIr,
    field: &str,
    value: &serde_json::Value,
) -> Result<Value, DataMutationError> {
    natural_json_to_value(value).map_err(|message| DataMutationError::InvalidValue {
        table: table.name.clone(),
        field: field.to_owned(),
        message,
    })
}

fn natural_json_to_value(value: &serde_json::Value) -> Result<Value, String> {
    Ok(match value {
        serde_json::Value::Null => Value::Null,
        serde_json::Value::Bool(value) => Value::Bool(*value),
        serde_json::Value::Number(value) => {
            if let Some(value) = value.as_i64() {
                Value::Integer(value)
            } else if let Some(value) = value.as_f64() {
                Value::Float(value)
            } else {
                return Err("number is outside the supported range".to_owned());
            }
        }
        serde_json::Value::String(value) => Value::String(value.clone()),
        serde_json::Value::Array(values) => Value::List(
            values
                .iter()
                .map(natural_json_to_value)
                .collect::<Result<_, _>>()?,
        ),
        serde_json::Value::Object(values) => Value::Object(
            values
                .iter()
                .map(|(key, value)| Ok((key.clone(), natural_json_to_value(value)?)))
                .collect::<Result<_, String>>()?,
        ),
    })
}

fn strip_materialized_fields(ir: &ConfigIr, base: &ConfigData) -> ConfigData {
    let mut data = base.clone();
    for table in &ir.tables {
        let materialized = table
            .fields
            .iter()
            .filter(|field| field.derived_from.is_some())
            .map(|field| field.name.as_str())
            .collect::<BTreeSet<_>>();
        if let Some(table_data) = data.tables.iter_mut().find(|item| item.name == table.name) {
            for row in &mut table_data.rows {
                row.values
                    .retain(|field, _| !materialized.contains(field.as_str()));
            }
        }
    }
    data
}

fn render_csv(
    ir: &ConfigIr,
    table: &TableIr,
    rows: &[RowData],
) -> Result<Vec<u8>, DataMutationError> {
    let columns = projected_columns(ir, table);
    let mut writer = csv::WriterBuilder::new().from_writer(Vec::new());
    writer
        .write_record(columns.iter().map(|column| column.name.as_str()))
        .map_err(|error| DataMutationError::Render(error.to_string()))?;
    for row in rows {
        writer
            .write_record(columns.iter().map(|column| projected_cell(ir, row, column)))
            .map_err(|error| DataMutationError::Render(error.to_string()))?;
    }
    writer
        .into_inner()
        .map_err(|error| DataMutationError::Render(error.to_string()))
}

fn render_localization_text(
    format: SourceFormat,
    source_columns: &[String],
    rows: &[LocalizationRowData],
) -> Result<Vec<u8>, DataMutationError> {
    match format {
        SourceFormat::Json => {
            let mut bytes =
                serde_json::to_vec_pretty(&rows.iter().map(|row| &row.values).collect::<Vec<_>>())
                    .map_err(|error| DataMutationError::Render(error.to_string()))?;
            bytes.push(b'\n');
            Ok(bytes)
        }
        SourceFormat::Yaml => {
            serde_yaml::to_string(&rows.iter().map(|row| &row.values).collect::<Vec<_>>())
                .map(String::into_bytes)
                .map_err(|error| DataMutationError::Render(error.to_string()))
        }
        SourceFormat::Toml => toml::to_string_pretty(&BTreeMap::from([(
            "rows",
            rows.iter().map(|row| &row.values).collect::<Vec<_>>(),
        )]))
        .map(String::into_bytes)
        .map_err(|error| DataMutationError::Render(error.to_string())),
        SourceFormat::Csv => {
            let columns = source_columns;
            let mut writer = csv::WriterBuilder::new().from_writer(Vec::new());
            writer
                .write_record(columns)
                .map_err(|error| DataMutationError::Render(error.to_string()))?;
            for row in rows {
                writer
                    .write_record(
                        columns
                            .iter()
                            .map(|column| row.values.get(column).map(String::as_str).unwrap_or("")),
                    )
                    .map_err(|error| DataMutationError::Render(error.to_string()))?;
            }
            writer
                .into_inner()
                .map_err(|error| DataMutationError::Render(error.to_string()))
        }
        SourceFormat::Xlsx => Err(DataMutationError::Render(
            "XLSX localization must be rendered as a workbook group".to_owned(),
        )),
    }
}

#[derive(Clone, Copy)]
enum ProjectedValue<'a> {
    Field(&'a FieldIr),
    StructField {
        owner: &'a FieldIr,
        field: &'a FieldIr,
    },
    UnionTag {
        owner: &'a FieldIr,
        tag: &'a str,
    },
    UnionField {
        owner: &'a FieldIr,
        field: &'a FieldIr,
    },
}

struct ProjectedColumn<'a> {
    name: String,
    value: ProjectedValue<'a>,
}

fn projected_columns<'a>(ir: &'a ConfigIr, table: &'a TableIr) -> Vec<ProjectedColumn<'a>> {
    table
        .fields
        .iter()
        .filter(|field| field.derived_from.is_none())
        .flat_map(|field| {
            if let Some(columns) = struct_columns(ir, field) {
                columns
                    .into_iter()
                    .map(|column| ProjectedColumn {
                        name: column.name,
                        value: ProjectedValue::StructField {
                            owner: field,
                            field: column.field,
                        },
                    })
                    .collect()
            } else if let Some(columns) = tagged_columns(ir, field) {
                let tag = tagged_columns_union(ir, field)
                    .map(|union| union.tag.as_str())
                    .unwrap_or("type");
                columns
                    .into_iter()
                    .map(|column| ProjectedColumn {
                        name: column.name,
                        value: match column.kind {
                            TaggedColumnKind::Tag => ProjectedValue::UnionTag { owner: field, tag },
                            TaggedColumnKind::VariantField(nested) => ProjectedValue::UnionField {
                                owner: field,
                                field: nested,
                            },
                        },
                    })
                    .collect()
            } else {
                vec![ProjectedColumn {
                    name: field.name.clone(),
                    value: ProjectedValue::Field(field),
                }]
            }
        })
        .collect()
}

fn projected_cell(ir: &ConfigIr, row: &RowData, column: &ProjectedColumn<'_>) -> String {
    match column.value {
        ProjectedValue::Field(field) => row
            .values
            .get(&field.name)
            .map(|value| cell_text(ir, field, value))
            .unwrap_or_default(),
        ProjectedValue::StructField { owner, field }
        | ProjectedValue::UnionField { owner, field } => row
            .values
            .get(&owner.name)
            .and_then(|value| match value {
                Value::Object(values) => values.get(&field.name),
                _ => None,
            })
            .map(|value| cell_text(ir, field, value))
            .unwrap_or_default(),
        ProjectedValue::UnionTag { owner, tag } => row
            .values
            .get(&owner.name)
            .and_then(|value| match value {
                Value::Object(values) => values.get(tag),
                _ => None,
            })
            .map(scalar_text)
            .unwrap_or_default(),
    }
}

fn cell_text(ir: &ConfigIr, field: &FieldIr, value: &Value) -> String {
    if matches!(value, Value::Null) {
        return String::new();
    }
    let parser = field.parser.as_ref();
    if parser.is_some_and(|parser| parser.kind == "json") {
        return serde_json::to_string(&natural_value(value)).unwrap_or_default();
    }
    if parser.is_some_and(|parser| parser.kind == "tuple")
        && let Value::Object(values) = value
    {
        let separator = parser
            .and_then(|parser| parser.options.get("separator"))
            .map(String::as_str)
            .unwrap_or(",");
        if let Some(structure) = struct_for_type(ir, &field.ty) {
            return structure
                .fields
                .iter()
                .map(|field| values.get(&field.name).map(scalar_text).unwrap_or_default())
                .collect::<Vec<_>>()
                .join(separator);
        }
    }
    if parser.is_some_and(|parser| parser.kind == "tuple_list")
        && let Value::List(values) = value
    {
        let separator = parser
            .and_then(|parser| parser.options.get("separator"))
            .map(String::as_str)
            .unwrap_or(",");
        let item_separator = parser
            .and_then(|parser| parser.options.get("item_separator"))
            .map(String::as_str)
            .unwrap_or("|");
        if let Some(structure) = list_struct_for_type(ir, &field.ty) {
            return values
                .iter()
                .map(|value| match value {
                    Value::Object(values) => structure
                        .fields
                        .iter()
                        .map(|field| values.get(&field.name).map(scalar_text).unwrap_or_default())
                        .collect::<Vec<_>>()
                        .join(separator),
                    _ => scalar_text(value),
                })
                .collect::<Vec<_>>()
                .join(item_separator);
        }
    }
    match (&field.ty, value) {
        (TypeIr::List(_) | TypeIr::Set(_) | TypeIr::Array { .. }, Value::List(values)) => {
            let separator = parser
                .and_then(|parser| parser.options.get("separator"))
                .map(String::as_str)
                .unwrap_or(",");
            values
                .iter()
                .map(scalar_text)
                .collect::<Vec<_>>()
                .join(separator)
        }
        (TypeIr::Map { .. }, Value::List(values)) => {
            let pair_separator = parser
                .and_then(|parser| parser.options.get("separator"))
                .map(String::as_str)
                .unwrap_or(",");
            let item_separator = parser
                .and_then(|parser| parser.options.get("item_separator"))
                .map(String::as_str)
                .unwrap_or("|");
            values
                .iter()
                .filter_map(|pair| match pair {
                    Value::List(pair) if pair.len() == 2 => Some(format!(
                        "{}{pair_separator}{}",
                        scalar_text(&pair[0]),
                        scalar_text(&pair[1])
                    )),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join(item_separator)
        }
        _ => scalar_text(value),
    }
}

fn struct_for_type<'a>(ir: &'a ConfigIr, ty: &TypeIr) -> Option<&'a sora_ir::model::StructIr> {
    match ty {
        TypeIr::Optional(inner) => struct_for_type(ir, inner),
        TypeIr::Struct(name) => ir.structs.iter().find(|item| item.name == *name),
        _ => None,
    }
}

fn list_struct_for_type<'a>(ir: &'a ConfigIr, ty: &TypeIr) -> Option<&'a sora_ir::model::StructIr> {
    match ty {
        TypeIr::Optional(inner) => list_struct_for_type(ir, inner),
        TypeIr::List(inner) | TypeIr::Set(inner) | TypeIr::Array { element: inner, .. } => {
            struct_for_type(ir, inner)
        }
        _ => None,
    }
}

fn render_xlsx(
    path: &Path,
    ir: &ConfigIr,
    tables: &[(&TableIr, &[RowData])],
    localization_sources: &[(
        &sora_ir::model::LocalizationSourceIr,
        &[LocalizationRowData],
    )],
) -> Result<Vec<u8>, DataMutationError> {
    let mut workbook = umya_spreadsheet::reader::xlsx::read(path)
        .map_err(|error| DataMutationError::Render(error.to_string()))?;
    for (table, rows) in tables {
        let source = table
            .source
            .as_ref()
            .ok_or_else(|| DataMutationError::MissingSource(table.name.clone()))?;
        let sheet_name = source.sheet.as_deref().unwrap_or(&table.name);
        let sheet = workbook.get_sheet_by_name_mut(sheet_name).ok_or_else(|| {
            DataMutationError::Render(format!("missing worksheet `{sheet_name}`"))
        })?;
        let columns = projected_columns(ir, table);
        let mut by_name = BTreeMap::new();
        let highest_column = sheet.get_highest_column();
        for column in 2..=highest_column {
            let name = sheet.get_value((column, 3));
            if !name.trim().is_empty() {
                by_name.insert(name.trim().to_owned(), column);
            }
        }
        for column in &columns {
            if !by_name.contains_key(&column.name) {
                return Err(DataMutationError::Render(format!(
                    "worksheet `{sheet_name}` is missing field column `{}`",
                    column.name
                )));
            }
        }
        let data_start = 8_u32;
        let old_last = sheet.get_highest_row();
        let last = old_last.max(data_start.saturating_add(rows.len() as u32));
        for row_number in data_start..=last {
            let row = rows.get((row_number - data_start) as usize);
            for column in &columns {
                let column_number = by_name[&column.name];
                let value = row
                    .map(|row| projected_cell(ir, row, column))
                    .unwrap_or_default();
                sheet
                    .get_cell_mut((column_number, row_number))
                    .set_value(value);
            }
        }
    }
    for (source, rows) in localization_sources {
        let sheet_name = source.sheet.as_deref().unwrap_or(&source.name);
        let sheet = workbook.get_sheet_by_name_mut(sheet_name).ok_or_else(|| {
            DataMutationError::Render(format!("missing worksheet `{sheet_name}`"))
        })?;
        let highest_column = sheet.get_highest_column();
        let mut columns = BTreeMap::new();
        for column in 2..=highest_column {
            let name = sheet.get_value((column, 3));
            if !name.trim().is_empty() {
                columns.insert(name.trim().to_owned(), column);
            }
        }
        let required = rows
            .iter()
            .flat_map(|row| row.values.keys())
            .collect::<BTreeSet<_>>();
        for field in required {
            if !columns.contains_key(field) {
                return Err(DataMutationError::Render(format!(
                    "worksheet `{sheet_name}` is missing localization column `{field}`"
                )));
            }
        }
        let data_start = 8_u32;
        let old_last = sheet.get_highest_row();
        let last = old_last.max(data_start.saturating_add(rows.len() as u32));
        for row_number in data_start..=last {
            let row = rows.get((row_number - data_start) as usize);
            for (field, column) in &columns {
                let value = row
                    .and_then(|row| row.values.get(field))
                    .cloned()
                    .unwrap_or_default();
                sheet.get_cell_mut((*column, row_number)).set_value(value);
            }
        }
    }
    let mut bytes = Vec::new();
    umya_spreadsheet::writer::xlsx::write_writer(&workbook, &mut bytes)
        .map_err(|error| DataMutationError::Render(error.to_string()))?;
    Ok(bytes)
}

fn natural_row(row: &RowData) -> BTreeMap<String, serde_json::Value> {
    row.values
        .iter()
        .map(|(field, value)| (field.clone(), natural_value(value)))
        .collect()
}

fn toml_row(row: &RowData) -> BTreeMap<String, serde_json::Value> {
    natural_row(row)
        .into_iter()
        .filter(|(_, value)| !value.is_null())
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

fn scalar_text(value: &Value) -> String {
    match value {
        Value::Bool(value) => value.to_string(),
        Value::Integer(value) => value.to_string(),
        Value::Float(value) => value.to_string(),
        Value::String(value) => value.clone(),
        Value::Null => String::new(),
        Value::List(_) | Value::Object(_) => {
            serde_json::to_string(&natural_value(value)).unwrap_or_default()
        }
    }
}

pub(crate) fn data_row_hash(row: &RowData) -> String {
    let bytes = serde_json::to_vec(&natural_row(row)).unwrap_or_default();
    let digest = Sha256::digest(bytes);
    format!(
        "row:{}",
        digest
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    )
}

fn row_change(
    table: &TableIr,
    kind: &str,
    before_index: Option<usize>,
    after_index: Option<usize>,
    before: Option<&RowData>,
    after: Option<&RowData>,
) -> RowChange {
    RowChange {
        table: table.name.clone(),
        kind: kind.to_owned(),
        before_index,
        after_index,
        before: before.map(natural_row),
        after: after.map(natural_row),
    }
}

fn operation_table(operation: &DataOperation) -> Option<&str> {
    match operation {
        DataOperation::InsertRow { table, .. }
        | DataOperation::UpsertRow { table, .. }
        | DataOperation::UpdateFields { table, .. }
        | DataOperation::DeleteRow { table, .. }
        | DataOperation::MoveListRow { table, .. } => Some(table),
        DataOperation::UpsertLocalization { .. }
        | DataOperation::UpdateLocalization { .. }
        | DataOperation::DeleteLocalization { .. } => None,
    }
}

fn operation_localization_source(operation: &DataOperation) -> Option<&str> {
    match operation {
        DataOperation::UpsertLocalization { source, .. }
        | DataOperation::UpdateLocalization { source, .. }
        | DataOperation::DeleteLocalization { source, .. } => Some(source),
        DataOperation::InsertRow { .. }
        | DataOperation::UpsertRow { .. }
        | DataOperation::UpdateFields { .. }
        | DataOperation::DeleteRow { .. }
        | DataOperation::MoveListRow { .. } => None,
    }
}

fn selector_name(selector: &RowSelector) -> &'static str {
    match selector {
        RowSelector::Map { .. } => "map",
        RowSelector::Singleton => "singleton",
        RowSelector::List { .. } => "list",
    }
}

fn relative_source_path(data_root: &Path, path: &Path) -> String {
    path.strip_prefix(data_root)
        .unwrap_or(path)
        .components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn source_data_start_row(format: SourceFormat) -> usize {
    match format {
        SourceFormat::Csv | SourceFormat::Json | SourceFormat::Toml | SourceFormat::Yaml => 1,
        SourceFormat::Xlsx => 8,
    }
}
