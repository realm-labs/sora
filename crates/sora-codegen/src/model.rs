use heck::{ToLowerCamelCase, ToPascalCase, ToSnakeCase};
use serde::Serialize;
use sora_diagnostics::Result;
use sora_ir::model::{ConfigIr, FieldIr, TableIr};

use crate::types::{kotlin_type_name, rust_type_name};

#[derive(Debug, Clone, Serialize)]
pub struct CodegenModel {
    pub package: String,
    pub enums: Vec<CodegenEnum>,
    pub records: Vec<CodegenRecord>,
    pub modules: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CodegenEnum {
    pub name: String,
    pub values: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CodegenRecord {
    pub name: String,
    pub pascal_name: String,
    pub snake_name: String,
    pub camel_name: String,
    pub fields: Vec<CodegenField>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CodegenField {
    pub raw_name: String,
    pub rust_name: String,
    pub rust_type: String,
    pub kotlin_name: String,
    pub kotlin_type: String,
    pub comment: Option<String>,
}

pub fn build_model(ir: &ConfigIr) -> Result<CodegenModel> {
    let enums = ir
        .enums
        .iter()
        .map(|item| CodegenEnum {
            name: item.name.clone(),
            values: item.values.clone(),
        })
        .collect::<Vec<_>>();

    let records = ir
        .structs
        .iter()
        .map(|item| {
            build_record(
                ir,
                &TableLike {
                    name: &item.name,
                    fields: &item.fields,
                },
            )
        })
        .chain(ir.tables.iter().map(|item| build_record(ir, &item.into())))
        .collect::<Result<Vec<_>>>()?;

    let modules = enums
        .iter()
        .map(|item| item.name.to_snake_case())
        .chain(records.iter().map(|item| item.snake_name.clone()))
        .collect();

    Ok(CodegenModel {
        package: ir.package.clone(),
        enums,
        records,
        modules,
    })
}

fn build_record(ir: &ConfigIr, item: &TableLike<'_>) -> Result<CodegenRecord> {
    let fields = item
        .fields
        .iter()
        .map(|field| {
            Ok(CodegenField {
                raw_name: field.name.clone(),
                rust_name: field.name.to_snake_case(),
                rust_type: rust_type_name(ir, &field.ty),
                kotlin_name: field.name.to_lower_camel_case(),
                kotlin_type: kotlin_type_name(ir, &field.ty),
                comment: field.comment.clone(),
            })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(CodegenRecord {
        name: item.name.to_owned(),
        pascal_name: item.name.to_pascal_case(),
        snake_name: item.name.to_snake_case(),
        camel_name: item.name.to_lower_camel_case(),
        fields,
    })
}

struct TableLike<'a> {
    name: &'a str,
    fields: &'a [FieldIr],
}

impl<'a> From<&'a TableIr> for TableLike<'a> {
    fn from(table: &'a TableIr) -> Self {
        Self {
            name: &table.name,
            fields: &table.fields,
        }
    }
}
