use std::path::{Path, PathBuf};

use heck::ToLowerCamelCase;
use minijinja::context;
use sora_diagnostics::{Result, SoraError};
use sora_ir::model::{ConfigIr, TableModeIr, TypeIr};

use crate::{
    generator::CodeGenerator,
    model::{LanguageBackend, TableNameParts, build_model},
    render::{ensure_dir, render_template, write_file},
    types::java_type_name,
};

pub struct JavaCodeGenerator;

impl CodeGenerator for JavaCodeGenerator {
    fn generate(&self, ir: &ConfigIr, out_dir: &Path) -> Result<()> {
        ensure_dir(out_dir)?;
        let backend = JavaBackend;
        let model = build_model(ir, &backend)?;
        let package_dir = java_package_dir(out_dir, &model.package)?;

        for item in &model.enums {
            let rendered = render_template(
                "java",
                "enum.java.j2",
                context! { package => &model.package, enum => item },
            )?;
            write_file(&package_dir.join(format!("{}.java", item.name)), rendered)?;
        }

        for record in &model.records {
            let rendered = render_template(
                "java",
                "record.java.j2",
                context! { package => &model.package, record => record },
            )?;
            write_file(
                &package_dir.join(format!("{}.java", record.pascal_name)),
                rendered,
            )?;
        }

        for union in &model.unions {
            let rendered = render_template(
                "java",
                "union.java.j2",
                context! { package => &model.package, union => union },
            )?;
            write_file(
                &package_dir.join(format!("{}.java", union.pascal_name)),
                rendered,
            )?;
        }

        let rendered = render_template(
            "java",
            "runtime.java.j2",
            context! { package => &model.package },
        )?;
        write_file(&package_dir.join("Runtime.java"), rendered)?;

        let rendered = render_template("java", "config.java.j2", context! { model => &model })?;
        write_file(&package_dir.join("SoraConfig.java"), rendered)
    }
}

struct JavaBackend;

impl LanguageBackend for JavaBackend {
    fn field_name(&self, raw_name: &str) -> String {
        raw_name.to_lower_camel_case()
    }

    fn type_name(&self, ir: &ConfigIr, ty: &TypeIr) -> String {
        java_type_name(ir, ty)
    }

    fn decode_expr(&self, ir: &ConfigIr, ty: &TypeIr) -> String {
        java_decode_expr(ir, ty)
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
            TableModeIr::List => format!("java.util.List<{row_type}>"),
            TableModeIr::Map => match key_type {
                Some(key_type) => format!("java.util.Map<{key_type}, {row_type}>"),
                None => format!("java.util.List<{row_type}>"),
            },
            TableModeIr::Singleton => row_type.to_owned(),
        }
    }
}

fn java_decode_expr(ir: &ConfigIr, ty: &TypeIr) -> String {
    match ty {
        TypeIr::Bool => "reader.readBool()".to_owned(),
        TypeIr::I32 => "reader.readI32()".to_owned(),
        TypeIr::I64 => "reader.readI64()".to_owned(),
        TypeIr::F32 => "reader.readF32()".to_owned(),
        TypeIr::F64 => "reader.readF64()".to_owned(),
        TypeIr::String => "reader.readString()".to_owned(),
        TypeIr::Enum(name) | TypeIr::Struct(name) | TypeIr::Union(name) => {
            format!("{name}.decode(reader)")
        }
        TypeIr::List(element) | TypeIr::Array { element, .. } => {
            format!("reader.readList(() -> {})", java_decode_expr(ir, element))
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
            .map(|field| java_decode_expr(ir, &field.ty))
            .unwrap_or_else(|| "reader.readI32()".to_owned()),
        TypeIr::Optional(element) => {
            format!(
                "reader.readOptional(() -> {})",
                java_decode_expr(ir, element)
            )
        }
    }
}

fn java_package_dir(out_dir: &Path, package: &str) -> Result<PathBuf> {
    let mut path = out_dir.to_path_buf();
    for segment in package.split('.') {
        if !is_java_package_segment(segment) {
            return Err(SoraError::InvalidSchema(format!(
                "java package `{package}` must use dot-separated identifier segments"
            )));
        }
        path.push(segment);
    }
    Ok(path)
}

fn is_java_package_segment(segment: &str) -> bool {
    let mut chars = segment.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || first == '_')
        && chars.all(|value| value.is_ascii_alphanumeric() || value == '_')
}
