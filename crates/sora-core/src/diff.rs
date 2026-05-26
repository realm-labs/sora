use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use sora_data::model::{ConfigData, RowData, TableData, Value};
use sora_diagnostics::{Result, SoraError};
use sora_ir::model::{ConfigIr, TableIr, TableModeIr};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfigDiff {
    pub package: String,
    pub tables: Vec<TableDiff>,
}

impl ConfigDiff {
    pub fn has_changes(&self) -> bool {
        self.tables.iter().any(TableDiff::has_changes)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TableDiff {
    pub name: String,
    pub added: Vec<RowSnapshot>,
    pub removed: Vec<RowSnapshot>,
    pub changed: Vec<RowDiff>,
}

impl TableDiff {
    pub fn has_changes(&self) -> bool {
        !(self.added.is_empty() && self.removed.is_empty() && self.changed.is_empty())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RowSnapshot {
    pub row_key: String,
    pub values: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RowDiff {
    pub row_key: String,
    pub fields: Vec<FieldDiff>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldDiff {
    pub name: String,
    pub old: Value,
    pub new: Value,
}

pub fn diff_config_data(
    ir: &ConfigIr,
    left: &ConfigData,
    right: &ConfigData,
) -> Result<ConfigDiff> {
    let left_tables = table_data_by_name(left);
    let right_tables = table_data_by_name(right);
    let mut tables = Vec::new();

    for table in &ir.tables {
        let left_rows = left_tables
            .get(table.name.as_str())
            .map(|data| indexed_rows(table, data))
            .transpose()?
            .unwrap_or_default();
        let right_rows = right_tables
            .get(table.name.as_str())
            .map(|data| indexed_rows(table, data))
            .transpose()?
            .unwrap_or_default();

        let table_diff = diff_table(table, &left_rows, &right_rows);
        if table_diff.has_changes() {
            tables.push(table_diff);
        }
    }

    Ok(ConfigDiff {
        package: ir.package.clone(),
        tables,
    })
}

fn table_data_by_name(data: &ConfigData) -> BTreeMap<&str, &TableData> {
    data.tables
        .iter()
        .map(|table| (table.name.as_str(), table))
        .collect()
}

fn indexed_rows<'a>(table: &TableIr, data: &'a TableData) -> Result<BTreeMap<String, &'a RowData>> {
    data.rows
        .iter()
        .enumerate()
        .map(|(index, row)| Ok((row_key(table, index, row)?, row)))
        .collect()
}

fn row_key(table: &TableIr, index: usize, row: &RowData) -> Result<String> {
    match table.mode {
        TableModeIr::Map => {
            let key_field = table
                .key
                .as_ref()
                .ok_or_else(|| SoraError::MissingTableKey {
                    table: table.name.clone(),
                    field: "<missing>".to_owned(),
                })?;
            let value =
                row.values
                    .get(key_field)
                    .ok_or_else(|| SoraError::MissingRequiredField {
                        table: table.name.clone(),
                        field: key_field.clone(),
                    })?;
            stable_value_key(value)
        }
        TableModeIr::Singleton => Ok("$singleton".to_owned()),
        TableModeIr::List => Ok(format!("#{index:06}")),
    }
}

fn stable_value_key(value: &Value) -> Result<String> {
    match value {
        Value::Bool(value) => Ok(value.to_string()),
        Value::Integer(value) => Ok(value.to_string()),
        Value::Float(value) => Ok(value.to_string()),
        Value::String(value) => Ok(value.clone()),
        Value::Null => Ok("$null".to_owned()),
        Value::List(_) | Value::Object(_) => {
            serde_json::to_string(value).map_err(SoraError::SerializeData)
        }
    }
}

fn diff_table(
    table: &TableIr,
    left_rows: &BTreeMap<String, &RowData>,
    right_rows: &BTreeMap<String, &RowData>,
) -> TableDiff {
    let mut added = Vec::new();
    let mut removed = Vec::new();
    let mut changed = Vec::new();
    let keys = left_rows
        .keys()
        .chain(right_rows.keys())
        .cloned()
        .collect::<BTreeSet<_>>();

    for key in keys {
        match (left_rows.get(&key), right_rows.get(&key)) {
            (None, Some(row)) => added.push(snapshot_row(key, row)),
            (Some(row), None) => removed.push(snapshot_row(key, row)),
            (Some(left), Some(right)) => {
                let fields = diff_row_fields(table, left, right);
                if !fields.is_empty() {
                    changed.push(RowDiff {
                        row_key: key,
                        fields,
                    });
                }
            }
            (None, None) => {}
        }
    }

    TableDiff {
        name: table.name.clone(),
        added,
        removed,
        changed,
    }
}

fn snapshot_row(row_key: String, row: &RowData) -> RowSnapshot {
    RowSnapshot {
        row_key,
        values: row.values.clone(),
    }
}

fn diff_row_fields(table: &TableIr, left: &RowData, right: &RowData) -> Vec<FieldDiff> {
    table
        .fields
        .iter()
        .filter_map(|field| {
            let old = left.values.get(&field.name).cloned().unwrap_or(Value::Null);
            let new = right
                .values
                .get(&field.name)
                .cloned()
                .unwrap_or(Value::Null);
            (old != new).then(|| FieldDiff {
                name: field.name.clone(),
                old,
                new,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use sora_ir::model::{FieldIr, ScopeIr, TableIr, TypeIr};

    #[test]
    fn diffs_added_removed_and_changed_map_rows() {
        let ir = ConfigIr {
            package: "game_config".to_owned(),
            localization: None,
            enums: Vec::new(),
            structs: Vec::new(),
            unions: Vec::new(),
            tables: vec![TableIr {
                name: "Item".to_owned(),
                scope: ScopeIr::default(),
                mode: TableModeIr::Map,
                key: Some("id".to_owned()),
                source: None,
                fields: vec![
                    field("id", TypeIr::I32),
                    field("name", TypeIr::String),
                    field("max_stack", TypeIr::I32),
                ],
                indexes: Vec::new(),
            }],
        };
        let left = ConfigData {
            tables: vec![TableData {
                name: "Item".to_owned(),
                rows: vec![row(1001, "Sword", 1), row(1002, "Shield", 1)],
            }],
        };
        let right = ConfigData {
            tables: vec![TableData {
                name: "Item".to_owned(),
                rows: vec![row(1001, "Sword", 99), row(1003, "Potion", 20)],
            }],
        };

        let diff = diff_config_data(&ir, &left, &right).unwrap();

        assert!(diff.has_changes());
        assert_eq!(diff.tables.len(), 1);
        assert_eq!(
            diff.tables[0].removed[0].values["name"],
            Value::String("Shield".to_owned())
        );
        assert_eq!(
            diff.tables[0].added[0].values["name"],
            Value::String("Potion".to_owned())
        );
        assert_eq!(diff.tables[0].changed[0].fields[0].name, "max_stack");
    }

    fn field(name: &str, ty: TypeIr) -> FieldIr {
        FieldIr {
            name: name.to_owned(),
            ty,
            scope: ScopeIr::default(),
            key: name == "id",
            comment: None,
            default: None,
            range: None,
            length: None,
            parser: None,
            derived_from: None,
        }
    }

    fn row(id: i64, name: &str, max_stack: i64) -> RowData {
        RowData {
            values: BTreeMap::from([
                ("id".to_owned(), Value::Integer(id)),
                ("name".to_owned(), Value::String(name.to_owned())),
                ("max_stack".to_owned(), Value::Integer(max_stack)),
            ]),
        }
    }
}
