use std::path::Path;

use heck::ToSnakeCase;
use minijinja::context;
use serde::Serialize;
use sora_diagnostics::Result;
use sora_ir::model::{ConfigIr, ErlangEnumReprIr, TableModeIr, TypeIr};

use crate::{
    generator::{CodeGenerator, ensure_sora_runtime_format},
    model::{LanguageBackend, TableNameParts, build_model},
    render::{ensure_dir, render_template, write_file},
};

pub struct ErlangCodeGenerator;

impl CodeGenerator for ErlangCodeGenerator {
    fn generate(&self, ir: &ConfigIr, out_dir: &Path) -> Result<()> {
        ensure_sora_runtime_format("erlang", ir.codegen.erlang.runtime_format)?;
        ensure_dir(out_dir)?;

        let options = ErlangOptionsView::new(ir.codegen.erlang.enum_repr);
        let backend = ErlangBackend;
        let model = build_model(ir, &backend)?;

        for item in &model.enums {
            let rendered = render_template(
                "erlang",
                "enum.erl.j2",
                context! { enum => item, options => &options },
            )?;
            write_file(
                &out_dir.join(format!("{}.erl", item.name.to_snake_case())),
                rendered,
            )?;
        }

        for record in &model.records {
            let rendered =
                render_template("erlang", "record.erl.j2", context! { record => record })?;
            write_file(
                &out_dir.join(format!("{}.erl", record.snake_name)),
                rendered,
            )?;
        }

        for union in &model.unions {
            let rendered = render_template("erlang", "union.erl.j2", context! { union => union })?;
            write_file(&out_dir.join(format!("{}.erl", union.snake_name)), rendered)?;
        }

        let rendered = render_template("erlang", "runtime.erl.j2", context! {})?;
        write_file(&out_dir.join("sora_runtime.erl"), rendered)?;

        let rendered = render_template("erlang", "config.erl.j2", context! { model => &model })?;
        write_file(&out_dir.join("sora_config.erl"), rendered)
    }
}

#[derive(Debug, Clone, Serialize)]
struct ErlangOptionsView {
    enum_is_integer: bool,
}

impl ErlangOptionsView {
    fn new(enum_repr: ErlangEnumReprIr) -> Self {
        Self {
            enum_is_integer: enum_repr == ErlangEnumReprIr::Integer,
        }
    }
}

#[derive(Debug, Clone)]
struct ErlangBackend;

impl LanguageBackend for ErlangBackend {
    fn field_name(&self, raw_name: &str) -> String {
        raw_name.to_snake_case()
    }

    fn type_name(&self, ir: &ConfigIr, ty: &TypeIr) -> String {
        erlang_type_name(ir, ty)
    }

    fn decode_expr(&self, ir: &ConfigIr, ty: &TypeIr) -> String {
        erlang_decode_fun(ir, ty)
    }

    fn row_type(&self, table: &TableNameParts<'_>) -> String {
        format!("{}:t()", table.snake_name)
    }

    fn container_type(
        &self,
        _table: &TableNameParts<'_>,
        mode: TableModeIr,
        row_type: &str,
        key_type: Option<&str>,
    ) -> String {
        match mode {
            TableModeIr::List => format!("[{row_type}]"),
            TableModeIr::Map => match key_type {
                Some(key_type) => format!("#{{{key_type} => {row_type}}}"),
                None => format!("[{row_type}]"),
            },
            TableModeIr::Singleton => row_type.to_owned(),
        }
    }
}

fn erlang_type_name(ir: &ConfigIr, ty: &TypeIr) -> String {
    match ty {
        TypeIr::Bool => "boolean()".to_owned(),
        TypeIr::I32 | TypeIr::I64 => "integer()".to_owned(),
        TypeIr::F32 | TypeIr::F64 => "float()".to_owned(),
        TypeIr::String => "binary()".to_owned(),
        TypeIr::Enum(name) | TypeIr::Struct(name) | TypeIr::Union(name) => {
            format!("{}:t()", name.to_snake_case())
        }
        TypeIr::List(element) | TypeIr::Array { element, .. } => {
            format!("[{}]", erlang_type_name(ir, element))
        }
        TypeIr::Ref { table, field } => ref_type(ir, table, field)
            .map(|ty| erlang_type_name(ir, ty))
            .unwrap_or_else(|| "integer()".to_owned()),
        TypeIr::Optional(element) => format!("{} | undefined", erlang_type_name(ir, element)),
    }
}

fn erlang_decode_fun(ir: &ConfigIr, ty: &TypeIr) -> String {
    match ty {
        TypeIr::Bool => "fun sora_runtime:read_bool/1".to_owned(),
        TypeIr::I32 => "fun sora_runtime:read_i32/1".to_owned(),
        TypeIr::I64 => "fun sora_runtime:read_i64/1".to_owned(),
        TypeIr::F32 => "fun sora_runtime:read_f32/1".to_owned(),
        TypeIr::F64 => "fun sora_runtime:read_f64/1".to_owned(),
        TypeIr::String => "fun sora_runtime:read_string/1".to_owned(),
        TypeIr::Enum(name) | TypeIr::Struct(name) | TypeIr::Union(name) => {
            format!("fun {}:decode/1", name.to_snake_case())
        }
        TypeIr::List(element) | TypeIr::Array { element, .. } => {
            format!(
                "fun(Reader) -> sora_runtime:read_list({}, Reader) end",
                erlang_decode_fun(ir, element)
            )
        }
        TypeIr::Ref { table, field } => ref_type(ir, table, field)
            .map(|ty| erlang_decode_fun(ir, ty))
            .unwrap_or_else(|| "fun sora_runtime:read_i32/1".to_owned()),
        TypeIr::Optional(element) => {
            format!(
                "fun(Reader) -> sora_runtime:read_optional({}, Reader) end",
                erlang_decode_fun(ir, element)
            )
        }
    }
}

fn ref_type<'a>(ir: &'a ConfigIr, table_name: &str, field_name: &str) -> Option<&'a TypeIr> {
    ir.tables
        .iter()
        .find(|table| table.name == table_name)
        .and_then(|table| table.fields.iter().find(|field| field.name == field_name))
        .map(|field| &field.ty)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sora_ir::{model::ErlangEnumReprIr, normalize::normalize_schema};
    use sora_schema::model::SchemaFile;
    use std::{
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };

    static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn generates_erlang_files() {
        let ir = example_ir();
        let base = temp_dir();

        ErlangCodeGenerator.generate(&ir, &base).unwrap();

        let item = std::fs::read_to_string(base.join("item.erl")).unwrap();
        let item_type = std::fs::read_to_string(base.join("item_type.erl")).unwrap();
        let action = std::fs::read_to_string(base.join("action.erl")).unwrap();
        let runtime = std::fs::read_to_string(base.join("sora_runtime.erl")).unwrap();
        let config = std::fs::read_to_string(base.join("sora_config.erl")).unwrap();

        assert!(item.contains("-module(item)."));
        assert!(item.contains("-type t() :: #{"));
        assert!(item.contains("'large_id' := integer()"));
        assert!(item.contains("{LargeId, Reader4} = (fun sora_runtime:read_i64/1)(Reader3)"));
        assert!(item_type.contains("-type t() ::"));
        assert!(item_type.contains("'weapon' |"));
        assert!(item_type.contains("'armor'."));
        assert!(item_type.contains("0 -> {'weapon', Reader1};"));
        assert!(action.contains("'type' := 'add_item'"));
        assert!(runtime.contains("read_i64(<<Value:64/little-signed-integer, Rest/binary>>)"));
        assert!(config.contains("item_get(Key, Config) ->"));
        assert!(config.contains("item_get_by_name(Name, Config) ->"));
        assert!(config.contains("item_find_by_item_type(ItemType, Config) ->"));
        assert!(config.ends_with('\n'));

        let _ = std::fs::remove_dir_all(base);
    }

    #[test]
    fn erlang_integer_enum_option_changes_api() {
        let mut ir = example_ir();
        ir.codegen.erlang.enum_repr = ErlangEnumReprIr::Integer;
        let base = temp_dir();

        ErlangCodeGenerator.generate(&ir, &base).unwrap();

        let item_type = std::fs::read_to_string(base.join("item_type.erl")).unwrap();
        assert!(item_type.contains("-type t() ::"));
        assert!(item_type.contains("0 |"));
        assert!(item_type.contains("1."));
        assert!(item_type.contains("0 -> {Ordinal, Reader1};"));

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
        std::env::temp_dir().join(format!("sora-erlang-codegen-test-{unique}"))
    }
}
