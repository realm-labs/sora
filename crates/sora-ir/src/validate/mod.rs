use std::collections::{BTreeMap, BTreeSet};

use sora_diagnostics::{Result, SoraError};

use crate::{
    input_projection::{
        TAGGED_COLUMNS_PARSER, tagged_columns, tagged_columns_prefix, tagged_columns_union,
    },
    model::{ConfigIr, FieldIr, StructIr, TableIr, TableModeIr},
};

mod aggregation;
mod type_ref;

use aggregation::validate_aggregation;
use type_ref::{
    TypeReferenceContext, validate_index_field_type, validate_map_key_type,
    validate_type_references,
};

pub fn validate_config_ir(ir: &ConfigIr) -> Result<()> {
    validate_unique_names("enum", ir.enums.iter().map(|item| item.name.as_str()))?;
    validate_unique_names("struct", ir.structs.iter().map(|item| item.name.as_str()))?;
    validate_unique_names("union", ir.unions.iter().map(|item| item.name.as_str()))?;
    validate_unique_names("table", ir.tables.iter().map(|item| item.name.as_str()))?;

    let enum_names = ir
        .enums
        .iter()
        .map(|item| item.name.as_str())
        .collect::<BTreeSet<_>>();
    let struct_names = ir
        .structs
        .iter()
        .map(|item| item.name.as_str())
        .collect::<BTreeSet<_>>();
    let union_names = ir
        .unions
        .iter()
        .map(|item| item.name.as_str())
        .collect::<BTreeSet<_>>();
    let table_names = ir
        .tables
        .iter()
        .map(|item| item.name.as_str())
        .collect::<BTreeSet<_>>();

    for item in &ir.enums {
        validate_unique_names("enum value", item.values.iter().map(String::as_str))?;
        validate_unique_names(
            "enum alias",
            item.aliases.iter().map(|alias| alias.alias.as_str()),
        )?;
        for alias in &item.aliases {
            if !item.values.iter().any(|value| value == &alias.name) {
                return Err(SoraError::InvalidSchema(format!(
                    "enum `{}` alias `{}` targets unknown value `{}`",
                    item.name, alias.alias, alias.name
                )));
            }
            if item.values.iter().any(|value| value == &alias.alias) {
                return Err(SoraError::InvalidSchema(format!(
                    "enum `{}` alias `{}` conflicts with an enum value",
                    item.name, alias.alias
                )));
            }
        }
    }

    for item in &ir.structs {
        validate_fields(
            "struct",
            &item.name,
            &item.fields,
            &ValidationContext {
                enum_names: &enum_names,
                struct_names: &struct_names,
                union_names: &union_names,
                table_names: &table_names,
                structs: &ir.structs,
                tables: &ir.tables,
            },
        )?;
    }

    for item in &ir.unions {
        validate_unique_names(
            "union variant",
            item.variants.iter().map(|variant| variant.name.as_str()),
        )?;
        for variant in &item.variants {
            if variant.fields.iter().any(|field| field.name == item.tag) {
                return Err(SoraError::InvalidSchema(format!(
                    "union `{}` variant `{}` field conflicts with tag `{}`",
                    item.name, variant.name, item.tag
                )));
            }
            validate_fields(
                "union",
                &item.name,
                &variant.fields,
                &ValidationContext {
                    enum_names: &enum_names,
                    struct_names: &struct_names,
                    union_names: &union_names,
                    table_names: &table_names,
                    structs: &ir.structs,
                    tables: &ir.tables,
                },
            )?;
        }
    }

    for table in &ir.tables {
        let field_names = validate_fields(
            "table",
            &table.name,
            &table.fields,
            &ValidationContext {
                enum_names: &enum_names,
                struct_names: &struct_names,
                union_names: &union_names,
                table_names: &table_names,
                structs: &ir.structs,
                tables: &ir.tables,
            },
        )?;
        validate_table_input_columns(ir, table)?;

        if table.mode == TableModeIr::Map && table.key.is_none() {
            return Err(SoraError::InvalidSchema(format!(
                "map table `{}` must declare `key`",
                table.name
            )));
        }

        if let Some(key) = &table.key
            && !field_names.contains(key.as_str())
        {
            return Err(SoraError::MissingTableKey {
                table: table.name.clone(),
                field: key.clone(),
            });
        }

        if table.mode == TableModeIr::Map
            && let Some(key) = &table.key
            && let Some(key_field) = table.fields.iter().find(|field| field.name == *key)
        {
            validate_map_key_type(table, key_field, &ir.tables)?;
        }

        validate_unique_names("index", table.indexes.iter().map(|item| item.name.as_str()))?;
        for index in &table.indexes {
            for field in &index.fields {
                if !field_names.contains(field.as_str()) {
                    return Err(SoraError::UnknownIndexField {
                        table: table.name.clone(),
                        index: index.name.clone(),
                        field: field.clone(),
                    });
                }
                if let Some(index_field) = table
                    .fields
                    .iter()
                    .find(|candidate| candidate.name == *field)
                {
                    validate_index_field_type(table, &index.name, index_field, &ir.tables)?;
                }
            }
        }
    }

    Ok(())
}

fn validate_unique_names<'a>(
    kind: &'static str,
    names: impl IntoIterator<Item = &'a str>,
) -> Result<()> {
    let mut seen = BTreeSet::new();
    for name in names {
        if !seen.insert(name) {
            return Err(SoraError::DuplicateSchemaName {
                kind,
                name: name.to_owned(),
            });
        }
    }
    Ok(())
}

fn validate_fields<'a>(
    owner_kind: &'static str,
    owner: &str,
    fields: &'a [FieldIr],
    context: &ValidationContext<'_>,
) -> Result<BTreeSet<&'a str>> {
    let mut field_names = BTreeSet::new();

    for field in fields {
        if !field_names.insert(field.name.as_str()) {
            return Err(SoraError::DuplicateFieldName {
                owner_kind,
                owner: owner.to_owned(),
                field: field.name.clone(),
            });
        }

        validate_type_references(
            owner_kind,
            owner,
            &field.name,
            &field.ty,
            &TypeReferenceContext {
                enum_names: context.enum_names,
                struct_names: context.struct_names,
                union_names: context.union_names,
                table_names: context.table_names,
                tables: context.tables,
            },
        )?;

        if field
            .parser
            .as_ref()
            .is_some_and(|parser| parser.kind == TAGGED_COLUMNS_PARSER)
            && owner_kind != "table"
        {
            return Err(SoraError::InvalidSchema(format!(
                "{owner_kind} `{owner}` field `{}` declares parser `tagged_columns`, but tagged columns are only supported on table fields",
                field.name
            )));
        }

        if let Some(aggregation) = &field.aggregation {
            validate_aggregation(
                owner_kind,
                owner,
                field,
                aggregation,
                context.structs,
                context.tables,
            )?;
        }
    }

    Ok(field_names)
}

fn validate_table_input_columns(ir: &ConfigIr, table: &TableIr) -> Result<()> {
    let mut columns = BTreeSet::<String>::new();

    for field in &table.fields {
        let Some(projected) = tagged_columns(ir, field) else {
            if !columns.insert(field.name.clone()) {
                return Err(input_column_conflict(&table.name, &field.name, &field.name));
            }
            continue;
        };

        validate_tagged_columns_internal(ir, table, field)?;
        for column in projected {
            if !columns.insert(column.name.clone()) {
                return Err(input_column_conflict(
                    &table.name,
                    &field.name,
                    &column.name,
                ));
            }
        }
    }

    Ok(())
}

fn validate_tagged_columns_internal(ir: &ConfigIr, table: &TableIr, field: &FieldIr) -> Result<()> {
    let Some(union) = tagged_columns_union(ir, field) else {
        return Ok(());
    };
    let prefix = tagged_columns_prefix(field).unwrap_or_default();
    let tag_column = format!("{prefix}{}", union.tag);
    let mut variant_columns = BTreeMap::<String, &FieldIr>::new();

    for variant in &union.variants {
        for variant_field in &variant.fields {
            let column = format!("{prefix}{}", variant_field.name);
            if column == tag_column {
                return Err(SoraError::InvalidSchema(format!(
                    "table `{}` field `{}` uses tagged_columns with column `{column}`, but it conflicts with union tag column `{tag_column}`",
                    table.name, field.name
                )));
            }

            if let Some(existing) = variant_columns.get(&column)
                && (existing.ty != variant_field.ty || existing.parser != variant_field.parser)
            {
                return Err(SoraError::InvalidSchema(format!(
                    "table `{}` field `{}` uses tagged_columns with incompatible repeated variant column `{column}`",
                    table.name, field.name
                )));
            }
            variant_columns.entry(column).or_insert(variant_field);
        }
    }

    Ok(())
}

fn input_column_conflict(table: &str, field: &str, column: &str) -> SoraError {
    SoraError::InvalidSchema(format!(
        "table `{table}` field `{field}` maps to input column `{column}`, but that column is already used"
    ))
}

struct ValidationContext<'a> {
    enum_names: &'a BTreeSet<&'a str>,
    struct_names: &'a BTreeSet<&'a str>,
    union_names: &'a BTreeSet<&'a str>,
    table_names: &'a BTreeSet<&'a str>,
    structs: &'a [StructIr],
    tables: &'a [TableIr],
}

#[cfg(test)]
mod tests;
