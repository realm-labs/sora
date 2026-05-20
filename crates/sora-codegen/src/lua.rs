use std::path::Path;

use heck::{ToLowerCamelCase, ToSnakeCase};
use minijinja::context;
use serde::Serialize;
use sora_diagnostics::Result;
use sora_ir::model::{ConfigIr, LuaEnumReprIr, LuaVersionIr, TableModeIr, TypeIr};

use crate::{
    generator::{CodeGenerator, ensure_sora_runtime_format},
    model::{LanguageBackend, TableNameParts, build_model},
    render::{ensure_dir, render_template, write_file},
};

pub struct LuaCodeGenerator;

impl CodeGenerator for LuaCodeGenerator {
    fn generate(&self, ir: &ConfigIr, out_dir: &Path) -> Result<()> {
        ensure_sora_runtime_format("lua", ir.codegen.lua.runtime_format)?;
        ensure_supported_lua_version(ir.codegen.lua.lua_version)?;
        ensure_dir(out_dir)?;

        let options = LuaOptionsView::new(
            ir.codegen.lua.module.as_deref(),
            ir.codegen.lua.lua_version,
            ir.codegen.lua.enum_repr,
        );
        let backend = LuaBackend {
            options: options.clone(),
        };
        let model = build_model(ir, &backend)?;

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
                context! { options => &options, record => record },
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
                context! { options => &options, union => union },
            )?;
            write_file(&out_dir.join(format!("{}.lua", union.snake_name)), rendered)?;
        }

        let rendered = render_template("lua", "runtime.lua.j2", context! { options => &options })?;
        write_file(&out_dir.join("sora_runtime.lua"), rendered)?;

        let rendered = render_template(
            "lua",
            "config.lua.j2",
            context! { model => &model, options => &options },
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
    fn new(module: Option<&str>, lua_version: LuaVersionIr, enum_repr: LuaEnumReprIr) -> Self {
        let require_prefix = module
            .filter(|module| !module.trim().is_empty())
            .map(|module| format!("{module}."))
            .unwrap_or_default();
        Self {
            require_prefix,
            uses_string_unpack: uses_string_unpack(lua_version),
            i64_type_name: lua_i64_type_name(lua_version),
            enum_is_integer: enum_repr == LuaEnumReprIr::Integer,
        }
    }
}

#[derive(Debug, Clone)]
struct LuaBackend {
    options: LuaOptionsView,
}

impl LanguageBackend for LuaBackend {
    fn field_name(&self, raw_name: &str) -> String {
        raw_name.to_lower_camel_case()
    }

    fn type_name(&self, ir: &ConfigIr, ty: &TypeIr) -> String {
        self.lua_type_name(ir, ty)
    }

    fn decode_expr(&self, ir: &ConfigIr, ty: &TypeIr) -> String {
        self.lua_decode_expr(ir, ty)
    }

    fn row_type(&self, table: &TableNameParts<'_>) -> String {
        table.pascal_name.to_owned()
    }

    fn container_type(
        &self,
        table: &TableNameParts<'_>,
        _mode: TableModeIr,
        _row_type: &str,
        _key_type: Option<&str>,
    ) -> String {
        format!("{}Table", table.pascal_name)
    }
}

impl LuaBackend {
    fn lua_type_name(&self, ir: &ConfigIr, ty: &TypeIr) -> String {
        match ty {
            TypeIr::Bool => "boolean".to_owned(),
            TypeIr::I32 => "integer".to_owned(),
            TypeIr::I64 => self.options.i64_type_name.to_owned(),
            TypeIr::F32 | TypeIr::F64 => "number".to_owned(),
            TypeIr::String => "string".to_owned(),
            TypeIr::Enum(name) | TypeIr::Struct(name) | TypeIr::Union(name) => name.clone(),
            TypeIr::List(element) | TypeIr::Array { element, .. } => {
                format!("{}[]", self.lua_type_name(ir, element))
            }
            TypeIr::Ref { table, field } => self
                .ref_target_type(ir, table, field)
                .map(|ty| self.lua_type_name(ir, ty))
                .unwrap_or_else(|| "integer".to_owned()),
            TypeIr::Optional(element) => format!("{}?", self.lua_type_name(ir, element)),
        }
    }

    fn lua_decode_expr(&self, ir: &ConfigIr, ty: &TypeIr) -> String {
        match ty {
            TypeIr::Bool => "reader:read_bool()".to_owned(),
            TypeIr::I32 => "reader:read_i32()".to_owned(),
            TypeIr::I64 => "reader:read_i64()".to_owned(),
            TypeIr::F32 => "reader:read_f32()".to_owned(),
            TypeIr::F64 => "reader:read_f64()".to_owned(),
            TypeIr::String => "reader:read_string()".to_owned(),
            TypeIr::Enum(name) | TypeIr::Struct(name) | TypeIr::Union(name) => {
                format!("{name}.decode(reader)")
            }
            TypeIr::List(element) | TypeIr::Array { element, .. } => {
                format!(
                    "reader:read_list(function() return {} end)",
                    self.lua_decode_expr(ir, element)
                )
            }
            TypeIr::Ref { table, field } => self
                .ref_target_type(ir, table, field)
                .map(|ty| self.lua_decode_expr(ir, ty))
                .unwrap_or_else(|| "reader:read_i32()".to_owned()),
            TypeIr::Optional(element) => {
                format!(
                    "reader:read_optional(function() return {} end)",
                    self.lua_decode_expr(ir, element)
                )
            }
        }
    }

    fn ref_target_type<'a>(
        &self,
        ir: &'a ConfigIr,
        table: &str,
        field: &str,
    ) -> Option<&'a TypeIr> {
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
}

fn ensure_supported_lua_version(version: LuaVersionIr) -> Result<()> {
    match version {
        LuaVersionIr::Lua51
        | LuaVersionIr::Lua52
        | LuaVersionIr::Lua53
        | LuaVersionIr::Lua54
        | LuaVersionIr::LuaJit => Ok(()),
    }
}

fn uses_string_unpack(version: LuaVersionIr) -> bool {
    matches!(version, LuaVersionIr::Lua53 | LuaVersionIr::Lua54)
}

fn lua_i64_type_name(version: LuaVersionIr) -> &'static str {
    match version {
        LuaVersionIr::Lua53 | LuaVersionIr::Lua54 => "integer",
        LuaVersionIr::Lua51 | LuaVersionIr::Lua52 | LuaVersionIr::LuaJit => "number",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sora_ir::normalize::normalize_schema;
    use sora_schema::model::SchemaFile;
    use std::{
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };

    static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn generates_lua_files_with_emmylua_annotations() {
        let mut ir = example_ir();
        ir.codegen.lua.module = Some("generated.lua".to_owned());
        let base = temp_dir();

        LuaCodeGenerator.generate(&ir, &base).unwrap();

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
        assert!(runtime.contains("function Runtime.parse_bundle(bytes)"));
        assert!(runtime.contains("string.unpack(\"<I4\""));
        assert!(config.contains("local Runtime = require(\"generated.lua.sora_runtime\")"));
        assert!(config.contains("---@class ItemTable"));
        assert!(config.contains("function ItemTable:get(key)"));
        assert!(config.contains("function ItemTable:get_by_name(name)"));
        assert!(config.contains("function ItemTable:find_by_item_type(itemType)"));
        assert!(config.contains("function SoraConfig.from_bytes(bytes)"));
        assert!(config.contains("function SoraConfig:item()"));
        assert!(config.ends_with('\n'));

        let _ = std::fs::remove_dir_all(base);
    }

    #[test]
    fn lua_version_changes_i64_api() {
        let mut ir = example_ir();
        ir.codegen.lua.lua_version = LuaVersionIr::LuaJit;
        let base = temp_dir();

        LuaCodeGenerator.generate(&ir, &base).unwrap();

        let item = std::fs::read_to_string(base.join("item.lua")).unwrap();

        assert!(item.contains("---@field largeId number"));
        assert!(item.contains("largeId = reader:read_i64()"));

        let _ = std::fs::remove_dir_all(base);
    }

    #[test]
    fn lua_enum_options_change_generated_api() {
        let mut ir = example_ir();
        ir.codegen.lua.enum_repr = LuaEnumReprIr::Integer;
        let base = temp_dir();

        LuaCodeGenerator.generate(&ir, &base).unwrap();

        let item_type = std::fs::read_to_string(base.join("item_type.lua")).unwrap();

        assert!(item_type.contains("---| integer"));
        assert!(item_type.contains("Weapon = 0"));
        assert!(item_type.contains("return ordinal"));

        let _ = std::fs::remove_dir_all(base);
    }

    #[test]
    fn lua_compat_runtime_avoids_string_unpack() {
        let mut ir = example_ir();
        ir.codegen.lua.lua_version = LuaVersionIr::LuaJit;
        let base = temp_dir();

        LuaCodeGenerator.generate(&ir, &base).unwrap();

        let runtime = std::fs::read_to_string(base.join("sora_runtime.lua")).unwrap();
        assert!(!runtime.contains("string.unpack"));
        assert!(runtime.contains("function read_f32_at(bytes, offset)"));
        assert!(runtime.contains("function read_i64_at(bytes, offset)"));
        assert!(runtime.contains("safe integer range"));

        let _ = std::fs::remove_dir_all(base);
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
name = "large_id"
type = "i64"
required = true

[[tables.fields]]
name = "action"
type = "union<Action>"
required = true

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
