use std::path::Path;

use heck::{ToPascalCase, ToSnakeCase};
use minijinja::context;
use sora_diagnostics::{Result, SoraError};
use sora_ir::model::{ConfigIr, TableModeIr, TypeIr};

use crate::{
    generator::{CodeGenerator, ensure_sora_runtime_format},
    model::{LanguageBackend, TableNameParts, build_model},
    render::{ensure_dir, render_template, write_file},
    types::go_type_name,
};

pub struct GoCodeGenerator;

impl CodeGenerator for GoCodeGenerator {
    fn generate(&self, ir: &ConfigIr, out_dir: &Path) -> Result<()> {
        ensure_sora_runtime_format("go", ir.codegen.go.runtime_format)?;
        ensure_dir(out_dir)?;
        let backend = GoBackend;
        let model = build_model(ir, &backend)?;
        let package = go_package_name(&model.package)?;

        for item in &model.enums {
            let rendered = render_template(
                "go",
                "enum.go.j2",
                context! { package => &package, enum => item },
            )?;
            write_file(
                &out_dir.join(format!("{}.go", item.name.to_snake_case())),
                rendered,
            )?;
        }

        for record in &model.records {
            let rendered = render_template(
                "go",
                "record.go.j2",
                context! { package => &package, record => record },
            )?;
            write_file(&out_dir.join(format!("{}.go", record.snake_name)), rendered)?;
        }

        for union in &model.unions {
            let rendered = render_template(
                "go",
                "union.go.j2",
                context! { package => &package, union => union },
            )?;
            write_file(&out_dir.join(format!("{}.go", union.snake_name)), rendered)?;
        }

        let rendered = render_template("go", "runtime.go.j2", context! { package => &package })?;
        write_file(&out_dir.join("runtime.go"), rendered)?;

        let rendered = render_template(
            "go",
            "config.go.j2",
            context! { package => &package, model => &model },
        )?;
        write_file(&out_dir.join("config.go"), rendered)
    }
}

struct GoBackend;

impl LanguageBackend for GoBackend {
    fn field_name(&self, raw_name: &str) -> String {
        raw_name.to_pascal_case()
    }

    fn type_name(&self, ir: &ConfigIr, ty: &TypeIr) -> String {
        go_type_name(ir, ty)
    }

    fn decode_expr(&self, ir: &ConfigIr, ty: &TypeIr) -> String {
        go_decode_expr(ir, ty)
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
            TableModeIr::List => format!("[]{row_type}"),
            TableModeIr::Map => match key_type {
                Some(key_type) => format!("map[{key_type}]{row_type}"),
                None => format!("[]{row_type}"),
            },
            TableModeIr::Singleton => row_type.to_owned(),
        }
    }
}

fn go_decode_expr(ir: &ConfigIr, ty: &TypeIr) -> String {
    match ty {
        TypeIr::Bool => "reader.ReadBool()".to_owned(),
        TypeIr::I32 => "reader.ReadInt32()".to_owned(),
        TypeIr::I64 => "reader.ReadInt64()".to_owned(),
        TypeIr::F32 => "reader.ReadFloat32()".to_owned(),
        TypeIr::F64 => "reader.ReadFloat64()".to_owned(),
        TypeIr::String => "reader.ReadString()".to_owned(),
        TypeIr::Enum(name) | TypeIr::Struct(name) | TypeIr::Union(name) => {
            format!("decode{name}(reader)")
        }
        TypeIr::List(element) | TypeIr::Array { element, .. } => {
            format!(
                "ReadList(reader, func(reader *SoraReader) ({}, error) {{ return {} }})",
                go_type_name(ir, element),
                go_decode_expr(ir, element)
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
            .map(|field| go_decode_expr(ir, &field.ty))
            .unwrap_or_else(|| "reader.ReadInt32()".to_owned()),
        TypeIr::Optional(element) => {
            format!(
                "ReadOptional(reader, func(reader *SoraReader) ({}, error) {{ return {} }})",
                go_type_name(ir, element),
                go_decode_expr(ir, element)
            )
        }
    }
}

fn go_package_name(package: &str) -> Result<String> {
    let Some(segment) = package.rsplit('.').next() else {
        return Err(SoraError::InvalidSchema(
            "go package must not be empty".to_owned(),
        ));
    };
    let package = segment.to_snake_case();
    if !is_go_package_name(&package) {
        return Err(SoraError::InvalidSchema(format!(
            "go package `{package}` must be a valid identifier"
        )));
    }
    Ok(package)
}

fn is_go_package_name(package: &str) -> bool {
    let mut chars = package.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || first == '_')
        && chars.all(|value| value.is_ascii_alphanumeric() || value == '_')
}
