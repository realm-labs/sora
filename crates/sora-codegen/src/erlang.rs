use std::path::Path;

use heck::ToSnakeCase;
use minijinja::context;
use serde::Serialize;
use sora_diagnostics::Result;
use sora_ir::model::{ConfigIr, ErlangEnumReprIr, TableModeIr, TypeIr};

use crate::{
    generator::{CodeGenerator, ensure_sora_runtime_format},
    model::{
        BaseField, BaseIndex, BaseModel, BaseRecord, BaseTable, BaseUnion, BaseUnionVariant,
        build_base_model,
    },
    render::{ensure_dir, render_template, write_file},
};

pub struct ErlangCodeGenerator;

impl CodeGenerator for ErlangCodeGenerator {
    fn generate(&self, ir: &ConfigIr, out_dir: &Path) -> Result<()> {
        ensure_sora_runtime_format("erlang", ir.codegen.erlang.runtime_format)?;
        ensure_dir(out_dir)?;

        let options = ErlangOptionsView::new(ir.codegen.erlang.enum_repr);
        let model = ErlangModel::from_base_model(ir, build_base_model(ir)?);

        for item in &model.enums {
            let rendered = render_template(
                "erlang",
                "enum.erl.j2",
                context! { enum => item, options => &options },
            )?;
            write_file(&out_dir.join(format!("{}.erl", item.snake_name)), rendered)?;
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

#[derive(Debug, Clone, Serialize)]
struct ErlangModel {
    package: String,
    schema_fingerprint: String,
    enums: Vec<ErlangEnum>,
    unions: Vec<ErlangUnion>,
    records: Vec<ErlangRecord>,
    tables: Vec<ErlangTable>,
}

#[derive(Debug, Clone, Serialize)]
struct ErlangEnum {
    name: String,
    snake_name: String,
    atom_values: Vec<String>,
    values: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct ErlangUnion {
    snake_name: String,
    tag: String,
    variants: Vec<ErlangUnionVariant>,
}

#[derive(Debug, Clone, Serialize)]
struct ErlangUnionVariant {
    snake_name: String,
    reader_var: String,
    fields: Vec<ErlangField>,
}

#[derive(Debug, Clone, Serialize)]
struct ErlangRecord {
    snake_name: String,
    reader_var: String,
    fields: Vec<ErlangField>,
    table: Option<ErlangTable>,
}

#[derive(Debug, Clone, Serialize)]
struct ErlangTable {
    name: String,
    pascal_name: String,
    snake_name: String,
    mode: String,
    container_type: String,
    row_type: String,
    key_name: Option<String>,
    key_field_name: Option<String>,
    key_type: Option<String>,
    unique_indexes: Vec<ErlangIndex>,
    non_unique_indexes: Vec<ErlangIndex>,
}

#[derive(Debug, Clone, Serialize)]
struct ErlangIndex {
    name: String,
    pascal_name: String,
    field_name: String,
    param_type: String,
    param_var_name: String,
}

#[derive(Debug, Clone, Serialize)]
struct ErlangField {
    name: String,
    var_name: String,
    type_name: String,
    decode: String,
}

impl ErlangModel {
    fn from_base_model(ir: &ConfigIr, model: BaseModel) -> Self {
        let enums = model
            .enums
            .into_iter()
            .map(|item| ErlangEnum {
                name: item.pascal_name,
                snake_name: item.snake_name,
                atom_values: item.atom_values,
                values: item.values,
            })
            .collect();
        let tables = model
            .tables
            .into_iter()
            .map(|item| erlang_table(ir, item))
            .collect::<Vec<_>>();
        let records = model
            .records
            .into_iter()
            .map(|item| {
                let table = tables
                    .iter()
                    .find(|table| table.snake_name == item.snake_name)
                    .cloned();
                erlang_record(ir, item, table)
            })
            .collect();
        let unions = model
            .unions
            .into_iter()
            .map(|item| erlang_union(ir, item))
            .collect();

        Self {
            package: model.package,
            schema_fingerprint: model.schema_fingerprint,
            enums,
            unions,
            records,
            tables,
        }
    }
}

fn erlang_union(ir: &ConfigIr, union: BaseUnion) -> ErlangUnion {
    ErlangUnion {
        snake_name: union.snake_name,
        tag: union.tag,
        variants: union
            .variants
            .into_iter()
            .map(|variant| erlang_variant(ir, variant))
            .collect(),
    }
}

fn erlang_variant(ir: &ConfigIr, variant: BaseUnionVariant) -> ErlangUnionVariant {
    let fields = variant
        .fields
        .into_iter()
        .map(|field| erlang_field(ir, field))
        .collect::<Vec<_>>();
    ErlangUnionVariant {
        snake_name: variant.snake_name,
        reader_var: format!("Reader{}", fields.len() + 1),
        fields,
    }
}

fn erlang_record(ir: &ConfigIr, record: BaseRecord, table: Option<ErlangTable>) -> ErlangRecord {
    let fields = record
        .fields
        .into_iter()
        .map(|field| erlang_field(ir, field))
        .collect::<Vec<_>>();
    ErlangRecord {
        snake_name: record.snake_name,
        reader_var: format!("Reader{}", fields.len()),
        fields,
        table,
    }
}

fn erlang_table(ir: &ConfigIr, table: BaseTable) -> ErlangTable {
    let row_type = format!("{}:t()", table.snake_name);
    let key_type = table
        .key_field
        .as_ref()
        .map(|field| erlang_type_name(ir, &field.ty));
    let container_type = erlang_container_type(table.mode, &row_type, key_type.as_deref());
    let key_field_name = table
        .key_field
        .as_ref()
        .map(|field| field.snake_name.clone());

    ErlangTable {
        name: table.name,
        pascal_name: table.pascal_name,
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
            .map(|index| erlang_index(ir, index))
            .collect(),
        non_unique_indexes: table
            .non_unique_indexes
            .into_iter()
            .map(|index| erlang_index(ir, index))
            .collect(),
    }
}

fn erlang_index(ir: &ConfigIr, index: BaseIndex) -> ErlangIndex {
    ErlangIndex {
        name: index.snake_name,
        pascal_name: index.pascal_name,
        field_name: index.field.snake_name.clone(),
        param_type: erlang_type_name(ir, &index.field.ty),
        param_var_name: index.field.pascal_name,
    }
}

fn erlang_field(ir: &ConfigIr, field: BaseField) -> ErlangField {
    ErlangField {
        name: field.snake_name,
        var_name: field.pascal_name,
        type_name: erlang_type_name(ir, &field.ty),
        decode: erlang_decode_fun(ir, &field.ty),
    }
}

fn erlang_container_type(mode: TableModeIr, row_type: &str, key_type: Option<&str>) -> String {
    match mode {
        TableModeIr::List => format!("[{row_type}]"),
        TableModeIr::Map => match key_type {
            Some(key_type) => format!("#{{{key_type} => {row_type}}}"),
            None => format!("[{row_type}]"),
        },
        TableModeIr::Singleton => row_type.to_owned(),
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
        assert!(item.contains("-export([decode/1, decode_table/1"));
        assert!(item.contains("get(Key, Table) ->"));
        assert!(item.contains("get_by_name(Name, Table) ->"));
        assert!(item.contains("find_by_item_type(ItemType, Table) ->"));
        assert!(config.contains("Item = item:decode_table(Bundle),"));
        assert!(!config.contains("item_get(Key, Config) ->"));
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
