use sora_diagnostics::{Result, SoraError};

use crate::model::{FieldIr, TableIr, TableModeIr, TypeIr};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableKeyIdentity<'a> {
    Primitive {
        table: &'a TableIr,
        field: &'a FieldIr,
        raw_type: &'a TypeIr,
    },
    Enum {
        name: &'a str,
    },
}

pub fn resolve_table_key_identity<'a>(
    tables: &'a [TableIr],
    table: &'a TableIr,
) -> Result<TableKeyIdentity<'a>> {
    if table.mode != TableModeIr::Map {
        return Err(SoraError::InvalidSchema(format!(
            "table `{}` is not a map table and has no key identity",
            table.name
        )));
    }
    let key_name = table.key.as_deref().ok_or_else(|| {
        SoraError::InvalidSchema(format!("map table `{}` must declare `key`", table.name))
    })?;
    let field = table
        .fields
        .iter()
        .find(|field| field.name == key_name)
        .ok_or_else(|| SoraError::MissingTableKey {
            table: table.name.clone(),
            field: key_name.to_owned(),
        })?;
    let mut chain = vec![(table.name.as_str(), field.name.as_str())];
    resolve_key_type(tables, table, field, &field.ty, &mut chain)
}

pub fn resolve_ref_key_identity<'a>(
    tables: &'a [TableIr],
    table: &str,
    field: &str,
) -> Result<TableKeyIdentity<'a>> {
    let table = tables
        .iter()
        .find(|candidate| candidate.name == table)
        .ok_or_else(|| SoraError::InvalidSchema(format!("unknown referenced table `{table}`")))?;
    if table.mode != TableModeIr::Map || table.key.as_deref() != Some(field) {
        return Err(SoraError::InvalidSchema(format!(
            "reference `{}`.`{field}` does not target the primary key of a map table",
            table.name
        )));
    }
    resolve_table_key_identity(tables, table)
}

pub fn validate_key_type(ty: &TypeIr, tables: &[TableIr]) -> Result<()> {
    let mut chain = Vec::new();
    validate_key_type_inner(ty, tables, &mut chain)
}

fn resolve_key_type<'a>(
    tables: &'a [TableIr],
    owner_table: &'a TableIr,
    owner_field: &'a FieldIr,
    ty: &'a TypeIr,
    chain: &mut Vec<(&'a str, &'a str)>,
) -> Result<TableKeyIdentity<'a>> {
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
        | TypeIr::Text => Ok(TableKeyIdentity::Primitive {
            table: owner_table,
            field: owner_field,
            raw_type: ty,
        }),
        TypeIr::Enum(name) => Ok(TableKeyIdentity::Enum { name }),
        TypeIr::Ref { table, field } => {
            let (target_table, target_field) = ref_target(tables, table, field)?;
            push_ref(&mut *chain, target_table, target_field)?;
            let identity =
                resolve_key_type(tables, target_table, target_field, &target_field.ty, chain);
            chain.pop();
            identity
        }
        _ => Err(unsupported_key_type(ty)),
    }
}

fn validate_key_type_inner<'a>(
    ty: &'a TypeIr,
    tables: &'a [TableIr],
    chain: &mut Vec<(&'a str, &'a str)>,
) -> Result<()> {
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
        | TypeIr::Enum(_) => Ok(()),
        TypeIr::Ref { table, field } => {
            let (target_table, target_field) = ref_target(tables, table, field)?;
            push_ref(&mut *chain, target_table, target_field)?;
            let result = validate_key_type_inner(&target_field.ty, tables, chain);
            chain.pop();
            result
        }
        _ => Err(unsupported_key_type(ty)),
    }
}

fn ref_target<'a>(
    tables: &'a [TableIr],
    table: &str,
    field: &str,
) -> Result<(&'a TableIr, &'a FieldIr)> {
    let table_ir = tables
        .iter()
        .find(|candidate| candidate.name == table)
        .ok_or_else(|| SoraError::InvalidSchema(format!("unknown referenced table `{table}`")))?;
    let field_ir = table_ir
        .fields
        .iter()
        .find(|candidate| candidate.name == field)
        .ok_or_else(|| {
            SoraError::InvalidSchema(format!("unknown referenced field `{table}.{field}`"))
        })?;
    Ok((table_ir, field_ir))
}

fn push_ref<'a>(
    chain: &mut Vec<(&'a str, &'a str)>,
    table: &'a TableIr,
    field: &'a FieldIr,
) -> Result<()> {
    if let Some(cycle_start) = chain.iter().position(|(candidate_table, candidate_field)| {
        *candidate_table == table.name && *candidate_field == field.name
    }) {
        let mut cycle = chain[cycle_start..]
            .iter()
            .map(|(table, field)| format!("{table}.{field}"))
            .collect::<Vec<_>>();
        cycle.push(format!("{}.{}", table.name, field.name));
        return Err(SoraError::InvalidSchema(format!(
            "cyclic table key reference: {}",
            cycle.join(" -> ")
        )));
    }
    chain.push((&table.name, &field.name));
    Ok(())
}

fn unsupported_key_type(ty: &TypeIr) -> SoraError {
    SoraError::InvalidSchema(format!("unsupported key type `{ty}`"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::GroupSetIr;

    fn field(name: &str, ty: TypeIr) -> FieldIr {
        FieldIr {
            name: name.to_owned(),
            ty,
            groups: GroupSetIr::default(),
            key: true,
            comment: None,
            default: None,
            range: None,
            length: None,
            parser: None,
            derived_from: None,
        }
    }

    fn table(name: &str, ty: TypeIr) -> TableIr {
        TableIr {
            id: name.to_owned(),
            name: name.to_owned(),
            canonical_name: name.to_owned(),
            groups: GroupSetIr::default(),
            mode: TableModeIr::Map,
            key: Some("id".to_owned()),
            source: None,
            fields: vec![field("id", ty)],
            indexes: Vec::new(),
        }
    }

    #[test]
    fn ref_key_reuses_the_originating_table_identity() {
        let tables = vec![
            table("Asset", TypeIr::String),
            table(
                "Item",
                TypeIr::Ref {
                    table: "Asset".to_owned(),
                    field: "id".to_owned(),
                },
            ),
        ];
        let identity = resolve_table_key_identity(&tables, &tables[1]).unwrap();
        let TableKeyIdentity::Primitive { table, field, .. } = identity else {
            panic!("expected a strong key identity");
        };
        assert_eq!(table.name, "Asset");
        assert_eq!(field.name, "id");
    }

    #[test]
    fn cyclic_ref_keys_are_rejected() {
        let tables = vec![
            table(
                "A",
                TypeIr::Ref {
                    table: "B".to_owned(),
                    field: "id".to_owned(),
                },
            ),
            table(
                "B",
                TypeIr::Ref {
                    table: "A".to_owned(),
                    field: "id".to_owned(),
                },
            ),
        ];
        let error = resolve_table_key_identity(&tables, &tables[0]).unwrap_err();
        assert!(error.to_string().contains("A.id -> B.id -> A.id"));
    }
}
