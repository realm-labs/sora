use std::path::{Path, PathBuf};

use minijinja::context;
use serde::Serialize;
use sora_diagnostics::Result;
use sora_ir::model::{ConfigIr, TableModeIr, TypeIr};

use crate::{
    generator::{CodeGenerator, runtime_format_name},
    model::{
        BaseField, BaseIndex, BaseModel, BaseRecord, BaseTable, BaseUnion, BaseUnionVariant,
        build_base_model,
    },
    render::{ensure_dir, render_template, write_file},
    types::kotlin_type_name,
};

pub struct KotlinCodeGenerator;

impl CodeGenerator for KotlinCodeGenerator {
    fn generate(&self, ir: &ConfigIr, out_dir: &Path) -> Result<()> {
        ensure_dir(out_dir)?;
        let model = KotlinModel::from_base_model(ir, build_base_model(ir)?);
        let package_dir = kotlin_package_dir(out_dir, &model.package)?;
        let runtime_format = runtime_format_name(ir.codegen.kotlin.runtime_format);

        for item in &model.enums {
            let rendered = render_template(
                "kotlin",
                "enum.kt.j2",
                context! { package => &model.package, enum => item, runtime_format => runtime_format },
            )?;
            write_file(&package_dir.join(format!("{}.kt", item.name)), rendered)?;
        }

        for record in &model.records {
            let rendered = render_template(
                "kotlin",
                "data_class.kt.j2",
                context! { package => &model.package, record => record, runtime_format => runtime_format },
            )?;
            write_file(
                &package_dir.join(format!("{}.kt", record.pascal_name)),
                rendered,
            )?;
        }

        for union in &model.unions {
            let rendered = render_template(
                "kotlin",
                "union.kt.j2",
                context! { package => &model.package, union => union, runtime_format => runtime_format },
            )?;
            write_file(
                &package_dir.join(format!("{}.kt", union.pascal_name)),
                rendered,
            )?;
        }

        let rendered = render_template(
            "kotlin",
            "runtime.kt.j2",
            context! { package => &model.package, runtime_format => runtime_format },
        )?;
        write_file(&package_dir.join("Runtime.kt"), rendered)?;

        let rendered = render_template(
            "kotlin",
            "config.kt.j2",
            context! { model => &model, runtime_format => runtime_format },
        )?;
        write_file(&package_dir.join("SoraConfig.kt"), rendered)?;

        let rendered = render_template("kotlin", "package.kt.j2", context! { model => &model })?;
        write_file(&package_dir.join("Package.kt"), rendered)
    }
}

#[derive(Debug, Clone, Serialize)]
struct KotlinModel {
    package: String,
    enums: Vec<KotlinEnum>,
    unions: Vec<KotlinUnion>,
    records: Vec<KotlinRecord>,
    tables: Vec<KotlinTable>,
}

#[derive(Debug, Clone, Serialize)]
struct KotlinEnum {
    name: String,
    values: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct KotlinUnion {
    pascal_name: String,
    tag: String,
    variants: Vec<KotlinUnionVariant>,
}

#[derive(Debug, Clone, Serialize)]
struct KotlinUnionVariant {
    name: String,
    fields: Vec<KotlinField>,
}

#[derive(Debug, Clone, Serialize)]
struct KotlinRecord {
    pascal_name: String,
    fields: Vec<KotlinField>,
}

#[derive(Debug, Clone, Serialize)]
struct KotlinTable {
    name: String,
    pascal_name: String,
    camel_name: String,
    snake_name: String,
    mode: String,
    container_type: String,
    row_type: String,
    key_name: Option<String>,
    key_field_name: Option<String>,
    key_type: Option<String>,
    unique_indexes: Vec<KotlinIndex>,
    non_unique_indexes: Vec<KotlinIndex>,
}

#[derive(Debug, Clone, Serialize)]
struct KotlinIndex {
    pascal_name: String,
    field_name: String,
    param_camel_name: String,
    key_type: String,
}

#[derive(Debug, Clone, Serialize)]
struct KotlinField {
    raw_name: String,
    name: String,
    type_name: String,
    decode: String,
    value_decode: String,
    comment: Option<String>,
}

impl KotlinModel {
    fn from_base_model(ir: &ConfigIr, model: BaseModel) -> Self {
        Self {
            package: model.package,
            enums: model
                .enums
                .into_iter()
                .map(|item| KotlinEnum {
                    name: item.pascal_name,
                    values: item.values,
                })
                .collect(),
            unions: model
                .unions
                .into_iter()
                .map(|item| kotlin_union(ir, item))
                .collect(),
            records: model
                .records
                .into_iter()
                .map(|item| kotlin_record(ir, item))
                .collect(),
            tables: model
                .tables
                .into_iter()
                .map(|item| kotlin_table(ir, item))
                .collect(),
        }
    }
}

fn kotlin_union(ir: &ConfigIr, union: BaseUnion) -> KotlinUnion {
    KotlinUnion {
        pascal_name: union.pascal_name,
        tag: union.tag,
        variants: union
            .variants
            .into_iter()
            .map(|variant| kotlin_variant(ir, variant))
            .collect(),
    }
}

fn kotlin_variant(ir: &ConfigIr, variant: BaseUnionVariant) -> KotlinUnionVariant {
    KotlinUnionVariant {
        name: variant.pascal_name,
        fields: variant
            .fields
            .into_iter()
            .map(|field| kotlin_field(ir, field))
            .collect(),
    }
}

fn kotlin_record(ir: &ConfigIr, record: BaseRecord) -> KotlinRecord {
    KotlinRecord {
        pascal_name: record.pascal_name,
        fields: record
            .fields
            .into_iter()
            .map(|field| kotlin_field(ir, field))
            .collect(),
    }
}

fn kotlin_table(ir: &ConfigIr, table: BaseTable) -> KotlinTable {
    let row_type = table.pascal_name.clone();
    let key_type = table
        .key_field
        .as_ref()
        .map(|field| kotlin_type_name(ir, &field.ty));
    let container_type = kotlin_container_type(table.mode, &row_type, key_type.as_deref());
    let key_field_name = table
        .key_field
        .as_ref()
        .map(|field| field.camel_name.clone());

    KotlinTable {
        name: table.name,
        pascal_name: table.pascal_name,
        camel_name: table.camel_name,
        snake_name: table.snake_name,
        mode: table.mode_name,
        container_type,
        row_type,
        key_name: table.key_name,
        key_field_name,
        key_type,
        unique_indexes: table
            .unique_indexes
            .into_iter()
            .map(|index| kotlin_index(ir, index))
            .collect(),
        non_unique_indexes: table
            .non_unique_indexes
            .into_iter()
            .map(|index| kotlin_index(ir, index))
            .collect(),
    }
}

fn kotlin_index(ir: &ConfigIr, index: BaseIndex) -> KotlinIndex {
    KotlinIndex {
        pascal_name: index.pascal_name,
        field_name: index.field.camel_name.clone(),
        param_camel_name: index.field.camel_name,
        key_type: kotlin_type_name(ir, &index.field.ty),
    }
}

fn kotlin_field(ir: &ConfigIr, field: BaseField) -> KotlinField {
    let value_decode = kotlin_value_decode_expr(ir, &field.ty, "__VALUE__");
    KotlinField {
        raw_name: field.raw_name,
        name: field.camel_name,
        type_name: kotlin_type_name(ir, &field.ty),
        decode: kotlin_decode_expr(ir, &field.ty),
        value_decode,
        comment: field.comment,
    }
}

fn kotlin_container_type(mode: TableModeIr, row_type: &str, key_type: Option<&str>) -> String {
    match mode {
        TableModeIr::List => format!("List<{row_type}>"),
        TableModeIr::Map => match key_type {
            Some(key_type) => format!("Map<{key_type}, {row_type}>"),
            None => format!("List<{row_type}>"),
        },
        TableModeIr::Singleton => row_type.to_owned(),
    }
}

fn kotlin_decode_expr(ir: &ConfigIr, ty: &TypeIr) -> String {
    match ty {
        TypeIr::Bool => "reader.readBool()".to_owned(),
        TypeIr::I32 => "reader.readI32()".to_owned(),
        TypeIr::I64 => "reader.readI64()".to_owned(),
        TypeIr::F32 => "reader.readF32()".to_owned(),
        TypeIr::F64 => "reader.readF64()".to_owned(),
        TypeIr::String => "reader.readString()".to_owned(),
        TypeIr::Enum(name) | TypeIr::Struct(name) | TypeIr::Union(name) => {
            format!("{name}.decode(reader)")
        }
        TypeIr::List(element) | TypeIr::Array { element, .. } => {
            format!("reader.readList {{ {} }}", kotlin_decode_expr(ir, element))
        }
        TypeIr::Ref { table, field } => ir
            .tables
            .iter()
            .find(|candidate| candidate.name == *table)
            .and_then(|table| {
                table
                    .fields
                    .iter()
                    .find(|candidate| candidate.name == *field)
            })
            .map(|field| kotlin_decode_expr(ir, &field.ty))
            .unwrap_or_else(|| "reader.readI32()".to_owned()),
        TypeIr::Optional(element) => {
            format!(
                "reader.readOptional {{ {} }}",
                kotlin_decode_expr(ir, element)
            )
        }
    }
}

fn kotlin_value_decode_expr(ir: &ConfigIr, ty: &TypeIr, value: &str) -> String {
    match ty {
        TypeIr::Bool => format!("{value}.asBool()"),
        TypeIr::I32 => format!("{value}.asInt()"),
        TypeIr::I64 => format!("{value}.asLong()"),
        TypeIr::F32 => format!("{value}.asFloat()"),
        TypeIr::F64 => format!("{value}.asDouble()"),
        TypeIr::String => format!("{value}.asString()"),
        TypeIr::Enum(name) | TypeIr::Struct(name) | TypeIr::Union(name) => {
            format!("{name}.decode({value})")
        }
        TypeIr::List(element) | TypeIr::Array { element, .. } => {
            format!(
                "{value}.asList {{ item -> {} }}",
                kotlin_value_decode_expr(ir, element, "item")
            )
        }
        TypeIr::Ref { table, field } => ir
            .tables
            .iter()
            .find(|candidate| candidate.name == *table)
            .and_then(|table| {
                table
                    .fields
                    .iter()
                    .find(|candidate| candidate.name == *field)
            })
            .map(|field| kotlin_value_decode_expr(ir, &field.ty, value))
            .unwrap_or_else(|| format!("{value}.asInt()")),
        TypeIr::Optional(element) => {
            format!(
                "if ({value}.isNull()) null else {}",
                kotlin_value_decode_expr(ir, element, value)
            )
        }
    }
}

fn kotlin_package_dir(out_dir: &Path, package: &str) -> Result<PathBuf> {
    let mut path = out_dir.to_path_buf();
    for segment in package.split('.') {
        if !is_kotlin_package_segment(segment) {
            return Err(sora_diagnostics::SoraError::InvalidSchema(format!(
                "kotlin package `{package}` must use dot-separated identifier segments"
            )));
        }
        path.push(segment);
    }
    Ok(path)
}

fn is_kotlin_package_segment(segment: &str) -> bool {
    let mut chars = segment.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || first == '_')
        && chars.all(|value| value.is_ascii_alphanumeric() || value == '_')
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        csharp::CSharpCodeGenerator, go::GoCodeGenerator, java::JavaCodeGenerator,
        rust::RustCodeGenerator,
    };
    use sora_ir::{
        model::{ConfigIr, RuntimeFormatIr},
        normalize::normalize_schema,
    };
    use sora_schema::model::SchemaFile;
    use std::{
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };

    static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn generates_rust_and_kotlin_files() {
        let ir = example_ir();
        let base = temp_dir();
        let rust_out = base.join("rust");
        let kotlin_out = base.join("kotlin");
        let kotlin_package_out = kotlin_out.join("game_config");

        RustCodeGenerator.generate(&ir, &rust_out).unwrap();
        KotlinCodeGenerator.generate(&ir, &kotlin_out).unwrap();

        let rust_item = std::fs::read_to_string(rust_out.join("item.rs")).unwrap();
        let rust_item_type = std::fs::read_to_string(rust_out.join("item_type.rs")).unwrap();
        let rust_action = std::fs::read_to_string(rust_out.join("action.rs")).unwrap();
        let rust_runtime = std::fs::read_to_string(rust_out.join("runtime.rs")).unwrap();
        let rust_mod = std::fs::read_to_string(rust_out.join("mod.rs")).unwrap();
        let kotlin_item = std::fs::read_to_string(kotlin_package_out.join("Item.kt")).unwrap();
        let kotlin_action = std::fs::read_to_string(kotlin_package_out.join("Action.kt")).unwrap();
        let kotlin_runtime =
            std::fs::read_to_string(kotlin_package_out.join("Runtime.kt")).unwrap();
        let kotlin_config =
            std::fs::read_to_string(kotlin_package_out.join("SoraConfig.kt")).unwrap();

        assert!(rust_item.starts_with("// This file was generated by Sora. Do not edit manually."));
        assert!(rust_item.contains("pub struct Item"));
        assert!(rust_item.contains("/// Item id"));
        assert!(rust_item.contains("pub item_type: ItemType"));
        assert!(rust_item.contains("pub action: Action"));
        assert!(rust_item.contains("impl super::runtime::SoraDecode for Item"));
        assert!(!rust_item.contains("impl std::fmt::Display for Item"));
        assert!(!rust_item_type.contains("impl std::fmt::Display for ItemType"));
        assert!(rust_action.contains("pub enum Action"));
        assert!(rust_action.contains("AddItem {"));
        assert!(rust_action.contains("impl super::runtime::SoraDecode for Action"));
        assert!(!rust_action.contains("impl std::fmt::Display for Action"));
        assert!(rust_runtime.contains("pub struct SoraBundle"));
        assert!(rust_mod.contains("pub struct SoraConfig"));
        assert!(rust_mod.contains("from_bytes"));
        assert!(rust_mod.contains("pub type SoraMap<K, V> = std::collections::HashMap<K, V>;"));
        assert!(rust_mod.contains("pub trait SoraTable: std::any::Any + Send + Sync"));
        assert!(rust_mod.contains("tables: SoraMap<&'static str, Box<dyn SoraTable>>"));
        assert!(rust_mod.contains("pub struct ItemTable"));
        assert!(rust_mod.contains("rows: SoraMap<i32, item::Item>"));
        assert!(rust_mod.contains("by_name: SoraMap<String, i32>"));
        assert!(rust_mod.contains("by_item_type: SoraMap<item_type::ItemType, Vec<i32>>"));
        assert!(rust_mod.contains("impl SoraTable for ItemTable"));
        assert!(rust_mod.contains("fn key(&self) -> Option<&'static str>"));
        assert!(rust_mod.contains("Some(\"id\")"));
        assert!(rust_mod.contains("pub fn tables(&self) -> impl Iterator<Item = &dyn SoraTable>"));
        assert!(rust_mod.contains("impl std::ops::Deref for ItemTable"));
        assert!(
            rust_mod.contains("tables.insert(\"Item\", Box::new(ItemTable::decode(&bundle)?));")
        );
        assert!(rust_mod.contains("|row| row.id"));
        assert!(rust_mod.contains("|row| row.name.clone()"));
        assert!(rust_mod.contains("|row| row.item_type"));
        assert!(
            rust_mod.contains("fn table<T: SoraTable + 'static>(&self, name: &'static str) -> &T")
        );
        assert!(!rust_mod.contains("as_any"));
        assert!(rust_mod.contains("let table: &dyn std::any::Any = table.as_ref();"));
        assert!(rust_mod.contains("table.downcast_ref::<T>()"));
        assert!(rust_mod.contains("pub fn item(&self) -> &ItemTable"));
        assert!(rust_mod.contains("pub fn get(&self, key: i32) -> Option<&item::Item>"));
        assert!(rust_mod.contains("pub fn get_by_name(&self, name: &str) -> Option<&item::Item>"));
        assert!(rust_mod.contains(
                "pub fn find_by_item_type(&self, item_type: item_type::ItemType) -> impl Iterator<Item = &item::Item>"
        ));
        assert!(!rust_mod.contains("pub fn get_item"));
        assert!(!rust_mod.contains("pub fn iter_item"));
        assert!(!rust_mod.contains("decode_singleton_table"));
        assert!(
            kotlin_item.starts_with("// This file was generated by Sora. Do not edit manually.")
        );
        assert!(kotlin_item.contains("data class Item"));
        assert!(kotlin_item.contains("package game_config"));
        assert!(kotlin_item.contains("/** Item id */"));
        assert!(kotlin_item.contains("val itemType: ItemType"));
        assert!(kotlin_item.contains("val action: Action"));
        assert!(kotlin_action.contains("sealed class Action"));
        assert!(kotlin_action.contains("data class AddItem"));
        assert!(kotlin_action.contains("fun decode(reader: SoraReader): Action"));
        assert!(kotlin_item.contains("fun decode(reader: SoraReader): Item"));
        assert!(kotlin_runtime.contains("class SoraBundle"));
        assert!(kotlin_config.contains("data class SoraConfig"));
        assert!(kotlin_config.contains("val item: Map<Int, Item>"));
        assert!(kotlin_config.contains("private val itemByName: Map<String, Item>"));
        assert!(kotlin_config.contains("private val itemByItemType: Map<ItemType, List<Item>>"));
        assert!(kotlin_config.contains("fun getItem(key: Int): Item? = item[key]"));
        assert!(
            kotlin_config.contains("fun getItemByName(name: String): Item? = itemByName[name]")
        );
        assert!(kotlin_config.contains(
            "fun findItemByItemType(itemType: ItemType): List<Item> = itemByItemType[itemType].orEmpty()"
        ));
        assert!(kotlin_config.contains("fun itemValues(): Collection<Item> = item.values"));
        assert!(kotlin_config.contains("fun fromBytes(bytes: ByteArray): SoraConfig"));

        let _ = std::fs::remove_dir_all(base);
    }

    #[test]
    fn kotlin_files_are_written_under_package_path() {
        let mut ir = example_ir();
        ir.package = "com.sora.game_config".to_owned();
        let base = temp_dir();
        let kotlin_out = base.join("kotlin");

        KotlinCodeGenerator.generate(&ir, &kotlin_out).unwrap();

        let item =
            std::fs::read_to_string(kotlin_out.join("com/sora/game_config/Item.kt")).unwrap();
        assert!(item.contains("package com.sora.game_config"));
        assert!(!kotlin_out.join("Item.kt").exists());

        let _ = std::fs::remove_dir_all(base);
    }

    #[test]
    fn kotlin_package_path_rejects_invalid_segments() {
        let mut ir = example_ir();
        ir.package = "com.sora-game".to_owned();
        let base = temp_dir();

        let error = KotlinCodeGenerator
            .generate(&ir, &base.join("kotlin"))
            .unwrap_err();

        assert!(
            error
                .to_string()
                .contains("dot-separated identifier segments")
        );

        let _ = std::fs::remove_dir_all(base);
    }

    #[test]
    fn rust_config_api_respects_table_modes() {
        let ir = table_mode_ir();
        let base = temp_dir();
        let rust_out = base.join("rust");

        RustCodeGenerator.generate(&ir, &rust_out).unwrap();

        let rust_mod = std::fs::read_to_string(rust_out.join("mod.rs")).unwrap();
        assert!(rust_mod.contains("pub struct ItemTable"));
        assert!(rust_mod.contains("rows: SoraMap<i32, item::Item>"));
        assert!(rust_mod.contains("pub struct SettingsTable"));
        assert!(rust_mod.contains("rows: settings::Settings"));
        assert!(rust_mod.contains("pub fn item(&self) -> &ItemTable"));
        assert!(rust_mod.contains("pub fn settings(&self) -> &SettingsTable"));
        assert!(!rust_mod.contains("pub fn settings_row"));
        assert!(rust_mod.contains("decode_singleton_table"));

        let _ = std::fs::remove_dir_all(base);
    }

    #[test]
    fn rust_config_api_can_use_fx_hash_map() {
        let mut ir = example_ir();
        ir.codegen.rust.map_type = sora_ir::model::RustMapTypeIr::FxHashMap;
        let base = temp_dir();
        let rust_out = base.join("rust");

        RustCodeGenerator.generate(&ir, &rust_out).unwrap();

        let rust_mod = std::fs::read_to_string(rust_out.join("mod.rs")).unwrap();
        assert!(rust_mod.contains("pub type SoraMap<K, V> = rustc_hash::FxHashMap<K, V>;"));
        assert!(rust_mod.contains("pub struct ItemTable"));
        assert!(rust_mod.contains("rows: SoraMap<i32, item::Item>"));
        assert!(rust_mod.contains("tables: SoraMap<&'static str, Box<dyn SoraTable>>"));

        let _ = std::fs::remove_dir_all(base);
    }

    #[test]
    fn csharp_supports_export_runtime_formats() {
        for (runtime_format, parse_function) in [
            (RuntimeFormatIr::Json, "ParseJson"),
            (RuntimeFormatIr::Cbor, "ParseCbor"),
            (RuntimeFormatIr::SoraProtobuf, "ParseProtobuf"),
        ] {
            let mut ir = example_ir();
            ir.codegen.csharp.runtime_format = runtime_format;
            let base = temp_dir();
            let csharp_out = base.join("csharp");

            CSharpCodeGenerator.generate(&ir, &csharp_out).unwrap();

            let runtime = std::fs::read_to_string(csharp_out.join("Runtime.cs")).unwrap();
            let config = std::fs::read_to_string(csharp_out.join("SoraConfig.cs")).unwrap();
            let item = std::fs::read_to_string(csharp_out.join("Item.cs")).unwrap();
            let action = std::fs::read_to_string(csharp_out.join("Action.cs")).unwrap();

            assert!(runtime.contains("internal sealed class SoraValueBundle"));
            assert!(runtime.contains(parse_function));
            assert!(config.contains(&format!("SoraValueBundle.{parse_function}(bytes)")));
            assert!(item.contains("internal static Item Decode(SoraValue value)"));
            assert!(item.contains("obj.Get(\"id\").AsInt32()"));
            assert!(item.contains("ItemTypeCodec.Decode(obj.Get(\"item_type\"))"));
            assert!(action.contains("internal static Action Decode(SoraValue value)"));

            let _ = std::fs::remove_dir_all(base);
        }
    }

    #[test]
    fn kotlin_supports_export_runtime_formats() {
        for (runtime_format, parse_function) in [
            (RuntimeFormatIr::Json, "parseJson"),
            (RuntimeFormatIr::Cbor, "parseCbor"),
            (RuntimeFormatIr::SoraProtobuf, "parseProtobuf"),
        ] {
            let mut ir = example_ir();
            ir.codegen.kotlin.runtime_format = runtime_format;
            let base = temp_dir();
            let kotlin_out = base.join("kotlin");

            KotlinCodeGenerator.generate(&ir, &kotlin_out).unwrap();

            let package_out = kotlin_out.join("game_config");
            let runtime = std::fs::read_to_string(package_out.join("Runtime.kt")).unwrap();
            let config = std::fs::read_to_string(package_out.join("SoraConfig.kt")).unwrap();
            let item = std::fs::read_to_string(package_out.join("Item.kt")).unwrap();
            let action = std::fs::read_to_string(package_out.join("Action.kt")).unwrap();

            assert!(runtime.contains("class SoraValueBundle"));
            assert!(runtime.contains(parse_function));
            assert!(config.contains(&format!("SoraValueBundle.{parse_function}(bytes)")));
            assert!(item.contains("fun decode(value: SoraValue): Item"));
            assert!(item.contains("obj.get(\"id\").asInt()"));
            assert!(item.contains("ItemType.decode(obj.get(\"item_type\"))"));
            assert!(action.contains("fun decode(value: SoraValue): Action"));

            let _ = std::fs::remove_dir_all(base);
        }
    }

    #[test]
    fn go_supports_export_runtime_formats() {
        for (runtime_format, parse_function) in [
            (RuntimeFormatIr::Json, "ParseJsonBundle"),
            (RuntimeFormatIr::Cbor, "ParseCborBundle"),
            (RuntimeFormatIr::SoraProtobuf, "ParseProtobufBundle"),
        ] {
            let mut ir = example_ir();
            ir.codegen.go.runtime_format = runtime_format;
            let base = temp_dir();
            let go_out = base.join("go");

            GoCodeGenerator.generate(&ir, &go_out).unwrap();

            let runtime = std::fs::read_to_string(go_out.join("runtime.go")).unwrap();
            let config = std::fs::read_to_string(go_out.join("config.go")).unwrap();
            let item = std::fs::read_to_string(go_out.join("item.go")).unwrap();
            let action = std::fs::read_to_string(go_out.join("action.go")).unwrap();

            assert!(runtime.contains("type SoraValueBundle struct"));
            assert!(runtime.contains(parse_function));
            assert!(config.contains(&format!("{parse_function}(bytes)")));
            assert!(item.contains("func decodeItemValue(input SoraValue) (Item, error)"));
            assert!(item.contains("obj.Get(\"id\").AsInt32()"));
            assert!(item.contains("decodeItemTypeValue(obj.Get(\"item_type\"))"));
            assert!(action.contains("func decodeActionValue(input SoraValue) (Action, error)"));

            let _ = std::fs::remove_dir_all(base);
        }
    }

    #[test]
    fn java_supports_export_runtime_formats() {
        for (runtime_format, parse_function) in [
            (RuntimeFormatIr::Json, "parseJson"),
            (RuntimeFormatIr::Cbor, "parseCbor"),
            (RuntimeFormatIr::SoraProtobuf, "parseProtobuf"),
        ] {
            let mut ir = example_ir();
            ir.codegen.java.runtime_format = runtime_format;
            let base = temp_dir();
            let java_out = base.join("java");

            JavaCodeGenerator.generate(&ir, &java_out).unwrap();

            let package_out = java_out.join("game_config");
            let runtime = std::fs::read_to_string(package_out.join("Runtime.java")).unwrap();
            let config = std::fs::read_to_string(package_out.join("SoraConfig.java")).unwrap();
            let item = std::fs::read_to_string(package_out.join("Item.java")).unwrap();
            let action = std::fs::read_to_string(package_out.join("Action.java")).unwrap();

            assert!(runtime.contains("final class SoraValueBundle"));
            assert!(runtime.contains(parse_function));
            assert!(config.contains(&format!("SoraValueBundle.{parse_function}(bytes)")));
            assert!(item.contains("static Item decode(SoraValue value)"));
            assert!(item.contains("obj.get(\"id\").asInt()"));
            assert!(item.contains("ItemType.decode(obj.get(\"item_type\"))"));
            assert!(action.contains("static Action decode(SoraValue value)"));

            let _ = std::fs::remove_dir_all(base);
        }
    }

    #[test]
    fn generates_csharp_java_and_go_files() {
        let mut ir = example_ir();
        ir.package = "com.sora.game_config".to_owned();
        let base = temp_dir();
        let csharp_out = base.join("csharp");
        let java_out = base.join("java");
        let go_out = base.join("go");

        CSharpCodeGenerator.generate(&ir, &csharp_out).unwrap();
        JavaCodeGenerator.generate(&ir, &java_out).unwrap();
        GoCodeGenerator.generate(&ir, &go_out).unwrap();

        let csharp_item = std::fs::read_to_string(csharp_out.join("Item.cs")).unwrap();
        let csharp_config = std::fs::read_to_string(csharp_out.join("SoraConfig.cs")).unwrap();
        let java_item =
            std::fs::read_to_string(java_out.join("com/sora/game_config/Item.java")).unwrap();
        let java_config =
            std::fs::read_to_string(java_out.join("com/sora/game_config/SoraConfig.java")).unwrap();
        let go_item = std::fs::read_to_string(go_out.join("item.go")).unwrap();
        let go_config = std::fs::read_to_string(go_out.join("config.go")).unwrap();

        assert!(
            csharp_item.starts_with("// This file was generated by Sora. Do not edit manually.")
        );
        assert!(csharp_item.contains("namespace com.sora.game_config;"));
        assert!(csharp_item.contains("public sealed record Item"));
        assert!(csharp_item.contains("// Item id"));
        assert!(csharp_config.contains("public sealed class SoraConfig"));
        assert!(csharp_config.contains("Dictionary<int, Item>"));
        assert!(csharp_config.contains("private readonly Dictionary<string, Item> byName"));
        assert!(
            csharp_config.contains("private readonly Dictionary<ItemType, List<Item>> byItemType")
        );
        assert!(csharp_config.contains("public Item? GetByName(string name)"));
        assert!(
            csharp_config.contains("public IReadOnlyList<Item> FindByItemType(ItemType itemType)")
        );
        assert!(java_item.starts_with("// This file was generated by Sora. Do not edit manually."));
        assert!(java_item.contains("package com.sora.game_config;"));
        assert!(java_item.contains("public final class Item"));
        assert!(java_item.contains("/** Item id */"));
        assert!(java_config.contains("public final class SoraConfig"));
        assert!(java_config.contains("java.util.Map<Integer, Item>"));
        assert!(java_config.contains("private final Map<String, Item> byName"));
        assert!(java_config.contains("private final Map<ItemType, List<Item>> byItemType"));
        assert!(java_config.contains("public Item getByName(String name)"));
        assert!(java_config.contains("public List<Item> findByItemType(ItemType itemType)"));
        assert!(go_item.starts_with("// This file was generated by Sora. Do not edit manually."));
        assert!(go_item.contains("package game_config"));
        assert!(go_item.contains("type Item struct"));
        assert!(go_item.contains("// Id: Item id"));
        assert!(go_config.contains("type SoraConfig struct"));
        assert!(go_config.contains("map[int32]Item"));
        assert!(go_config.contains("byName map[string]Item"));
        assert!(go_config.contains("byItemType map[ItemType][]Item"));
        assert!(go_config.contains("func (table *ItemTable) GetByName(name string) (Item, bool)"));
        assert!(
            go_config.contains("func (table *ItemTable) FindByItemType(itemType ItemType) []Item")
        );

        let _ = std::fs::remove_dir_all(base);
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
name = "name"
type = "string"
required = true
comment = "Item name"

[[tables.fields]]
name = "item_type"
type = "enum<ItemType>"
required = true
comment = "Item type"

[[tables.fields]]
name = "action"
type = "union<Action>"
required = true
comment = "Action"

[[tables.indexes]]
name = "by_name"
fields = ["name"]
unique = true

[[tables.indexes]]
name = "by_item_type"
fields = ["item_type"]
"#,
        )
        .unwrap();

        normalize_schema(schema).unwrap()
    }

    fn table_mode_ir() -> ConfigIr {
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

[[tables]]
name = "Settings"
mode = "singleton"

[[tables.fields]]
name = "version"
type = "string"
required = true
"#,
        )
        .unwrap();

        normalize_schema(schema).unwrap()
    }

    fn temp_dir() -> PathBuf {
        let unique = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("sora-codegen-test-{unique}"))
    }
}
