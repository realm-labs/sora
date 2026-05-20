use std::collections::{BTreeMap, BTreeSet};

use sora_diagnostics::{Result, SoraError};
use sora_ir::model::{ConfigIr, FieldIr, StructIr, TableIr, TableModeIr, TypeIr};

use crate::model::{ConfigData, RowData, TableData, Value};

pub fn validate_config_data(ir: &ConfigIr, data: &ConfigData) -> Result<()> {
    let tables_by_name = data
        .tables
        .iter()
        .map(|table| (table.name.as_str(), table))
        .collect::<BTreeMap<_, _>>();

    for table in &ir.tables {
        match tables_by_name.get(table.name.as_str()) {
            Some(table_data) => validate_table_data_with_config(ir, data, table, table_data)?,
            None if table.mode == TableModeIr::Singleton => {
                return Err(SoraError::InvalidTableRowCount {
                    table: table.name.clone(),
                    mode: table_mode_name(table.mode),
                    expected: "exactly 1",
                    actual: 0,
                });
            }
            None => {}
        }
    }

    Ok(())
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
    validate_table_row_count(table, data)?;

    let field_names = table
        .fields
        .iter()
        .map(|field| field.name.as_str())
        .collect::<BTreeSet<_>>();
    let mut seen_keys = BTreeSet::new();

    for row in &data.rows {
        validate_row_fields(
            ir,
            config_data,
            &table.name,
            &table.fields,
            &field_names,
            row,
        )?;
        validate_map_key(table, row, &mut seen_keys)?;
    }

    Ok(())
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
            None if field.required => {
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

fn validate_field_value(
    ir: &ConfigIr,
    config_data: &ConfigData,
    table: &str,
    field: &FieldIr,
    path: &str,
    value: &Value,
) -> Result<()> {
    validate_typed_value(ir, config_data, table, path, &field.ty, field.range, value)
}

fn validate_typed_value(
    ir: &ConfigIr,
    config_data: &ConfigData,
    table: &str,
    path: &str,
    ty: &TypeIr,
    range: Option<[i64; 2]>,
    value: &Value,
) -> Result<()> {
    match ty {
        TypeIr::Optional(element) if matches!(value, Value::Null) => Ok(()),
        TypeIr::Optional(element) => {
            validate_typed_value(ir, config_data, table, path, element, range, value)
        }
        TypeIr::Bool => expect_type(table, path, ty, value, matches!(value, Value::Bool(_))),
        TypeIr::I32 => match value {
            Value::Integer(number) if i32::try_from(*number).is_ok() => {
                validate_integer_range(table, path, *number, range)
            }
            _ => type_mismatch(table, path, ty, value),
        },
        TypeIr::I64 => match value {
            Value::Integer(number) => validate_integer_range(table, path, *number, range),
            _ => type_mismatch(table, path, ty, value),
        },
        TypeIr::F32 | TypeIr::F64 => match value {
            Value::Integer(number) => validate_float_range(table, path, *number as f64, range),
            Value::Float(number) => validate_float_range(table, path, *number, range),
            _ => type_mismatch(table, path, ty, value),
        },
        TypeIr::String => expect_type(table, path, ty, value, matches!(value, Value::String(_))),
        TypeIr::Enum(enum_name) => validate_enum(ir, table, path, ty, enum_name, value),
        TypeIr::Struct(struct_name) => {
            validate_struct(ir, config_data, table, path, ty, struct_name, value)
        }
        TypeIr::List(element) => validate_list(ir, config_data, table, path, element, range, value),
        TypeIr::Array { element, len } => validate_array(
            ir,
            config_data,
            table,
            path,
            ty,
            element,
            *len,
            range,
            value,
        ),
        TypeIr::Ref {
            table: ref_table,
            field: ref_field,
        } => validate_ref(config_data, table, path, ty, ref_table, ref_field, value),
    }
}

fn validate_enum(
    ir: &ConfigIr,
    table: &str,
    path: &str,
    ty: &TypeIr,
    enum_name: &str,
    value: &Value,
) -> Result<()> {
    let Value::String(item) = value else {
        return type_mismatch(table, path, ty, value);
    };

    let is_valid = ir
        .enums
        .iter()
        .find(|candidate| candidate.name == enum_name)
        .is_some_and(|candidate| candidate.values.contains(item));
    if is_valid {
        Ok(())
    } else {
        Err(SoraError::InvalidEnumValue {
            table: table.to_owned(),
            field: path.to_owned(),
            value: item.clone(),
        })
    }
}

fn validate_struct(
    ir: &ConfigIr,
    config_data: &ConfigData,
    table: &str,
    path: &str,
    ty: &TypeIr,
    struct_name: &str,
    value: &Value,
) -> Result<()> {
    let Value::Object(values) = value else {
        return type_mismatch(table, path, ty, value);
    };
    let Some(struct_ir) = ir
        .structs
        .iter()
        .find(|candidate| candidate.name == struct_name)
    else {
        return type_mismatch(table, path, ty, value);
    };

    validate_object_fields(ir, config_data, table, path, struct_ir, values)
}

fn validate_object_fields(
    ir: &ConfigIr,
    config_data: &ConfigData,
    table: &str,
    path: &str,
    struct_ir: &StructIr,
    values: &BTreeMap<String, Value>,
) -> Result<()> {
    let field_names = struct_ir
        .fields
        .iter()
        .map(|field| field.name.as_str())
        .collect::<BTreeSet<_>>();
    for field_name in values.keys() {
        if !field_names.contains(field_name.as_str()) {
            return Err(SoraError::UnknownField {
                table: table.to_owned(),
                field: format!("{path}.{field_name}"),
            });
        }
    }

    for field in &struct_ir.fields {
        let child_path = format!("{path}.{}", field.name);
        match values.get(&field.name) {
            Some(value) => {
                validate_field_value(ir, config_data, table, field, &child_path, value)?;
            }
            None if field.required => {
                return Err(SoraError::MissingRequiredField {
                    table: table.to_owned(),
                    field: child_path,
                });
            }
            None => {}
        }
    }

    Ok(())
}

fn validate_list(
    ir: &ConfigIr,
    config_data: &ConfigData,
    table: &str,
    path: &str,
    element: &TypeIr,
    range: Option<[i64; 2]>,
    value: &Value,
) -> Result<()> {
    let Value::List(items) = value else {
        return type_mismatch(table, path, &TypeIr::List(Box::new(element.clone())), value);
    };

    for (index, item) in items.iter().enumerate() {
        validate_typed_value(
            ir,
            config_data,
            table,
            &format!("{path}[{index}]"),
            element,
            range,
            item,
        )?;
    }

    Ok(())
}

fn validate_array(
    ir: &ConfigIr,
    config_data: &ConfigData,
    table: &str,
    path: &str,
    ty: &TypeIr,
    element: &TypeIr,
    len: usize,
    range: Option<[i64; 2]>,
    value: &Value,
) -> Result<()> {
    let Value::List(items) = value else {
        return type_mismatch(table, path, ty, value);
    };
    if items.len() != len {
        return type_mismatch(table, path, ty, value);
    }

    for (index, item) in items.iter().enumerate() {
        validate_typed_value(
            ir,
            config_data,
            table,
            &format!("{path}[{index}]"),
            element,
            range,
            item,
        )?;
    }

    Ok(())
}

fn validate_ref(
    config_data: &ConfigData,
    table: &str,
    path: &str,
    ty: &TypeIr,
    ref_table: &str,
    ref_field: &str,
    value: &Value,
) -> Result<()> {
    if !matches!(value, Value::Integer(_) | Value::String(_)) {
        return type_mismatch(table, path, ty, value);
    }

    let key = stable_key(value);
    let exists = config_data
        .tables
        .iter()
        .find(|candidate| candidate.name == ref_table)
        .is_some_and(|table_data| {
            table_data
                .rows
                .iter()
                .filter_map(|row| row.values.get(ref_field))
                .any(|candidate| stable_key(candidate) == key)
        });

    if exists {
        Ok(())
    } else {
        Err(SoraError::MissingReference {
            table: table.to_owned(),
            field: path.to_owned(),
            ref_table: ref_table.to_owned(),
            ref_field: ref_field.to_owned(),
            value: key,
        })
    }
}

fn expect_type(table: &str, path: &str, ty: &TypeIr, value: &Value, matches: bool) -> Result<()> {
    if matches {
        Ok(())
    } else {
        type_mismatch(table, path, ty, value)
    }
}

fn type_mismatch(table: &str, path: &str, ty: &TypeIr, value: &Value) -> Result<()> {
    Err(SoraError::TypeMismatch {
        table: table.to_owned(),
        field: path.to_owned(),
        expected: ty.to_string(),
        actual: value.kind_name().to_owned(),
    })
}

fn validate_integer_range(
    table: &str,
    path: &str,
    value: i64,
    range: Option<[i64; 2]>,
) -> Result<()> {
    let Some([min, max]) = range else {
        return Ok(());
    };
    if value < min || value > max {
        Err(SoraError::RangeOutOfBounds {
            table: table.to_owned(),
            field: path.to_owned(),
            value: value.to_string(),
            min,
            max,
        })
    } else {
        Ok(())
    }
}

fn validate_float_range(
    table: &str,
    path: &str,
    value: f64,
    range: Option<[i64; 2]>,
) -> Result<()> {
    let Some([min, max]) = range else {
        return Ok(());
    };
    if value < min as f64 || value > max as f64 {
        Err(SoraError::RangeOutOfBounds {
            table: table.to_owned(),
            field: path.to_owned(),
            value: value.to_string(),
            min,
            max,
        })
    } else {
        Ok(())
    }
}

fn stable_key(value: &Value) -> String {
    match value {
        Value::Bool(value) => value.to_string(),
        Value::Integer(value) => value.to_string(),
        Value::Float(value) => value.to_string(),
        Value::String(value) => value.clone(),
        Value::List(_) => "<list>".to_owned(),
        Value::Object(_) => "<object>".to_owned(),
        Value::Null => "<null>".to_owned(),
    }
}

fn table_mode_name(mode: TableModeIr) -> &'static str {
    match mode {
        TableModeIr::List => "list",
        TableModeIr::Map => "map",
        TableModeIr::Singleton => "singleton",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ConfigData, RowData, TableData, Value};
    use sora_ir::normalize::normalize_schema;
    use sora_schema::model::SchemaFile;
    use std::collections::BTreeMap;

    #[test]
    fn validates_simple_table_data() {
        let ir = example_ir();
        let data = ConfigData {
            tables: vec![TableData {
                name: "Item".to_owned(),
                rows: vec![RowData {
                    values: BTreeMap::from([
                        ("id".to_owned(), Value::Integer(1001)),
                        ("name".to_owned(), Value::String("Iron Sword".to_owned())),
                        ("item_type".to_owned(), Value::String("Weapon".to_owned())),
                        ("max_stack".to_owned(), Value::Integer(1)),
                    ]),
                }],
            }],
        };

        validate_config_data(&ir, &data).unwrap();
    }

    #[test]
    fn rejects_invalid_data() {
        assert_validation_error(
            BTreeMap::from([
                ("id".to_owned(), Value::Integer(1001)),
                ("item_type".to_owned(), Value::String("Weapon".to_owned())),
                ("max_stack".to_owned(), Value::Integer(1)),
            ]),
            |error| matches!(error, SoraError::MissingRequiredField { field, .. } if field == "name"),
        );

        assert_validation_error(
            BTreeMap::from([
                ("id".to_owned(), Value::Integer(1001)),
                ("name".to_owned(), Value::String("Iron Sword".to_owned())),
                ("unknown".to_owned(), Value::Integer(1)),
                ("item_type".to_owned(), Value::String("Weapon".to_owned())),
                ("max_stack".to_owned(), Value::Integer(1)),
            ]),
            |error| matches!(error, SoraError::UnknownField { field, .. } if field == "unknown"),
        );

        assert_validation_error(
            BTreeMap::from([
                ("id".to_owned(), Value::Integer(1001)),
                ("name".to_owned(), Value::String("Iron Sword".to_owned())),
                ("item_type".to_owned(), Value::String("Invalid".to_owned())),
                ("max_stack".to_owned(), Value::Integer(1)),
            ]),
            |error| matches!(error, SoraError::InvalidEnumValue { value, .. } if value == "Invalid"),
        );

        assert_validation_error(
            BTreeMap::from([
                ("id".to_owned(), Value::Integer(1001)),
                ("name".to_owned(), Value::String("Iron Sword".to_owned())),
                ("item_type".to_owned(), Value::String("Weapon".to_owned())),
                ("max_stack".to_owned(), Value::String("one".to_owned())),
            ]),
            |error| matches!(error, SoraError::TypeMismatch { field, .. } if field == "max_stack"),
        );
    }

    #[test]
    fn rejects_duplicate_and_missing_map_keys() {
        let ir = example_ir();
        let duplicate_data = ConfigData {
            tables: vec![TableData {
                name: "Item".to_owned(),
                rows: vec![
                    RowData {
                        values: valid_row(1001),
                    },
                    RowData {
                        values: valid_row(1001),
                    },
                ],
            }],
        };

        let error = validate_config_data(&ir, &duplicate_data).unwrap_err();
        assert!(matches!(error, SoraError::DuplicateKey { key, .. } if key == "1001"));

        let missing_key_data = ConfigData {
            tables: vec![TableData {
                name: "Item".to_owned(),
                rows: vec![RowData {
                    values: BTreeMap::from([
                        ("name".to_owned(), Value::String("Iron Sword".to_owned())),
                        ("item_type".to_owned(), Value::String("Weapon".to_owned())),
                        ("max_stack".to_owned(), Value::Integer(1)),
                    ]),
                }],
            }],
        };

        let error = validate_config_data(&ir, &missing_key_data).unwrap_err();
        assert!(matches!(
            error,
            SoraError::MissingRequiredField { field, .. } if field == "id"
        ));
    }

    #[test]
    fn validates_ranges_and_struct_fields() {
        let ir = complex_ir();
        let data = ConfigData {
            tables: vec![
                TableData {
                    name: "Item".to_owned(),
                    rows: vec![RowData {
                        values: BTreeMap::from([
                            ("id".to_owned(), Value::Integer(1001)),
                            (
                                "reward".to_owned(),
                                Value::Object(BTreeMap::from([
                                    ("item_id".to_owned(), Value::Integer(1001)),
                                    ("count".to_owned(), Value::Integer(2)),
                                ])),
                            ),
                            (
                                "rolls".to_owned(),
                                Value::List(vec![Value::Integer(1), Value::Integer(3)]),
                            ),
                        ]),
                    }],
                },
                TableData {
                    name: "RewardSource".to_owned(),
                    rows: vec![RowData {
                        values: BTreeMap::from([("id".to_owned(), Value::Integer(1001))]),
                    }],
                },
                TableData {
                    name: "Settings".to_owned(),
                    rows: vec![RowData {
                        values: BTreeMap::from([("id".to_owned(), Value::Integer(1))]),
                    }],
                },
            ],
        };

        validate_config_data(&ir, &data).unwrap();
    }

    #[test]
    fn rejects_range_struct_ref_and_singleton_errors() {
        let ir = complex_ir();

        let range_error = validate_config_data(
            &ir,
            &ConfigData {
                tables: vec![
                    TableData {
                        name: "Item".to_owned(),
                        rows: vec![RowData {
                            values: BTreeMap::from([
                                ("id".to_owned(), Value::Integer(1001)),
                                (
                                    "reward".to_owned(),
                                    Value::Object(BTreeMap::from([
                                        ("item_id".to_owned(), Value::Integer(1001)),
                                        ("count".to_owned(), Value::Integer(99)),
                                    ])),
                                ),
                                (
                                    "rolls".to_owned(),
                                    Value::List(vec![Value::Integer(1), Value::Integer(3)]),
                                ),
                            ]),
                        }],
                    },
                    reward_source_table(),
                    singleton_table(1),
                ],
            },
        )
        .unwrap_err();
        assert!(matches!(
            range_error,
            SoraError::RangeOutOfBounds { field, .. } if field == "reward.count"
        ));

        let ref_error = validate_config_data(
            &ir,
            &ConfigData {
                tables: vec![
                    TableData {
                        name: "Item".to_owned(),
                        rows: vec![RowData {
                            values: BTreeMap::from([
                                ("id".to_owned(), Value::Integer(1001)),
                                (
                                    "reward".to_owned(),
                                    Value::Object(BTreeMap::from([
                                        ("item_id".to_owned(), Value::Integer(404)),
                                        ("count".to_owned(), Value::Integer(2)),
                                    ])),
                                ),
                                (
                                    "rolls".to_owned(),
                                    Value::List(vec![Value::Integer(1), Value::Integer(3)]),
                                ),
                            ]),
                        }],
                    },
                    reward_source_table(),
                    singleton_table(1),
                ],
            },
        )
        .unwrap_err();
        assert!(matches!(
            ref_error,
            SoraError::MissingReference { field, value, .. } if field == "reward.item_id" && value == "404"
        ));

        let singleton_error = validate_config_data(
            &ir,
            &ConfigData {
                tables: vec![
                    valid_complex_item_table(),
                    reward_source_table(),
                    TableData {
                        name: "Settings".to_owned(),
                        rows: vec![],
                    },
                ],
            },
        )
        .unwrap_err();
        assert!(matches!(
            singleton_error,
            SoraError::InvalidTableRowCount { table, actual: 0, .. } if table == "Settings"
        ));
    }

    fn assert_validation_error(
        values: BTreeMap<String, Value>,
        predicate: impl FnOnce(SoraError) -> bool,
    ) {
        let ir = example_ir();
        let data = ConfigData {
            tables: vec![TableData {
                name: "Item".to_owned(),
                rows: vec![RowData { values }],
            }],
        };

        let error = validate_config_data(&ir, &data).unwrap_err();
        assert!(predicate(error));
    }

    fn valid_row(id: i64) -> BTreeMap<String, Value> {
        BTreeMap::from([
            ("id".to_owned(), Value::Integer(id)),
            ("name".to_owned(), Value::String("Iron Sword".to_owned())),
            ("item_type".to_owned(), Value::String("Weapon".to_owned())),
            ("max_stack".to_owned(), Value::Integer(1)),
        ])
    }

    fn valid_complex_item_table() -> TableData {
        TableData {
            name: "Item".to_owned(),
            rows: vec![RowData {
                values: BTreeMap::from([
                    ("id".to_owned(), Value::Integer(1001)),
                    (
                        "reward".to_owned(),
                        Value::Object(BTreeMap::from([
                            ("item_id".to_owned(), Value::Integer(1001)),
                            ("count".to_owned(), Value::Integer(2)),
                        ])),
                    ),
                    (
                        "rolls".to_owned(),
                        Value::List(vec![Value::Integer(1), Value::Integer(3)]),
                    ),
                ]),
            }],
        }
    }

    fn reward_source_table() -> TableData {
        TableData {
            name: "RewardSource".to_owned(),
            rows: vec![RowData {
                values: BTreeMap::from([("id".to_owned(), Value::Integer(1001))]),
            }],
        }
    }

    fn singleton_table(id: i64) -> TableData {
        TableData {
            name: "Settings".to_owned(),
            rows: vec![RowData {
                values: BTreeMap::from([("id".to_owned(), Value::Integer(id))]),
            }],
        }
    }

    fn example_ir() -> ConfigIr {
        let schema: SchemaFile = toml::from_str(
            r#"
package = "game_config"

[[enums]]
name = "ItemType"
values = ["Weapon", "Armor", "Material", "Consumable"]

[[tables]]
name = "Item"
mode = "map"
key = "id"
[tables.source]
format = "toml"
file = "items.toml"

[[tables.fields]]
name = "id"
type = "i32"
key = true
required = true

[[tables.fields]]
name = "name"
type = "string"
required = true

[[tables.fields]]
name = "item_type"
type = "enum<ItemType>"
required = true

[[tables.fields]]
name = "max_stack"
type = "i32"
required = true
"#,
        )
        .unwrap();

        normalize_schema(schema).unwrap()
    }

    fn complex_ir() -> ConfigIr {
        let schema: SchemaFile = toml::from_str(
            r#"
package = "game_config"

[[structs]]
name = "Reward"

[[structs.fields]]
name = "item_id"
type = "ref<RewardSource.id>"
required = true

[[structs.fields]]
name = "count"
type = "i32"
required = true
range = [1, 10]

[[tables]]
name = "RewardSource"
mode = "map"
key = "id"

[[tables.fields]]
name = "id"
type = "i32"
required = true

[[tables]]
name = "Item"
mode = "map"
key = "id"

[[tables.fields]]
name = "id"
type = "i32"
required = true

[[tables.fields]]
name = "reward"
type = "struct<Reward>"
required = true

[[tables.fields]]
name = "rolls"
type = "array<i32,2>"
required = true
range = [1, 6]
separator = ","

[[tables]]
name = "Settings"
mode = "singleton"

[[tables.fields]]
name = "id"
type = "i32"
required = true
"#,
        )
        .unwrap();

        normalize_schema(schema).unwrap()
    }
}
