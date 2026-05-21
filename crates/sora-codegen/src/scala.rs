use std::path::{Path, PathBuf};

use minijinja::context;
use serde::Serialize;
use sora_diagnostics::{Result, SoraError};
use sora_ir::model::{ConfigIr, ScalaVersionIr, TableModeIr, TypeIr};

use crate::{
    generator::{CodeGenerator, ensure_sora_runtime_format},
    model::{
        BaseField, BaseIndex, BaseModel, BaseRecord, BaseTable, BaseUnion, BaseUnionVariant,
        build_base_model,
    },
    render::{ensure_dir, render_template, write_file},
    types::scala_type_name,
};

pub struct ScalaCodeGenerator;

impl CodeGenerator for ScalaCodeGenerator {
    fn generate(&self, ir: &ConfigIr, out_dir: &Path) -> Result<()> {
        ensure_sora_runtime_format("scala", ir.codegen.scala.runtime_format)?;
        ensure_dir(out_dir)?;

        let model = ScalaModel::from_base_model(ir, build_base_model(ir)?);
        let package_dir = scala_package_dir(out_dir, &model.package)?;

        for item in &model.enums {
            let rendered = render_template(
                "scala",
                "enum.scala.j2",
                context! { package => &model.package, enum => item },
            )?;
            write_file(&package_dir.join(format!("{}.scala", item.name)), rendered)?;
        }

        for record in &model.records {
            let rendered = render_template(
                "scala",
                "record.scala.j2",
                context! { package => &model.package, record => record },
            )?;
            write_file(
                &package_dir.join(format!("{}.scala", record.pascal_name)),
                rendered,
            )?;
        }

        for union in &model.unions {
            let rendered = render_template(
                "scala",
                "union.scala.j2",
                context! { package => &model.package, union => union },
            )?;
            write_file(
                &package_dir.join(format!("{}.scala", union.pascal_name)),
                rendered,
            )?;
        }

        let rendered = render_template(
            "scala",
            "runtime.scala.j2",
            context! { package => &model.package },
        )?;
        write_file(&package_dir.join("SoraRuntime.scala"), rendered)?;

        let rendered = render_template("scala", "config.scala.j2", context! { model => &model })?;
        write_file(&package_dir.join("SoraConfig.scala"), rendered)
    }
}

#[derive(Debug, Clone, Serialize)]
struct ScalaModel {
    package: String,
    enums: Vec<ScalaEnum>,
    unions: Vec<ScalaUnion>,
    records: Vec<ScalaRecord>,
    tables: Vec<ScalaTable>,
}

#[derive(Debug, Clone, Serialize)]
struct ScalaEnum {
    name: String,
    values: Vec<String>,
    is_scala3: bool,
}

#[derive(Debug, Clone, Serialize)]
struct ScalaUnion {
    pascal_name: String,
    tag: String,
    variants: Vec<ScalaUnionVariant>,
}

#[derive(Debug, Clone, Serialize)]
struct ScalaUnionVariant {
    name: String,
    fields: Vec<ScalaField>,
}

#[derive(Debug, Clone, Serialize)]
struct ScalaRecord {
    pascal_name: String,
    fields: Vec<ScalaField>,
    table: Option<ScalaTable>,
}

#[derive(Debug, Clone, Serialize)]
struct ScalaTable {
    name: String,
    pascal_name: String,
    camel_name: String,
    mode: String,
    container_type: String,
    row_type: String,
    key_name: Option<String>,
    key_field_name: Option<String>,
    key_type: Option<String>,
    unique_indexes: Vec<ScalaIndex>,
    non_unique_indexes: Vec<ScalaIndex>,
}

#[derive(Debug, Clone, Serialize)]
struct ScalaIndex {
    pascal_name: String,
    field_name: String,
    param_camel_name: String,
    key_type: String,
}

#[derive(Debug, Clone, Serialize)]
struct ScalaField {
    raw_name: String,
    name: String,
    type_name: String,
    decode: String,
    comment: Option<String>,
}

impl ScalaModel {
    fn from_base_model(ir: &ConfigIr, model: BaseModel) -> Self {
        let is_scala3 = ir.codegen.scala.scala_version == ScalaVersionIr::Scala3;
        let tables = model
            .tables
            .into_iter()
            .map(|item| scala_table(ir, item))
            .collect::<Vec<_>>();

        Self {
            package: model.package,
            enums: model
                .enums
                .into_iter()
                .map(|item| ScalaEnum {
                    name: item.pascal_name,
                    values: item.values,
                    is_scala3,
                })
                .collect(),
            unions: model
                .unions
                .into_iter()
                .map(|item| scala_union(ir, item))
                .collect(),
            records: model
                .records
                .into_iter()
                .map(|item| {
                    let table = tables
                        .iter()
                        .find(|table| table.row_type == item.pascal_name)
                        .cloned();
                    scala_record(ir, item, table)
                })
                .collect(),
            tables,
        }
    }
}

fn scala_union(ir: &ConfigIr, union: BaseUnion) -> ScalaUnion {
    ScalaUnion {
        pascal_name: union.pascal_name,
        tag: union.tag,
        variants: union
            .variants
            .into_iter()
            .map(|variant| scala_variant(ir, variant))
            .collect(),
    }
}

fn scala_variant(ir: &ConfigIr, variant: BaseUnionVariant) -> ScalaUnionVariant {
    ScalaUnionVariant {
        name: variant.pascal_name,
        fields: variant
            .fields
            .into_iter()
            .map(|field| scala_field(ir, field))
            .collect(),
    }
}

fn scala_record(ir: &ConfigIr, record: BaseRecord, table: Option<ScalaTable>) -> ScalaRecord {
    ScalaRecord {
        pascal_name: record.pascal_name,
        fields: record
            .fields
            .into_iter()
            .map(|field| scala_field(ir, field))
            .collect(),
        table,
    }
}

fn scala_table(ir: &ConfigIr, table: BaseTable) -> ScalaTable {
    let row_type = table.pascal_name.clone();
    let key_type = table
        .key_field
        .as_ref()
        .map(|field| scala_type_name(ir, &field.ty));
    let container_type = scala_container_type(table.mode, &row_type, key_type.as_deref());
    let key_field_name = table
        .key_field
        .as_ref()
        .map(|field| field.camel_name.clone());

    ScalaTable {
        name: table.name,
        pascal_name: table.pascal_name,
        camel_name: table.camel_name,
        mode: table.mode_name,
        container_type,
        row_type,
        key_name: table.key_name,
        key_field_name,
        key_type,
        unique_indexes: table
            .unique_indexes
            .into_iter()
            .map(|index| scala_index(ir, index))
            .collect(),
        non_unique_indexes: table
            .non_unique_indexes
            .into_iter()
            .map(|index| scala_index(ir, index))
            .collect(),
    }
}

fn scala_index(ir: &ConfigIr, index: BaseIndex) -> ScalaIndex {
    ScalaIndex {
        pascal_name: index.pascal_name,
        field_name: index.field.camel_name.clone(),
        param_camel_name: index.field.camel_name,
        key_type: scala_type_name(ir, &index.field.ty),
    }
}

fn scala_field(ir: &ConfigIr, field: BaseField) -> ScalaField {
    ScalaField {
        raw_name: field.raw_name,
        name: field.camel_name,
        type_name: scala_type_name(ir, &field.ty),
        decode: scala_decode_expr(ir, &field.ty),
        comment: field.comment,
    }
}

fn scala_container_type(mode: TableModeIr, row_type: &str, key_type: Option<&str>) -> String {
    match mode {
        TableModeIr::List => format!("Vector[{row_type}]"),
        TableModeIr::Map => match key_type {
            Some(key_type) => format!("Map[{key_type}, {row_type}]"),
            None => format!("Vector[{row_type}]"),
        },
        TableModeIr::Singleton => row_type.to_owned(),
    }
}

fn scala_decode_expr(ir: &ConfigIr, ty: &TypeIr) -> String {
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
            format!("reader.readList({})", scala_decode_expr(ir, element))
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
            .map(|field| scala_decode_expr(ir, &field.ty))
            .unwrap_or_else(|| "reader.readI32()".to_owned()),
        TypeIr::Optional(element) => {
            format!("reader.readOptional({})", scala_decode_expr(ir, element))
        }
    }
}

fn scala_package_dir(out_dir: &Path, package: &str) -> Result<PathBuf> {
    let mut path = out_dir.to_path_buf();
    for segment in package.split('.') {
        if !is_scala_package_segment(segment) {
            return Err(SoraError::InvalidSchema(format!(
                "scala package `{package}` must use dot-separated identifier segments"
            )));
        }
        path.push(segment);
    }
    Ok(path)
}

fn is_scala_package_segment(segment: &str) -> bool {
    let mut chars = segment.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || first == '_')
        && chars.all(|value| value.is_ascii_alphanumeric() || value == '_')
}
