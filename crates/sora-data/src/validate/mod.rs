use std::collections::{BTreeMap, BTreeSet};

use sora_diagnostics::{Result, SoraError};
use sora_ir::model::{ConfigIr, FieldIr, IndexIr, TableIr, TableModeIr};

use crate::model::{ConfigData, RowData, TableData, Value};

mod value;

use value::{stable_key, table_mode_name, validate_field_value};

pub fn validate_config_data(ir: &ConfigIr, data: &ConfigData) -> Result<()> {
    let errors = collect_config_data_errors(ir, data);
    finish_validation(errors)
}

pub fn validate_config_data_all(ir: &ConfigIr, data: &ConfigData) -> Result<()> {
    validate_config_data(ir, data)
}

fn collect_config_data_errors(ir: &ConfigIr, data: &ConfigData) -> Vec<SoraError> {
    let tables_by_name = data
        .tables
        .iter()
        .map(|table| (table.name.as_str(), table))
        .collect::<BTreeMap<_, _>>();

    let mut errors = Vec::new();
    for table in &ir.tables {
        match tables_by_name.get(table.name.as_str()) {
            Some(table_data) => {
                errors.extend(collect_table_data_errors(ir, data, table, table_data));
            }
            None if table.mode == TableModeIr::Singleton => {
                errors.push(SoraError::InvalidTableRowCount {
                    table: table.name.clone(),
                    mode: table_mode_name(table.mode),
                    expected: "exactly 1",
                    actual: 0,
                });
            }
            None => {}
        }
    }

    errors
}

pub fn validate_table_data(ir: &ConfigIr, table: &TableIr, data: &TableData) -> Result<()> {
    let config_data = ConfigData {
        tables: vec![data.clone()],
    };
    validate_table_data_with_config(ir, &config_data, table, data)
}

fn validate_table_data_with_config(
    ir: &ConfigIr,
    config_data: &ConfigData,
    table: &TableIr,
    data: &TableData,
) -> Result<()> {
    finish_validation(collect_table_data_errors(ir, config_data, table, data))
}

fn collect_table_data_errors(
    ir: &ConfigIr,
    config_data: &ConfigData,
    table: &TableIr,
    data: &TableData,
) -> Vec<SoraError> {
    let mut errors = Vec::new();
    if let Err(error) = validate_table_row_count(table, data) {
        errors.push(error);
    }

    let field_names = table
        .fields
        .iter()
        .map(|field| field.name.as_str())
        .collect::<BTreeSet<_>>();
    let mut seen_keys = BTreeSet::new();
    let mut unique_indexes = table
        .indexes
        .iter()
        .filter(|index| index.unique)
        .map(|index| UniqueIndexState {
            index,
            seen: BTreeSet::new(),
        })
        .collect::<Vec<_>>();

    for row in &data.rows {
        let row_result = validate_row_fields(
            ir,
            config_data,
            &table.name,
            &table.fields,
            &field_names,
            row,
        );
        if let Err(error) = row_result {
            errors.push(error);
            continue;
        }
        if let Err(error) = validate_map_key(table, row, &mut seen_keys) {
            errors.push(error);
            continue;
        }
        if let Err(error) = validate_unique_indexes(table, row, &mut unique_indexes) {
            errors.push(error);
        }
    }

    errors
}

fn finish_validation(errors: Vec<SoraError>) -> Result<()> {
    match errors.len() {
        0 => Ok(()),
        1 => Err(errors.into_iter().next().expect("one validation error")),
        _ => Err(SoraError::validation_errors(errors)),
    }
}

fn validate_table_row_count(table: &TableIr, data: &TableData) -> Result<()> {
    match table.mode {
        TableModeIr::Singleton if data.rows.len() != 1 => Err(SoraError::InvalidTableRowCount {
            table: table.name.clone(),
            mode: table_mode_name(table.mode),
            expected: "exactly 1",
            actual: data.rows.len(),
        }),
        _ => Ok(()),
    }
}

fn validate_row_fields(
    ir: &ConfigIr,
    config_data: &ConfigData,
    table_name: &str,
    fields: &[FieldIr],
    field_names: &BTreeSet<&str>,
    row: &RowData,
) -> Result<()> {
    for field_name in row.values.keys() {
        if !field_names.contains(field_name.as_str()) {
            return Err(SoraError::UnknownField {
                table: table_name.to_owned(),
                field: field_name.clone(),
            });
        }
    }

    for field in fields {
        match row.values.get(&field.name) {
            Some(value) => {
                validate_field_value(ir, config_data, table_name, field, &field.name, value)?
            }
            None if field.is_required() => {
                return Err(SoraError::MissingRequiredField {
                    table: table_name.to_owned(),
                    field: field.name.clone(),
                });
            }
            None => {}
        }
    }

    Ok(())
}

struct UniqueIndexState<'a> {
    index: &'a IndexIr,
    seen: BTreeSet<String>,
}

fn validate_unique_indexes(
    table: &TableIr,
    row: &RowData,
    indexes: &mut [UniqueIndexState<'_>],
) -> Result<()> {
    for state in indexes {
        let key = unique_index_key(state.index, row);
        if !state.seen.insert(key.clone()) {
            return Err(SoraError::DuplicateIndexKey {
                table: table.name.clone(),
                index: state.index.name.clone(),
                key,
            });
        }
    }

    Ok(())
}

fn unique_index_key(index: &IndexIr, row: &RowData) -> String {
    index
        .fields
        .iter()
        .map(|field| {
            let value = row.values.get(field).unwrap_or(&Value::Null);
            format!("{field}={}", stable_key(value))
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn validate_map_key(
    table: &TableIr,
    row: &RowData,
    seen_keys: &mut BTreeSet<String>,
) -> Result<()> {
    if table.mode != TableModeIr::Map {
        return Ok(());
    }

    let Some(key_field) = table.key.as_deref() else {
        return Ok(());
    };
    let Some(value) = row.values.get(key_field) else {
        return Err(SoraError::MissingRequiredField {
            table: table.name.clone(),
            field: key_field.to_owned(),
        });
    };
    if matches!(value, Value::Null) {
        return Err(SoraError::MissingRequiredField {
            table: table.name.clone(),
            field: key_field.to_owned(),
        });
    }

    let key = stable_key(value);
    if !seen_keys.insert(key.clone()) {
        return Err(SoraError::DuplicateKey {
            table: table.name.clone(),
            key,
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests;
