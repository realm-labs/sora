use std::path::Path;

use heck::ToLowerCamelCase;
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
    options::{LanguageCodegenOptions, RuntimeFormat},
    render::{ensure_dir, render_template, write_file},
    types::csharp_type_name,
};

pub struct CSharpCodeGenerator;
crate::impl_test_codegen_generate!(CSharpCodeGenerator, "csharp");

impl CodeGenerator for CSharpCodeGenerator {
    fn generate(&self, context: CodegenContext<'_>, out_dir: &Path) -> Result<()> {
        let ir = context.ir;
        let options = context.options::<LanguageCodegenOptions>()?;
        ensure_dir(out_dir)?;
        let model = CSharpModel::from_base_model(ir, build_base_model(ir)?);
        let runtime_format = runtime_format_name(options.runtime_format);

        for item in &model.enums {
            let rendered = render_template(
                "csharp",
                "enum.cs.j2",
                context! { namespace => &model.package, enum => item, runtime_format => runtime_format },
            )?;
            write_file(&out_dir.join(format!("{}.cs", item.name)), rendered)?;
        }

        for record in &model.records {
            let rendered = render_template(
                "csharp",
                "record.cs.j2",
                context! { namespace => &model.package, record => record, runtime_format => runtime_format },
            )?;
            write_file(
                &out_dir.join(format!("{}.cs", record.pascal_name)),
                rendered,
            )?;
        }

        for union in &model.unions {
            let rendered = render_template(
                "csharp",
                "union.cs.j2",
                context! { namespace => &model.package, union => union, runtime_format => runtime_format },
            )?;
            write_file(&out_dir.join(format!("{}.cs", union.pascal_name)), rendered)?;
        }

        let rendered = render_template(
            "csharp",
            "runtime.cs.j2",
            context! { namespace => &model.package, runtime_format => runtime_format },
        )?;
        write_file(&out_dir.join("Runtime.cs"), rendered)?;

        if options.runtime_format == RuntimeFormat::SoraProtobuf {
            let rendered = render_template("csharp", "protobuf_bundle.cs.j2", context! {})?;
            write_file(&out_dir.join("SoraRuntimeBundle.cs"), rendered)?;
        }

        let rendered = render_template(
            "csharp",
            "config.cs.j2",
            context! { model => &model, runtime_format => runtime_format },
        )?;
        write_file(&out_dir.join("SoraConfig.cs"), rendered)
    }
}

#[derive(Debug, Clone, Serialize)]
struct CSharpModel {
    package: String,
    schema_fingerprint: String,
    enums: Vec<CSharpEnum>,
    unions: Vec<CSharpUnion>,
    records: Vec<CSharpRecord>,
    tables: Vec<CSharpTable>,
    has_unique_indexes: bool,
    has_non_unique_indexes: bool,
    has_localization: bool,
    locales: Vec<String>,
    default_locale: String,
}

#[derive(Debug, Clone, Serialize)]
struct CSharpEnum {
    name: String,
    values: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct CSharpUnion {
    pascal_name: String,
    tag: String,
    variants: Vec<CSharpUnionVariant>,
}

#[derive(Debug, Clone, Serialize)]
struct CSharpUnionVariant {
    name: String,
    fields: Vec<CSharpField>,
    has_text_keys: bool,
}

#[derive(Debug, Clone, Serialize)]
struct CSharpRecord {
    pascal_name: String,
    fields: Vec<CSharpField>,
    has_text_keys: bool,
    table: Option<CSharpTable>,
}

#[derive(Debug, Clone, Serialize)]
struct CSharpTable {
    name: String,
    pascal_name: String,
    mode: String,
    container_type: String,
    row_type: String,
    key_name: Option<String>,
    key_field_name: Option<String>,
    key_type: Option<String>,
    unique_indexes: Vec<CSharpIndex>,
    non_unique_indexes: Vec<CSharpIndex>,
}

#[derive(Debug, Clone, Serialize)]
struct CSharpIndex {
    pascal_name: String,
    camel_name: String,
    field_name: String,
    param_camel_name: String,
    key_type: String,
}

#[derive(Debug, Clone, Serialize)]
struct CSharpField {
    raw_name: String,
    name: String,
    type_name: String,
    decode: String,
    value_decode: String,
    collect_text_keys: String,
    comment: Option<String>,
}

impl CSharpModel {
    fn from_base_model(ir: &ConfigIr, model: BaseModel) -> Self {
        let tables = model
            .tables
            .into_iter()
            .map(|table| csharp_table(ir, table))
            .collect::<Vec<_>>();
        Self {
            package: model.package,
            schema_fingerprint: model.schema_fingerprint,
            enums: model
                .enums
                .into_iter()
                .map(|item| CSharpEnum {
                    name: item.pascal_name,
                    values: item.values,
                })
                .collect(),
            unions: model
                .unions
                .into_iter()
                .map(|item| csharp_union(ir, item))
                .collect(),
            records: model
                .records
                .into_iter()
                .map(|item| {
                    let table = tables
                        .iter()
                        .find(|table| table.row_type == item.pascal_name)
                        .cloned();
                    csharp_record(ir, item, table)
                })
                .collect(),
            has_unique_indexes: tables.iter().any(|table| !table.unique_indexes.is_empty()),
            has_non_unique_indexes: tables
                .iter()
                .any(|table| !table.non_unique_indexes.is_empty()),
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

fn csharp_union(ir: &ConfigIr, union: BaseUnion) -> CSharpUnion {
    CSharpUnion {
        pascal_name: union.pascal_name,
        tag: union.tag,
        variants: union
            .variants
            .into_iter()
            .map(|variant| csharp_variant(ir, variant))
            .collect(),
    }
}

fn csharp_variant(ir: &ConfigIr, variant: BaseUnionVariant) -> CSharpUnionVariant {
    let fields = variant
        .fields
        .into_iter()
        .map(|field| csharp_field(ir, field))
        .collect::<Vec<_>>();
    let has_text_keys = fields
        .iter()
        .any(|field| !field.collect_text_keys.is_empty());
    CSharpUnionVariant {
        name: variant.pascal_name,
        fields,
        has_text_keys,
    }
}

fn csharp_record(ir: &ConfigIr, record: BaseRecord, table: Option<CSharpTable>) -> CSharpRecord {
    let fields = record
        .fields
        .into_iter()
        .map(|field| csharp_field(ir, field))
        .collect::<Vec<_>>();
    let has_text_keys = fields
        .iter()
        .any(|field| !field.collect_text_keys.is_empty());
    CSharpRecord {
        pascal_name: record.pascal_name,
        fields,
        has_text_keys,
        table,
    }
}

fn csharp_table(ir: &ConfigIr, table: BaseTable) -> CSharpTable {
    let row_type = table.pascal_name.clone();
    let key_type = table
        .key_field
        .as_ref()
        .map(|field| csharp_type_name(ir, &field.ty));
    let container_type = csharp_container_type(table.mode, &row_type, key_type.as_deref());
    let key_field_name = table
        .key_field
        .as_ref()
        .map(|field| field.pascal_name.clone());

    CSharpTable {
        name: table.name,
        pascal_name: table.pascal_name,
        mode: table.mode_name,
        container_type,
        row_type,
        key_name: table.key_name,
        key_field_name,
        key_type,
        unique_indexes: table
            .unique_indexes
            .into_iter()
            .map(|index| csharp_index(ir, index))
            .collect(),
        non_unique_indexes: table
            .non_unique_indexes
            .into_iter()
            .map(|index| csharp_index(ir, index))
            .collect(),
    }
}

fn csharp_index(ir: &ConfigIr, index: BaseIndex) -> CSharpIndex {
    CSharpIndex {
        pascal_name: index.pascal_name,
        camel_name: index.name.to_lower_camel_case(),
        field_name: index.field.pascal_name.clone(),
        param_camel_name: index.field.camel_name,
        key_type: csharp_type_name(ir, &index.field.ty),
    }
}

fn csharp_field(ir: &ConfigIr, field: BaseField) -> CSharpField {
    let value_decode = csharp_value_decode_expr(ir, &field.ty, "__VALUE__");
    let collect_text_keys =
        csharp_collect_text_keys(ir, &field.ty, &format!("this.{}", field.pascal_name), 8);
    CSharpField {
        raw_name: field.raw_name,
        name: field.pascal_name,
        type_name: csharp_type_name(ir, &field.ty),
        decode: csharp_decode_expr(ir, &field.ty),
        value_decode,
        collect_text_keys,
        comment: field.comment,
    }
}

fn csharp_container_type(mode: TableModeIr, row_type: &str, key_type: Option<&str>) -> String {
    match mode {
        TableModeIr::List => format!("List<{row_type}>"),
        TableModeIr::Map => match key_type {
            Some(key_type) => format!("Dictionary<{key_type}, {row_type}>"),
            None => format!("List<{row_type}>"),
        },
        TableModeIr::Singleton => row_type.to_owned(),
    }
}

fn csharp_decode_expr(ir: &ConfigIr, ty: &TypeIr) -> String {
    match ty {
        TypeIr::Bool => "reader.ReadBool()".to_owned(),
        TypeIr::I8 => "(sbyte)reader.ReadInt32()".to_owned(),
        TypeIr::U8 => "(byte)reader.ReadUInt32()".to_owned(),
        TypeIr::I16 => "(short)reader.ReadInt32()".to_owned(),
        TypeIr::U16 => "(ushort)reader.ReadUInt32()".to_owned(),
        TypeIr::I32 => "reader.ReadInt32()".to_owned(),
        TypeIr::U32 => "(uint)reader.ReadUInt32()".to_owned(),
        TypeIr::I64 => "reader.ReadInt64()".to_owned(),
        TypeIr::F32 => "reader.ReadFloat()".to_owned(),
        TypeIr::F64 => "reader.ReadDouble()".to_owned(),
        TypeIr::String => "reader.ReadString()".to_owned(),
        TypeIr::Text => "new TextKey(reader.ReadString())".to_owned(),
        TypeIr::Enum(name) => format!("{name}Codec.Decode(reader)"),
        TypeIr::Struct(name) | TypeIr::Union(name) => format!("{name}.Decode(reader)"),
        TypeIr::List(element) | TypeIr::Set(element) | TypeIr::Array { element, .. } => {
            format!("reader.ReadList(() => {})", csharp_decode_expr(ir, element))
        }
        TypeIr::Map { key, value } => {
            format!(
                "reader.ReadMap(() => {}, () => {})",
                csharp_decode_expr(ir, key),
                csharp_decode_expr(ir, value)
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
            .map(|field| csharp_decode_expr(ir, &field.ty))
            .unwrap_or_else(|| "reader.ReadInt32()".to_owned()),
        TypeIr::Optional(element) => {
            format!(
                "reader.ReadOptional(() => {})",
                csharp_decode_expr(ir, element)
            )
        }
    }
}

fn csharp_value_decode_expr(ir: &ConfigIr, ty: &TypeIr, value: &str) -> String {
    match ty {
        TypeIr::Bool => format!("{value}.AsBool()"),
        TypeIr::I8 => format!("(sbyte){value}.AsInt32()"),
        TypeIr::U8 => format!("(byte){value}.AsInt32()"),
        TypeIr::I16 => format!("(short){value}.AsInt32()"),
        TypeIr::U16 => format!("(ushort){value}.AsInt32()"),
        TypeIr::I32 => format!("{value}.AsInt32()"),
        TypeIr::U32 => format!("(uint){value}.AsInt64()"),
        TypeIr::I64 => format!("{value}.AsInt64()"),
        TypeIr::F32 => format!("{value}.AsFloat()"),
        TypeIr::F64 => format!("{value}.AsDouble()"),
        TypeIr::String => format!("{value}.AsString()"),
        TypeIr::Text => format!("new TextKey({value}.AsString())"),
        TypeIr::Enum(name) => format!("{name}Codec.Decode({value})"),
        TypeIr::Struct(name) | TypeIr::Union(name) => format!("{name}.Decode({value})"),
        TypeIr::List(element) | TypeIr::Set(element) | TypeIr::Array { element, .. } => {
            format!(
                "{value}.AsList(item => {})",
                csharp_value_decode_expr(ir, element, "item")
            )
        }
        TypeIr::Map {
            key,
            value: element,
        } => {
            format!(
                "{value}.AsMap(item => {}, item => {})",
                csharp_value_decode_expr(ir, key, "item"),
                csharp_value_decode_expr(ir, element, "item")
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
            .map(|field| csharp_value_decode_expr(ir, &field.ty, value))
            .unwrap_or_else(|| format!("{value}.AsInt32()")),
        TypeIr::Optional(element) => {
            format!(
                "{value}.IsNull ? default : {}",
                csharp_value_decode_expr(ir, element, value)
            )
        }
    }
}

fn csharp_collect_text_keys(ir: &ConfigIr, ty: &TypeIr, value: &str, indent: usize) -> String {
    let pad = " ".repeat(indent);
    match ty {
        TypeIr::Text => format!("{pad}keys.Add({value});"),
        TypeIr::Optional(element) => {
            let inner = csharp_collect_text_keys(ir, element, "optionalValue", indent + 4);
            if inner.is_empty() {
                String::new()
            } else {
                format!("{pad}if ({value} is {{ }} optionalValue)\n{pad}{{\n{inner}\n{pad}}}")
            }
        }
        TypeIr::List(element) | TypeIr::Set(element) | TypeIr::Array { element, .. } => {
            let inner = csharp_collect_text_keys(ir, element, "element", indent + 4);
            if inner.is_empty() {
                String::new()
            } else {
                format!("{pad}foreach (var element in {value})\n{pad}{{\n{inner}\n{pad}}}")
            }
        }
        TypeIr::Map {
            key,
            value: element,
        } => {
            let key_inner = csharp_collect_text_keys(ir, key, "entryKey", indent + 4);
            let value_inner = csharp_collect_text_keys(ir, element, "entryValue", indent + 4);
            if key_inner.is_empty() && value_inner.is_empty() {
                String::new()
            } else {
                format!(
                    "{pad}foreach (var entry in {value})\n{pad}{{\n{pad}    var entryKey = entry.Key;\n{pad}    var entryValue = entry.Value;\n{key_inner}\n{value_inner}\n{pad}}}"
                )
            }
        }
        TypeIr::Struct(_) => format!("{pad}{value}.CollectTextKeys(keys);"),
        TypeIr::Union(name) => format!("{pad}{name}.CollectTextKeys({value}, keys);"),
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
            .map(|field| csharp_collect_text_keys(ir, &field.ty, value, indent))
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
