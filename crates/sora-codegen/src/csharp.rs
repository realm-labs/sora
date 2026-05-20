use std::path::Path;

use heck::ToPascalCase;
use minijinja::context;
use sora_diagnostics::Result;
use sora_ir::model::{ConfigIr, TableModeIr, TypeIr};

use crate::{
    generator::{CodeGenerator, runtime_format_name},
    model::{LanguageBackend, TableNameParts, build_model},
    render::{ensure_dir, render_template, write_file},
    types::csharp_type_name,
};

pub struct CSharpCodeGenerator;

impl CodeGenerator for CSharpCodeGenerator {
    fn generate(&self, ir: &ConfigIr, out_dir: &Path) -> Result<()> {
        ensure_dir(out_dir)?;
        let backend = CSharpBackend;
        let model = build_model(ir, &backend)?;
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

        let rendered = render_template(
            "csharp",
            "config.cs.j2",
            context! { model => &model, runtime_format => runtime_format },
        )?;
        write_file(&out_dir.join("SoraConfig.cs"), rendered)
    }
}

struct CSharpBackend;

impl LanguageBackend for CSharpBackend {
    fn field_name(&self, raw_name: &str) -> String {
        raw_name.to_pascal_case()
    }

    fn type_name(&self, ir: &ConfigIr, ty: &TypeIr) -> String {
        csharp_type_name(ir, ty)
    }

    fn decode_expr(&self, ir: &ConfigIr, ty: &TypeIr) -> String {
        csharp_decode_expr(ir, ty)
    }

    fn value_decode_expr(&self, ir: &ConfigIr, ty: &TypeIr) -> String {
        csharp_value_decode_expr(ir, ty, "__VALUE__")
    }

    fn row_type(&self, table: &TableNameParts<'_>) -> String {
        table.pascal_name.to_owned()
    }

    fn container_type(
        &self,
        _table: &TableNameParts<'_>,
        mode: TableModeIr,
        row_type: &str,
        key_type: Option<&str>,
    ) -> String {
        match mode {
            TableModeIr::List => format!("List<{row_type}>"),
            TableModeIr::Map => match key_type {
                Some(key_type) => format!("Dictionary<{key_type}, {row_type}>"),
                None => format!("List<{row_type}>"),
            },
            TableModeIr::Singleton => row_type.to_owned(),
        }
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
