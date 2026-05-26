use std::{collections::BTreeMap, path::Path};

use heck::{ToPascalCase, ToShoutySnakeCase, ToSnakeCase};
use minijinja::context;
use serde::Serialize;
use sora_diagnostics::{Result, SoraError};
use sora_ir::model::{ConfigIr, TypeIr};

use crate::{
    generator::{CodeGenerator, CodegenContext, runtime_format_name},
    model::{
        BaseField, BaseIndex, BaseModel, BaseRecord, BaseTable, BaseUnion, BaseUnionVariant,
        build_base_model,
    },
    options::{LanguageCodegenOptions, RuntimeFormat},
    render::{ensure_dir, render_template, write_file},
    types::godot_type_name,
};

pub struct GodotCodeGenerator;
crate::impl_test_codegen_generate!(GodotCodeGenerator, "godot");

impl CodeGenerator for GodotCodeGenerator {
    fn generate(&self, context: CodegenContext<'_>, out_dir: &Path) -> Result<()> {
        let ir = context.ir;
        let options = context.options::<LanguageCodegenOptions>()?;
        if options.runtime_format != RuntimeFormat::Json {
            return Err(SoraError::InvalidSchema(format!(
                "godot codegen runtime_format `{}` is not implemented yet; supported runtime_format: json",
                runtime_format_name(options.runtime_format)
            )));
        }

        ensure_dir(out_dir)?;
        let model = GodotModel::from_base_model(ir, build_base_model(ir)?);

        for item in &model.enums {
            let rendered = render_template("godot", "enum.gd.j2", context! { enum => item })?;
            write_file(&out_dir.join(format!("{}.gd", item.file_name)), rendered)?;
        }

        for record in &model.records {
            let rendered = render_template("godot", "record.gd.j2", context! { record => record })?;
            write_file(&out_dir.join(format!("{}.gd", record.file_name)), rendered)?;
        }

        for union in &model.unions {
            let rendered = render_template("godot", "union.gd.j2", context! { union => union })?;
            write_file(&out_dir.join(format!("{}.gd", union.file_name)), rendered)?;
        }

        let rendered = render_template("godot", "runtime.gd.j2", context! {})?;
        write_file(&out_dir.join("sora_runtime.gd"), rendered)?;

        let rendered = render_template("godot", "config.gd.j2", context! { model => &model })?;
        write_file(&out_dir.join("sora_config.gd"), rendered)
    }
}

#[derive(Debug, Clone, Serialize)]
struct GodotModel {
    schema_fingerprint: String,
    enums: Vec<GodotEnum>,
    records: Vec<GodotRecord>,
    unions: Vec<GodotUnion>,
    tables: Vec<GodotTable>,
}

#[derive(Debug, Clone, Serialize)]
struct GodotEnum {
    class_name: String,
    file_name: String,
    values: Vec<GodotEnumValue>,
}

#[derive(Debug, Clone, Serialize)]
struct GodotEnumValue {
    raw_name: String,
    const_name: String,
}

#[derive(Debug, Clone, Serialize)]
struct GodotRecord {
    class_name: String,
    file_name: String,
    fields: Vec<GodotField>,
    table: Option<GodotTable>,
}

#[derive(Debug, Clone, Serialize)]
struct GodotUnion {
    class_name: String,
    file_name: String,
    raw_tag: String,
    tag: String,
    fields: Vec<GodotField>,
    variants: Vec<GodotUnionVariant>,
}

#[derive(Debug, Clone, Serialize)]
struct GodotUnionVariant {
    raw_name: String,
    fields: Vec<GodotField>,
}

#[derive(Debug, Clone, Serialize)]
struct GodotField {
    raw_name: String,
    name: String,
    type_name: String,
    default_value: String,
    value_decode: String,
    comment: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct GodotTable {
    name: String,
    class_name: String,
    field_name: String,
    mode: String,
    row_type: String,
    key_name: Option<String>,
    key_field_name: Option<String>,
    unique_indexes: Vec<GodotIndex>,
    non_unique_indexes: Vec<GodotIndex>,
}

#[derive(Debug, Clone, Serialize)]
struct GodotIndex {
    method_name: String,
    field_name: String,
    param_name: String,
}

impl GodotModel {
    fn from_base_model(ir: &ConfigIr, model: BaseModel) -> Self {
        let enums = model
            .enums
            .into_iter()
            .map(|item| GodotEnum {
                class_name: godot_type_identifier(&item.pascal_name),
                file_name: godot_file_name(&item.snake_name),
                values: item
                    .values
                    .into_iter()
                    .map(|value| GodotEnumValue {
                        const_name: godot_const_identifier(&value),
                        raw_name: value,
                    })
                    .collect(),
            })
            .collect();
        let tables = model
            .tables
            .into_iter()
            .map(godot_table)
            .collect::<Vec<_>>();
        let records = model
            .records
            .into_iter()
            .map(|item| {
                let class_name = godot_type_identifier(&item.pascal_name);
                let table = tables
                    .iter()
                    .find(|table| table.row_type == class_name)
                    .cloned();
                godot_record(ir, item, table)
            })
            .collect();
        let unions = model
            .unions
            .into_iter()
            .map(|item| godot_union(ir, item))
            .collect();

        Self {
            schema_fingerprint: model.schema_fingerprint,
            enums,
            records,
            unions,
            tables,
        }
    }
}

fn godot_record(ir: &ConfigIr, record: BaseRecord, table: Option<GodotTable>) -> GodotRecord {
    GodotRecord {
        class_name: godot_type_identifier(&record.pascal_name),
        file_name: godot_file_name(&record.snake_name),
        fields: record
            .fields
            .into_iter()
            .map(|field| godot_field(ir, field))
            .collect(),
        table,
    }
}

fn godot_union(ir: &ConfigIr, union: BaseUnion) -> GodotUnion {
    let variants = union
        .variants
        .into_iter()
        .map(|variant| godot_union_variant(ir, variant))
        .collect::<Vec<_>>();
    let fields = flattened_union_fields(&variants);

    GodotUnion {
        class_name: godot_type_identifier(&union.pascal_name),
        file_name: godot_file_name(&union.snake_name),
        raw_tag: union.tag.clone(),
        tag: godot_field_identifier(&union.tag),
        fields,
        variants,
    }
}

fn godot_union_variant(ir: &ConfigIr, variant: BaseUnionVariant) -> GodotUnionVariant {
    GodotUnionVariant {
        raw_name: variant.name,
        fields: variant
            .fields
            .into_iter()
            .map(|field| godot_field(ir, field))
            .collect(),
    }
}

fn flattened_union_fields(variants: &[GodotUnionVariant]) -> Vec<GodotField> {
    let mut fields = BTreeMap::<String, GodotField>::new();
    for variant in variants {
        for field in &variant.fields {
            fields
                .entry(field.raw_name.clone())
                .or_insert_with(|| field.clone());
        }
    }
    fields.into_values().collect()
}

fn godot_table(table: BaseTable) -> GodotTable {
    let key_field_name = table
        .key_field
        .as_ref()
        .map(|field| godot_field_identifier(&field.snake_name));

    GodotTable {
        name: table.name,
        class_name: godot_type_identifier(&format!("{}Table", table.pascal_name)),
        field_name: godot_field_identifier(&table.snake_name),
        mode: table.mode_name,
        row_type: godot_type_identifier(&table.pascal_name),
        key_name: table.key_name,
        key_field_name,
        unique_indexes: table.unique_indexes.into_iter().map(godot_index).collect(),
        non_unique_indexes: table
            .non_unique_indexes
            .into_iter()
            .map(godot_index)
            .collect(),
    }
}

fn godot_index(index: BaseIndex) -> GodotIndex {
    let field_name = godot_field_identifier(&index.field.snake_name);
    GodotIndex {
        method_name: godot_field_identifier(&index.snake_name),
        param_name: field_name.clone(),
        field_name,
    }
}

fn godot_field(ir: &ConfigIr, field: BaseField) -> GodotField {
    GodotField {
        raw_name: field.raw_name,
        name: godot_field_identifier(&field.snake_name),
        type_name: godot_type_name(ir, &field.ty),
        default_value: godot_default_value(ir, &field.ty),
        value_decode: godot_value_decode_expr(ir, &field.ty, "__VALUE__"),
        comment: field.comment,
    }
}

fn godot_value_decode_expr(ir: &ConfigIr, ty: &TypeIr, value: &str) -> String {
    match ty {
        TypeIr::Bool => format!("bool({value})"),
        TypeIr::I32 | TypeIr::I64 => format!("int({value})"),
        TypeIr::F32 | TypeIr::F64 => format!("float({value})"),
        TypeIr::String | TypeIr::Text => format!("str({value})"),
        TypeIr::Enum(name) => format!("{}.decode({value})", godot_type_identifier(name)),
        TypeIr::Struct(name) | TypeIr::Union(name) => {
            format!("{}.decode({value})", godot_type_identifier(name))
        }
        TypeIr::List(element) | TypeIr::Set(element) | TypeIr::Array { element, .. } => format!(
            "SoraRuntime.decode_array({value}, func(item): return {})",
            godot_value_decode_expr(ir, element, "item")
        ),
        TypeIr::Map {
            key,
            value: element,
        } => format!(
            "SoraRuntime.decode_map({value}, func(item): return {}, func(item): return {})",
            godot_value_decode_expr(ir, key, "item"),
            godot_value_decode_expr(ir, element, "item")
        ),
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
            .map(|field| godot_value_decode_expr(ir, &field.ty, value))
            .unwrap_or_else(|| format!("{value}.as_int()")),
        TypeIr::Optional(element) => {
            format!(
                "null if {value}.is_null() else {}",
                godot_value_decode_expr(ir, element, value)
            )
        }
    }
}

fn godot_default_value(ir: &ConfigIr, ty: &TypeIr) -> String {
    match ty {
        TypeIr::Bool => "false".to_owned(),
        TypeIr::I32 | TypeIr::I64 => "0".to_owned(),
        TypeIr::F32 | TypeIr::F64 => "0.0".to_owned(),
        TypeIr::String | TypeIr::Text | TypeIr::Enum(_) => "\"\"".to_owned(),
        TypeIr::List(_) | TypeIr::Set(_) | TypeIr::Map { .. } | TypeIr::Array { .. } => {
            "[]".to_owned()
        }
        TypeIr::Optional(_) | TypeIr::Struct(_) | TypeIr::Union(_) => "null".to_owned(),
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
            .map(|field| godot_default_value(ir, &field.ty))
            .unwrap_or_else(|| "0".to_owned()),
    }
}

fn godot_file_name(value: &str) -> String {
    sanitize_identifier(&value.to_snake_case(), CaseKind::Snake)
}

fn godot_type_identifier(value: &str) -> String {
    sanitize_identifier(&value.to_pascal_case(), CaseKind::Pascal)
}

fn godot_field_identifier(value: &str) -> String {
    sanitize_identifier(&value.to_snake_case(), CaseKind::Snake)
}

fn godot_const_identifier(value: &str) -> String {
    sanitize_identifier(&value.to_shouty_snake_case(), CaseKind::Const)
}

#[derive(Clone, Copy)]
enum CaseKind {
    Snake,
    Pascal,
    Const,
}

fn sanitize_identifier(value: &str, case: CaseKind) -> String {
    let mut out = String::with_capacity(value.len());
    for (index, ch) in value.chars().enumerate() {
        let valid = ch == '_' || ch.is_ascii_alphanumeric();
        let ch = if valid { ch } else { '_' };
        if index == 0 && ch.is_ascii_digit() {
            out.push(match case {
                CaseKind::Pascal => 'T',
                CaseKind::Snake => 'v',
                CaseKind::Const => '_',
            });
        }
        out.push(ch);
    }
    if out.is_empty() || out == "_" {
        out = match case {
            CaseKind::Pascal => "Value".to_owned(),
            CaseKind::Snake => "value".to_owned(),
            CaseKind::Const => "VALUE".to_owned(),
        };
    }
    if is_godot_keyword(&out) {
        out.push('_');
    }
    out
}

fn is_godot_keyword(value: &str) -> bool {
    matches!(
        value,
        "Array"
            | "Dictionary"
            | "String"
            | "Variant"
            | "bool"
            | "break"
            | "class"
            | "class_name"
            | "const"
            | "continue"
            | "else"
            | "extends"
            | "false"
            | "float"
            | "for"
            | "func"
            | "if"
            | "in"
            | "int"
            | "match"
            | "null"
            | "pass"
            | "return"
            | "self"
            | "static"
            | "true"
            | "var"
            | "void"
            | "while"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::options::{LanguageCodegenOptions, RuntimeFormat};
    use sora_ir::normalize::normalize_schema;
    use sora_schema::model::SchemaFile;

    #[test]
    fn generates_godot_json_runtime_files() {
        let schema: SchemaFile = toml::from_str(
            r#"
package = "game_config"

[codegen.godot]
runtime_format = "json"

[[enums]]
name = "ItemType"
values = ["Weapon", "Armor"]

[[structs]]
name = "Cost"

[[structs.fields]]
name = "item_id"
type = "i32"

[[unions]]
name = "RewardAction"

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
name = "item_type"
type = "enum<ItemType>"

[[tables.indexes]]
name = "by_item_type"
fields = ["item_type"]
"#,
        )
        .unwrap();
        let ir = normalize_schema(schema).unwrap();
        let out = std::env::temp_dir().join("sora-codegen-godot-test");
        let _ = std::fs::remove_dir_all(&out);

        GodotCodeGenerator
            .generate_with_options(
                &ir,
                LanguageCodegenOptions {
                    runtime_format: RuntimeFormat::Json,
                },
                &out,
            )
            .unwrap();

        assert!(out.join("sora_runtime.gd").exists());
        assert!(out.join("sora_config.gd").exists());
        assert!(out.join("item.gd").exists());
        assert!(out.join("item_type.gd").exists());
        assert!(out.join("reward_action.gd").exists());
        let item = std::fs::read_to_string(out.join("item.gd")).unwrap();
        let config = std::fs::read_to_string(out.join("sora_config.gd")).unwrap();
        assert!(item.contains("class ItemTable"));
        assert!(item.contains("func find_by_item_type"));
        assert!(!config.contains("class ItemTable"));

        let _ = std::fs::remove_dir_all(out);
    }
}
