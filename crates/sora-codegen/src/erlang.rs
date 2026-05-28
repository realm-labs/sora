use std::path::Path;

use heck::ToSnakeCase;
use minijinja::context;
use serde::Serialize;
use sora_diagnostics::Result;
use sora_ir::model::{ConfigIr, TableModeIr, TypeIr};

use crate::{
    generator::{CodeGenerator, CodegenContext, runtime_format_name},
    model::{
        BaseField, BaseIndex, BaseModel, BaseRecord, BaseTable, BaseUnion, BaseUnionVariant,
        build_base_model,
    },
    options::{ErlangCodegenOptions, ErlangEnumRepr},
    render::{ensure_dir, render_template, write_file},
    type_mapping::{TypeMapping, TypeMappingContext, TypeMappingRegistry},
};

pub struct ErlangCodeGenerator;
crate::impl_test_codegen_generate!(ErlangCodeGenerator, "erlang");

impl CodeGenerator for ErlangCodeGenerator {
    fn generate(&self, context: CodegenContext<'_>, out_dir: &Path) -> Result<()> {
        let ir = context.ir;
        let codegen_options = context.options::<ErlangCodegenOptions>()?;
        ensure_dir(out_dir)?;
        let runtime_format = runtime_format_name(codegen_options.runtime_format);

        let options = ErlangOptionsView::new(codegen_options.enum_repr);
        let mapper = ErlangTypeMapper::new(context.target, ir, context.type_mappings);
        let model = ErlangModel::from_base_model(ir, build_base_model(ir)?, &mapper);

        for item in &model.enums {
            let rendered = render_template(
                "erlang",
                "enum.erl.j2",
                context! { enum => item, options => &options },
            )?;
            write_file(&out_dir.join(format!("{}.erl", item.snake_name)), rendered)?;
        }

        for record in &model.records {
            let rendered = render_template(
                "erlang",
                "record.erl.j2",
                context! { record => record, runtime_format => runtime_format },
            )?;
            write_file(
                &out_dir.join(format!("{}.erl", record.snake_name)),
                rendered,
            )?;
        }

        for union in &model.unions {
            let rendered = render_template(
                "erlang",
                "union.erl.j2",
                context! { union => union, runtime_format => runtime_format },
            )?;
            write_file(&out_dir.join(format!("{}.erl", union.snake_name)), rendered)?;
        }

        let rendered = render_template(
            "erlang",
            "runtime.erl.j2",
            context! { runtime_format => runtime_format },
        )?;
        write_file(&out_dir.join("sora_runtime.erl"), rendered)?;

        let rendered = render_template(
            "erlang",
            "config.erl.j2",
            context! { model => &model, runtime_format => runtime_format },
        )?;
        write_file(&out_dir.join("sora_config.erl"), rendered)
    }
}

#[derive(Debug, Clone, Serialize)]
struct ErlangOptionsView {
    enum_is_integer: bool,
}

impl ErlangOptionsView {
    fn new(enum_repr: ErlangEnumRepr) -> Self {
        Self {
            enum_is_integer: enum_repr == ErlangEnumRepr::Integer,
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
    raw_name: String,
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
    raw_name: String,
    name: String,
    var_name: String,
    type_name: String,
    decode: String,
    value_decode: String,
}

impl ErlangModel {
    fn from_base_model(ir: &ConfigIr, model: BaseModel, mapper: &ErlangTypeMapper<'_>) -> Self {
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
            .map(|item| erlang_table(ir, item, mapper))
            .collect::<Vec<_>>();
        let records = model
            .records
            .into_iter()
            .map(|item| {
                let table = tables
                    .iter()
                    .find(|table| table.snake_name == item.snake_name)
                    .cloned();
                erlang_record(ir, item, table, mapper)
            })
            .collect();
        let unions = model
            .unions
            .into_iter()
            .map(|item| erlang_union(ir, item, mapper))
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

fn erlang_union(ir: &ConfigIr, union: BaseUnion, mapper: &ErlangTypeMapper<'_>) -> ErlangUnion {
    ErlangUnion {
        snake_name: union.snake_name,
        tag: union.tag,
        variants: union
            .variants
            .into_iter()
            .map(|variant| erlang_variant(ir, variant, mapper))
            .collect(),
    }
}

fn erlang_variant(
    ir: &ConfigIr,
    variant: BaseUnionVariant,
    mapper: &ErlangTypeMapper<'_>,
) -> ErlangUnionVariant {
    let fields = variant
        .fields
        .into_iter()
        .map(|field| erlang_field(ir, field, mapper))
        .collect::<Vec<_>>();
    ErlangUnionVariant {
        raw_name: variant.name,
        snake_name: variant.snake_name,
        reader_var: format!("Reader{}", fields.len() + 1),
        fields,
    }
}

fn erlang_record(
    ir: &ConfigIr,
    record: BaseRecord,
    table: Option<ErlangTable>,
    mapper: &ErlangTypeMapper<'_>,
) -> ErlangRecord {
    let fields = record
        .fields
        .into_iter()
        .map(|field| erlang_field(ir, field, mapper))
        .collect::<Vec<_>>();
    ErlangRecord {
        snake_name: record.snake_name,
        reader_var: format!("Reader{}", fields.len()),
        fields,
        table,
    }
}

fn erlang_table(ir: &ConfigIr, table: BaseTable, mapper: &ErlangTypeMapper<'_>) -> ErlangTable {
    let row_type = format!("{}:t()", table.snake_name);
    let key_type = table
        .key_field
        .as_ref()
        .map(|field| mapper.type_name(&field.ty));
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
            .map(|index| erlang_index(ir, index, mapper))
            .collect(),
        non_unique_indexes: table
            .non_unique_indexes
            .into_iter()
            .map(|index| erlang_index(ir, index, mapper))
            .collect(),
    }
}

fn erlang_index(_ir: &ConfigIr, index: BaseIndex, mapper: &ErlangTypeMapper<'_>) -> ErlangIndex {
    ErlangIndex {
        name: index.snake_name,
        pascal_name: index.pascal_name,
        field_name: index.field.snake_name.clone(),
        param_type: mapper.type_name(&index.field.ty),
        param_var_name: index.field.pascal_name,
    }
}

fn erlang_field(ir: &ConfigIr, field: BaseField, mapper: &ErlangTypeMapper<'_>) -> ErlangField {
    ErlangField {
        raw_name: field.raw_name,
        name: field.snake_name,
        var_name: field.pascal_name,
        type_name: mapper.type_name(&field.ty),
        decode: erlang_decode_fun(ir, &field.ty, mapper),
        value_decode: erlang_value_decode_expr(ir, &field.ty, "__VALUE__", mapper),
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

struct ErlangTypeMapper<'a> {
    target: &'a str,
    ir: &'a ConfigIr,
    mappings: &'a TypeMappingRegistry,
}

impl<'a> ErlangTypeMapper<'a> {
    fn new(target: &'a str, ir: &'a ConfigIr, mappings: &'a TypeMappingRegistry) -> Self {
        Self {
            target,
            ir,
            mappings,
        }
    }

    fn type_name(&self, ty: &TypeIr) -> String {
        if let Some(mapping) = self.mapping(ty) {
            return mapping.type_name;
        }

        match ty {
            TypeIr::Bool => "boolean()".to_owned(),
            TypeIr::I8
            | TypeIr::U8
            | TypeIr::I16
            | TypeIr::U16
            | TypeIr::I32
            | TypeIr::U32
            | TypeIr::I64
            | TypeIr::Duration
            | TypeIr::DateTime => "integer()".to_owned(),
            TypeIr::F32 | TypeIr::F64 => "float()".to_owned(),
            TypeIr::String => "binary()".to_owned(),
            TypeIr::Text => "sora_runtime:text_key()".to_owned(),
            TypeIr::Enum(name) | TypeIr::Struct(name) | TypeIr::Union(name) => {
                format!("{}:t()", name.to_snake_case())
            }
            TypeIr::List(element) | TypeIr::Set(element) | TypeIr::Array { element, .. } => {
                format!("[{}]", self.type_name(element))
            }
            TypeIr::Map { key, value } => {
                format!("#{{{} => {}}}", self.type_name(key), self.type_name(value))
            }
            TypeIr::Ref { table, field } => ref_type(self.ir, table, field)
                .map(|ty| self.type_name(ty))
                .unwrap_or_else(|| "integer()".to_owned()),
            TypeIr::Optional(element) => format!("{} | undefined", self.type_name(element)),
        }
    }

    fn mapping(&self, ty: &TypeIr) -> Option<TypeMapping> {
        self.mappings.map_type(TypeMappingContext {
            target: self.target,
            ir: self.ir,
            ty,
        })
    }

    fn wrap_decode(&self, ty: &TypeIr, base_expr: String) -> String {
        self.mapping(ty)
            .map(|mapping| mapping.wrap_decode(&base_expr))
            .unwrap_or(base_expr)
    }

    fn wrap_value_decode(&self, ty: &TypeIr, base_expr: String) -> String {
        self.mapping(ty)
            .map(|mapping| mapping.wrap_value_decode(&base_expr))
            .unwrap_or(base_expr)
    }
}

fn erlang_decode_fun(ir: &ConfigIr, ty: &TypeIr, mapper: &ErlangTypeMapper<'_>) -> String {
    match ty {
        TypeIr::Bool => "fun sora_runtime:read_bool/1".to_owned(),
        TypeIr::I8 | TypeIr::I16 | TypeIr::I32 => "fun sora_runtime:read_i32/1".to_owned(),
        TypeIr::U8 | TypeIr::U16 | TypeIr::U32 => "fun sora_runtime:read_u32/1".to_owned(),
        TypeIr::I64 | TypeIr::Duration | TypeIr::DateTime => {
            "fun sora_runtime:read_i64/1".to_owned()
        }
        TypeIr::F32 => "fun sora_runtime:read_f32/1".to_owned(),
        TypeIr::F64 => "fun sora_runtime:read_f64/1".to_owned(),
        TypeIr::String => "fun sora_runtime:read_string/1".to_owned(),
        TypeIr::Text => "fun sora_runtime:read_text_key/1".to_owned(),
        TypeIr::Enum(name) | TypeIr::Struct(name) | TypeIr::Union(name) => {
            mapper.wrap_decode(ty, format!("fun {}:decode/1", name.to_snake_case()))
        }
        TypeIr::List(element) | TypeIr::Set(element) | TypeIr::Array { element, .. } => {
            format!(
                "fun(Reader) -> sora_runtime:read_list({}, Reader) end",
                erlang_decode_fun(ir, element, mapper)
            )
        }
        TypeIr::Map { key, value } => format!(
            "fun(Reader) -> sora_runtime:read_map({}, {}, Reader) end",
            erlang_decode_fun(ir, key, mapper),
            erlang_decode_fun(ir, value, mapper)
        ),
        TypeIr::Ref { table, field } => ref_type(ir, table, field)
            .map(|ty| erlang_decode_fun(ir, ty, mapper))
            .unwrap_or_else(|| "fun sora_runtime:read_i32/1".to_owned()),
        TypeIr::Optional(element) => {
            format!(
                "fun(Reader) -> sora_runtime:read_optional({}, Reader) end",
                erlang_decode_fun(ir, element, mapper)
            )
        }
    }
}

fn erlang_value_decode_expr(
    ir: &ConfigIr,
    ty: &TypeIr,
    value: &str,
    mapper: &ErlangTypeMapper<'_>,
) -> String {
    match ty {
        TypeIr::Bool => format!("sora_runtime:expect_boolean({value})"),
        TypeIr::I8
        | TypeIr::U8
        | TypeIr::I16
        | TypeIr::U16
        | TypeIr::I32
        | TypeIr::U32
        | TypeIr::I64
        | TypeIr::Duration
        | TypeIr::DateTime => format!("sora_runtime:expect_integer({value})"),
        TypeIr::F32 | TypeIr::F64 => format!("sora_runtime:expect_float({value})"),
        TypeIr::String => format!("sora_runtime:expect_binary({value})"),
        TypeIr::Text => format!("sora_runtime:expect_text_key({value})"),
        TypeIr::Enum(name) | TypeIr::Struct(name) | TypeIr::Union(name) => mapper
            .wrap_value_decode(
                ty,
                format!("{}:decode_value({value})", name.to_snake_case()),
            ),
        TypeIr::List(element) | TypeIr::Set(element) | TypeIr::Array { element, .. } => {
            format!(
                "sora_runtime:decode_value_list(fun(Item) -> {} end, {value})",
                erlang_value_decode_expr(ir, element, "Item", mapper)
            )
        }
        TypeIr::Map {
            key,
            value: element,
        } => format!(
            "sora_runtime:decode_value_map(fun(Item) -> {} end, fun(Item) -> {} end, {value})",
            erlang_value_decode_expr(ir, key, "Item", mapper),
            erlang_value_decode_expr(ir, element, "Item", mapper)
        ),
        TypeIr::Ref { table, field } => ref_type(ir, table, field)
            .map(|ty| erlang_value_decode_expr(ir, ty, value, mapper))
            .unwrap_or_else(|| format!("sora_runtime:expect_integer({value})")),
        TypeIr::Optional(element) => {
            format!(
                "(fun(OptionalValue) -> case OptionalValue of undefined -> undefined; _ -> {} end end)({value})",
                erlang_value_decode_expr(ir, element, "OptionalValue", mapper)
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
    use crate::options::{ErlangCodegenOptions, ErlangEnumRepr, RuntimeFormat};
    use sora_ir::normalize::normalize_schema;
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
        assert!(runtime.contains("read_i64(Reader0) ->"));
        assert!(runtime.contains("zigzag_decode(Value)"));
        assert!(!runtime.contains("read_u64_at("));
        assert!(item.contains("-export([decode/1, decode_value/1, decode_table/1"));
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
        let ir = example_ir();
        let base = temp_dir();

        ErlangCodeGenerator
            .generate_with_options(
                &ir,
                ErlangCodegenOptions {
                    enum_repr: ErlangEnumRepr::Integer,
                    ..Default::default()
                },
                &base,
            )
            .unwrap();

        let item_type = std::fs::read_to_string(base.join("item_type.erl")).unwrap();
        assert!(item_type.contains("-type t() ::"));
        assert!(item_type.contains("0 |"));
        assert!(item_type.contains("1."));
        assert!(item_type.contains("0 -> {Ordinal, Reader1};"));

        let _ = std::fs::remove_dir_all(base);
    }

    #[test]
    fn erlang_supports_adapter_export_runtime_formats() {
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

            ErlangCodeGenerator
                .generate_with_options(
                    &ir,
                    ErlangCodegenOptions {
                        runtime_format,
                        ..Default::default()
                    },
                    &base,
                )
                .unwrap();

            let item = std::fs::read_to_string(base.join("item.erl")).unwrap();
            let item_type = std::fs::read_to_string(base.join("item_type.erl")).unwrap();
            let action = std::fs::read_to_string(base.join("action.erl")).unwrap();
            let runtime = std::fs::read_to_string(base.join("sora_runtime.erl")).unwrap();
            let config = std::fs::read_to_string(base.join("sora_config.erl")).unwrap();

            assert!(runtime.contains(&format!("{parse_function}(Bytes, Options) ->")));
            assert!(runtime.contains(adapter));
            assert!(runtime.contains("-type value_bundle() ::"));
            assert!(config.contains("from_bundle/1"));
            assert!(config.contains(parse_function));
            assert!(item.contains("-export([decode/1, decode_value/1, decode_table/1"));
            assert!(item.contains("decode_value(Value) ->"));
            assert!(
                item.contains(
                    "sora_runtime:expect_integer(sora_runtime:value_get(<<\"id\">>, Obj))"
                )
            );
            assert!(item_type.contains("-export([decode/1, decode_value/1])."));
            assert!(action.contains("decode_value(Value) ->"));

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
        std::env::temp_dir().join(format!("sora-erlang-codegen-test-{unique}"))
    }
}
