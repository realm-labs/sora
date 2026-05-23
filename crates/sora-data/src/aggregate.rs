use std::{cmp::Ordering, collections::BTreeMap};

use sora_diagnostics::{Result, SoraError};
use sora_ir::model::{AggregationIr, ConfigIr, FieldIr, StructIr, TableIr, TypeIr};

use crate::model::{ConfigData, RowData, Value};

pub fn materialize_aggregations(ir: &ConfigIr, data: &ConfigData) -> Result<ConfigData> {
    let mut materialized = data.clone();

    for table in &ir.tables {
        for field in table
            .fields
            .iter()
            .filter(|field| field.aggregation.is_some())
        {
            materialize_table_aggregation(ir, data, &mut materialized, table, field)?;
        }
    }

    Ok(materialized)
}

fn materialize_table_aggregation(
    ir: &ConfigIr,
    source_data: &ConfigData,
    materialized: &mut ConfigData,
    parent_table: &TableIr,
    field: &FieldIr,
) -> Result<()> {
    let aggregation = field
        .aggregation
        .as_ref()
        .expect("caller filters to aggregation fields");
    let shape = aggregation_shape(ir, field)?;
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
        .find(|table| table.name == aggregation.source_table)
        .map(|table| table.rows.as_slice())
        .unwrap_or(&[]);

    for parent_row in &mut parent_data.rows {
        let parent_key = parent_row
            .values
            .get(&aggregation.parent_key)
            .ok_or_else(|| SoraError::MissingRequiredField {
                table: parent_table.name.clone(),
                field: aggregation.parent_key.clone(),
            })?;
        let mut child_rows = matching_child_rows(source_rows, aggregation, parent_key)?;
        if let Some(order_by) = &aggregation.order_by {
            child_rows.sort_by(|left, right| compare_order_field(left, right, order_by));
        }

        let values = child_rows
            .into_iter()
            .map(|row| aggregate_child_value(&aggregation.source_table, row, &shape.value))
            .collect::<Result<Vec<_>>>()?;
        let value = match shape.cardinality {
            AggregationCardinality::List => Value::List(values),
            AggregationCardinality::RequiredOne => {
                if values.len() != 1 {
                    return Err(aggregation_row_count_error(
                        parent_table,
                        field,
                        aggregation,
                        parent_key,
                        "exactly 1",
                        values.len(),
                    ));
                }
                values.into_iter().next().expect("checked one value")
            }
            AggregationCardinality::OptionalOne => {
                if values.len() > 1 {
                    return Err(aggregation_row_count_error(
                        parent_table,
                        field,
                        aggregation,
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

struct AggregationShape<'a> {
    cardinality: AggregationCardinality,
    value: AggregationValue<'a>,
}

#[derive(Debug, Clone, Copy)]
enum AggregationCardinality {
    List,
    RequiredOne,
    OptionalOne,
}

enum AggregationValue<'a> {
    Struct(&'a StructIr),
    Field(&'a str),
}

fn aggregation_shape<'a>(ir: &'a ConfigIr, field: &'a FieldIr) -> Result<AggregationShape<'a>> {
    let aggregation = field
        .aggregation
        .as_ref()
        .expect("caller filters to aggregation fields");
    let (cardinality, value_ty) = match &field.ty {
        TypeIr::List(element) => (AggregationCardinality::List, element.as_ref()),
        TypeIr::Optional(element) => (AggregationCardinality::OptionalOne, element.as_ref()),
        ty => (AggregationCardinality::RequiredOne, ty),
    };

    if let Some(value_field) = &aggregation.value_field {
        return Ok(AggregationShape {
            cardinality,
            value: AggregationValue::Field(value_field),
        });
    }

    let TypeIr::Struct(struct_name) = value_ty else {
        return Err(SoraError::InvalidSchema(format!(
            "aggregation field `{}` must aggregate struct values or declare `value_field`",
            field.name
        )));
    };

    let struct_ir = ir
        .structs
        .iter()
        .find(|item| item.name == *struct_name)
        .ok_or_else(|| {
            SoraError::InvalidSchema(format!(
                "aggregation field `{}` references unknown struct `{struct_name}`",
                field.name
            ))
        })?;

    Ok(AggregationShape {
        cardinality,
        value: AggregationValue::Struct(struct_ir),
    })
}

fn matching_child_rows<'a>(
    source_rows: &'a [RowData],
    aggregation: &AggregationIr,
    parent_key: &Value,
) -> Result<Vec<&'a RowData>> {
    let mut rows = Vec::new();
    for row in source_rows {
        let Some(child_key) = row.values.get(&aggregation.child_key) else {
            return Err(SoraError::MissingRequiredField {
                table: aggregation.source_table.clone(),
                field: aggregation.child_key.clone(),
            });
        };
        if stable_key(child_key) == stable_key(parent_key) {
            rows.push(row);
        }
    }
    Ok(rows)
}

fn aggregate_struct_value(
    source_table: &str,
    row: &RowData,
    struct_ir: &StructIr,
) -> Result<Value> {
    let mut values = BTreeMap::new();
    for field in &struct_ir.fields {
        if let Some(value) = row.values.get(&field.name) {
            values.insert(field.name.clone(), value.clone());
        } else if field.required {
            return Err(SoraError::MissingRequiredField {
                table: source_table.to_owned(),
                field: field.name.clone(),
            });
        }
    }
    Ok(Value::Object(values))
}

fn aggregate_child_value(
    source_table: &str,
    row: &RowData,
    value: &AggregationValue<'_>,
) -> Result<Value> {
    match value {
        AggregationValue::Struct(struct_ir) => aggregate_struct_value(source_table, row, struct_ir),
        AggregationValue::Field(field) => {
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

fn aggregation_row_count_error(
    parent_table: &TableIr,
    field: &FieldIr,
    aggregation: &AggregationIr,
    parent_key: &Value,
    expected: &'static str,
    actual: usize,
) -> SoraError {
    SoraError::InvalidSchema(format!(
        "aggregation field `{}` in table `{}` expected {} row from `{}` where `{}` = `{}`, but found {}",
        field.name,
        parent_table.name,
        expected,
        aggregation.source_table,
        aggregation.child_key,
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
    use sora_schema::model::SchemaFile;

    #[test]
    fn materializes_child_rows_into_parent_list_field() {
        let ir = aggregation_ir();
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

        let materialized = materialize_aggregations(&ir, &data).unwrap();
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
        let ir = single_value_aggregation_ir("string");
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

        let materialized = materialize_aggregations(&ir, &data).unwrap();

        assert_eq!(
            materialized.tables[0].rows[0].values["display_name"],
            Value::String("Iron Sword".to_owned())
        );
    }

    #[test]
    fn materializes_missing_optional_child_value_as_null() {
        let ir = single_value_aggregation_ir("optional<string>");
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

        let materialized = materialize_aggregations(&ir, &data).unwrap();

        assert_eq!(
            materialized.tables[0].rows[0].values["display_name"],
            Value::Null
        );
    }

    #[test]
    fn rejects_missing_required_single_child_value() {
        let ir = single_value_aggregation_ir("string");
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

        let error = materialize_aggregations(&ir, &data).unwrap_err();

        assert!(
            error
                .to_string()
                .contains("expected exactly 1 row from `ItemProfile`")
        );
    }

    #[test]
    fn rejects_multiple_single_child_values() {
        let ir = single_value_aggregation_ir("optional<string>");
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

        let error = materialize_aggregations(&ir, &data).unwrap_err();

        assert!(
            error
                .to_string()
                .contains("expected at most 1 row from `ItemProfile`")
        );
    }

    fn aggregation_ir() -> ConfigIr {
        let schema: SchemaFile = toml::from_str(
            r#"
package = "game_config"

[[structs]]
name = "Reward"

[[structs.fields]]
name = "reward_item_id"
type = "i32"
required = true

[[structs.fields]]
name = "count"
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
name = "name"
type = "string"
required = true

[[tables.fields]]
name = "rewards"
type = "list<Reward>"
source_table = "ItemReward"
parent_key = "id"
child_key = "item_id"
order_by = "seq"

[[tables]]
name = "ItemReward"
mode = "list"

[[tables.fields]]
name = "item_id"
type = "i32"
required = true

[[tables.fields]]
name = "seq"
type = "i32"
required = true

[[tables.fields]]
name = "reward_item_id"
type = "i32"
required = true

[[tables.fields]]
name = "count"
type = "i32"
required = true
"#,
        )
        .unwrap();
        let ir = normalize_schema(schema).unwrap();
        validate_config_ir(&ir).unwrap();
        ir
    }

    fn single_value_aggregation_ir(field_type: &str) -> ConfigIr {
        let schema: SchemaFile = toml::from_str(&format!(
            r#"
package = "game_config"

[[tables]]
name = "Item"
mode = "map"
key = "id"

[[tables.fields]]
name = "id"
type = "i32"
required = true

[[tables.fields]]
name = "display_name"
type = "{field_type}"
source_table = "ItemProfile"
parent_key = "id"
child_key = "item_id"
value_field = "name"

[[tables]]
name = "ItemProfile"
mode = "list"

[[tables.fields]]
name = "item_id"
type = "i32"
required = true

[[tables.fields]]
name = "name"
type = "string"
required = true

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
}
