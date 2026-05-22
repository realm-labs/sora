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
                    ("item_type".to_owned(), Value::String("weapon".to_owned())),
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
fn rejects_duplicate_unique_index_keys() {
    let ir = index_ir();
    let data = ConfigData {
        tables: vec![TableData {
            name: "Item".to_owned(),
            rows: vec![
                RowData {
                    values: BTreeMap::from([
                        ("id".to_owned(), Value::Integer(1001)),
                        ("item_type".to_owned(), Value::String("Weapon".to_owned())),
                        ("name".to_owned(), Value::String("Iron Sword".to_owned())),
                    ]),
                },
                RowData {
                    values: BTreeMap::from([
                        ("id".to_owned(), Value::Integer(1002)),
                        ("item_type".to_owned(), Value::String("Weapon".to_owned())),
                        ("name".to_owned(), Value::String("Iron Sword".to_owned())),
                    ]),
                },
            ],
        }],
    };

    let error = validate_config_data(&ir, &data).unwrap_err();

    assert!(matches!(
        error,
        SoraError::DuplicateIndexKey { table, index, key }
            if table == "Item" && index == "by_type_name" && key == "item_type=Weapon,name=Iron Sword"
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
fn validates_string_and_collection_lengths() {
    let ir = length_ir();
    let valid = ConfigData {
        tables: vec![TableData {
            name: "Item".to_owned(),
            rows: vec![RowData {
                values: BTreeMap::from([
                    ("id".to_owned(), Value::Integer(1001)),
                    ("name".to_owned(), Value::String("Sword".to_owned())),
                    (
                        "tags".to_owned(),
                        Value::List(vec![
                            Value::String("sharp".to_owned()),
                            Value::String("rare".to_owned()),
                        ]),
                    ),
                ]),
            }],
        }],
    };
    validate_config_data(&ir, &valid).unwrap();

    let invalid_name = ConfigData {
        tables: vec![TableData {
            name: "Item".to_owned(),
            rows: vec![RowData {
                values: BTreeMap::from([
                    ("id".to_owned(), Value::Integer(1001)),
                    ("name".to_owned(), Value::String("A".to_owned())),
                    (
                        "tags".to_owned(),
                        Value::List(vec![Value::String("x".to_owned())]),
                    ),
                ]),
            }],
        }],
    };
    let error = validate_config_data(&ir, &invalid_name).unwrap_err();
    assert!(matches!(
        error,
        SoraError::LengthOutOfBounds { field, actual: 1, min: 2, max: 8, .. }
            if field == "name"
    ));

    let invalid_tags = ConfigData {
        tables: vec![TableData {
            name: "Item".to_owned(),
            rows: vec![RowData {
                values: BTreeMap::from([
                    ("id".to_owned(), Value::Integer(1001)),
                    ("name".to_owned(), Value::String("Sword".to_owned())),
                    (
                        "tags".to_owned(),
                        Value::List(vec![
                            Value::String("a".to_owned()),
                            Value::String("b".to_owned()),
                            Value::String("c".to_owned()),
                        ]),
                    ),
                ]),
            }],
        }],
    };
    let error = validate_config_data(&ir, &invalid_tags).unwrap_err();
    assert!(matches!(
        error,
        SoraError::LengthOutOfBounds { field, actual: 3, min: 1, max: 2, .. }
            if field == "tags"
    ));
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

[[enums.aliases]]
name = "Weapon"
alias = "weapon"

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

fn index_ir() -> ConfigIr {
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

[[tables.fields]]
name = "id"
type = "i32"
required = true

[[tables.fields]]
name = "item_type"
type = "enum<ItemType>"
required = true

[[tables.fields]]
name = "name"
type = "string"
required = true

[[tables.indexes]]
name = "by_type_name"
fields = ["item_type", "name"]
unique = true
"#,
    )
    .unwrap();

    normalize_schema(schema).unwrap()
}

fn length_ir() -> ConfigIr {
    let schema: SchemaFile = toml::from_str(
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
name = "name"
type = "string"
required = true
length = [2, 8]

[[tables.fields]]
name = "tags"
type = "list<string>"
length = [1, 2]
"#,
    )
    .unwrap();

    normalize_schema(schema).unwrap()
}
