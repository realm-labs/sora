use sora_ir::model::{ConfigIr, TypeIr};

pub fn rust_type_name(ir: &ConfigIr, ty: &TypeIr) -> String {
    rust_type_name_inner(ir, ty)
}

pub fn kotlin_type_name(ir: &ConfigIr, ty: &TypeIr) -> String {
    kotlin_type_name_inner(ir, ty)
}

fn rust_type_name_inner(ir: &ConfigIr, ty: &TypeIr) -> String {
    match ty {
        TypeIr::Bool => "bool".to_owned(),
        TypeIr::I32 => "i32".to_owned(),
        TypeIr::I64 => "i64".to_owned(),
        TypeIr::F32 => "f32".to_owned(),
        TypeIr::F64 => "f64".to_owned(),
        TypeIr::String => "String".to_owned(),
        TypeIr::Enum(name) | TypeIr::Struct(name) => name.clone(),
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
        TypeIr::Enum(name) | TypeIr::Struct(name) => name.clone(),
        TypeIr::List(element) | TypeIr::Array { element, .. } => {
            format!("List<{}>", kotlin_type_name_inner(ir, element))
        }
        TypeIr::Ref { table, field } => ref_type(ir, table, field, kotlin_type_name_inner, "Int"),
        TypeIr::Optional(element) => format!("{}?", kotlin_type_name_inner(ir, element)),
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
