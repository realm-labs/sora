use std::path::{Path, PathBuf};

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
    options::LanguageCodegenOptions,
    render::{ensure_dir, render_template, write_file},
    types::kotlin_type_name,
};

pub struct KotlinCodeGenerator;
crate::impl_test_codegen_generate!(KotlinCodeGenerator, "kotlin");

impl CodeGenerator for KotlinCodeGenerator {
    fn generate(&self, context: CodegenContext<'_>, out_dir: &Path) -> Result<()> {
        let ir = context.ir;
        let options = context.options::<LanguageCodegenOptions>()?;
        ensure_dir(out_dir)?;
        let model = KotlinModel::from_base_model(ir, build_base_model(ir)?);
        let package_dir = kotlin_package_dir(out_dir, &model.package)?;
        let runtime_format = runtime_format_name(options.runtime_format);

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
    schema_fingerprint: String,
    enums: Vec<KotlinEnum>,
    unions: Vec<KotlinUnion>,
    records: Vec<KotlinRecord>,
    tables: Vec<KotlinTable>,
    has_localization: bool,
    locales: Vec<String>,
    default_locale: String,
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
    has_text_keys: bool,
}

#[derive(Debug, Clone, Serialize)]
struct KotlinRecord {
    pascal_name: String,
    fields: Vec<KotlinField>,
    has_text_keys: bool,
    table: Option<KotlinTable>,
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
    collect_text_keys: String,
    comment: Option<String>,
}

impl KotlinModel {
    fn from_base_model(ir: &ConfigIr, model: BaseModel) -> Self {
        let tables = model
            .tables
            .into_iter()
            .map(|item| kotlin_table(ir, item))
            .collect::<Vec<_>>();
        Self {
            package: model.package,
            schema_fingerprint: model.schema_fingerprint,
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
                .map(|item| {
                    let table = tables
                        .iter()
                        .find(|table| table.row_type == item.pascal_name)
                        .cloned();
                    kotlin_record(ir, item, table)
                })
                .collect(),
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
            tables,
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
    let fields = variant
        .fields
        .into_iter()
        .map(|field| kotlin_field(ir, field))
        .collect::<Vec<_>>();
    let has_text_keys = fields
        .iter()
        .any(|field| !field.collect_text_keys.is_empty());
    KotlinUnionVariant {
        name: variant.pascal_name,
        fields,
        has_text_keys,
    }
}

fn kotlin_record(ir: &ConfigIr, record: BaseRecord, table: Option<KotlinTable>) -> KotlinRecord {
    let fields = record
        .fields
        .into_iter()
        .map(|field| kotlin_field(ir, field))
        .collect::<Vec<_>>();
    let has_text_keys = fields
        .iter()
        .any(|field| !field.collect_text_keys.is_empty());
    KotlinRecord {
        pascal_name: record.pascal_name,
        fields,
        has_text_keys,
        table,
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
    let collect_text_keys =
        kotlin_collect_text_keys(ir, &field.ty, &format!("this.{}", field.camel_name), 8);
    KotlinField {
        raw_name: field.raw_name,
        name: field.camel_name,
        type_name: kotlin_type_name(ir, &field.ty),
        decode: kotlin_decode_expr(ir, &field.ty),
        value_decode,
        collect_text_keys,
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
        TypeIr::I8 => "reader.readI32().toByte()".to_owned(),
        TypeIr::U8 => "reader.readU32().toShort()".to_owned(),
        TypeIr::I16 => "reader.readI32().toShort()".to_owned(),
        TypeIr::U16 => "reader.readU32()".to_owned(),
        TypeIr::I32 => "reader.readI32()".to_owned(),
        TypeIr::U32 => "reader.readU32().toLong() and 0xffffffffL".to_owned(),
        TypeIr::I64 | TypeIr::Duration => "reader.readI64()".to_owned(),
        TypeIr::F32 => "reader.readF32()".to_owned(),
        TypeIr::F64 => "reader.readF64()".to_owned(),
        TypeIr::String => "reader.readString()".to_owned(),
        TypeIr::Text => "TextKey(reader.readString())".to_owned(),
        TypeIr::Enum(name) | TypeIr::Struct(name) | TypeIr::Union(name) => {
            format!("{name}.decode(reader)")
        }
        TypeIr::List(element) | TypeIr::Set(element) | TypeIr::Array { element, .. } => {
            format!("reader.readList {{ {} }}", kotlin_decode_expr(ir, element))
        }
        TypeIr::Map { key, value } => format!(
            "reader.readMap({{ {} }}, {{ {} }})",
            kotlin_decode_expr(ir, key),
            kotlin_decode_expr(ir, value)
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
        TypeIr::I8 => format!("{value}.asInt().toByte()"),
        TypeIr::U8 | TypeIr::I16 => format!("{value}.asInt().toShort()"),
        TypeIr::U16 => format!("{value}.asInt()"),
        TypeIr::I32 => format!("{value}.asInt()"),
        TypeIr::U32 => format!("{value}.asLong()"),
        TypeIr::I64 | TypeIr::Duration => format!("{value}.asLong()"),
        TypeIr::F32 => format!("{value}.asFloat()"),
        TypeIr::F64 => format!("{value}.asDouble()"),
        TypeIr::String => format!("{value}.asString()"),
        TypeIr::Text => format!("TextKey({value}.asString())"),
        TypeIr::Enum(name) | TypeIr::Struct(name) | TypeIr::Union(name) => {
            format!("{name}.decode({value})")
        }
        TypeIr::List(element) | TypeIr::Set(element) | TypeIr::Array { element, .. } => {
            format!(
                "{value}.asList {{ item -> {} }}",
                kotlin_value_decode_expr(ir, element, "item")
            )
        }
        TypeIr::Map {
            key,
            value: element,
        } => format!(
            "{value}.asMap({{ item -> {} }}, {{ item -> {} }})",
            kotlin_value_decode_expr(ir, key, "item"),
            kotlin_value_decode_expr(ir, element, "item")
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

fn kotlin_collect_text_keys(ir: &ConfigIr, ty: &TypeIr, value: &str, indent: usize) -> String {
    let pad = " ".repeat(indent);
    match ty {
        TypeIr::Text => format!("{pad}out.add({value})"),
        TypeIr::Optional(element) => {
            let inner = kotlin_collect_text_keys(ir, element, "item", indent + 4);
            if inner.is_empty() {
                String::new()
            } else {
                format!("{pad}{value}?.let {{ item ->\n{inner}\n{pad}}}")
            }
        }
        TypeIr::List(element) | TypeIr::Set(element) | TypeIr::Array { element, .. } => {
            let inner = kotlin_collect_text_keys(ir, element, "item", indent + 4);
            if inner.is_empty() {
                String::new()
            } else {
                format!("{pad}for (item in {value}) {{\n{inner}\n{pad}}}")
            }
        }
        TypeIr::Map {
            key,
            value: element,
        } => {
            let key_inner = kotlin_collect_text_keys(ir, key, "key", indent + 4);
            let value_inner = kotlin_collect_text_keys(ir, element, "item", indent + 4);
            if key_inner.is_empty() && value_inner.is_empty() {
                String::new()
            } else {
                format!("{pad}for ((key, item) in {value}) {{\n{key_inner}\n{value_inner}\n{pad}}}")
            }
        }
        TypeIr::Struct(_) => format!("{pad}{value}.collectTextKeys(out)"),
        TypeIr::Union(name) => format!("{pad}{name}.collectTextKeys({value}, out)"),
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
            .map(|field| kotlin_collect_text_keys(ir, &field.ty, value, indent))
            .unwrap_or_default(),
        TypeIr::Bool
        | TypeIr::I8
        | TypeIr::U8
        | TypeIr::I16
        | TypeIr::U16
        | TypeIr::I32
        | TypeIr::U32
        | TypeIr::I64
        | TypeIr::Duration
        | TypeIr::F32
        | TypeIr::F64
        | TypeIr::String
        | TypeIr::Enum(_) => String::new(),
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
#[path = "kotlin/tests.rs"]
mod tests;
