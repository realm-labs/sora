use std::path::Path;

use heck::ToSnakeCase;
use minijinja::context;
use sora_diagnostics::Result;
use sora_ir::model::{ConfigIr, TableModeIr, TypeIr};

use crate::{
    generator::CodeGenerator,
    model::{LanguageBackend, TableNameParts, build_model},
    render::{ensure_dir, render_template, write_file},
    types::rust_type_name,
};

pub struct RustCodeGenerator;

impl CodeGenerator for RustCodeGenerator {
    fn generate(&self, ir: &ConfigIr, out_dir: &Path) -> Result<()> {
        ensure_dir(out_dir)?;
        let backend = RustBackend;
        let model = build_model(ir, &backend)?;

        for item in &model.enums {
            let rendered = render_template("rust", "enum.rs.j2", context! { enum => item })?;
            write_file(
                &out_dir.join(format!("{}.rs", item.name.to_snake_case())),
                rendered,
            )?;
        }

        for record in &model.records {
            let rendered = render_template("rust", "struct.rs.j2", context! { record => record })?;
            write_file(&out_dir.join(format!("{}.rs", record.snake_name)), rendered)?;
        }

        for union in &model.unions {
            let rendered = render_template("rust", "union.rs.j2", context! { union => union })?;
            write_file(&out_dir.join(format!("{}.rs", union.snake_name)), rendered)?;
        }

        let rust_map_type = match ir.codegen.rust.map_type {
            sora_ir::model::RustMapTypeIr::Std => "std",
            sora_ir::model::RustMapTypeIr::FxHashMap => "fx_hash_map",
        };
        let rendered = render_template(
            "rust",
            "mod.rs.j2",
            context! { model => &model, rust_map_type => rust_map_type },
        )?;
        write_file(&out_dir.join("mod.rs"), rendered)?;

        let rendered = render_template("rust", "runtime.rs.j2", context! {})?;
        write_file(&out_dir.join("runtime.rs"), rendered)
    }
}

struct RustBackend;

impl LanguageBackend for RustBackend {
    fn field_name(&self, raw_name: &str) -> String {
        raw_name.to_snake_case()
    }

    fn type_name(&self, ir: &ConfigIr, ty: &TypeIr) -> String {
        rust_type_name(ir, ty)
    }

    fn decode_expr(&self, _ir: &ConfigIr, _ty: &TypeIr) -> String {
        String::new()
    }

    fn row_type(&self, table: &TableNameParts<'_>) -> String {
        format!("{}::{}", table.snake_name, table.pascal_name)
    }

    fn container_type(
        &self,
        _table: &TableNameParts<'_>,
        mode: TableModeIr,
        row_type: &str,
        key_type: Option<&str>,
    ) -> String {
        match mode {
            TableModeIr::List => format!("Vec<{row_type}>"),
            TableModeIr::Map => match key_type {
                Some(key_type) => format!("SoraMap<{key_type}, {row_type}>"),
                None => format!("Vec<{row_type}>"),
            },
            TableModeIr::Singleton => row_type.to_owned(),
        }
    }

    fn key_is_copy(&self, ir: &ConfigIr, ty: &TypeIr) -> bool {
        rust_key_type_is_copy(ir, ty)
    }

    fn key_param_type(&self, ir: &ConfigIr, ty: &TypeIr) -> String {
        let type_name = rust_type_name(ir, ty);
        if type_name == "String" {
            "str".to_owned()
        } else {
            type_name
        }
    }
}

fn rust_key_type_is_copy(ir: &ConfigIr, ty: &TypeIr) -> bool {
    match ty {
        TypeIr::Bool | TypeIr::I32 | TypeIr::I64 | TypeIr::F32 | TypeIr::F64 | TypeIr::Enum(_) => {
            true
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
            .is_some_and(|field| rust_key_type_is_copy(ir, &field.ty)),
        TypeIr::Optional(element) => rust_key_type_is_copy(ir, element),
        TypeIr::String
        | TypeIr::Struct(_)
        | TypeIr::Union(_)
        | TypeIr::List(_)
        | TypeIr::Array { .. } => false,
    }
}
