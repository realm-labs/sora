use std::path::Path;

use heck::ToSnakeCase;
use minijinja::context;
use serde::Serialize;
use sora_diagnostics::Result;
use sora_ir::model::{ConfigIr, TypeIr};

use crate::{
    generator::{CodeGenerator, CodegenContext, runtime_format_name},
    model::{
        BaseField, BaseImport, BaseIndex, BaseModel, BaseRecord, BaseTable, BaseUnion,
        BaseUnionVariant, build_base_model,
    },
    options::{LuaCodegenOptions, LuaEnumRepr, LuaVersion},
    render::{ensure_dir, render_template, write_file},
};

pub struct LuaCodeGenerator;
crate::impl_test_codegen_generate!(LuaCodeGenerator, "lua");

impl CodeGenerator for LuaCodeGenerator {
    fn generate(&self, context: CodegenContext<'_>, out_dir: &Path) -> Result<()> {
        let ir = context.ir;
        let codegen_options = context.options::<LuaCodegenOptions>()?;
        ensure_supported_lua_version(codegen_options.lua_version)?;
        ensure_dir(out_dir)?;
        let runtime_format = runtime_format_name(codegen_options.runtime_format);

        let options = LuaOptionsView::new(
            codegen_options.module.as_deref(),
            codegen_options.lua_version,
            codegen_options.enum_repr,
        );
        let model = LuaModel::from_base_model(ir, build_base_model(ir)?, &options);

        for item in &model.enums {
            let rendered = render_template(
                "lua",
                "enum.lua.j2",
                context! { enum => item, options => &options },
            )?;
            write_file(
                &out_dir.join(format!("{}.lua", item.name.to_snake_case())),
                rendered,
            )?;
        }

        for record in &model.records {
            let rendered = render_template(
                "lua",
                "record.lua.j2",
                context! { options => &options, record => record, runtime_format => runtime_format },
            )?;
            write_file(
                &out_dir.join(format!("{}.lua", record.snake_name)),
                rendered,
            )?;
        }

        for union in &model.unions {
            let rendered = render_template(
                "lua",
                "union.lua.j2",
                context! { options => &options, union => union, runtime_format => runtime_format },
            )?;
            write_file(&out_dir.join(format!("{}.lua", union.snake_name)), rendered)?;
        }

        let rendered = render_template(
            "lua",
            "runtime.lua.j2",
            context! { options => &options, runtime_format => runtime_format },
        )?;
        write_file(&out_dir.join("sora_runtime.lua"), rendered)?;

        let rendered = render_template(
            "lua",
            "config.lua.j2",
            context! { model => &model, options => &options, runtime_format => runtime_format },
        )?;
        write_file(&out_dir.join("sora_config.lua"), rendered)
    }
}

#[derive(Debug, Clone, Serialize)]
struct LuaOptionsView {
    require_prefix: String,
    uses_string_unpack: bool,
    i64_type_name: &'static str,
    enum_is_integer: bool,
}

impl LuaOptionsView {
    fn new(module: Option<&str>, lua_version: LuaVersion, enum_repr: LuaEnumRepr) -> Self {
        let require_prefix = module
            .filter(|module| !module.trim().is_empty())
            .map(|module| format!("{module}."))
            .unwrap_or_default();
        Self {
            require_prefix,
            uses_string_unpack: uses_string_unpack(lua_version),
            i64_type_name: lua_i64_type_name(lua_version),
            enum_is_integer: enum_repr == LuaEnumRepr::Integer,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
struct LuaModel {
    package: String,
    schema_fingerprint: String,
    enums: Vec<LuaEnum>,
    unions: Vec<LuaUnion>,
    records: Vec<LuaRecord>,
    tables: Vec<LuaTable>,
    has_localization: bool,
    locales: Vec<String>,
    default_locale: String,
}

#[derive(Debug, Clone, Serialize)]
struct LuaEnum {
    name: String,
    values: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct LuaUnion {
    pascal_name: String,
    snake_name: String,
    tag: String,
    variants: Vec<LuaUnionVariant>,
    imports: Vec<LuaImport>,
}

#[derive(Debug, Clone, Serialize)]
struct LuaUnionVariant {
    name: String,
    fields: Vec<LuaField>,
    has_text_keys: bool,
}

#[derive(Debug, Clone, Serialize)]
struct LuaRecord {
    pascal_name: String,
    snake_name: String,
    imports: Vec<LuaImport>,
    fields: Vec<LuaField>,
    table: Option<LuaTable>,
}

#[derive(Debug, Clone, Serialize)]
struct LuaImport {
    module: String,
    name: String,
}

#[derive(Debug, Clone, Serialize)]
struct LuaTable {
    name: String,
    pascal_name: String,
    snake_name: String,
    mode: String,
    row_type: String,
    key_name: Option<String>,
    key_field_name: Option<String>,
    key_type: Option<String>,
    unique_indexes: Vec<LuaIndex>,
    non_unique_indexes: Vec<LuaIndex>,
}

#[derive(Debug, Clone, Serialize)]
struct LuaIndex {
    name: String,
    field_name: String,
    param_camel_name: String,
    key_type: String,
}

#[derive(Debug, Clone, Serialize)]
struct LuaField {
    raw_name: String,
    name: String,
    type_name: String,
    decode: String,
    value_decode: String,
    collect_text_keys: String,
    comment: Option<String>,
}

impl LuaModel {
    fn from_base_model(ir: &ConfigIr, model: BaseModel, options: &LuaOptionsView) -> Self {
        let enums = model
            .enums
            .into_iter()
            .map(|item| LuaEnum {
                name: item.pascal_name,
                values: item.values,
            })
            .collect();
        let tables = model
            .tables
            .into_iter()
            .map(|item| lua_table(ir, item, options))
            .collect::<Vec<_>>();
        let records = model
            .records
            .into_iter()
            .map(|item| {
                let table = tables
                    .iter()
                    .find(|table| table.row_type == item.pascal_name)
                    .cloned();
                lua_record(ir, item, options, table)
            })
            .collect();
        let unions = model
            .unions
            .into_iter()
            .map(|item| lua_union(ir, item, options))
            .collect();

        Self {
            package: model.package,
            schema_fingerprint: model.schema_fingerprint,
            enums,
            unions,
            records,
            tables,
            has_localization: ir.localization.is_some(),
            locales: ir
                .localization
                .as_ref()
                .map(|item| item.locales.clone())
                .unwrap_or_default(),
            default_locale: ir
                .localization
                .as_ref()
                .map(|item| item.default_locale.clone())
                .unwrap_or_default(),
        }
    }
}

fn lua_union(ir: &ConfigIr, union: BaseUnion, options: &LuaOptionsView) -> LuaUnion {
    LuaUnion {
        pascal_name: union.pascal_name,
        snake_name: union.snake_name,
        tag: union.tag,
        variants: union
            .variants
            .into_iter()
            .map(|variant| lua_variant(ir, variant, options))
            .collect(),
        imports: union.imports.into_iter().map(lua_import).collect(),
    }
}

fn lua_variant(
    ir: &ConfigIr,
    variant: BaseUnionVariant,
    options: &LuaOptionsView,
) -> LuaUnionVariant {
    let fields = variant
        .fields
        .into_iter()
        .map(|field| lua_field(ir, field, options))
        .collect::<Vec<_>>();
    let has_text_keys = fields
        .iter()
        .any(|field| !field.collect_text_keys.is_empty());
    LuaUnionVariant {
        name: variant.pascal_name,
        fields,
        has_text_keys,
    }
}

fn lua_record(
    ir: &ConfigIr,
    record: BaseRecord,
    options: &LuaOptionsView,
    table: Option<LuaTable>,
) -> LuaRecord {
    LuaRecord {
        pascal_name: record.pascal_name,
        snake_name: record.snake_name,
        imports: record.imports.into_iter().map(lua_import).collect(),
        fields: record
            .fields
            .into_iter()
            .map(|field| lua_field(ir, field, options))
            .collect(),
        table,
    }
}

fn lua_table(ir: &ConfigIr, table: BaseTable, options: &LuaOptionsView) -> LuaTable {
    let row_type = table.pascal_name.clone();
    let key_type = table
        .key_field
        .as_ref()
        .map(|field| lua_type_name(ir, &field.ty, options));
    let key_field_name = table
        .key_field
        .as_ref()
        .map(|field| field.camel_name.clone());

    LuaTable {
        name: table.name,
        pascal_name: table.pascal_name,
        snake_name: table.snake_name,
        mode: table.mode_name,
        row_type,
        key_name: table.key_name,
        key_field_name,
        key_type,
        unique_indexes: table
            .unique_indexes
            .into_iter()
            .map(|index| lua_index(ir, index, options))
            .collect(),
        non_unique_indexes: table
            .non_unique_indexes
            .into_iter()
            .map(|index| lua_index(ir, index, options))
            .collect(),
    }
}

fn lua_index(ir: &ConfigIr, index: BaseIndex, options: &LuaOptionsView) -> LuaIndex {
    LuaIndex {
        name: index.snake_name,
        field_name: index.field.camel_name.clone(),
        param_camel_name: index.field.camel_name,
        key_type: lua_type_name(ir, &index.field.ty, options),
    }
}

fn lua_field(ir: &ConfigIr, field: BaseField, options: &LuaOptionsView) -> LuaField {
    let collect_text_keys =
        lua_collect_text_keys(ir, &field.ty, &format!("value.{}", field.camel_name));
    LuaField {
        raw_name: field.raw_name,
        name: field.camel_name,
        type_name: lua_type_name(ir, &field.ty, options),
        decode: lua_decode_expr(ir, &field.ty, options),
        value_decode: lua_value_decode_expr(ir, &field.ty, "__VALUE__"),
        collect_text_keys,
        comment: field.comment,
    }
}

fn lua_import(import: BaseImport) -> LuaImport {
    LuaImport {
        module: import.module,
        name: import.name,
    }
}

fn lua_type_name(ir: &ConfigIr, ty: &TypeIr, options: &LuaOptionsView) -> String {
    match ty {
        TypeIr::Bool => "boolean".to_owned(),
        TypeIr::I8 | TypeIr::U8 | TypeIr::I16 | TypeIr::U16 | TypeIr::I32 | TypeIr::U32 => {
            "integer".to_owned()
        }
        TypeIr::I64 => options.i64_type_name.to_owned(),
        TypeIr::F32 | TypeIr::F64 => "number".to_owned(),
        TypeIr::String => "string".to_owned(),
        TypeIr::Text => "TextKey".to_owned(),
        TypeIr::Enum(name) | TypeIr::Struct(name) | TypeIr::Union(name) => name.clone(),
        TypeIr::List(element) | TypeIr::Set(element) | TypeIr::Array { element, .. } => {
            format!("{}[]", lua_type_name(ir, element, options))
        }
        TypeIr::Map { key, value } => format!(
            "{{[{}]: {}}}",
            lua_type_name(ir, key, options),
            lua_type_name(ir, value, options)
        ),
        TypeIr::Ref { table, field } => ref_target_type(ir, table, field)
            .map(|ty| lua_type_name(ir, ty, options))
            .unwrap_or_else(|| "integer".to_owned()),
        TypeIr::Optional(element) => format!("{}?", lua_type_name(ir, element, options)),
    }
}

fn lua_decode_expr(ir: &ConfigIr, ty: &TypeIr, _options: &LuaOptionsView) -> String {
    match ty {
        TypeIr::Bool => "reader:read_bool()".to_owned(),
        TypeIr::I8 | TypeIr::I16 | TypeIr::I32 => "reader:read_i32()".to_owned(),
        TypeIr::U8 | TypeIr::U16 | TypeIr::U32 => "reader:read_u32()".to_owned(),
        TypeIr::I64 => "reader:read_i64()".to_owned(),
        TypeIr::F32 => "reader:read_f32()".to_owned(),
        TypeIr::F64 => "reader:read_f64()".to_owned(),
        TypeIr::String => "reader:read_string()".to_owned(),
        TypeIr::Text => "Runtime.new_text_key(reader:read_string())".to_owned(),
        TypeIr::Enum(name) | TypeIr::Struct(name) | TypeIr::Union(name) => {
            format!("{name}.decode(reader)")
        }
        TypeIr::List(element) | TypeIr::Set(element) | TypeIr::Array { element, .. } => {
            format!(
                "reader:read_list(function() return {} end)",
                lua_decode_expr(ir, element, _options)
            )
        }
        TypeIr::Map { key, value } => format!(
            "reader:read_map(function() return {} end, function() return {} end)",
            lua_decode_expr(ir, key, _options),
            lua_decode_expr(ir, value, _options)
        ),
        TypeIr::Ref { table, field } => ref_target_type(ir, table, field)
            .map(|ty| lua_decode_expr(ir, ty, _options))
            .unwrap_or_else(|| "reader:read_i32()".to_owned()),
        TypeIr::Optional(element) => {
            format!(
                "reader:read_optional(function() return {} end)",
                lua_decode_expr(ir, element, _options)
            )
        }
    }
}

fn lua_value_decode_expr(ir: &ConfigIr, ty: &TypeIr, value: &str) -> String {
    match ty {
        TypeIr::Bool => format!("Runtime.expect_boolean({value})"),
        TypeIr::I8
        | TypeIr::U8
        | TypeIr::I16
        | TypeIr::U16
        | TypeIr::I32
        | TypeIr::U32
        | TypeIr::I64 => format!("Runtime.expect_integer({value})"),
        TypeIr::F32 | TypeIr::F64 => format!("Runtime.expect_number({value})"),
        TypeIr::String => format!("Runtime.expect_string({value})"),
        TypeIr::Text => format!("Runtime.new_text_key(Runtime.expect_string({value}))"),
        TypeIr::Enum(name) | TypeIr::Struct(name) | TypeIr::Union(name) => {
            format!("{name}.decode_value({value})")
        }
        TypeIr::List(element) | TypeIr::Set(element) | TypeIr::Array { element, .. } => {
            format!(
                "Runtime.decode_value_list({value}, function(item) return {} end)",
                lua_value_decode_expr(ir, element, "item")
            )
        }
        TypeIr::Map {
            key,
            value: element,
        } => format!(
            "Runtime.decode_value_map({value}, function(item) return {} end, function(item) return {} end)",
            lua_value_decode_expr(ir, key, "item"),
            lua_value_decode_expr(ir, element, "item")
        ),
        TypeIr::Ref { table, field } => ref_target_type(ir, table, field)
            .map(|ty| lua_value_decode_expr(ir, ty, value))
            .unwrap_or_else(|| format!("Runtime.expect_integer({value})")),
        TypeIr::Optional(element) => {
            format!(
                "{value} == nil and nil or {}",
                lua_value_decode_expr(ir, element, value)
            )
        }
    }
}

fn lua_collect_text_keys(ir: &ConfigIr, ty: &TypeIr, value: &str) -> String {
    match ty {
        TypeIr::Text => format!("out[#out + 1] = {value}"),
        TypeIr::Optional(element) => {
            let inner = lua_collect_text_keys(ir, element, "__sora_value");
            if inner.is_empty() {
                String::new()
            } else {
                format!("if {value} ~= nil then local __sora_value = {value}; {inner} end")
            }
        }
        TypeIr::List(element) | TypeIr::Set(element) | TypeIr::Array { element, .. } => {
            let inner = lua_collect_text_keys(ir, element, "__sora_value");
            if inner.is_empty() {
                String::new()
            } else {
                format!("for _, __sora_value in ipairs({value}) do {inner} end")
            }
        }
        TypeIr::Map {
            key,
            value: element,
        } => {
            let key_inner = lua_collect_text_keys(ir, key, "__sora_key");
            let value_inner = lua_collect_text_keys(ir, element, "__sora_value");
            if key_inner.is_empty() && value_inner.is_empty() {
                String::new()
            } else {
                format!(
                    "for __sora_key, __sora_value in pairs({value}) do {key_inner} {value_inner} end"
                )
            }
        }
        TypeIr::Struct(name) | TypeIr::Union(name) => {
            format!("{name}.collect_text_keys({value}, out)")
        }
        TypeIr::Ref { table, field } => ref_target_type(ir, table, field)
            .map(|ty| lua_collect_text_keys(ir, ty, value))
            .unwrap_or_default(),
        TypeIr::Bool
        | TypeIr::I8
        | TypeIr::U8
        | TypeIr::I16
        | TypeIr::U16
        | TypeIr::I32
        | TypeIr::U32
        | TypeIr::I64
        | TypeIr::F32
        | TypeIr::F64
        | TypeIr::String
        | TypeIr::Enum(_) => String::new(),
    }
}

fn ref_target_type<'a>(ir: &'a ConfigIr, table: &str, field: &str) -> Option<&'a TypeIr> {
    ir.tables
        .iter()
        .find(|candidate| candidate.name == table)
        .and_then(|table| {
            table
                .fields
                .iter()
                .find(|candidate| candidate.name == *field)
        })
        .map(|field| &field.ty)
}

fn ensure_supported_lua_version(version: LuaVersion) -> Result<()> {
    match version {
        LuaVersion::Lua51
        | LuaVersion::Lua52
        | LuaVersion::Lua53
        | LuaVersion::Lua54
        | LuaVersion::LuaJit => Ok(()),
    }
}

fn uses_string_unpack(version: LuaVersion) -> bool {
    matches!(version, LuaVersion::Lua53 | LuaVersion::Lua54)
}

fn lua_i64_type_name(version: LuaVersion) -> &'static str {
    match version {
        LuaVersion::Lua53 | LuaVersion::Lua54 => "integer",
        LuaVersion::Lua51 | LuaVersion::Lua52 | LuaVersion::LuaJit => "number",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::options::{LuaCodegenOptions, LuaEnumRepr, LuaVersion, RuntimeFormat};
    use sora_ir::normalize::normalize_schema;
    use sora_schema::model::SchemaFile;
    use std::{
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };

    static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn generates_lua_files_with_emmylua_annotations() {
        let ir = example_ir();
        let base = temp_dir();

        LuaCodeGenerator
            .generate_with_options(
                &ir,
                LuaCodegenOptions {
                    module: Some("generated.lua".to_owned()),
                    ..Default::default()
                },
                &base,
            )
            .unwrap();

        let item = std::fs::read_to_string(base.join("item.lua")).unwrap();
        let item_type = std::fs::read_to_string(base.join("item_type.lua")).unwrap();
        let action = std::fs::read_to_string(base.join("action.lua")).unwrap();
        let runtime = std::fs::read_to_string(base.join("sora_runtime.lua")).unwrap();
        let config = std::fs::read_to_string(base.join("sora_config.lua")).unwrap();

        assert!(item.contains("---@class Item"));
        assert!(item.ends_with('\n'));
        assert!(item.contains("---@field itemType ItemType"));
        assert!(item.contains("---@field largeId integer"));
        assert!(item.contains("local ItemType = require(\"generated.lua.item_type\")"));
        assert!(item.contains("function Item.decode(reader)"));
        assert!(item.contains("itemType = ItemType.decode(reader)"));
        assert!(item.contains("largeId = reader:read_i64()"));
        assert!(item_type.contains("---@alias ItemType"));
        assert!(item_type.contains("---| '\"Weapon\"'"));
        assert!(action.contains("---@alias Action"));
        assert!(action.contains("---@class ActionAddItem"));
        assert!(action.contains("type = \"AddItem\""));
        assert!(runtime.contains("function Runtime.parse_bundle(bytes, options)"));
        assert!(runtime.contains("string.unpack(\"<I4\""));
        assert!(config.contains("local Runtime = require(\"generated.lua.sora_runtime\")"));
        assert!(item.contains("---@class ItemTable"));
        assert!(item.contains("function ItemTable:get(key)"));
        assert!(item.contains("function ItemTable:get_by_name(name)"));
        assert!(item.contains("function ItemTable:find_by_item_type(itemType)"));
        assert!(!config.contains("---@class ItemTable"));
        assert!(config.contains("function SoraConfig.from_bytes(bytes, options)"));
        assert!(config.contains("function SoraConfig:item()"));
        assert!(config.ends_with('\n'));

        let _ = std::fs::remove_dir_all(base);
    }

    #[test]
    fn lua_version_changes_i64_api() {
        let ir = example_ir();
        let base = temp_dir();

        LuaCodeGenerator
            .generate_with_options(
                &ir,
                LuaCodegenOptions {
                    lua_version: LuaVersion::LuaJit,
                    ..Default::default()
                },
                &base,
            )
            .unwrap();

        let item = std::fs::read_to_string(base.join("item.lua")).unwrap();

        assert!(item.contains("---@field largeId number"));
        assert!(item.contains("largeId = reader:read_i64()"));

        let _ = std::fs::remove_dir_all(base);
    }

    #[test]
    fn lua_enum_options_change_generated_api() {
        let ir = example_ir();
        let base = temp_dir();

        LuaCodeGenerator
            .generate_with_options(
                &ir,
                LuaCodegenOptions {
                    enum_repr: LuaEnumRepr::Integer,
                    ..Default::default()
                },
                &base,
            )
            .unwrap();

        let item_type = std::fs::read_to_string(base.join("item_type.lua")).unwrap();

        assert!(item_type.contains("---| integer"));
        assert!(item_type.contains("Weapon = 0"));
        assert!(item_type.contains("return ordinal"));

        let _ = std::fs::remove_dir_all(base);
    }

    #[test]
    fn lua_compat_runtime_avoids_string_unpack() {
        let ir = example_ir();
        let base = temp_dir();

        LuaCodeGenerator
            .generate_with_options(
                &ir,
                LuaCodegenOptions {
                    lua_version: LuaVersion::LuaJit,
                    ..Default::default()
                },
                &base,
            )
            .unwrap();

        let runtime = std::fs::read_to_string(base.join("sora_runtime.lua")).unwrap();
        assert!(!runtime.contains("string.unpack"));
        assert!(runtime.contains("function read_f32_at(bytes, offset)"));
        assert!(runtime.contains("function read_i64_at(bytes, offset)"));
        assert!(runtime.contains("safe integer range"));

        let _ = std::fs::remove_dir_all(base);
    }

    #[test]
    fn lua_supports_adapter_export_runtime_formats() {
        for (runtime_format, parse_function, adapter) in [
            (RuntimeFormat::Json, "parse_json_bundle", "decode_json"),
            (RuntimeFormat::Cbor, "parse_cbor_bundle", "decode_cbor"),
            (
                RuntimeFormat::SoraProtobuf,
                "parse_protobuf_bundle",
                "decode_protobuf",
            ),
        ] {
            let ir = example_ir();
            let base = temp_dir();

            LuaCodeGenerator
                .generate_with_options(
                    &ir,
                    LuaCodegenOptions {
                        runtime_format,
                        ..Default::default()
                    },
                    &base,
                )
                .unwrap();

            let item = std::fs::read_to_string(base.join("item.lua")).unwrap();
            let item_type = std::fs::read_to_string(base.join("item_type.lua")).unwrap();
            let action = std::fs::read_to_string(base.join("action.lua")).unwrap();
            let runtime = std::fs::read_to_string(base.join("sora_runtime.lua")).unwrap();
            let config = std::fs::read_to_string(base.join("sora_config.lua")).unwrap();

            assert!(runtime.contains(&format!(
                "function Runtime.{parse_function}(bytes, options)"
            )));
            assert!(runtime.contains(adapter));
            assert!(runtime.contains("local SoraValueBundle = {}"));
            assert!(config.contains("function SoraConfig.from_bundle(bundle)"));
            assert!(config.contains(parse_function));
            assert!(config.contains("Item.decode_value"));
            assert!(item.contains("function Item.decode_value(value)"));
            assert!(item.contains("Runtime.expect_integer(obj[\"id\"])"));
            assert!(item_type.contains("function ItemType.decode_value(value)"));
            assert!(action.contains("function Action.decode_value(value)"));

            let _ = std::fs::remove_dir_all(base);
        }
    }

    fn example_ir() -> ConfigIr {
        let schema: SchemaFile = toml::from_str(
            r#"
package = "game_config"

[[enums]]
name = "ItemType"
values = ["Weapon", "Armor"]

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

[[tables.fields]]
name = "name"
type = "string"

[[tables.fields]]
name = "item_type"
type = "enum<ItemType>"

[[tables.fields]]
name = "large_id"
type = "i64"

[[tables.fields]]
name = "action"
type = "union<Action>"

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

    fn temp_dir() -> PathBuf {
        let unique = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("sora-lua-codegen-test-{unique}"))
    }
}
