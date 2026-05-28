use sora_ir::model::{ConfigIr, TypeIr};

use crate::options::{RustCodegenOptions, RustDateTimeType, RustStringStorage};

pub fn rust_type_name(ir: &ConfigIr, ty: &TypeIr) -> String {
    rust_type_name_with_options(ir, ty, &RustCodegenOptions::default())
}

pub fn rust_type_name_with_options(
    ir: &ConfigIr,
    ty: &TypeIr,
    options: &RustCodegenOptions,
) -> String {
    rust_type_name_inner(ir, ty, options)
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

pub fn scala_type_name(ir: &ConfigIr, ty: &TypeIr) -> String {
    scala_type_name_inner(ir, ty)
}

pub fn go_type_name(ir: &ConfigIr, ty: &TypeIr) -> String {
    go_type_name_inner(ir, ty)
}

pub fn dart_type_name(ir: &ConfigIr, ty: &TypeIr) -> String {
    dart_type_name_inner(ir, ty)
}

pub fn godot_type_name(ir: &ConfigIr, ty: &TypeIr) -> String {
    godot_type_name_inner(ir, ty)
}

pub fn python_type_name(ir: &ConfigIr, ty: &TypeIr) -> String {
    python_type_name_inner(ir, ty)
}

fn rust_type_name_inner(ir: &ConfigIr, ty: &TypeIr, options: &RustCodegenOptions) -> String {
    match ty {
        TypeIr::Bool => "bool".to_owned(),
        TypeIr::I8 => "i8".to_owned(),
        TypeIr::U8 => "u8".to_owned(),
        TypeIr::I16 => "i16".to_owned(),
        TypeIr::U16 => "u16".to_owned(),
        TypeIr::I32 => "i32".to_owned(),
        TypeIr::U32 => "u32".to_owned(),
        TypeIr::I64 => "i64".to_owned(),
        TypeIr::Duration => "std::time::Duration".to_owned(),
        TypeIr::DateTime => rust_datetime_type(options),
        TypeIr::F32 => "f32".to_owned(),
        TypeIr::F64 => "f64".to_owned(),
        TypeIr::String => rust_string_type(options),
        TypeIr::Text => "super::runtime::TextKey".to_owned(),
        TypeIr::Enum(name) | TypeIr::Struct(name) | TypeIr::Union(name) => name.clone(),
        TypeIr::List(element) => format!("Vec<{}>", rust_type_name_inner(ir, element, options)),
        TypeIr::Set(element) => {
            format!(
                "std::collections::HashSet<{}>",
                rust_type_name_inner(ir, element, options)
            )
        }
        TypeIr::Map { key, value } => format!(
            "std::collections::HashMap<{}, {}>",
            rust_type_name_inner(ir, key, options),
            rust_type_name_inner(ir, value, options)
        ),
        TypeIr::Array { element, len } => {
            format!("[{}; {len}]", rust_type_name_inner(ir, element, options))
        }
        TypeIr::Ref { table, field } => {
            ref_type_with_options(ir, table, field, options, rust_type_name_inner, "i32")
        }
        TypeIr::Optional(element) => {
            format!("Option<{}>", rust_type_name_inner(ir, element, options))
        }
    }
}

fn rust_string_type(options: &RustCodegenOptions) -> String {
    match options.string_storage {
        RustStringStorage::Owned => "String".to_owned(),
        RustStringStorage::Arc => "std::sync::Arc<str>".to_owned(),
    }
}

fn rust_datetime_type(options: &RustCodegenOptions) -> String {
    match options.datetime_type {
        RustDateTimeType::SystemTime => "std::time::SystemTime".to_owned(),
        RustDateTimeType::Chrono => "chrono::DateTime<chrono::Utc>".to_owned(),
    }
}

fn kotlin_type_name_inner(ir: &ConfigIr, ty: &TypeIr) -> String {
    match ty {
        TypeIr::Bool => "Boolean".to_owned(),
        TypeIr::I8 => "Byte".to_owned(),
        TypeIr::U8 | TypeIr::I16 => "Short".to_owned(),
        TypeIr::U16 | TypeIr::I32 => "Int".to_owned(),
        TypeIr::U32 | TypeIr::I64 => "Long".to_owned(),
        TypeIr::Duration => "kotlin.time.Duration".to_owned(),
        TypeIr::DateTime => "java.time.Instant".to_owned(),
        TypeIr::F32 => "Float".to_owned(),
        TypeIr::F64 => "Double".to_owned(),
        TypeIr::String => "String".to_owned(),
        TypeIr::Text => "TextKey".to_owned(),
        TypeIr::Enum(name) | TypeIr::Struct(name) | TypeIr::Union(name) => name.clone(),
        TypeIr::List(element) | TypeIr::Set(element) | TypeIr::Array { element, .. } => {
            format!("List<{}>", kotlin_type_name_inner(ir, element))
        }
        TypeIr::Map { key, value } => format!(
            "Map<{}, {}>",
            kotlin_type_name_inner(ir, key),
            kotlin_type_name_inner(ir, value)
        ),
        TypeIr::Ref { table, field } => ref_type(ir, table, field, kotlin_type_name_inner, "Int"),
        TypeIr::Optional(element) => format!("{}?", kotlin_type_name_inner(ir, element)),
    }
}

fn csharp_type_name_inner(ir: &ConfigIr, ty: &TypeIr) -> String {
    match ty {
        TypeIr::Bool => "bool".to_owned(),
        TypeIr::I8 => "sbyte".to_owned(),
        TypeIr::U8 => "byte".to_owned(),
        TypeIr::I16 => "short".to_owned(),
        TypeIr::U16 => "ushort".to_owned(),
        TypeIr::I32 => "int".to_owned(),
        TypeIr::U32 => "uint".to_owned(),
        TypeIr::I64 => "long".to_owned(),
        TypeIr::Duration => "global::System.TimeSpan".to_owned(),
        TypeIr::DateTime => "global::System.DateTimeOffset".to_owned(),
        TypeIr::F32 => "float".to_owned(),
        TypeIr::F64 => "double".to_owned(),
        TypeIr::String => "string".to_owned(),
        TypeIr::Text => "TextKey".to_owned(),
        TypeIr::Enum(name) | TypeIr::Struct(name) | TypeIr::Union(name) => name.clone(),
        TypeIr::List(element) | TypeIr::Set(element) | TypeIr::Array { element, .. } => {
            format!("List<{}>", csharp_type_name_inner(ir, element))
        }
        TypeIr::Map { key, value } => format!(
            "Dictionary<{}, {}>",
            csharp_type_name_inner(ir, key),
            csharp_type_name_inner(ir, value)
        ),
        TypeIr::Ref { table, field } => ref_type(ir, table, field, csharp_type_name_inner, "int"),
        TypeIr::Optional(element) => format!("{}?", csharp_type_name_inner(ir, element)),
    }
}

fn java_type_name_inner(ir: &ConfigIr, ty: &TypeIr) -> String {
    match ty {
        TypeIr::Bool => "Boolean".to_owned(),
        TypeIr::I8 | TypeIr::U8 | TypeIr::I16 => "Integer".to_owned(),
        TypeIr::U16 | TypeIr::I32 => "Integer".to_owned(),
        TypeIr::U32 | TypeIr::I64 => "Long".to_owned(),
        TypeIr::Duration => "java.time.Duration".to_owned(),
        TypeIr::DateTime => "java.time.Instant".to_owned(),
        TypeIr::F32 => "Float".to_owned(),
        TypeIr::F64 => "Double".to_owned(),
        TypeIr::String => "String".to_owned(),
        TypeIr::Text => "TextKey".to_owned(),
        TypeIr::Enum(name) | TypeIr::Struct(name) | TypeIr::Union(name) => name.clone(),
        TypeIr::List(element) | TypeIr::Set(element) | TypeIr::Array { element, .. } => {
            format!("java.util.List<{}>", java_type_name_inner(ir, element))
        }
        TypeIr::Map { key, value } => format!(
            "java.util.Map<{}, {}>",
            java_type_name_inner(ir, key),
            java_type_name_inner(ir, value)
        ),
        TypeIr::Ref { table, field } => ref_type(ir, table, field, java_type_name_inner, "Integer"),
        TypeIr::Optional(element) => java_type_name_inner(ir, element),
    }
}

fn scala_type_name_inner(ir: &ConfigIr, ty: &TypeIr) -> String {
    match ty {
        TypeIr::Bool => "Boolean".to_owned(),
        TypeIr::I8 | TypeIr::U8 | TypeIr::I16 | TypeIr::U16 | TypeIr::I32 => "Int".to_owned(),
        TypeIr::U32 | TypeIr::I64 => "Long".to_owned(),
        TypeIr::Duration => "scala.concurrent.duration.FiniteDuration".to_owned(),
        TypeIr::DateTime => "java.time.Instant".to_owned(),
        TypeIr::F32 => "Float".to_owned(),
        TypeIr::F64 => "Double".to_owned(),
        TypeIr::String => "String".to_owned(),
        TypeIr::Text => "TextKey".to_owned(),
        TypeIr::Enum(name) | TypeIr::Struct(name) | TypeIr::Union(name) => name.clone(),
        TypeIr::List(element) | TypeIr::Set(element) | TypeIr::Array { element, .. } => {
            format!("Vector[{}]", scala_type_name_inner(ir, element))
        }
        TypeIr::Map { key, value } => format!(
            "Map[{}, {}]",
            scala_type_name_inner(ir, key),
            scala_type_name_inner(ir, value)
        ),
        TypeIr::Ref { table, field } => ref_type(ir, table, field, scala_type_name_inner, "Int"),
        TypeIr::Optional(element) => format!("Option[{}]", scala_type_name_inner(ir, element)),
    }
}

fn go_type_name_inner(ir: &ConfigIr, ty: &TypeIr) -> String {
    match ty {
        TypeIr::Bool => "bool".to_owned(),
        TypeIr::I8 => "int8".to_owned(),
        TypeIr::U8 => "uint8".to_owned(),
        TypeIr::I16 => "int16".to_owned(),
        TypeIr::U16 => "uint16".to_owned(),
        TypeIr::I32 => "int32".to_owned(),
        TypeIr::U32 => "uint32".to_owned(),
        TypeIr::I64 => "int64".to_owned(),
        TypeIr::Duration => "time.Duration".to_owned(),
        TypeIr::DateTime => "time.Time".to_owned(),
        TypeIr::F32 => "float32".to_owned(),
        TypeIr::F64 => "float64".to_owned(),
        TypeIr::String => "string".to_owned(),
        TypeIr::Text => "TextKey".to_owned(),
        TypeIr::Enum(name) | TypeIr::Struct(name) | TypeIr::Union(name) => name.clone(),
        TypeIr::List(element) | TypeIr::Set(element) | TypeIr::Array { element, .. } => {
            format!("[]{}", go_type_name_inner(ir, element))
        }
        TypeIr::Map { key, value } => format!(
            "map[{}]{}",
            go_type_name_inner(ir, key),
            go_type_name_inner(ir, value)
        ),
        TypeIr::Ref { table, field } => ref_type(ir, table, field, go_type_name_inner, "int32"),
        TypeIr::Optional(element) => format!("*{}", go_type_name_inner(ir, element)),
    }
}

fn dart_type_name_inner(ir: &ConfigIr, ty: &TypeIr) -> String {
    match ty {
        TypeIr::Bool => "bool".to_owned(),
        TypeIr::I8
        | TypeIr::U8
        | TypeIr::I16
        | TypeIr::U16
        | TypeIr::I32
        | TypeIr::U32
        | TypeIr::I64 => "int".to_owned(),
        TypeIr::Duration => "Duration".to_owned(),
        TypeIr::DateTime => "DateTime".to_owned(),
        TypeIr::F32 | TypeIr::F64 => "double".to_owned(),
        TypeIr::String => "String".to_owned(),
        TypeIr::Text => "TextKey".to_owned(),
        TypeIr::Enum(name) | TypeIr::Struct(name) | TypeIr::Union(name) => name.clone(),
        TypeIr::List(element) | TypeIr::Set(element) | TypeIr::Array { element, .. } => {
            format!("List<{}>", dart_type_name_inner(ir, element))
        }
        TypeIr::Map { key, value } => format!(
            "Map<{}, {}>",
            dart_type_name_inner(ir, key),
            dart_type_name_inner(ir, value)
        ),
        TypeIr::Ref { table, field } => ref_type(ir, table, field, dart_type_name_inner, "int"),
        TypeIr::Optional(element) => format!("{}?", dart_type_name_inner(ir, element)),
    }
}

fn godot_type_name_inner(ir: &ConfigIr, ty: &TypeIr) -> String {
    match ty {
        TypeIr::Bool => "bool".to_owned(),
        TypeIr::I8
        | TypeIr::U8
        | TypeIr::I16
        | TypeIr::U16
        | TypeIr::I32
        | TypeIr::U32
        | TypeIr::I64
        | TypeIr::Duration
        | TypeIr::DateTime => "int".to_owned(),
        TypeIr::F32 | TypeIr::F64 => "float".to_owned(),
        TypeIr::String | TypeIr::Enum(_) => "String".to_owned(),
        TypeIr::Text => "SoraRuntime.TextKey".to_owned(),
        TypeIr::Struct(name) | TypeIr::Union(name) => name.clone(),
        TypeIr::List(_) | TypeIr::Set(_) | TypeIr::Map { .. } | TypeIr::Array { .. } => {
            "Array".to_owned()
        }
        TypeIr::Ref { table, field } => ref_type(ir, table, field, godot_type_name_inner, "int"),
        TypeIr::Optional(_) => "Variant".to_owned(),
    }
}

fn python_type_name_inner(ir: &ConfigIr, ty: &TypeIr) -> String {
    match ty {
        TypeIr::Bool => "bool".to_owned(),
        TypeIr::I8
        | TypeIr::U8
        | TypeIr::I16
        | TypeIr::U16
        | TypeIr::I32
        | TypeIr::U32
        | TypeIr::I64 => "int".to_owned(),
        TypeIr::Duration => "datetime.timedelta".to_owned(),
        TypeIr::DateTime => "datetime.datetime".to_owned(),
        TypeIr::F32 | TypeIr::F64 => "float".to_owned(),
        TypeIr::String => "str".to_owned(),
        TypeIr::Text => "TextKey".to_owned(),
        TypeIr::Enum(name) | TypeIr::Struct(name) | TypeIr::Union(name) => name.clone(),
        TypeIr::List(element) | TypeIr::Set(element) | TypeIr::Array { element, .. } => {
            format!("list[{}]", python_type_name_inner(ir, element))
        }
        TypeIr::Map { key, value } => format!(
            "dict[{}, {}]",
            python_type_name_inner(ir, key),
            python_type_name_inner(ir, value)
        ),
        TypeIr::Ref { table, field } => ref_type(ir, table, field, python_type_name_inner, "int"),
        TypeIr::Optional(element) => format!("{} | None", python_type_name_inner(ir, element)),
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

fn ref_type_with_options<T>(
    ir: &ConfigIr,
    table_name: &str,
    field_name: &str,
    options: &T,
    mapper: fn(&ConfigIr, &TypeIr, &T) -> String,
    fallback: &str,
) -> String {
    ir.tables
        .iter()
        .find(|table| table.name == table_name)
        .and_then(|table| table.fields.iter().find(|field| field.name == field_name))
        .map(|field| mapper(ir, &field.ty, options))
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
            ("duration", "std::time::Duration"),
            ("datetime", "std::time::SystemTime"),
            ("f32", "f32"),
            ("f64", "f64"),
            ("string", "String"),
            ("enum<ItemType>", "ItemType"),
            ("struct<Reward>", "Reward"),
            ("union<Action>", "Action"),
            ("list<i32>", "Vec<i32>"),
            ("set<string>", "std::collections::HashSet<String>"),
            ("map<string,i32>", "std::collections::HashMap<String, i32>"),
            ("array<i32,3>", "[i32; 3]"),
            ("optional<string>", "Option<String>"),
            ("ref<Item.id>", "i32"),
        ];

        for (source, expected) in cases {
            assert_eq!(rust_type_name(&ir, &parse_type(source).unwrap()), expected);
        }
    }

    #[test]
    fn maps_rust_datetime_chrono_option() {
        let ir = example_ir();
        let options = RustCodegenOptions {
            datetime_type: RustDateTimeType::Chrono,
            ..RustCodegenOptions::default()
        };

        assert_eq!(
            rust_type_name_with_options(&ir, &parse_type("datetime").unwrap(), &options),
            "chrono::DateTime<chrono::Utc>"
        );
    }

    #[test]
    fn maps_kotlin_types() {
        let ir = example_ir();
        let cases = [
            ("bool", "Boolean"),
            ("i32", "Int"),
            ("i64", "Long"),
            ("duration", "kotlin.time.Duration"),
            ("datetime", "java.time.Instant"),
            ("f32", "Float"),
            ("f64", "Double"),
            ("string", "String"),
            ("enum<ItemType>", "ItemType"),
            ("struct<Reward>", "Reward"),
            ("union<Action>", "Action"),
            ("list<i32>", "List<Int>"),
            ("set<string>", "List<String>"),
            ("map<string,i32>", "Map<String, Int>"),
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
            ("duration", "global::System.TimeSpan"),
            ("datetime", "global::System.DateTimeOffset"),
            ("f32", "float"),
            ("f64", "double"),
            ("string", "string"),
            ("enum<ItemType>", "ItemType"),
            ("struct<Reward>", "Reward"),
            ("union<Action>", "Action"),
            ("list<i32>", "List<int>"),
            ("set<string>", "List<string>"),
            ("map<string,i32>", "Dictionary<string, int>"),
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
            ("duration", "java.time.Duration"),
            ("datetime", "java.time.Instant"),
            ("f32", "Float"),
            ("f64", "Double"),
            ("string", "String"),
            ("enum<ItemType>", "ItemType"),
            ("struct<Reward>", "Reward"),
            ("union<Action>", "Action"),
            ("list<i32>", "java.util.List<Integer>"),
            ("set<string>", "java.util.List<String>"),
            ("map<string,i32>", "java.util.Map<String, Integer>"),
            ("array<i32,3>", "java.util.List<Integer>"),
            ("optional<string>", "String"),
            ("ref<Item.id>", "Integer"),
        ];

        for (source, expected) in cases {
            assert_eq!(java_type_name(&ir, &parse_type(source).unwrap()), expected);
        }
    }

    #[test]
    fn maps_scala_types() {
        let ir = example_ir();
        let cases = [
            ("bool", "Boolean"),
            ("i32", "Int"),
            ("i64", "Long"),
            ("duration", "scala.concurrent.duration.FiniteDuration"),
            ("datetime", "java.time.Instant"),
            ("f32", "Float"),
            ("f64", "Double"),
            ("string", "String"),
            ("enum<ItemType>", "ItemType"),
            ("struct<Reward>", "Reward"),
            ("union<Action>", "Action"),
            ("list<i32>", "Vector[Int]"),
            ("set<string>", "Vector[String]"),
            ("map<string,i32>", "Map[String, Int]"),
            ("array<i32,3>", "Vector[Int]"),
            ("optional<string>", "Option[String]"),
            ("ref<Item.id>", "Int"),
        ];

        for (source, expected) in cases {
            assert_eq!(scala_type_name(&ir, &parse_type(source).unwrap()), expected);
        }
    }

    #[test]
    fn maps_go_types() {
        let ir = example_ir();
        let cases = [
            ("bool", "bool"),
            ("i32", "int32"),
            ("i64", "int64"),
            ("duration", "time.Duration"),
            ("datetime", "time.Time"),
            ("f32", "float32"),
            ("f64", "float64"),
            ("string", "string"),
            ("enum<ItemType>", "ItemType"),
            ("struct<Reward>", "Reward"),
            ("union<Action>", "Action"),
            ("list<i32>", "[]int32"),
            ("set<string>", "[]string"),
            ("map<string,i32>", "map[string]int32"),
            ("array<i32,3>", "[]int32"),
            ("optional<string>", "*string"),
            ("ref<Item.id>", "int32"),
        ];

        for (source, expected) in cases {
            assert_eq!(go_type_name(&ir, &parse_type(source).unwrap()), expected);
        }
    }

    #[test]
    fn maps_dart_types() {
        let ir = example_ir();
        let cases = [
            ("bool", "bool"),
            ("i32", "int"),
            ("i64", "int"),
            ("duration", "Duration"),
            ("datetime", "DateTime"),
            ("f32", "double"),
            ("f64", "double"),
            ("string", "String"),
            ("enum<ItemType>", "ItemType"),
            ("struct<Reward>", "Reward"),
            ("union<Action>", "Action"),
            ("list<i32>", "List<int>"),
            ("set<string>", "List<String>"),
            ("map<string,i32>", "Map<String, int>"),
            ("array<i32,3>", "List<int>"),
            ("optional<string>", "String?"),
            ("ref<Item.id>", "int"),
        ];

        for (source, expected) in cases {
            assert_eq!(dart_type_name(&ir, &parse_type(source).unwrap()), expected);
        }
    }

    #[test]
    fn maps_godot_types() {
        let ir = example_ir();
        let cases = [
            ("bool", "bool"),
            ("i32", "int"),
            ("i64", "int"),
            ("duration", "int"),
            ("datetime", "int"),
            ("f32", "float"),
            ("f64", "float"),
            ("string", "String"),
            ("enum<ItemType>", "String"),
            ("struct<Reward>", "Reward"),
            ("union<Action>", "Action"),
            ("list<i32>", "Array"),
            ("set<string>", "Array"),
            ("map<string,i32>", "Array"),
            ("array<i32,3>", "Array"),
            ("optional<string>", "Variant"),
            ("ref<Item.id>", "int"),
        ];

        for (source, expected) in cases {
            assert_eq!(godot_type_name(&ir, &parse_type(source).unwrap()), expected);
        }
    }

    #[test]
    fn maps_python_types() {
        let ir = example_ir();
        let cases = [
            ("bool", "bool"),
            ("i32", "int"),
            ("i64", "int"),
            ("duration", "datetime.timedelta"),
            ("datetime", "datetime.datetime"),
            ("f32", "float"),
            ("f64", "float"),
            ("string", "str"),
            ("enum<ItemType>", "ItemType"),
            ("struct<Reward>", "Reward"),
            ("union<Action>", "Action"),
            ("list<i32>", "list[int]"),
            ("set<string>", "list[str]"),
            ("map<string,i32>", "dict[str, int]"),
            ("array<i32,3>", "list[int]"),
            ("optional<string>", "str | None"),
            ("ref<Item.id>", "int"),
        ];

        for (source, expected) in cases {
            assert_eq!(
                python_type_name(&ir, &parse_type(source).unwrap()),
                expected
            );
        }
    }

    #[test]
    fn maps_integer_width_types() {
        let ir = example_ir();
        let cases = [
            (
                "i8", "i8", "Byte", "sbyte", "Integer", "Int", "int8", "int", "int", "int",
            ),
            (
                "u8", "u8", "Short", "byte", "Integer", "Int", "uint8", "int", "int", "int",
            ),
            (
                "i16", "i16", "Short", "short", "Integer", "Int", "int16", "int", "int", "int",
            ),
            (
                "u16", "u16", "Int", "ushort", "Integer", "Int", "uint16", "int", "int", "int",
            ),
            (
                "u32", "u32", "Long", "uint", "Long", "Long", "uint32", "int", "int", "int",
            ),
        ];

        for (source, rust, kotlin, csharp, java, scala, go, dart, godot, python) in cases {
            let ty = parse_type(source).unwrap();
            assert_eq!(rust_type_name(&ir, &ty), rust);
            assert_eq!(kotlin_type_name(&ir, &ty), kotlin);
            assert_eq!(csharp_type_name(&ir, &ty), csharp);
            assert_eq!(java_type_name(&ir, &ty), java);
            assert_eq!(scala_type_name(&ir, &ty), scala);
            assert_eq!(go_type_name(&ir, &ty), go);
            assert_eq!(dart_type_name(&ir, &ty), dart);
            assert_eq!(godot_type_name(&ir, &ty), godot);
            assert_eq!(python_type_name(&ir, &ty), python);
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
comment = "Item id"

[[tables.fields]]
name = "item_type"
type = "enum<ItemType>"
comment = "Item type"
"#,
        )
        .unwrap();

        normalize_schema(schema).unwrap()
    }
}
