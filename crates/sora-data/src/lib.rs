use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use sora_diagnostics::{Result, SoraError};
use sora_ir::{ConfigIr, FieldIr, TableIr, TableModeIr, TypeIr};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfigData {
    pub tables: Vec<TableData>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TableData {
    pub name: String,
    pub rows: Vec<RowData>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RowData {
    pub values: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    Bool(bool),
    Integer(i64),
    Float(f64),
    String(String),
    List(Vec<Value>),
    Object(BTreeMap<String, Value>),
    Null,
}

pub fn validate_config_data(ir: &ConfigIr, data: &ConfigData) -> Result<()> {
    for table in &ir.tables {
        if let Some(table_data) = data.tables.iter().find(|item| item.name == table.name) {
            validate_table_data(ir, table, table_data)?;
        }
    }

    Ok(())
}

pub fn validate_table_data(ir: &ConfigIr, table: &TableIr, data: &TableData) -> Result<()> {
    let field_names = table
        .fields
        .iter()
        .map(|field| field.name.as_str())
        .collect::<BTreeSet<_>>();
    let mut seen_keys = BTreeSet::new();

    for row in &data.rows {
        for field_name in row.values.keys() {
            if !field_names.contains(field_name.as_str()) {
                return Err(SoraError::UnknownField {
                    table: table.name.clone(),
                    field: field_name.clone(),
                });
            }
        }

        for field in &table.fields {
            match row.values.get(&field.name) {
                Some(value) => validate_value(ir, &table.name, field, value)?,
                None if field.required => {
                    return Err(SoraError::MissingRequiredField {
                        table: table.name.clone(),
                        field: field.name.clone(),
                    });
                }
                None => {}
            }
        }

        if table.mode == TableModeIr::Map {
            let Some(key_field) = table.key.as_deref() else {
                continue;
            };
            if let Some(value) = row.values.get(key_field) {
                let key = stable_key(value);
                if !seen_keys.insert(key.clone()) {
                    return Err(SoraError::DuplicateKey {
                        table: table.name.clone(),
                        key,
                    });
                }
            }
        }
    }

    Ok(())
}

fn validate_value(ir: &ConfigIr, table: &str, field: &FieldIr, value: &Value) -> Result<()> {
    if value_matches_type(ir, &field.ty, value, table, &field.name)? {
        Ok(())
    } else {
        Err(SoraError::TypeMismatch {
            table: table.to_owned(),
            field: field.name.clone(),
            expected: field.ty.to_string(),
            actual: value.kind_name().to_owned(),
        })
    }
}

fn value_matches_type(
    ir: &ConfigIr,
    ty: &TypeIr,
    value: &Value,
    table: &str,
    field: &str,
) -> Result<bool> {
    Ok(match ty {
        TypeIr::Bool => matches!(value, Value::Bool(_)),
        TypeIr::I32 => matches!(value, Value::Integer(number) if i32::try_from(*number).is_ok()),
        TypeIr::I64 => matches!(value, Value::Integer(_)),
        TypeIr::F32 | TypeIr::F64 => matches!(value, Value::Integer(_) | Value::Float(_)),
        TypeIr::String => matches!(value, Value::String(_)),
        TypeIr::Enum(enum_name) => match value {
            Value::String(item) => {
                let is_valid = ir
                    .enums
                    .iter()
                    .find(|candidate| candidate.name == *enum_name)
                    .is_some_and(|candidate| candidate.values.contains(item));
                if !is_valid {
                    return Err(SoraError::InvalidEnumValue {
                        table: table.to_owned(),
                        field: field.to_owned(),
                        value: item.clone(),
                    });
                }
                true
            }
            _ => false,
        },
        TypeIr::Struct(_) => matches!(value, Value::Object(_)),
        TypeIr::List(element) => match value {
            Value::List(items) => {
                for item in items {
                    if !value_matches_type(ir, element, item, table, field)? {
                        return Ok(false);
                    }
                }
                true
            }
            _ => false,
        },
        TypeIr::Array { element, len } => match value {
            Value::List(items) if items.len() == *len => {
                for item in items {
                    if !value_matches_type(ir, element, item, table, field)? {
                        return Ok(false);
                    }
                }
                true
            }
            _ => false,
        },
        TypeIr::Ref { .. } => matches!(value, Value::Integer(_) | Value::String(_)),
        TypeIr::Optional(element) => {
            matches!(value, Value::Null) || value_matches_type(ir, element, value, table, field)?
        }
    })
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

impl Value {
    fn kind_name(&self) -> &'static str {
        match self {
            Self::Bool(_) => "bool",
            Self::Integer(_) => "integer",
            Self::Float(_) => "float",
            Self::String(_) => "string",
            Self::List(_) => "list",
            Self::Object(_) => "object",
            Self::Null => "null",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sora_ir::normalize_schema;
    use sora_schema::SchemaFile;

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
    fn rejects_duplicate_map_keys() {
        let ir = example_ir();
        let data = ConfigData {
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

        let error = validate_config_data(&ir, &data).unwrap_err();
        assert!(matches!(error, SoraError::DuplicateKey { key, .. } if key == "1001"));
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
}
