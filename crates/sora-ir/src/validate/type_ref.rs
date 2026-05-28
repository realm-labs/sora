use std::collections::BTreeSet;

use sora_diagnostics::{Result, SoraError};

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
    if is_valid_map_key_type(&field.ty, tables) {
        return Ok(());
    }

    Err(SoraError::InvalidSchema(format!(
        "map table `{}` key field `{}` has unsupported key type `{}`",
        table.name, field.name, field.ty
    )))
}

pub(super) fn validate_index_field_type(
    table: &TableIr,
    index_name: &str,
    field: &FieldIr,
    tables: &[TableIr],
) -> Result<()> {
    if is_valid_map_key_type(&field.ty, tables) {
        return Ok(());
    }

    Err(SoraError::InvalidSchema(format!(
        "index `{}` in table `{}` field `{}` has unsupported key type `{}`",
        index_name, table.name, field.name, field.ty
    )))
}

fn is_valid_map_key_type(ty: &TypeIr, tables: &[TableIr]) -> bool {
    match ty {
        TypeIr::Bool
        | TypeIr::I8
        | TypeIr::U8
        | TypeIr::I16
        | TypeIr::U16
        | TypeIr::I32
        | TypeIr::U32
        | TypeIr::I64
        | TypeIr::Duration
        | TypeIr::DateTime
        | TypeIr::String
        | TypeIr::Text
        | TypeIr::Enum(_) => true,
        TypeIr::Ref { table, field } => tables
            .iter()
            .find(|candidate| candidate.name == *table)
            .and_then(|table| {
                table
                    .fields
                    .iter()
                    .find(|candidate| candidate.name == *field)
            })
            .is_some_and(|field| is_valid_map_key_type(&field.ty, tables)),
        TypeIr::F32
        | TypeIr::F64
        | TypeIr::Struct(_)
        | TypeIr::Union(_)
        | TypeIr::List(_)
        | TypeIr::Set(_)
        | TypeIr::Map { .. }
        | TypeIr::Array { .. }
        | TypeIr::Optional(_) => false,
    }
}
