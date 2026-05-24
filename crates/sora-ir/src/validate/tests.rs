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

    let non_key_ref = example_ir(
        r#"
[[tables]]
name = "Item"
mode = "map"
key = "id"

[[tables.fields]]
name = "id"
type = "i32"

[[tables.fields]]
name = "name"
type = "string"

[[tables]]
name = "Reward"
mode = "list"

[[tables.fields]]
name = "item_name"
type = "ref<Item.name>"
"#,
    );
    assert!(matches!(
        validate_config_ir(&non_key_ref).unwrap_err(),
        SoraError::InvalidSchema(message)
            if message.contains("references `Item.name`")
                && message.contains("primary key")
    ));

    let list_ref_to_primary_key = example_ir(
        r#"
[[tables]]
name = "Item"
mode = "map"
key = "id"

[[tables.fields]]
name = "id"
type = "i32"

[[tables]]
name = "Reward"
mode = "list"

[[tables.fields]]
name = "item_ids"
type = "list<ref<Item.id>>"
"#,
    );
    validate_config_ir(&list_ref_to_primary_key).unwrap();

    let ref_to_list_table_field = example_ir(
        r#"
[[tables]]
name = "RewardSource"
mode = "list"

[[tables.fields]]
name = "id"
type = "i32"

[[tables]]
name = "Reward"
mode = "list"

[[tables.fields]]
name = "source_id"
type = "ref<RewardSource.id>"
"#,
    );
    assert!(matches!(
        validate_config_ir(&ref_to_list_table_field).unwrap_err(),
        SoraError::InvalidSchema(message)
            if message.contains("references `RewardSource.id`")
                && message.contains("map table")
    ));
}

#[test]
fn derived_field_key_compatibility_resolves_ref_types() {
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
from = { table = "QuestReward", parent_key = "id", child_key = "quest_id" }

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
fn validates_single_value_derived_field() {
    let ir = example_ir(
        r#"
[[unions]]
name = "EventCondition"
tag = "type"

[[unions.variants]]
name = "QuestCompleted"

[[unions.variants.fields]]
name = "quest_id"
type = "i32"

[[tables]]
name = "Event"
mode = "map"
key = "id"

[[tables.fields]]
name = "id"
type = "i32"

[[tables.fields]]
name = "condition"
type = "union<EventCondition>"
from = { table = "EventConditionEntry", parent_key = "id", child_key = "event_id", field = "value" }

[[tables]]
name = "EventConditionEntry"
mode = "list"

[[tables.fields]]
name = "event_id"
type = "ref<Event.id>"

[[tables.fields]]
name = "value"
type = "union<EventCondition>"
"#,
    );

    validate_config_ir(&ir).unwrap();
}

#[test]
fn validates_optional_value_derived_field() {
    let ir = example_ir(
        r#"
[[tables]]
name = "Item"
mode = "map"
key = "id"

[[tables.fields]]
name = "id"
type = "i32"

[[tables.fields]]
name = "display_name"
type = "optional<string>"
from = { table = "ItemProfile", parent_key = "id", child_key = "item_id", field = "name" }

[[tables]]
name = "ItemProfile"
mode = "list"

[[tables.fields]]
name = "item_id"
type = "ref<Item.id>"

[[tables.fields]]
name = "name"
type = "string"
"#,
    );

    validate_config_ir(&ir).unwrap();
}

#[test]
fn rejects_scalar_derived_field_without_from_field() {
    let ir = example_ir(
        r#"
[[tables]]
name = "Item"
mode = "map"
key = "id"

[[tables.fields]]
name = "id"
type = "i32"

[[tables.fields]]
name = "display_name"
type = "string"
from = { table = "ItemProfile", parent_key = "id", child_key = "item_id" }

[[tables]]
name = "ItemProfile"
mode = "list"

[[tables.fields]]
name = "item_id"
type = "ref<Item.id>"
"#,
    );

    assert!(matches!(
        validate_config_ir(&ir).unwrap_err(),
        SoraError::InvalidSchema(message)
            if message.contains("must assemble struct values or declare `from.field`")
    ));
}

#[test]
fn rejects_unknown_derived_source_field() {
    let ir = example_ir(
        r#"
[[tables]]
name = "Item"
mode = "map"
key = "id"

[[tables.fields]]
name = "id"
type = "i32"

[[tables.fields]]
name = "display_name"
type = "string"
from = { table = "ItemProfile", parent_key = "id", child_key = "item_id", field = "missing" }

[[tables]]
name = "ItemProfile"
mode = "list"

[[tables.fields]]
name = "item_id"
type = "ref<Item.id>"
"#,
    );

    assert!(matches!(
        validate_config_ir(&ir).unwrap_err(),
        SoraError::UnknownRefField { ref_field, .. } if ref_field == "missing"
    ));
}

#[test]
fn rejects_incompatible_derived_source_field_type() {
    let ir = example_ir(
        r#"
[[tables]]
name = "Item"
mode = "map"
key = "id"

[[tables.fields]]
name = "id"
type = "i32"

[[tables.fields]]
name = "display_name"
type = "string"
from = { table = "ItemProfile", parent_key = "id", child_key = "item_id", field = "level" }

[[tables]]
name = "ItemProfile"
mode = "list"

[[tables.fields]]
name = "item_id"
type = "ref<Item.id>"

[[tables.fields]]
name = "level"
type = "i32"
"#,
    );

    assert!(matches!(
        validate_config_ir(&ir).unwrap_err(),
        SoraError::InvalidSchema(message)
            if message.contains("maps source field `level` with incompatible type")
    ));
}

#[test]
fn rejects_optional_source_for_required_derived_field() {
    let ir = example_ir(
        r#"
[[tables]]
name = "Item"
mode = "map"
key = "id"

[[tables.fields]]
name = "id"
type = "i32"

[[tables.fields]]
name = "display_name"
type = "string"
from = { table = "ItemProfile", parent_key = "id", child_key = "item_id", field = "name" }

[[tables]]
name = "ItemProfile"
mode = "list"

[[tables.fields]]
name = "item_id"
type = "ref<Item.id>"

[[tables.fields]]
name = "name"
type = "optional<string>"
"#,
    );

    assert!(matches!(
        validate_config_ir(&ir).unwrap_err(),
        SoraError::InvalidSchema(message)
            if message.contains("maps source field `name` with incompatible type")
    ));
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
            if message.contains("column projection parsers are only supported on table fields")
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

#[test]
fn validates_struct_columns_projection() {
    let ir = example_ir(
        r#"
[[structs]]
name = "SkillEffect"

[[structs.fields]]
name = "element"
type = "enum<ItemType>"

[[structs.fields]]
name = "power"
type = "i32"

[[tables]]
name = "EquipmentSet"
mode = "list"

[[tables.fields]]
name = "bonus"
type = "struct<SkillEffect>"
parser = { kind = "columns", prefix = "bonus_" }
"#,
    );

    validate_config_ir(&ir).unwrap();
}

#[test]
fn rejects_struct_columns_input_column_conflicts() {
    let ir = example_ir(
        r#"
[[structs]]
name = "SkillEffect"

[[structs.fields]]
name = "power"
type = "i32"

[[tables]]
name = "EquipmentSet"
mode = "list"

[[tables.fields]]
name = "power"
type = "i32"

[[tables.fields]]
name = "bonus"
type = "struct<SkillEffect>"
parser = { kind = "columns", prefix = "" }
"#,
    );

    assert!(matches!(
        validate_config_ir(&ir).unwrap_err(),
        SoraError::InvalidSchema(message)
            if message.contains("maps to input column `power`")
                && message.contains("already used")
    ));
}

#[test]
fn rejects_columns_outside_table_fields() {
    let ir = example_ir(
        r#"
[[structs]]
name = "SkillEffect"

[[structs.fields]]
name = "power"
type = "i32"

[[structs.fields]]
name = "nested"
type = "struct<SkillEffect>"
parser = { kind = "columns" }
"#,
    );

    assert!(matches!(
        validate_config_ir(&ir).unwrap_err(),
        SoraError::InvalidSchema(message)
            if message.contains("column projection parsers are only supported on table fields")
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
