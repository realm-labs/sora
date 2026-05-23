use super::*;
use crate::model::{TableModeIr, TypeIr};

#[test]
fn normalizes_schema() {
    let schema: SchemaFile = toml::from_str(
        r#"
package = "game_config"

[[enums]]
name = "ItemType"
values = ["Weapon"]

[[enums.aliases]]
name = "Weapon"
alias = "weapon"

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
name = "tags"
type = "list<string>"
parser = { kind = "split", separator = "|" }
default = "starter"
length = [1, 3]
"#,
    )
    .unwrap();

    let ir = normalize_schema(schema).unwrap();
    assert_eq!(ir.package, "game_config");
    assert_eq!(ir.enums[0].name, "ItemType");
    assert_eq!(ir.enums[0].aliases[0].name, "Weapon");
    assert_eq!(ir.enums[0].aliases[0].alias, "weapon");
    assert_eq!(ir.tables[0].mode, TableModeIr::Map);
    assert!(ir.tables[0].fields[0].required);
    assert_eq!(ir.tables[0].fields[0].ty, TypeIr::I32);
    let parser = ir.tables[0].fields[1].parser.as_ref().unwrap();
    assert_eq!(parser.kind, "split");
    assert_eq!(parser.options["separator"], "|");
    assert_eq!(ir.tables[0].fields[1].default.as_deref(), Some("starter"));
    assert_eq!(ir.tables[0].fields[1].length, Some([1, 3]));
}

#[test]
fn normalizes_tuple_struct_parser() {
    let schema: SchemaFile = toml::from_str(
        r#"
package = "game_config"

[[structs]]
name = "Vec3"

[[structs.fields]]
name = "x"
type = "f32"

[[structs.fields]]
name = "y"
type = "f32"

[[structs.fields]]
name = "z"
type = "f32"

[[tables]]
name = "Spawn"
mode = "list"

[[tables.fields]]
name = "pos"
type = "struct<Vec3>"
parser = { kind = "tuple" }
"#,
    )
    .unwrap();

    let ir = normalize_schema(schema).unwrap();
    assert_eq!(
        ir.tables[0].fields[0]
            .parser
            .as_ref()
            .map(|parser| parser.kind.as_str()),
        Some("tuple")
    );
}

#[test]
fn normalizes_tuple_list_parser() {
    let schema: SchemaFile = toml::from_str(
        r#"
package = "game_config"

[[structs]]
name = "ResourceCost"

[[structs.fields]]
name = "kind"
type = "string"

[[structs.fields]]
name = "id"
type = "i32"

[[structs.fields]]
name = "count"
type = "i32"

[[tables]]
name = "Recipe"
mode = "list"

[[tables.fields]]
name = "materials"
type = "list<ResourceCost>"
parser = { kind = "tuple_list", item_separator = ";", separator = "," }
"#,
    )
    .unwrap();

    let ir = normalize_schema(schema).unwrap();
    let parser = ir.tables[0].fields[0].parser.as_ref().unwrap();
    assert_eq!(parser.kind, "tuple_list");
    assert_eq!(parser.options["item_separator"], ";");
    assert_eq!(parser.options["separator"], ",");
}

#[test]
fn normalizes_map_parser() {
    let schema: SchemaFile = toml::from_str(
        r#"
package = "game_config"

[[tables]]
name = "Item"
mode = "list"

[[tables.fields]]
name = "attributes"
type = "map<string,i32>"
parser = { kind = "map", item_separator = ";", separator = ":" }
"#,
    )
    .unwrap();

    let ir = normalize_schema(schema).unwrap();
    let parser = ir.tables[0].fields[0].parser.as_ref().unwrap();
    assert_eq!(parser.kind, "map");
    assert_eq!(parser.options["item_separator"], ";");
    assert_eq!(parser.options["separator"], ":");
}

#[test]
fn normalizes_tagged_columns_parser() {
    let schema: SchemaFile = toml::from_str(
        r#"
package = "game_config"

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
name = "action"
type = "union<Action>"
parser = { kind = "tagged_columns", prefix = "" }
"#,
    )
    .unwrap();

    let ir = normalize_schema(schema).unwrap();
    let parser = ir.tables[0].fields[0].parser.as_ref().unwrap();
    assert_eq!(parser.kind, "tagged_columns");
    assert_eq!(parser.options["prefix"], "");
}

#[test]
fn default_collections_do_not_need_parser_metadata() {
    let schema: SchemaFile = toml::from_str(
        r#"
package = "game_config"

[[tables]]
name = "Item"
mode = "list"

[[tables.fields]]
name = "tags"
type = "list<string>"
"#,
    )
    .unwrap();

    let ir = normalize_schema(schema).unwrap();
    assert_eq!(ir.tables[0].fields[0].parser, None);
}

#[test]
fn rejects_invalid_parser_metadata() {
    let scalar_split: SchemaFile = toml::from_str(
        r#"
package = "game_config"

[[tables]]
name = "Item"
mode = "list"

[[tables.fields]]
name = "name"
type = "string"
parser = { kind = "split", separator = "|" }
"#,
    )
    .unwrap();
    assert!(matches!(
        normalize_schema(scalar_split).unwrap_err(),
        SoraError::InvalidSchema(message) if message.contains("is not list or array")
    ));

    let scalar_tuple_list: SchemaFile = toml::from_str(
        r#"
package = "game_config"

[[tables]]
name = "Recipe"
mode = "list"

[[tables.fields]]
name = "materials"
type = "list<string>"
parser = { kind = "tuple_list" }
"#,
    )
    .unwrap();
    assert!(matches!(
        normalize_schema(scalar_tuple_list).unwrap_err(),
        SoraError::InvalidSchema(message) if message.contains("not list or array of struct")
    ));

    let scalar_map: SchemaFile = toml::from_str(
        r#"
package = "game_config"

[[tables]]
name = "Item"
mode = "list"

[[tables.fields]]
name = "name"
type = "string"
parser = { kind = "map" }
"#,
    )
    .unwrap();
    assert!(matches!(
        normalize_schema(scalar_map).unwrap_err(),
        SoraError::InvalidSchema(message) if message.contains("is not map")
    ));

    let split_map: SchemaFile = toml::from_str(
        r#"
package = "game_config"

[[tables]]
name = "Item"
mode = "list"

[[tables.fields]]
name = "attributes"
type = "map<string,i32>"
parser = { kind = "split" }
"#,
    )
    .unwrap();
    assert!(matches!(
        normalize_schema(split_map).unwrap_err(),
        SoraError::InvalidSchema(message) if message.contains("is not list or array")
    ));

    let unknown_parser: SchemaFile = toml::from_str(
        r#"
package = "game_config"

[[tables]]
name = "Item"
mode = "list"

[[tables.fields]]
name = "name"
type = "string"
parser = { kind = "lua" }
"#,
    )
    .unwrap();
    assert!(matches!(
        normalize_schema(unknown_parser).unwrap_err(),
        SoraError::InvalidSchema(message) if message.contains("unsupported parser")
    ));

    let list_union_tagged_columns: SchemaFile = toml::from_str(
        r#"
package = "game_config"

[[unions]]
name = "Action"
tag = "type"

[[unions.variants]]
name = "AddItem"

[[tables]]
name = "Event"
mode = "list"

[[tables.fields]]
name = "actions"
type = "list<union<Action>>"
parser = { kind = "tagged_columns" }
"#,
    )
    .unwrap();
    assert!(matches!(
        normalize_schema(list_union_tagged_columns).unwrap_err(),
        SoraError::InvalidSchema(message) if message.contains("is not union")
    ));

    let tagged_columns_bad_option: SchemaFile = toml::from_str(
        r#"
package = "game_config"

[[unions]]
name = "Action"
tag = "type"

[[unions.variants]]
name = "AddItem"

[[tables]]
name = "Event"
mode = "list"

[[tables.fields]]
name = "action"
type = "union<Action>"
parser = { kind = "tagged_columns", separator = "," }
"#,
    )
    .unwrap();
    assert!(matches!(
        normalize_schema(tagged_columns_bad_option).unwrap_err(),
        SoraError::InvalidSchema(message) if message.contains("unsupported option `separator`")
    ));
}

#[test]
fn validates_length_constraints() {
    let invalid_type: SchemaFile = toml::from_str(
        r#"
package = "game_config"

[[tables]]
name = "Item"
mode = "list"

[[tables.fields]]
name = "id"
type = "i32"
length = [1, 4]
"#,
    )
    .unwrap();
    assert!(matches!(
        normalize_schema(invalid_type).unwrap_err(),
        SoraError::InvalidSchema(message) if message.contains("declares `length`")
    ));

    let invalid_range: SchemaFile = toml::from_str(
        r#"
package = "game_config"

[[tables]]
name = "Item"
mode = "list"

[[tables.fields]]
name = "name"
type = "string"
length = [4, 1]
"#,
    )
    .unwrap();
    assert!(matches!(
        normalize_schema(invalid_range).unwrap_err(),
        SoraError::InvalidSchema(message) if message.contains("invalid `length`")
    ));
}

#[test]
fn rejects_invalid_tuple_parser_metadata() {
    let scalar_parser: SchemaFile = toml::from_str(
        r#"
package = "game_config"

[[tables]]
name = "Spawn"
mode = "list"

[[tables.fields]]
name = "pos"
type = "string"
parser = { kind = "tuple" }
"#,
    )
    .unwrap();
    assert!(matches!(
        normalize_schema(scalar_parser).unwrap_err(),
        SoraError::InvalidSchema(message) if message.contains("is not struct")
    ));
}

#[test]
fn derived_list_fields_do_not_need_separator_metadata() {
    let schema: SchemaFile = toml::from_str(
        r#"
package = "game_config"

[[structs]]
name = "Reward"

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

[[tables.fields]]
name = "rewards"
type = "list<Reward>"
from = { table = "ItemReward", parent_key = "id", child_key = "item_id" }

[[tables]]
name = "ItemReward"
mode = "list"
"#,
    )
    .unwrap();

    let ir = normalize_schema(schema).unwrap();
    assert!(ir.tables[0].fields[1].derived_from.is_some());
    assert_eq!(ir.tables[0].fields[1].parser, None);
}

#[test]
fn normalizes_derived_from_field() {
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

[[tables.fields]]
name = "display_name"
type = "string"
from = { table = "ItemProfile", parent_key = "id", child_key = "item_id", field = "name" }

[[tables]]
name = "ItemProfile"
mode = "list"
"#,
    )
    .unwrap();

    let ir = normalize_schema(schema).unwrap();
    let derived_from = ir.tables[0].fields[1].derived_from.as_ref().unwrap();
    assert_eq!(derived_from.value_field.as_deref(), Some("name"));
}

#[test]
fn rejects_incomplete_from_metadata() {
    let schema: SchemaFile = toml::from_str(
        r#"
package = "game_config"

[[tables]]
name = "Item"
mode = "list"

[[tables.fields]]
name = "display_name"
type = "string"
from = { table = "ItemProfile", field = "name" }
"#,
    )
    .unwrap();

    assert!(matches!(
        normalize_schema(schema).unwrap_err(),
        SoraError::InvalidSchema(message)
            if message.contains("incomplete `from` metadata")
    ));
}

#[test]
fn rejects_default_on_derived_fields() {
    let schema: SchemaFile = toml::from_str(
        r#"
package = "game_config"

[[structs]]
name = "Reward"

[[tables]]
name = "Item"
mode = "map"
key = "id"

[[tables.fields]]
name = "id"
type = "i32"

[[tables.fields]]
name = "rewards"
type = "list<Reward>"
from = { table = "ItemReward", parent_key = "id", child_key = "item_id" }
default = "[]"

[[tables]]
name = "ItemReward"
mode = "list"
"#,
    )
    .unwrap();

    assert!(matches!(
        normalize_schema(schema).unwrap_err(),
        SoraError::InvalidSchema(message)
            if message.contains("declares both `default` and `from`")
    ));
}

#[test]
fn rejects_default_on_tagged_columns_fields() {
    let schema: SchemaFile = toml::from_str(
        r#"
package = "game_config"

[[unions]]
name = "Action"
tag = "type"

[[unions.variants]]
name = "AddItem"

[[tables]]
name = "Event"
mode = "list"

[[tables.fields]]
name = "action"
type = "union<Action>"
parser = { kind = "tagged_columns" }
default = "{\"type\":\"AddItem\"}"
"#,
    )
    .unwrap();

    assert!(matches!(
        normalize_schema(schema).unwrap_err(),
        SoraError::InvalidSchema(message)
            if message.contains("declares both `default` and parser `tagged_columns`")
    ));
}
