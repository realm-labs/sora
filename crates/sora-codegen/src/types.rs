use sora_ir::model::{ConfigIr, TypeIr};

pub fn rust_type_name(ir: &ConfigIr, ty: &TypeIr) -> String {
    rust_type_name_inner(ir, ty)
}

pub fn kotlin_type_name(ir: &ConfigIr, ty: &TypeIr) -> String {
    kotlin_type_name_inner(ir, ty)
}

pub fn csharp_type_name(ir: &ConfigIr, ty: &TypeIr) -> String {
    csharp_type_name_inner(ir, ty)
}

pub fn java_type_name(ir: &ConfigIr, ty: &TypeIr) -> String {
    java_type_name_inner(ir, ty)
}

pub fn go_type_name(ir: &ConfigIr, ty: &TypeIr) -> String {
    go_type_name_inner(ir, ty)
}

pub fn lua_type_name(ir: &ConfigIr, ty: &TypeIr) -> String {
    lua_type_name_inner(ir, ty)
}

fn rust_type_name_inner(ir: &ConfigIr, ty: &TypeIr) -> String {
    match ty {
        TypeIr::Bool => "bool".to_owned(),
        TypeIr::I32 => "i32".to_owned(),
        TypeIr::I64 => "i64".to_owned(),
        TypeIr::F32 => "f32".to_owned(),
        TypeIr::F64 => "f64".to_owned(),
        TypeIr::String => "String".to_owned(),
        TypeIr::Enum(name) | TypeIr::Struct(name) | TypeIr::Union(name) => name.clone(),
        TypeIr::List(element) => format!("Vec<{}>", rust_type_name_inner(ir, element)),
        TypeIr::Array { element, len } => {
            format!("[{}; {len}]", rust_type_name_inner(ir, element))
        }
        TypeIr::Ref { table, field } => ref_type(ir, table, field, rust_type_name_inner, "i32"),
        TypeIr::Optional(element) => format!("Option<{}>", rust_type_name_inner(ir, element)),
    }
}

fn kotlin_type_name_inner(ir: &ConfigIr, ty: &TypeIr) -> String {
    match ty {
        TypeIr::Bool => "Boolean".to_owned(),
        TypeIr::I32 => "Int".to_owned(),
        TypeIr::I64 => "Long".to_owned(),
        TypeIr::F32 => "Float".to_owned(),
        TypeIr::F64 => "Double".to_owned(),
        TypeIr::String => "String".to_owned(),
        TypeIr::Enum(name) | TypeIr::Struct(name) | TypeIr::Union(name) => name.clone(),
        TypeIr::List(element) | TypeIr::Array { element, .. } => {
            format!("List<{}>", kotlin_type_name_inner(ir, element))
        }
        TypeIr::Ref { table, field } => ref_type(ir, table, field, kotlin_type_name_inner, "Int"),
        TypeIr::Optional(element) => format!("{}?", kotlin_type_name_inner(ir, element)),
    }
}

fn csharp_type_name_inner(ir: &ConfigIr, ty: &TypeIr) -> String {
    match ty {
        TypeIr::Bool => "bool".to_owned(),
        TypeIr::I32 => "int".to_owned(),
        TypeIr::I64 => "long".to_owned(),
        TypeIr::F32 => "float".to_owned(),
        TypeIr::F64 => "double".to_owned(),
        TypeIr::String => "string".to_owned(),
        TypeIr::Enum(name) | TypeIr::Struct(name) | TypeIr::Union(name) => name.clone(),
        TypeIr::List(element) | TypeIr::Array { element, .. } => {
            format!("List<{}>", csharp_type_name_inner(ir, element))
        }
        TypeIr::Ref { table, field } => ref_type(ir, table, field, csharp_type_name_inner, "int"),
        TypeIr::Optional(element) => format!("{}?", csharp_type_name_inner(ir, element)),
    }
}

fn java_type_name_inner(ir: &ConfigIr, ty: &TypeIr) -> String {
    match ty {
        TypeIr::Bool => "Boolean".to_owned(),
        TypeIr::I32 => "Integer".to_owned(),
        TypeIr::I64 => "Long".to_owned(),
        TypeIr::F32 => "Float".to_owned(),
        TypeIr::F64 => "Double".to_owned(),
        TypeIr::String => "String".to_owned(),
        TypeIr::Enum(name) | TypeIr::Struct(name) | TypeIr::Union(name) => name.clone(),
        TypeIr::List(element) | TypeIr::Array { element, .. } => {
            format!("java.util.List<{}>", java_type_name_inner(ir, element))
        }
        TypeIr::Ref { table, field } => ref_type(ir, table, field, java_type_name_inner, "Integer"),
        TypeIr::Optional(element) => java_type_name_inner(ir, element),
    }
}

fn go_type_name_inner(ir: &ConfigIr, ty: &TypeIr) -> String {
    match ty {
        TypeIr::Bool => "bool".to_owned(),
        TypeIr::I32 => "int32".to_owned(),
        TypeIr::I64 => "int64".to_owned(),
        TypeIr::F32 => "float32".to_owned(),
        TypeIr::F64 => "float64".to_owned(),
        TypeIr::String => "string".to_owned(),
        TypeIr::Enum(name) | TypeIr::Struct(name) | TypeIr::Union(name) => name.clone(),
        TypeIr::List(element) | TypeIr::Array { element, .. } => {
            format!("[]{}", go_type_name_inner(ir, element))
        }
        TypeIr::Ref { table, field } => ref_type(ir, table, field, go_type_name_inner, "int32"),
        TypeIr::Optional(element) => format!("*{}", go_type_name_inner(ir, element)),
    }
}

fn lua_type_name_inner(ir: &ConfigIr, ty: &TypeIr) -> String {
    match ty {
        TypeIr::Bool => "boolean".to_owned(),
        TypeIr::I32 | TypeIr::I64 => "integer".to_owned(),
        TypeIr::F32 | TypeIr::F64 => "number".to_owned(),
        TypeIr::String => "string".to_owned(),
        TypeIr::Enum(name) | TypeIr::Struct(name) | TypeIr::Union(name) => name.clone(),
        TypeIr::List(element) | TypeIr::Array { element, .. } => {
            format!("{}[]", lua_type_name_inner(ir, element))
        }
        TypeIr::Ref { table, field } => ref_type(ir, table, field, lua_type_name_inner, "integer"),
        TypeIr::Optional(element) => format!("{}?", lua_type_name_inner(ir, element)),
    }
}

fn ref_type(
    ir: &ConfigIr,
    table_name: &str,
    field_name: &str,
    mapper: fn(&ConfigIr, &TypeIr) -> String,
    fallback: &str,
) -> String {
    ir.tables
        .iter()
        .find(|table| table.name == table_name)
        .and_then(|table| table.fields.iter().find(|field| field.name == field_name))
        .map(|field| mapper(ir, &field.ty))
        .unwrap_or_else(|| fallback.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sora_ir::{normalize::normalize_schema, parse::parse_type};
    use sora_schema::model::SchemaFile;

    #[test]
    fn maps_rust_types() {
        let ir = example_ir();
        let cases = [
            ("bool", "bool"),
            ("i32", "i32"),
            ("i64", "i64"),
            ("f32", "f32"),
            ("f64", "f64"),
            ("string", "String"),
            ("enum<ItemType>", "ItemType"),
            ("struct<Reward>", "Reward"),
            ("union<Action>", "Action"),
            ("list<i32>", "Vec<i32>"),
            ("array<i32,3>", "[i32; 3]"),
            ("optional<string>", "Option<String>"),
            ("ref<Item.id>", "i32"),
        ];

        for (source, expected) in cases {
            assert_eq!(rust_type_name(&ir, &parse_type(source).unwrap()), expected);
        }
    }

    #[test]
    fn maps_kotlin_types() {
        let ir = example_ir();
        let cases = [
            ("bool", "Boolean"),
            ("i32", "Int"),
            ("i64", "Long"),
            ("f32", "Float"),
            ("f64", "Double"),
            ("string", "String"),
            ("enum<ItemType>", "ItemType"),
            ("struct<Reward>", "Reward"),
            ("union<Action>", "Action"),
            ("list<i32>", "List<Int>"),
            ("array<i32,3>", "List<Int>"),
            ("optional<string>", "String?"),
            ("ref<Item.id>", "Int"),
        ];

        for (source, expected) in cases {
            assert_eq!(
                kotlin_type_name(&ir, &parse_type(source).unwrap()),
                expected
            );
        }
    }

    #[test]
    fn maps_csharp_types() {
        let ir = example_ir();
        let cases = [
            ("bool", "bool"),
            ("i32", "int"),
            ("i64", "long"),
            ("f32", "float"),
            ("f64", "double"),
            ("string", "string"),
            ("enum<ItemType>", "ItemType"),
            ("struct<Reward>", "Reward"),
            ("union<Action>", "Action"),
            ("list<i32>", "List<int>"),
            ("array<i32,3>", "List<int>"),
            ("optional<string>", "string?"),
            ("optional<i32>", "int?"),
            ("ref<Item.id>", "int"),
        ];

        for (source, expected) in cases {
            assert_eq!(
                csharp_type_name(&ir, &parse_type(source).unwrap()),
                expected
            );
        }
    }

    #[test]
    fn maps_java_types() {
        let ir = example_ir();
        let cases = [
            ("bool", "Boolean"),
            ("i32", "Integer"),
            ("i64", "Long"),
            ("f32", "Float"),
            ("f64", "Double"),
            ("string", "String"),
            ("enum<ItemType>", "ItemType"),
            ("struct<Reward>", "Reward"),
            ("union<Action>", "Action"),
            ("list<i32>", "java.util.List<Integer>"),
            ("array<i32,3>", "java.util.List<Integer>"),
            ("optional<string>", "String"),
            ("ref<Item.id>", "Integer"),
        ];

        for (source, expected) in cases {
            assert_eq!(java_type_name(&ir, &parse_type(source).unwrap()), expected);
        }
    }

    #[test]
    fn maps_go_types() {
        let ir = example_ir();
        let cases = [
            ("bool", "bool"),
            ("i32", "int32"),
            ("i64", "int64"),
            ("f32", "float32"),
            ("f64", "float64"),
            ("string", "string"),
            ("enum<ItemType>", "ItemType"),
            ("struct<Reward>", "Reward"),
            ("union<Action>", "Action"),
            ("list<i32>", "[]int32"),
            ("array<i32,3>", "[]int32"),
            ("optional<string>", "*string"),
            ("ref<Item.id>", "int32"),
        ];

        for (source, expected) in cases {
            assert_eq!(go_type_name(&ir, &parse_type(source).unwrap()), expected);
        }
    }

    #[test]
    fn maps_lua_types() {
        let ir = example_ir();
        let cases = [
            ("bool", "boolean"),
            ("i32", "integer"),
            ("i64", "integer"),
            ("f32", "number"),
            ("f64", "number"),
            ("string", "string"),
            ("enum<ItemType>", "ItemType"),
            ("struct<Reward>", "Reward"),
            ("union<Action>", "Action"),
            ("list<i32>", "integer[]"),
            ("array<i32,3>", "integer[]"),
            ("optional<string>", "string?"),
            ("ref<Item.id>", "integer"),
        ];

        for (source, expected) in cases {
            assert_eq!(lua_type_name(&ir, &parse_type(source).unwrap()), expected);
        }
    }

    fn example_ir() -> ConfigIr {
        let schema: SchemaFile = toml::from_str(
            r#"
package = "game_config"

[[enums]]
name = "ItemType"
values = ["Weapon", "Armor", "Material", "Consumable"]

[[unions]]
name = "Action"

[[unions.variants]]
name = "AddItem"

[[unions.variants.fields]]
name = "item_id"
type = "i32"

[[tables]]
name = "Item"
mode = "map"
key = "id"

[[tables.fields]]
name = "id"
type = "i32"
key = true
required = true
comment = "Item id"

[[tables.fields]]
name = "item_type"
type = "enum<ItemType>"
required = true
comment = "Item type"
"#,
        )
        .unwrap();

        normalize_schema(schema).unwrap()
    }
}
