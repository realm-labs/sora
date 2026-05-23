use super::*;
use crate::normalize::normalize_schema;
use sora_schema::model::SchemaFile;

#[test]
fn validates_valid_ir() {
    let ir = example_ir(
        r#"
[[structs]]
name = "Reward"

[[structs.fields]]
name = "item_id"
type = "ref<Item.id>"

[[tables]]
name = "Item"
mode = "map"
key = "id"

[[tables.fields]]
name = "id"
type = "i32"
key = true
required = true

[[tables.fields]]
name = "item_type"
type = "enum<ItemType>"

[[tables.fields]]
name = "reward"
type = "struct<Reward>"

[[tables.indexes]]
name = "by_type"
fields = ["item_type"]
"#,
    );

    validate_config_ir(&ir).unwrap();
}

#[test]
fn rejects_duplicate_names_and_fields() {
    let duplicate_table = example_ir(
        r#"
[[tables]]
name = "Item"
mode = "list"

[[tables]]
name = "Item"
mode = "list"
"#,
    );
    assert!(matches!(
        validate_config_ir(&duplicate_table).unwrap_err(),
        SoraError::DuplicateSchemaName { kind: "table", name } if name == "Item"
    ));

    let duplicate_field = example_ir(
        r#"
[[tables]]
name = "Item"
mode = "list"

[[tables.fields]]
name = "id"
type = "i32"

[[tables.fields]]
name = "id"
type = "i64"
"#,
    );
    assert!(matches!(
        validate_config_ir(&duplicate_field).unwrap_err(),
        SoraError::DuplicateFieldName { owner_kind: "table", owner, field }
            if owner == "Item" && field == "id"
    ));
}

#[test]
fn rejects_unknown_type_references() {
    let ir = example_ir(
        r#"
[[tables]]
name = "Item"
mode = "list"

[[tables.fields]]
name = "kind"
type = "enum<Missing>"
"#,
    );

    assert!(matches!(
        validate_config_ir(&ir).unwrap_err(),
        SoraError::UnknownTypeReference { kind: "enum", name, .. } if name == "Missing"
    ));
}

#[test]
fn rejects_invalid_enum_aliases() {
    let unknown_target = example_ir(
        r#"
[[enums.aliases]]
name = "Missing"
alias = "weapon"
"#,
    );
    assert!(matches!(
        validate_config_ir(&unknown_target).unwrap_err(),
        SoraError::InvalidSchema(message)
            if message.contains("targets unknown value")
    ));

    let value_conflict = example_ir(
        r#"
[[enums.aliases]]
name = "Weapon"
alias = "Armor"
"#,
    );
    assert!(matches!(
        validate_config_ir(&value_conflict).unwrap_err(),
        SoraError::InvalidSchema(message)
            if message.contains("conflicts with an enum value")
    ));
}

#[test]
fn rejects_invalid_table_key_index_and_ref() {
    let missing_key = example_ir(
        r#"
[[tables]]
name = "Item"
mode = "map"
key = "id"
"#,
    );
    assert!(matches!(
        validate_config_ir(&missing_key).unwrap_err(),
        SoraError::MissingTableKey { table, field } if table == "Item" && field == "id"
    ));

    let map_without_key = example_ir(
        r#"
[[tables]]
name = "Item"
mode = "map"

[[tables.fields]]
name = "id"
type = "i32"
"#,
    );
    assert!(matches!(
        validate_config_ir(&map_without_key).unwrap_err(),
        SoraError::InvalidSchema(message) if message.contains("must declare `key`")
    ));

    let invalid_map_key_type = example_ir(
        r#"
[[tables]]
name = "Item"
mode = "map"
key = "weight"

[[tables.fields]]
name = "weight"
type = "f32"
"#,
    );
    assert!(matches!(
        validate_config_ir(&invalid_map_key_type).unwrap_err(),
        SoraError::InvalidSchema(message) if message.contains("unsupported key type")
    ));

    let bad_index = example_ir(
        r#"
[[tables]]
name = "Item"
mode = "list"

[[tables.fields]]
name = "id"
type = "i32"

[[tables.indexes]]
name = "bad"
fields = ["missing"]
"#,
    );
    assert!(matches!(
        validate_config_ir(&bad_index).unwrap_err(),
        SoraError::UnknownIndexField { table, index, field }
            if table == "Item" && index == "bad" && field == "missing"
    ));

    let bad_unique_index_type = example_ir(
        r#"
[[structs]]
name = "Tag"

[[structs.fields]]
name = "name"
type = "string"

[[tables]]
name = "Item"
mode = "map"
key = "id"

[[tables.fields]]
name = "id"
type = "i32"

[[tables.fields]]
name = "tag"
type = "struct<Tag>"

[[tables.indexes]]
name = "by_tag"
fields = ["tag"]
unique = true
"#,
    );
    assert!(matches!(
        validate_config_ir(&bad_unique_index_type).unwrap_err(),
        SoraError::InvalidSchema(message)
            if message.contains("index `by_tag`") && message.contains("unsupported key type")
    ));

    let bad_non_unique_index_type = example_ir(
        r#"
[[structs]]
name = "Tag"

[[structs.fields]]
name = "name"
type = "string"

[[tables]]
name = "Item"
mode = "map"
key = "id"

[[tables.fields]]
name = "id"
type = "i32"

[[tables.fields]]
name = "tag"
type = "struct<Tag>"

[[tables.indexes]]
name = "by_tag"
fields = ["tag"]
"#,
    );
    assert!(matches!(
        validate_config_ir(&bad_non_unique_index_type).unwrap_err(),
        SoraError::InvalidSchema(message)
            if message.contains("index `by_tag`") && message.contains("unsupported key type")
    ));

    let bad_ref = example_ir(
        r#"
[[tables]]
name = "Item"
mode = "list"

[[tables.fields]]
name = "id"
type = "ref<Missing.id>"
"#,
    );
    assert!(matches!(
        validate_config_ir(&bad_ref).unwrap_err(),
        SoraError::UnknownRefTable { table, .. } if table == "Missing"
    ));
}

#[test]
fn aggregation_key_compatibility_resolves_ref_types() {
    let ir = example_ir(
        r#"
[[structs]]
name = "Reward"

[[structs.fields]]
name = "item_id"
type = "ref<Item.id>"

[[structs.fields]]
name = "count"
type = "i32"

[[tables]]
name = "Item"
mode = "map"
key = "id"

[[tables.fields]]
name = "id"
type = "i32"

[[tables]]
name = "Quest"
mode = "map"
key = "id"

[[tables.fields]]
name = "id"
type = "i32"

[[tables.fields]]
name = "rewards"
type = "list<Reward>"
source_table = "QuestReward"
parent_key = "id"
child_key = "quest_id"

[[tables]]
name = "QuestReward"
mode = "list"

[[tables.fields]]
name = "quest_id"
type = "ref<Quest.id>"

[[tables.fields]]
name = "item_id"
type = "ref<Item.id>"

[[tables.fields]]
name = "count"
type = "i32"
"#,
    );

    validate_config_ir(&ir).unwrap();
}

#[test]
fn validates_tagged_columns_projection() {
    let ir = example_ir(
        r#"
[[unions]]
name = "Action"
tag = "type"

[[unions.variants]]
name = "AddItem"

[[unions.variants.fields]]
name = "item_id"
type = "i32"

[[unions.variants]]
name = "AddBuff"

[[unions.variants.fields]]
name = "buff_id"
type = "i32"

[[tables]]
name = "Event"
mode = "list"

[[tables.fields]]
name = "action"
type = "union<Action>"
parser = { kind = "tagged_columns", prefix = "" }
"#,
    );

    validate_config_ir(&ir).unwrap();
}

#[test]
fn rejects_tagged_columns_input_column_conflicts() {
    let ir = example_ir(
        r#"
[[unions]]
name = "Action"
tag = "type"

[[unions.variants]]
name = "AddItem"

[[unions.variants.fields]]
name = "item_id"
type = "i32"

[[tables]]
name = "Event"
mode = "list"

[[tables.fields]]
name = "type"
type = "string"

[[tables.fields]]
name = "action"
type = "union<Action>"
parser = { kind = "tagged_columns", prefix = "" }
"#,
    );

    assert!(matches!(
        validate_config_ir(&ir).unwrap_err(),
        SoraError::InvalidSchema(message)
            if message.contains("maps to input column `type`")
                && message.contains("already used")
    ));
}

#[test]
fn rejects_tagged_columns_outside_table_fields() {
    let ir = example_ir(
        r#"
[[unions]]
name = "Action"
tag = "type"

[[unions.variants]]
name = "AddItem"

[[unions.variants.fields]]
name = "nested"
type = "union<Action>"
parser = { kind = "tagged_columns" }
"#,
    );

    assert!(matches!(
        validate_config_ir(&ir).unwrap_err(),
        SoraError::InvalidSchema(message)
            if message.contains("tagged columns are only supported on table fields")
    ));
}

#[test]
fn rejects_incompatible_repeated_tagged_columns() {
    let ir = example_ir(
        r#"
[[unions]]
name = "Action"
tag = "type"

[[unions.variants]]
name = "AddItem"

[[unions.variants.fields]]
name = "value"
type = "i32"

[[unions.variants]]
name = "SetName"

[[unions.variants.fields]]
name = "value"
type = "string"

[[tables]]
name = "Event"
mode = "list"

[[tables.fields]]
name = "action"
type = "union<Action>"
parser = { kind = "tagged_columns" }
"#,
    );

    assert!(matches!(
        validate_config_ir(&ir).unwrap_err(),
        SoraError::InvalidSchema(message)
            if message.contains("incompatible repeated variant column `action.value`")
    ));
}

fn example_ir(extra: &str) -> ConfigIr {
    let source = format!(
        r#"
package = "game_config"

[[enums]]
name = "ItemType"
values = ["Weapon", "Armor"]

{extra}
"#
    );
    let schema: SchemaFile = toml::from_str(&source).unwrap();
    normalize_schema(schema).unwrap()
}
