use std::path::Path;

use heck::ToLowerCamelCase;
use minijinja::context;
use serde::Serialize;
use sora_diagnostics::Result;
use sora_ir::model::{ConfigIr, RuntimeFormatIr, TableModeIr, TypeIr};

use crate::{
    generator::{CodeGenerator, runtime_format_name},
    model::{
        BaseField, BaseIndex, BaseModel, BaseRecord, BaseTable, BaseUnion, BaseUnionVariant,
        build_base_model,
    },
    render::{ensure_dir, render_template, write_file},
    types::csharp_type_name,
};

pub struct CSharpCodeGenerator;

impl CodeGenerator for CSharpCodeGenerator {
    fn generate(&self, ir: &ConfigIr, out_dir: &Path) -> Result<()> {
        ensure_dir(out_dir)?;
        let model = CSharpModel::from_base_model(ir, build_base_model(ir)?);
        let runtime_format = runtime_format_name(ir.codegen.csharp.runtime_format);

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

        if ir.codegen.csharp.runtime_format == RuntimeFormatIr::SoraProtobuf {
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
    enums: Vec<CSharpEnum>,
    unions: Vec<CSharpUnion>,
    records: Vec<CSharpRecord>,
    tables: Vec<CSharpTable>,
    has_unique_indexes: bool,
    has_non_unique_indexes: bool,
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
}

#[derive(Debug, Clone, Serialize)]
struct CSharpRecord {
    pascal_name: String,
    fields: Vec<CSharpField>,
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
    CSharpUnionVariant {
        name: variant.pascal_name,
        fields: variant
            .fields
            .into_iter()
            .map(|field| csharp_field(ir, field))
            .collect(),
    }
}

fn csharp_record(ir: &ConfigIr, record: BaseRecord, table: Option<CSharpTable>) -> CSharpRecord {
    CSharpRecord {
        pascal_name: record.pascal_name,
        fields: record
            .fields
            .into_iter()
            .map(|field| csharp_field(ir, field))
            .collect(),
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
    CSharpField {
        raw_name: field.raw_name,
        name: field.pascal_name,
        type_name: csharp_type_name(ir, &field.ty),
        decode: csharp_decode_expr(ir, &field.ty),
        value_decode,
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
        TypeIr::I32 => "reader.ReadInt32()".to_owned(),
        TypeIr::I64 => "reader.ReadInt64()".to_owned(),
        TypeIr::F32 => "reader.ReadFloat()".to_owned(),
        TypeIr::F64 => "reader.ReadDouble()".to_owned(),
        TypeIr::String => "reader.ReadString()".to_owned(),
        TypeIr::Enum(name) => format!("{name}Codec.Decode(reader)"),
        TypeIr::Struct(name) | TypeIr::Union(name) => format!("{name}.Decode(reader)"),
        TypeIr::List(element) | TypeIr::Array { element, .. } => {
            format!("reader.ReadList(() => {})", csharp_decode_expr(ir, element))
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
        TypeIr::I32 => format!("{value}.AsInt32()"),
        TypeIr::I64 => format!("{value}.AsInt64()"),
        TypeIr::F32 => format!("{value}.AsFloat()"),
        TypeIr::F64 => format!("{value}.AsDouble()"),
        TypeIr::String => format!("{value}.AsString()"),
        TypeIr::Enum(name) => format!("{name}Codec.Decode({value})"),
        TypeIr::Struct(name) | TypeIr::Union(name) => format!("{name}.Decode({value})"),
        TypeIr::List(element) | TypeIr::Array { element, .. } => {
            format!(
                "{value}.AsList(item => {})",
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
