use sora_diagnostics::{Result, SoraError};

use crate::model::{AggregationIr, FieldIr, StructIr, TableIr, TypeIr};
pub(super) fn validate_aggregation(
    owner_kind: &'static str,
    owner: &str,
    field: &FieldIr,
    aggregation: &AggregationIr,
    structs: &[StructIr],
    tables: &[TableIr],
) -> Result<()> {
    if owner_kind != "table" {
        return Err(SoraError::InvalidSchema(format!(
            "aggregation field `{}` is only supported on tables",
            field.name
        )));
    }

    let Some(owner_table) = tables.iter().find(|table| table.name == owner) else {
        return Err(SoraError::InvalidSchema(format!(
            "aggregation owner table `{owner}` does not exist"
        )));
    };
    let Some(source_table) = tables
        .iter()
        .find(|table| table.name == aggregation.source_table)
    else {
        return Err(SoraError::UnknownRefTable {
            owner_kind,
            owner: owner.to_owned(),
            field: field.name.clone(),
            table: aggregation.source_table.clone(),
        });
    };

    let Some(parent_key_field) = owner_table
        .fields
        .iter()
        .find(|candidate| candidate.name == aggregation.parent_key)
    else {
        return Err(SoraError::UnknownRefField {
            owner_kind,
            owner: owner.to_owned(),
            field: field.name.clone(),
            table: owner.to_owned(),
            ref_field: aggregation.parent_key.clone(),
        });
    };

    let Some(child_key_field) = source_table
        .fields
        .iter()
        .find(|candidate| candidate.name == aggregation.child_key)
    else {
        return Err(SoraError::UnknownRefField {
            owner_kind,
            owner: owner.to_owned(),
            field: field.name.clone(),
            table: aggregation.source_table.clone(),
            ref_field: aggregation.child_key.clone(),
        });
    };

    if !types_compatible(&parent_key_field.ty, &child_key_field.ty, tables) {
        return Err(SoraError::InvalidSchema(format!(
            "aggregation field `{}` joins `{}` and `{}` with incompatible key types `{}` and `{}`",
            field.name,
            aggregation.parent_key,
            aggregation.child_key,
            parent_key_field.ty,
            child_key_field.ty
        )));
    }

    validate_aggregation_result_type(field, source_table, structs, tables)?;

    if let Some(order_by) = &aggregation.order_by
        && !source_table
            .fields
            .iter()
            .any(|field| field.name == *order_by)
    {
        return Err(SoraError::UnknownRefField {
            owner_kind,
            owner: owner.to_owned(),
            field: field.name.clone(),
            table: aggregation.source_table.clone(),
            ref_field: order_by.clone(),
        });
    }

    Ok(())
}

fn validate_aggregation_result_type(
    field: &FieldIr,
    source_table: &TableIr,
    structs: &[StructIr],
    tables: &[TableIr],
) -> Result<()> {
    let TypeIr::List(element) = &field.ty else {
        return Err(SoraError::InvalidSchema(format!(
            "aggregation field `{}` must have type `list<struct>`",
            field.name
        )));
    };
    let TypeIr::Struct(struct_name) = element.as_ref() else {
        return Err(SoraError::InvalidSchema(format!(
            "aggregation field `{}` must aggregate struct values",
            field.name
        )));
    };
    let Some(struct_ir) = structs.iter().find(|item| item.name == *struct_name) else {
        return Err(SoraError::InvalidSchema(format!(
            "aggregation field `{}` references unknown struct `{struct_name}`",
            field.name
        )));
    };

    for struct_field in &struct_ir.fields {
        let Some(source_field) = source_table
            .fields
            .iter()
            .find(|candidate| candidate.name == struct_field.name)
        else {
            return Err(SoraError::UnknownRefField {
                owner_kind: "table",
                owner: source_table.name.clone(),
                field: field.name.clone(),
                table: source_table.name.clone(),
                ref_field: struct_field.name.clone(),
            });
        };
        if !types_compatible(&struct_field.ty, &source_field.ty, tables) {
            return Err(SoraError::InvalidSchema(format!(
                "aggregation field `{}` maps source field `{}` with incompatible type `{}` into `{}`",
                field.name, source_field.name, source_field.ty, struct_field.ty
            )));
        }
    }

    Ok(())
}

fn types_compatible(left: &TypeIr, right: &TypeIr, tables: &[TableIr]) -> bool {
    resolve_ref_type(left, tables) == resolve_ref_type(right, tables)
}

fn resolve_ref_type<'a>(ty: &'a TypeIr, tables: &'a [TableIr]) -> Option<&'a TypeIr> {
    let mut current = ty;
    let max_depth = tables.len().saturating_mul(8).saturating_add(8);
    for _ in 0..max_depth {
        match current {
            TypeIr::Ref { table, field } => {
                current = &tables
                    .iter()
                    .find(|candidate| candidate.name == *table)?
                    .fields
                    .iter()
                    .find(|candidate| candidate.name == *field)?
                    .ty;
            }
            _ => return Some(current),
        }
    }

    None
}
