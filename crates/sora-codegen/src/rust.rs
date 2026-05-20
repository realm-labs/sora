use std::path::Path;

use heck::ToSnakeCase;
use minijinja::context;
use serde::Serialize;
use sora_diagnostics::Result;
use sora_ir::model::{ConfigIr, TableModeIr, TypeIr};

use crate::{
    generator::{CodeGenerator, runtime_format_name},
    model::{
        BaseField, BaseImport, BaseIndex, BaseModel, BaseRecord, BaseTable, BaseUnion,
        BaseUnionVariant, build_base_model,
    },
    render::{ensure_dir, render_template, write_file},
    types::rust_type_name,
};

pub struct RustCodeGenerator;

impl CodeGenerator for RustCodeGenerator {
    fn generate(&self, ir: &ConfigIr, out_dir: &Path) -> Result<()> {
        ensure_dir(out_dir)?;
        let model = RustModel::from_base_model(ir, build_base_model(ir)?);
        let runtime_format = runtime_format_name(ir.codegen.rust.runtime_format);

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
            context! { model => &model, rust_map_type => rust_map_type, runtime_format => runtime_format },
        )?;
        write_file(&out_dir.join("mod.rs"), rendered)?;

        let rendered = render_template(
            "rust",
            "runtime.rs.j2",
            context! { runtime_format => runtime_format },
        )?;
        write_file(&out_dir.join("runtime.rs"), rendered)
    }
}

#[derive(Debug, Clone, Serialize)]
struct RustModel {
    package: String,
    enums: Vec<RustEnum>,
    unions: Vec<RustUnion>,
    records: Vec<RustRecord>,
    tables: Vec<RustTable>,
    modules: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct RustEnum {
    name: String,
    snake_name: String,
    values: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct RustUnion {
    pascal_name: String,
    snake_name: String,
    tag: String,
    variants: Vec<RustUnionVariant>,
    imports: Vec<RustImport>,
}

#[derive(Debug, Clone, Serialize)]
struct RustUnionVariant {
    name: String,
    fields: Vec<RustField>,
}

#[derive(Debug, Clone, Serialize)]
struct RustRecord {
    pascal_name: String,
    snake_name: String,
    imports: Vec<RustImport>,
    fields: Vec<RustField>,
}

#[derive(Debug, Clone, Serialize)]
struct RustImport {
    module: String,
    name: String,
}

#[derive(Debug, Clone, Serialize)]
struct RustTable {
    name: String,
    pascal_name: String,
    snake_name: String,
    mode: String,
    container_type: String,
    row_type: String,
    key_name: Option<String>,
    key_field_name: Option<String>,
    key_type: Option<String>,
    key_is_copy: bool,
    unique_indexes: Vec<RustIndex>,
    non_unique_indexes: Vec<RustIndex>,
}

#[derive(Debug, Clone, Serialize)]
struct RustIndex {
    name: String,
    method_name: String,
    field_name: String,
    param_name: String,
    param_type: String,
    key_type: String,
    key_is_copy: bool,
}

#[derive(Debug, Clone, Serialize)]
struct RustField {
    raw_name: String,
    name: String,
    type_name: String,
    comment: Option<String>,
}

impl RustModel {
    fn from_base_model(ir: &ConfigIr, model: BaseModel) -> Self {
        Self {
            package: model.package,
            enums: model
                .enums
                .into_iter()
                .map(|item| RustEnum {
                    name: item.pascal_name,
                    snake_name: item.snake_name,
                    values: item.values,
                })
                .collect(),
            unions: model
                .unions
                .into_iter()
                .map(|item| rust_union(ir, item))
                .collect(),
            records: model
                .records
                .into_iter()
                .map(|item| rust_record(ir, item))
                .collect(),
            tables: model
                .tables
                .into_iter()
                .map(|item| rust_table(ir, item))
                .collect(),
            modules: model.modules,
        }
    }
}

fn rust_union(ir: &ConfigIr, union: BaseUnion) -> RustUnion {
    RustUnion {
        pascal_name: union.pascal_name,
        snake_name: union.snake_name,
        tag: union.tag,
        variants: union
            .variants
            .into_iter()
            .map(|variant| rust_variant(ir, variant))
            .collect(),
        imports: union.imports.into_iter().map(rust_import).collect(),
    }
}

fn rust_variant(ir: &ConfigIr, variant: BaseUnionVariant) -> RustUnionVariant {
    RustUnionVariant {
        name: variant.pascal_name,
        fields: variant
            .fields
            .into_iter()
            .map(|field| rust_field(ir, field))
            .collect(),
    }
}

fn rust_record(ir: &ConfigIr, record: BaseRecord) -> RustRecord {
    RustRecord {
        pascal_name: record.pascal_name,
        snake_name: record.snake_name,
        imports: record.imports.into_iter().map(rust_import).collect(),
        fields: record
            .fields
            .into_iter()
            .map(|field| rust_field(ir, field))
            .collect(),
    }
}

fn rust_table(ir: &ConfigIr, table: BaseTable) -> RustTable {
    let row_type = format!("{}::{}", table.snake_name, table.pascal_name);
    let key_type = table
        .key_field
        .as_ref()
        .map(|field| rust_table_key_type(ir, &field.ty));
    let container_type = rust_container_type(table.mode, &row_type, key_type.as_deref());
    let key_field_name = table
        .key_field
        .as_ref()
        .map(|field| field.snake_name.clone());
    let key_is_copy = table
        .key_field
        .as_ref()
        .is_some_and(|field| rust_key_type_is_copy(ir, &field.ty));

    RustTable {
        name: table.name,
        pascal_name: table.pascal_name,
        snake_name: table.snake_name,
        mode: table.mode_name,
        container_type,
        row_type,
        key_name: table.key_name,
        key_field_name,
        key_type,
        key_is_copy,
        unique_indexes: table
            .unique_indexes
            .into_iter()
            .map(|index| rust_index(ir, index))
            .collect(),
        non_unique_indexes: table
            .non_unique_indexes
            .into_iter()
            .map(|index| rust_index(ir, index))
            .collect(),
    }
}

fn rust_index(ir: &ConfigIr, index: BaseIndex) -> RustIndex {
    RustIndex {
        name: index.snake_name,
        method_name: index.method_name,
        field_name: index.field.snake_name.clone(),
        param_name: index.field.snake_name.clone(),
        param_type: rust_key_param_type(ir, &index.field.ty),
        key_type: rust_table_key_type(ir, &index.field.ty),
        key_is_copy: rust_key_type_is_copy(ir, &index.field.ty),
    }
}

fn rust_field(ir: &ConfigIr, field: BaseField) -> RustField {
    RustField {
        raw_name: field.raw_name,
        name: field.snake_name,
        type_name: rust_type_name(ir, &field.ty),
        comment: field.comment,
    }
}

fn rust_import(import: BaseImport) -> RustImport {
    RustImport {
        module: import.module,
        name: import.name,
    }
}

fn rust_container_type(mode: TableModeIr, row_type: &str, key_type: Option<&str>) -> String {
    match mode {
        TableModeIr::List => format!("Vec<{row_type}>"),
        TableModeIr::Map => match key_type {
            Some(key_type) => format!("SoraMap<{key_type}, {row_type}>"),
            None => format!("Vec<{row_type}>"),
        },
        TableModeIr::Singleton => row_type.to_owned(),
    }
}

fn rust_key_param_type(ir: &ConfigIr, ty: &TypeIr) -> String {
    let type_name = rust_type_name(ir, ty);
    if type_name == "String" {
        "str".to_owned()
    } else {
        type_name
    }
}

fn rust_table_key_type(ir: &ConfigIr, ty: &TypeIr) -> String {
    match ty {
        TypeIr::Enum(name) | TypeIr::Struct(name) | TypeIr::Union(name) => {
            format!("{}::{}", name.to_snake_case(), rust_type_name(ir, ty))
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
            .map(|field| rust_table_key_type(ir, &field.ty))
            .unwrap_or_else(|| rust_type_name(ir, ty)),
        _ => rust_type_name(ir, ty),
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
