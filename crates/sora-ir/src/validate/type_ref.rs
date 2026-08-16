use std::collections::BTreeSet;

use sora_diagnostics::{Result, SoraError};

use crate::key::validate_key_type;
use crate::model::{FieldIr, TableIr, TableModeIr, TypeIr};
pub(super) struct TypeReferenceContext<'a> {
    pub(super) enum_names: &'a BTreeSet<&'a str>,
    pub(super) struct_names: &'a BTreeSet<&'a str>,
    pub(super) union_names: &'a BTreeSet<&'a str>,
    pub(super) table_names: &'a BTreeSet<&'a str>,
    pub(super) tables: &'a [TableIr],
}

pub(super) fn validate_type_references(
    owner_kind: &'static str,
    owner: &str,
    field_name: &str,
    ty: &TypeIr,
    context: &TypeReferenceContext<'_>,
) -> Result<()> {
    match ty {
        TypeIr::Enum(name) if !context.enum_names.contains(name.as_str()) => {
            Err(SoraError::UnknownTypeReference {
                kind: "enum",
                name: name.clone(),
                owner_kind,
                owner: owner.to_owned(),
                field: field_name.to_owned(),
            })
        }
        TypeIr::Struct(name) if !context.struct_names.contains(name.as_str()) => {
            Err(SoraError::UnknownTypeReference {
                kind: "struct",
                name: name.clone(),
                owner_kind,
                owner: owner.to_owned(),
                field: field_name.to_owned(),
            })
        }
        TypeIr::Union(name) if !context.union_names.contains(name.as_str()) => {
            Err(SoraError::UnknownTypeReference {
                kind: "union",
                name: name.clone(),
                owner_kind,
                owner: owner.to_owned(),
                field: field_name.to_owned(),
            })
        }
        TypeIr::List(element) | TypeIr::Set(element) | TypeIr::Optional(element) => {
            validate_type_references(owner_kind, owner, field_name, element, context)
        }
        TypeIr::Map { key, value } => {
            validate_type_references(owner_kind, owner, field_name, key, context)?;
            validate_type_references(owner_kind, owner, field_name, value, context)
        }
        TypeIr::Array { element, .. } => {
            validate_type_references(owner_kind, owner, field_name, element, context)
        }
        TypeIr::Ref { table, field } => {
            if !context.table_names.contains(table.as_str()) {
                return Err(SoraError::UnknownRefTable {
                    owner_kind,
                    owner: owner.to_owned(),
                    field: field_name.to_owned(),
                    table: table.clone(),
                });
            }

            let table_ir = context
                .tables
                .iter()
                .find(|candidate| candidate.name == *table)
                .expect("table_names and tables should match");
            if !table_ir
                .fields
                .iter()
                .any(|candidate| candidate.name == *field)
            {
                return Err(SoraError::UnknownRefField {
                    owner_kind,
                    owner: owner.to_owned(),
                    field: field_name.to_owned(),
                    table: table.clone(),
                    ref_field: field.clone(),
                });
            }
            if table_ir.mode != TableModeIr::Map || table_ir.key.as_deref() != Some(field) {
                let primary_key = table_ir.key.as_deref().unwrap_or("<none>");
                return Err(SoraError::InvalidSchema(format!(
                    "field `{owner}.{field_name}` references `{table}.{field}`, but references can only target the primary key of a map table; `{table}` primary key is `{primary_key}`"
                )));
            }

            Ok(())
        }
        _ => Ok(()),
    }
}

pub(super) fn validate_map_key_type(
    table: &TableIr,
    field: &FieldIr,
    tables: &[TableIr],
) -> Result<()> {
    validate_key_type(&field.ty, tables).map_err(|error| {
        SoraError::InvalidSchema(format!(
            "map table `{}` key field `{}` is invalid: {}",
            table.name,
            field.name,
            key_error_message(error)
        ))
    })
}

pub(super) fn validate_index_field_type(
    table: &TableIr,
    index_name: &str,
    field: &FieldIr,
    tables: &[TableIr],
) -> Result<()> {
    validate_key_type(&field.ty, tables).map_err(|error| {
        SoraError::InvalidSchema(format!(
            "index `{index_name}` in table `{}` field `{}` is invalid: {}",
            table.name,
            field.name,
            key_error_message(error)
        ))
    })
}

fn key_error_message(error: SoraError) -> String {
    match error {
        SoraError::InvalidSchema(message) => message,
        error => error.to_string(),
    }
}
