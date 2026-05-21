use super::*;
use crate::model::{
    CStandardIr, CppStandardIr, EnumReprIr, ErlangEnumReprIr, LuaEnumReprIr, LuaVersionIr,
    RuntimeFormatIr, TableModeIr, TypeIr,
};

#[test]
fn normalizes_schema() {
    let schema: SchemaFile = toml::from_str(
        r#"
package = "game_config"

[[enums]]
name = "ItemType"
values = ["Weapon"]

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
    assert_eq!(ir.codegen.rust.runtime_format, RuntimeFormatIr::Sora);
    assert_eq!(ir.codegen.kotlin.runtime_format, RuntimeFormatIr::Sora);
    assert_eq!(ir.codegen.dart.runtime_format, RuntimeFormatIr::Sora);
    assert_eq!(ir.codegen.godot.runtime_format, RuntimeFormatIr::Sora);
    assert_eq!(ir.codegen.c.runtime_format, RuntimeFormatIr::Sora);
    assert_eq!(ir.codegen.c.c_standard, CStandardIr::C11);
    assert_eq!(ir.codegen.cpp.runtime_format, RuntimeFormatIr::Sora);
    assert_eq!(ir.codegen.cpp.cpp_standard, CppStandardIr::Cpp17);
    assert_eq!(ir.codegen.typescript.runtime_format, RuntimeFormatIr::Sora);
    assert_eq!(ir.codegen.typescript.enum_repr, EnumReprIr::String);
    assert_eq!(ir.codegen.javascript.runtime_format, RuntimeFormatIr::Sora);
    assert_eq!(ir.codegen.javascript.enum_repr, EnumReprIr::String);
    assert!(ir.codegen.javascript.emit_dts);
    assert_eq!(ir.codegen.erlang.runtime_format, RuntimeFormatIr::Sora);
    assert_eq!(ir.codegen.erlang.enum_repr, ErlangEnumReprIr::Atom);
    assert_eq!(ir.codegen.lua.runtime_format, RuntimeFormatIr::Sora);
    assert_eq!(ir.codegen.lua.lua_version, LuaVersionIr::Lua54);
    assert_eq!(ir.codegen.lua.enum_repr, LuaEnumReprIr::String);
    assert_eq!(ir.enums[0].name, "ItemType");
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
fn aggregation_list_fields_do_not_need_separator_metadata() {
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
source_table = "ItemReward"
parent_key = "id"
child_key = "item_id"

[[tables]]
name = "ItemReward"
mode = "list"
"#,
    )
    .unwrap();

    let ir = normalize_schema(schema).unwrap();
    assert!(ir.tables[0].fields[1].aggregation.is_some());
    assert_eq!(ir.tables[0].fields[1].parser, None);
}

#[test]
fn rejects_default_on_aggregation_fields() {
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
source_table = "ItemReward"
parent_key = "id"
child_key = "item_id"
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
            if message.contains("declares both `default` and aggregation")
    ));
}
