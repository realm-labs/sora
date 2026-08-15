use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet},
};

use sora_diagnostics::{Result, SoraError};
use sora_ir::model::{ConfigIr, DerivedFieldIr, FieldIr, StructIr, TableIr, TypeIr};

use crate::model::{ConfigData, RowData, Value};

pub fn materialize_derived_fields(ir: &ConfigIr, data: &ConfigData) -> Result<ConfigData> {
    let mut materialized = data.clone();

    for table in &ir.tables {
        for field in table
            .fields
            .iter()
            .filter(|field| field.derived_from.is_some())
        {
            materialize_table_derived_field(ir, data, &mut materialized, table, field)?;
        }
    }

    Ok(materialized)
}

fn materialize_table_derived_field(
    ir: &ConfigIr,
    source_data: &ConfigData,
    materialized: &mut ConfigData,
    parent_table: &TableIr,
    field: &FieldIr,
) -> Result<()> {
    let derived_from = field
        .derived_from
        .as_ref()
        .expect("caller filters to derived fields");
    let shape = derived_field_shape(ir, field)?;
    let Some(parent_data) = materialized
        .tables
        .iter_mut()
        .find(|table| table.name == parent_table.name)
    else {
        return Ok(());
    };
    let source_rows = source_data
        .tables
        .iter()
        .find(|table| table.name == derived_from.source_table)
        .map(|table| table.rows.as_slice())
        .unwrap_or(&[]);

    for parent_row in &mut parent_data.rows {
        let parent_key = parent_row
            .values
            .get(&derived_from.parent_key)
            .ok_or_else(|| SoraError::MissingRequiredField {
                table: parent_table.name.clone(),
                field: derived_from.parent_key.clone(),
            })?;
        let mut child_rows = matching_child_rows(source_rows, derived_from, parent_key)?;
        if let Some(order_by) = &derived_from.order_by {
            child_rows.sort_by(|left, right| compare_order_field(left, right, order_by));
        }

        let value = match shape.cardinality {
            DerivedFieldCardinality::Map { key_field, key_ty } => Value::List(derive_map_entries(
                ir,
                parent_table,
                field,
                derived_from,
                parent_key,
                child_rows,
                key_field,
                key_ty,
                &shape.value,
            )?),
            DerivedFieldCardinality::List => {
                Value::List(derive_child_values(child_rows, derived_from, &shape.value)?)
            }
            DerivedFieldCardinality::RequiredOne => {
                let values = derive_child_values(child_rows, derived_from, &shape.value)?;
                if values.len() != 1 {
                    return Err(derived_field_row_count_error(
                        parent_table,
                        field,
                        derived_from,
                        parent_key,
                        "exactly 1",
                        values.len(),
                    ));
                }
                values.into_iter().next().expect("checked one value")
            }
            DerivedFieldCardinality::OptionalOne => {
                let values = derive_child_values(child_rows, derived_from, &shape.value)?;
                if values.len() > 1 {
                    return Err(derived_field_row_count_error(
                        parent_table,
                        field,
                        derived_from,
                        parent_key,
                        "at most 1",
                        values.len(),
                    ));
                }
                values.into_iter().next().unwrap_or(Value::Null)
            }
        };
        parent_row.values.insert(field.name.clone(), value);
    }

    Ok(())
}

struct DerivedFieldShape<'a> {
    cardinality: DerivedFieldCardinality<'a>,
    value: DerivedFieldValue<'a>,
}

#[derive(Debug, Clone, Copy)]
enum DerivedFieldCardinality<'a> {
    List,
    RequiredOne,
    OptionalOne,
    Map {
        key_field: &'a str,
        key_ty: &'a TypeIr,
    },
}

enum DerivedFieldValue<'a> {
    Struct(&'a StructIr),
    Field(&'a str),
}

fn derived_field_shape<'a>(ir: &'a ConfigIr, field: &'a FieldIr) -> Result<DerivedFieldShape<'a>> {
    let derived_from = field
        .derived_from
        .as_ref()
        .expect("caller filters to derived fields");
    let (cardinality, value_ty) = match &field.ty {
        TypeIr::List(element) => (DerivedFieldCardinality::List, element.as_ref()),
        TypeIr::Optional(element) => (DerivedFieldCardinality::OptionalOne, element.as_ref()),
        TypeIr::Map { key, value } => {
            let key_field = derived_from.map_key.as_deref().ok_or_else(|| {
                SoraError::InvalidSchema(format!(
                    "derived map field `{}` must declare `from.key`",
                    field.name
                ))
            })?;
            (
                DerivedFieldCardinality::Map {
                    key_field,
                    key_ty: key.as_ref(),
                },
                value.as_ref(),
            )
        }
        ty => (DerivedFieldCardinality::RequiredOne, ty),
    };

    if let Some(value_field) = &derived_from.value_field {
        return Ok(DerivedFieldShape {
            cardinality,
            value: DerivedFieldValue::Field(value_field),
        });
    }

    let TypeIr::Struct(struct_name) = value_ty else {
        return Err(SoraError::InvalidSchema(format!(
            "derived field `{}` must assemble struct values or declare `from.field`",
            field.name
        )));
    };

    let struct_ir = ir
        .structs
        .iter()
        .find(|item| item.name == *struct_name)
        .ok_or_else(|| {
            SoraError::InvalidSchema(format!(
                "derived field `{}` references unknown struct `{struct_name}`",
                field.name
            ))
        })?;

    Ok(DerivedFieldShape {
        cardinality,
        value: DerivedFieldValue::Struct(struct_ir),
    })
}

fn matching_child_rows<'a>(
    source_rows: &'a [RowData],
    derived_from: &DerivedFieldIr,
    parent_key: &Value,
) -> Result<Vec<&'a RowData>> {
    let mut rows = Vec::new();
    for row in source_rows {
        let Some(child_key) = row.values.get(&derived_from.child_key) else {
            return Err(SoraError::MissingRequiredField {
                table: derived_from.source_table.clone(),
                field: derived_from.child_key.clone(),
            });
        };
        if stable_key(child_key) == stable_key(parent_key) {
            rows.push(row);
        }
    }
    Ok(rows)
}

fn derive_struct_value(source_table: &str, row: &RowData, struct_ir: &StructIr) -> Result<Value> {
    let mut values = BTreeMap::new();
    for field in &struct_ir.fields {
        if let Some(value) = row.values.get(&field.name) {
            values.insert(field.name.clone(), value.clone());
        } else if field.is_required() {
            return Err(SoraError::MissingRequiredField {
                table: source_table.to_owned(),
                field: field.name.clone(),
            });
        }
    }
    Ok(Value::Object(values))
}

fn derive_child_value(
    source_table: &str,
    row: &RowData,
    value: &DerivedFieldValue<'_>,
) -> Result<Value> {
    match value {
        DerivedFieldValue::Struct(struct_ir) => derive_struct_value(source_table, row, struct_ir),
        DerivedFieldValue::Field(field) => {
            row.values
                .get(*field)
                .cloned()
                .ok_or_else(|| SoraError::MissingRequiredField {
                    table: source_table.to_owned(),
                    field: (*field).to_owned(),
                })
        }
    }
}

fn derive_child_values(
    child_rows: Vec<&RowData>,
    derived_from: &DerivedFieldIr,
    value: &DerivedFieldValue<'_>,
) -> Result<Vec<Value>> {
    child_rows
        .into_iter()
        .map(|row| derive_child_value(&derived_from.source_table, row, value))
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn derive_map_entries(
    ir: &ConfigIr,
    parent_table: &TableIr,
    field: &FieldIr,
    derived_from: &DerivedFieldIr,
    parent_key: &Value,
    child_rows: Vec<&RowData>,
    key_field: &str,
    key_ty: &TypeIr,
    value: &DerivedFieldValue<'_>,
) -> Result<Vec<Value>> {
    let mut seen = BTreeSet::new();
    let mut entries = Vec::with_capacity(child_rows.len());
    for row in child_rows {
        let key = row.values.get(key_field).ok_or_else(|| {
            SoraError::InvalidSchema(format!(
                "derived map field `{}.{}` is missing source key `{}.{key_field}` for parent key `{}`",
                parent_table.name,
                field.name,
                derived_from.source_table,
                stable_key(parent_key)
            ))
        })?;
        let identity = canonical_typed_key(ir, key_ty, key);
        if !seen.insert(identity.clone()) {
            return Err(SoraError::InvalidSchema(format!(
                "derived map field `{}.{}` has duplicate key `{identity}` from `{}` for parent key `{}`",
                parent_table.name,
                field.name,
                derived_from.source_table,
                stable_key(parent_key)
            )));
        }
        let value = derive_child_value(&derived_from.source_table, row, value)?;
        entries.push(Value::List(vec![key.clone(), value]));
    }
    Ok(entries)
}

fn canonical_typed_key(ir: &ConfigIr, ty: &TypeIr, value: &Value) -> String {
    match (ty, value) {
        (TypeIr::Enum(name), Value::String(value)) => ir
            .enums
            .iter()
            .find(|candidate| candidate.name == *name)
            .and_then(|item| {
                item.aliases
                    .iter()
                    .find(|alias| alias.alias == *value)
                    .map(|alias| alias.name.clone())
            })
            .unwrap_or_else(|| value.clone()),
        (TypeIr::Ref { table, field }, value) => ir
            .tables
            .iter()
            .find(|candidate| candidate.name == *table)
            .and_then(|table| {
                table
                    .fields
                    .iter()
                    .find(|candidate| candidate.name == *field)
            })
            .map_or_else(
                || stable_key(value),
                |field| canonical_typed_key(ir, &field.ty, value),
            ),
        _ => stable_key(value),
    }
}

fn derived_field_row_count_error(
    parent_table: &TableIr,
    field: &FieldIr,
    derived_from: &DerivedFieldIr,
    parent_key: &Value,
    expected: &'static str,
    actual: usize,
) -> SoraError {
    SoraError::InvalidSchema(format!(
        "derived field `{}` in table `{}` expected {} row from `{}` where `{}` = `{}`, but found {}",
        field.name,
        parent_table.name,
        expected,
        derived_from.source_table,
        derived_from.child_key,
        stable_key(parent_key),
        actual
    ))
}

fn compare_order_field(left: &RowData, right: &RowData, order_by: &str) -> Ordering {
    let left = left.values.get(order_by);
    let right = right.values.get(order_by);
    compare_optional_values(left, right)
}

fn compare_optional_values(left: Option<&Value>, right: Option<&Value>) -> Ordering {
    match (left, right) {
        (Some(left), Some(right)) => compare_values(left, right),
        (None, Some(_)) => Ordering::Less,
        (Some(_), None) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

fn compare_values(left: &Value, right: &Value) -> Ordering {
    match (left, right) {
        (Value::Bool(left), Value::Bool(right)) => left.cmp(right),
        (Value::Integer(left), Value::Integer(right)) => left.cmp(right),
        (Value::Float(left), Value::Float(right)) => {
            left.partial_cmp(right).unwrap_or(Ordering::Equal)
        }
        (Value::String(left), Value::String(right)) => left.cmp(right),
        _ => stable_key(left).cmp(&stable_key(right)),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::TableData;
    use sora_ir::{normalize::normalize_schema, validate::validate_config_ir};
    use sora_schema::model::ProjectSchema;

    #[test]
    fn materializes_child_rows_into_parent_list_field() {
        let ir = derived_field_ir();
        let data = ConfigData {
            tables: vec![
                TableData {
                    name: "Item".to_owned(),
                    rows: vec![RowData {
                        values: BTreeMap::from([
                            ("id".to_owned(), Value::Integer(1001)),
                            ("name".to_owned(), Value::String("Iron Sword".to_owned())),
                        ]),
                    }],
                },
                TableData {
                    name: "ItemReward".to_owned(),
                    rows: vec![
                        RowData {
                            values: BTreeMap::from([
                                ("item_id".to_owned(), Value::Integer(1001)),
                                ("seq".to_owned(), Value::Integer(2)),
                                ("reward_item_id".to_owned(), Value::Integer(3002)),
                                ("count".to_owned(), Value::Integer(5)),
                            ]),
                        },
                        RowData {
                            values: BTreeMap::from([
                                ("item_id".to_owned(), Value::Integer(1001)),
                                ("seq".to_owned(), Value::Integer(1)),
                                ("reward_item_id".to_owned(), Value::Integer(3001)),
                                ("count".to_owned(), Value::Integer(2)),
                            ]),
                        },
                    ],
                },
            ],
        };

        let materialized = materialize_derived_fields(&ir, &data).unwrap();
        let rewards = &materialized.tables[0].rows[0].values["rewards"];

        assert_eq!(
            rewards,
            &Value::List(vec![
                Value::Object(BTreeMap::from([
                    ("count".to_owned(), Value::Integer(2)),
                    ("reward_item_id".to_owned(), Value::Integer(3001)),
                ])),
                Value::Object(BTreeMap::from([
                    ("count".to_owned(), Value::Integer(5)),
                    ("reward_item_id".to_owned(), Value::Integer(3002)),
                ])),
            ])
        );
    }

    #[test]
    fn materializes_single_child_value_field() {
        let ir = single_value_derived_field_ir("string");
        let data = ConfigData {
            tables: vec![
                TableData {
                    name: "Item".to_owned(),
                    rows: vec![RowData {
                        values: BTreeMap::from([("id".to_owned(), Value::Integer(1001))]),
                    }],
                },
                TableData {
                    name: "ItemProfile".to_owned(),
                    rows: vec![RowData {
                        values: BTreeMap::from([
                            ("item_id".to_owned(), Value::Integer(1001)),
                            ("name".to_owned(), Value::String("Iron Sword".to_owned())),
                            ("notes".to_owned(), Value::String("ignored".to_owned())),
                        ]),
                    }],
                },
            ],
        };

        let materialized = materialize_derived_fields(&ir, &data).unwrap();

        assert_eq!(
            materialized.tables[0].rows[0].values["display_name"],
            Value::String("Iron Sword".to_owned())
        );
    }

    #[test]
    fn materializes_missing_optional_child_value_as_null() {
        let ir = single_value_derived_field_ir("optional<string>");
        let data = ConfigData {
            tables: vec![
                TableData {
                    name: "Item".to_owned(),
                    rows: vec![RowData {
                        values: BTreeMap::from([("id".to_owned(), Value::Integer(1001))]),
                    }],
                },
                TableData {
                    name: "ItemProfile".to_owned(),
                    rows: Vec::new(),
                },
            ],
        };

        let materialized = materialize_derived_fields(&ir, &data).unwrap();

        assert_eq!(
            materialized.tables[0].rows[0].values["display_name"],
            Value::Null
        );
    }

    #[test]
    fn materializes_child_rows_into_map_field() {
        let ir = map_derived_field_ir();
        let data = ConfigData {
            tables: vec![
                TableData {
                    name: "Item".to_owned(),
                    rows: vec![RowData {
                        values: BTreeMap::from([("id".to_owned(), Value::Integer(1001))]),
                    }],
                },
                TableData {
                    name: "ItemRate".to_owned(),
                    rows: vec![
                        RowData {
                            values: BTreeMap::from([
                                ("item_id".to_owned(), Value::Integer(1001)),
                                ("slot".to_owned(), Value::String("normal".to_owned())),
                                ("rate".to_owned(), Value::Integer(80)),
                            ]),
                        },
                        RowData {
                            values: BTreeMap::from([
                                ("item_id".to_owned(), Value::Integer(1001)),
                                ("slot".to_owned(), Value::String("epic".to_owned())),
                                ("rate".to_owned(), Value::Integer(20)),
                            ]),
                        },
                    ],
                },
            ],
        };

        let materialized = materialize_derived_fields(&ir, &data).unwrap();

        assert_eq!(
            materialized.tables[0].rows[0].values["rates"],
            Value::List(vec![
                Value::List(vec![Value::String("normal".to_owned()), Value::Integer(80),]),
                Value::List(vec![Value::String("epic".to_owned()), Value::Integer(20),]),
            ])
        );
    }

    #[test]
    fn rejects_duplicate_derived_map_keys() {
        let ir = map_derived_field_ir();
        let data = ConfigData {
            tables: vec![
                TableData {
                    name: "Item".to_owned(),
                    rows: vec![RowData {
                        values: BTreeMap::from([("id".to_owned(), Value::Integer(1001))]),
                    }],
                },
                TableData {
                    name: "ItemRate".to_owned(),
                    rows: vec![
                        RowData {
                            values: BTreeMap::from([
                                ("item_id".to_owned(), Value::Integer(1001)),
                                ("slot".to_owned(), Value::String("normal".to_owned())),
                                ("rate".to_owned(), Value::Integer(80)),
                            ]),
                        },
                        RowData {
                            values: BTreeMap::from([
                                ("item_id".to_owned(), Value::Integer(1001)),
                                ("slot".to_owned(), Value::String("normal".to_owned())),
                                ("rate".to_owned(), Value::Integer(20)),
                            ]),
                        },
                    ],
                },
            ],
        };

        let error = materialize_derived_fields(&ir, &data).unwrap_err();

        assert!(
            error
                .to_string()
                .contains("derived map field `Item.rates` has duplicate key `normal`")
        );
    }

    #[test]
    fn materializes_struct_values_into_map_field() {
        let schema: ProjectSchema = toml::from_str(
            r#"
project = { id = "game_config" }
groups = { common = { default = true } }
views = { default = { contract = "game_config/default", groups = ["common"] } }

[[structs]]
name = "Rate"

[[structs.fields]]
name = "limit"
type = "i32"

[[structs.fields]]
name = "rate"
type = "i32"

[[tables]]
id = "item"
name = "Item"
mode = "map"
key = "id"

[[tables.fields]]
name = "id"
type = "i32"

[[tables.fields]]
name = "rates"
type = "map<string,struct<Rate>>"
from = { table = "ItemRate", parent_key = "id", child_key = "item_id", key = "slot" }

[[tables]]
id = "item_rate"
name = "ItemRate"
mode = "list"

[[tables.fields]]
name = "item_id"
type = "i32"

[[tables.fields]]
name = "slot"
type = "string"

[[tables.fields]]
name = "limit"
type = "i32"

[[tables.fields]]
name = "rate"
type = "i32"
"#,
        )
        .unwrap();
        let ir = normalize_schema(schema).unwrap();
        validate_config_ir(&ir).unwrap();
        let data = ConfigData {
            tables: vec![
                TableData {
                    name: "Item".to_owned(),
                    rows: vec![RowData {
                        values: BTreeMap::from([("id".to_owned(), Value::Integer(1001))]),
                    }],
                },
                TableData {
                    name: "ItemRate".to_owned(),
                    rows: vec![RowData {
                        values: BTreeMap::from([
                            ("item_id".to_owned(), Value::Integer(1001)),
                            ("slot".to_owned(), Value::String("epic".to_owned())),
                            ("limit".to_owned(), Value::Integer(10)),
                            ("rate".to_owned(), Value::Integer(20)),
                        ]),
                    }],
                },
            ],
        };

        let materialized = materialize_derived_fields(&ir, &data).unwrap();

        assert_eq!(
            materialized.tables[0].rows[0].values["rates"],
            Value::List(vec![Value::List(vec![
                Value::String("epic".to_owned()),
                Value::Object(BTreeMap::from([
                    ("limit".to_owned(), Value::Integer(10)),
                    ("rate".to_owned(), Value::Integer(20)),
                ])),
            ])])
        );
    }

    #[test]
    fn rejects_missing_required_single_child_value() {
        let ir = single_value_derived_field_ir("string");
        let data = ConfigData {
            tables: vec![
                TableData {
                    name: "Item".to_owned(),
                    rows: vec![RowData {
                        values: BTreeMap::from([("id".to_owned(), Value::Integer(1001))]),
                    }],
                },
                TableData {
                    name: "ItemProfile".to_owned(),
                    rows: Vec::new(),
                },
            ],
        };

        let error = materialize_derived_fields(&ir, &data).unwrap_err();

        assert!(
            error
                .to_string()
                .contains("expected exactly 1 row from `ItemProfile`")
        );
    }

    #[test]
    fn rejects_multiple_single_child_values() {
        let ir = single_value_derived_field_ir("optional<string>");
        let data = ConfigData {
            tables: vec![
                TableData {
                    name: "Item".to_owned(),
                    rows: vec![RowData {
                        values: BTreeMap::from([("id".to_owned(), Value::Integer(1001))]),
                    }],
                },
                TableData {
                    name: "ItemProfile".to_owned(),
                    rows: vec![
                        RowData {
                            values: BTreeMap::from([
                                ("item_id".to_owned(), Value::Integer(1001)),
                                ("name".to_owned(), Value::String("Iron Sword".to_owned())),
                            ]),
                        },
                        RowData {
                            values: BTreeMap::from([
                                ("item_id".to_owned(), Value::Integer(1001)),
                                ("name".to_owned(), Value::String("Sword".to_owned())),
                            ]),
                        },
                    ],
                },
            ],
        };

        let error = materialize_derived_fields(&ir, &data).unwrap_err();

        assert!(
            error
                .to_string()
                .contains("expected at most 1 row from `ItemProfile`")
        );
    }

    fn derived_field_ir() -> ConfigIr {
        let schema: ProjectSchema = toml::from_str(
            r#"
project = { id = "game_config" }
groups = { common = { default = true } }
views = { default = { contract = "game_config/default", groups = ["common"] } }

[[structs]]
name = "Reward"

[[structs.fields]]
name = "reward_item_id"
type = "i32"

[[structs.fields]]
name = "count"
type = "i32"

[[tables]]
id = "item"
name = "Item"
mode = "map"
key = "id"

[[tables.fields]]
name = "id"
type = "i32"

[[tables.fields]]
name = "name"
type = "string"

[[tables.fields]]
name = "rewards"
type = "list<Reward>"
from = { table = "ItemReward", parent_key = "id", child_key = "item_id", order_by = "seq" }

[[tables]]
id = "item_reward"
name = "ItemReward"
mode = "list"

[[tables.fields]]
name = "item_id"
type = "i32"

[[tables.fields]]
name = "seq"
type = "i32"

[[tables.fields]]
name = "reward_item_id"
type = "i32"

[[tables.fields]]
name = "count"
type = "i32"
"#,
        )
        .unwrap();
        let ir = normalize_schema(schema).unwrap();
        validate_config_ir(&ir).unwrap();
        ir
    }

    fn single_value_derived_field_ir(field_type: &str) -> ConfigIr {
        let schema: ProjectSchema = toml::from_str(&format!(
            r#"
project = {{ id = "game_config" }}
groups = {{ common = {{ default = true }} }}
views = {{ default = {{ contract = "game_config/default", groups = ["common"] }} }}

[[tables]]
id = "item"
name = "Item"
mode = "map"
key = "id"

[[tables.fields]]
name = "id"
type = "i32"

[[tables.fields]]
name = "display_name"
type = "{field_type}"
from = {{ table = "ItemProfile", parent_key = "id", child_key = "item_id", field = "name" }}

[[tables]]
id = "item_profile"
name = "ItemProfile"
mode = "list"

[[tables.fields]]
name = "item_id"
type = "i32"

[[tables.fields]]
name = "name"
type = "string"

[[tables.fields]]
name = "notes"
type = "string"
"#
        ))
        .unwrap();
        let ir = normalize_schema(schema).unwrap();
        validate_config_ir(&ir).unwrap();
        ir
    }

    fn map_derived_field_ir() -> ConfigIr {
        let schema: ProjectSchema = toml::from_str(
            r#"
project = { id = "game_config" }
groups = { common = { default = true } }
views = { default = { contract = "game_config/default", groups = ["common"] } }

[[tables]]
id = "item"
name = "Item"
mode = "map"
key = "id"

[[tables.fields]]
name = "id"
type = "i32"

[[tables.fields]]
name = "rates"
type = "map<string,i32>"
from = { table = "ItemRate", parent_key = "id", child_key = "item_id", key = "slot", field = "rate" }

[[tables]]
id = "item_rate"
name = "ItemRate"
mode = "list"

[[tables.fields]]
name = "item_id"
type = "i32"

[[tables.fields]]
name = "slot"
type = "string"

[[tables.fields]]
name = "rate"
type = "i32"
"#,
        )
        .unwrap();
        let ir = normalize_schema(schema).unwrap();
        validate_config_ir(&ir).unwrap();
        ir
    }
}
