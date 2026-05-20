use heck::{ToLowerCamelCase, ToPascalCase, ToSnakeCase};
use serde::Serialize;
use sora_diagnostics::Result;
use sora_ir::model::{ConfigIr, FieldIr, TableIr, TableModeIr, TypeIr};

use crate::types::{kotlin_type_name, rust_type_name};

#[derive(Debug, Clone, Serialize)]
pub struct CodegenModel {
    pub package: String,
    pub enums: Vec<CodegenEnum>,
    pub records: Vec<CodegenRecord>,
    pub tables: Vec<CodegenTable>,
    pub modules: Vec<String>,
    pub has_map_tables: bool,
    pub has_singleton_tables: bool,
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
    pub imports: Vec<CodegenImport>,
    pub fields: Vec<CodegenField>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CodegenImport {
    pub module: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CodegenTable {
    pub name: String,
    pub pascal_name: String,
    pub snake_name: String,
    pub mode: String,
    pub rust_container_type: String,
    pub rust_row_type: String,
    pub key_rust_name: Option<String>,
    pub key_rust_type: Option<String>,
    pub key_is_copy: bool,
    pub kotlin_container_type: String,
    pub kotlin_row_type: String,
    pub key_kotlin_name: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CodegenField {
    pub raw_name: String,
    pub rust_name: String,
    pub rust_type: String,
    pub kotlin_name: String,
    pub kotlin_type: String,
    pub kotlin_decode: String,
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

    let tables = ir
        .tables
        .iter()
        .map(|item| build_table(ir, item))
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
        has_map_tables: tables
            .iter()
            .any(|table| table.mode == "map" && table.key_rust_name.is_some()),
        has_singleton_tables: tables.iter().any(|table| table.mode == "singleton"),
        tables,
        modules,
    })
}

fn build_table(ir: &ConfigIr, table: &TableIr) -> Result<CodegenTable> {
    let pascal_name = table.name.to_pascal_case();
    let snake_name = table.name.to_snake_case();
    let rust_row_type = format!("{snake_name}::{pascal_name}");
    let kotlin_row_type = pascal_name.clone();
    let key_field = table.key.as_ref().and_then(|key| {
        table
            .fields
            .iter()
            .find(|field| field.name == *key)
            .map(|field| {
                (
                    field.name.to_snake_case(),
                    field.name.to_lower_camel_case(),
                    rust_type_name(ir, &field.ty),
                    kotlin_type_name(ir, &field.ty),
                    rust_key_type_is_copy(ir, &field.ty),
                )
            })
    });
    let rust_container_type = match table.mode {
        TableModeIr::List => format!("Vec<{rust_row_type}>"),
        TableModeIr::Map => match &key_field {
            Some((_, _, key_type, _, _)) => {
                format!("std::collections::HashMap<{key_type}, {rust_row_type}>")
            }
            None => format!("Vec<{rust_row_type}>"),
        },
        TableModeIr::Singleton => rust_row_type.clone(),
    };
    let kotlin_container_type = match table.mode {
        TableModeIr::List => format!("List<{kotlin_row_type}>"),
        TableModeIr::Map => match &key_field {
            Some((_, _, _, key_type, _)) => format!("Map<{key_type}, {kotlin_row_type}>"),
            None => format!("List<{kotlin_row_type}>"),
        },
        TableModeIr::Singleton => kotlin_row_type.clone(),
    };

    Ok(CodegenTable {
        name: table.name.clone(),
        pascal_name,
        snake_name,
        mode: match table.mode {
            TableModeIr::List => "list",
            TableModeIr::Map => "map",
            TableModeIr::Singleton => "singleton",
        }
        .to_owned(),
        rust_container_type,
        rust_row_type,
        key_rust_name: key_field.as_ref().map(|(name, _, _, _, _)| name.clone()),
        key_rust_type: key_field.as_ref().map(|(_, _, ty, _, _)| ty.clone()),
        key_is_copy: key_field
            .as_ref()
            .is_some_and(|(_, _, _, _, is_copy)| *is_copy),
        kotlin_container_type,
        kotlin_row_type,
        key_kotlin_name: key_field.as_ref().map(|(_, name, _, _, _)| name.clone()),
    })
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
        TypeIr::String | TypeIr::Struct(_) | TypeIr::List(_) | TypeIr::Array { .. } => false,
    }
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
                kotlin_decode: kotlin_decode_expr(ir, &field.ty),
                comment: field.comment.clone(),
            })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(CodegenRecord {
        name: item.name.to_owned(),
        pascal_name: item.name.to_pascal_case(),
        snake_name: item.name.to_snake_case(),
        camel_name: item.name.to_lower_camel_case(),
        imports: build_imports(ir, item),
        fields,
    })
}

fn kotlin_decode_expr(ir: &ConfigIr, ty: &TypeIr) -> String {
    match ty {
        TypeIr::Bool => "reader.readBool()".to_owned(),
        TypeIr::I32 => "reader.readI32()".to_owned(),
        TypeIr::I64 => "reader.readI64()".to_owned(),
        TypeIr::F32 => "reader.readF32()".to_owned(),
        TypeIr::F64 => "reader.readF64()".to_owned(),
        TypeIr::String => "reader.readString()".to_owned(),
        TypeIr::Enum(name) | TypeIr::Struct(name) => format!("{name}.decode(reader)"),
        TypeIr::List(element) | TypeIr::Array { element, .. } => {
            format!("reader.readList {{ {} }}", kotlin_decode_expr(ir, element))
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

fn build_imports(ir: &ConfigIr, item: &TableLike<'_>) -> Vec<CodegenImport> {
    let mut imports = Vec::new();
    for field in item.fields {
        collect_type_imports(ir, item.name, &field.ty, &mut imports);
    }
    imports.sort_by(|a, b| a.module.cmp(&b.module).then(a.name.cmp(&b.name)));
    imports.dedup_by(|a, b| a.module == b.module && a.name == b.name);
    imports
}

fn collect_type_imports(
    ir: &ConfigIr,
    owner_name: &str,
    ty: &TypeIr,
    imports: &mut Vec<CodegenImport>,
) {
    match ty {
        TypeIr::Enum(name) | TypeIr::Struct(name) => push_named_import(owner_name, name, imports),
        TypeIr::List(element) | TypeIr::Optional(element) => {
            collect_type_imports(ir, owner_name, element, imports);
        }
        TypeIr::Array { element, .. } => collect_type_imports(ir, owner_name, element, imports),
        TypeIr::Ref { table, field } => {
            if let Some(target_field) = ir
                .tables
                .iter()
                .find(|candidate| candidate.name == *table)
                .and_then(|table| {
                    table
                        .fields
                        .iter()
                        .find(|candidate| candidate.name == *field)
                })
            {
                collect_type_imports(ir, owner_name, &target_field.ty, imports);
            }
        }
        _ => {}
    }
}

fn push_named_import(owner_name: &str, name: &str, imports: &mut Vec<CodegenImport>) {
    if name == owner_name {
        return;
    }
    imports.push(CodegenImport {
        module: name.to_snake_case(),
        name: name.to_pascal_case(),
    });
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
