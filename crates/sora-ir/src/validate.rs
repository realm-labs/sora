use std::collections::BTreeSet;

use sora_diagnostics::{Result, SoraError};

use crate::model::{AggregationIr, ConfigIr, FieldIr, StructIr, TableIr, TableModeIr, TypeIr};

pub fn validate_config_ir(ir: &ConfigIr) -> Result<()> {
    validate_unique_names("enum", ir.enums.iter().map(|item| item.name.as_str()))?;
    validate_unique_names("struct", ir.structs.iter().map(|item| item.name.as_str()))?;
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
    let table_names = ir
        .tables
        .iter()
        .map(|item| item.name.as_str())
        .collect::<BTreeSet<_>>();

    for item in &ir.enums {
        validate_unique_names("enum value", item.values.iter().map(String::as_str))?;
    }

    for item in &ir.structs {
        validate_fields(
            "struct",
            &item.name,
            &item.fields,
            &ValidationContext {
                enum_names: &enum_names,
                struct_names: &struct_names,
                table_names: &table_names,
                structs: &ir.structs,
                tables: &ir.tables,
            },
        )?;
    }

    for table in &ir.tables {
        let field_names = validate_fields(
            "table",
            &table.name,
            &table.fields,
            &ValidationContext {
                enum_names: &enum_names,
                struct_names: &struct_names,
                table_names: &table_names,
                structs: &ir.structs,
                tables: &ir.tables,
            },
        )?;

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
                if index.unique
                    && let Some(index_field) = table
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
                table_names: context.table_names,
                tables: context.tables,
            },
        )?;

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

struct ValidationContext<'a> {
    enum_names: &'a BTreeSet<&'a str>,
    struct_names: &'a BTreeSet<&'a str>,
    table_names: &'a BTreeSet<&'a str>,
    structs: &'a [StructIr],
    tables: &'a [TableIr],
}

struct TypeReferenceContext<'a> {
    enum_names: &'a BTreeSet<&'a str>,
    struct_names: &'a BTreeSet<&'a str>,
    table_names: &'a BTreeSet<&'a str>,
    tables: &'a [TableIr],
}

fn validate_type_references(
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
        TypeIr::List(element) | TypeIr::Optional(element) => {
            validate_type_references(owner_kind, owner, field_name, element, context)
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

            Ok(())
        }
        _ => Ok(()),
    }
}

fn validate_map_key_type(table: &TableIr, field: &FieldIr, tables: &[TableIr]) -> Result<()> {
    if is_valid_map_key_type(&field.ty, tables) {
        return Ok(());
    }

    Err(SoraError::InvalidSchema(format!(
        "map table `{}` key field `{}` has unsupported key type `{}`",
        table.name, field.name, field.ty
    )))
}

fn validate_index_field_type(
    table: &TableIr,
    index_name: &str,
    field: &FieldIr,
    tables: &[TableIr],
) -> Result<()> {
    if is_valid_map_key_type(&field.ty, tables) {
        return Ok(());
    }

    Err(SoraError::InvalidSchema(format!(
        "unique index `{}` in table `{}` field `{}` has unsupported key type `{}`",
        index_name, table.name, field.name, field.ty
    )))
}

fn is_valid_map_key_type(ty: &TypeIr, tables: &[TableIr]) -> bool {
    match ty {
        TypeIr::Bool | TypeIr::I32 | TypeIr::I64 | TypeIr::String | TypeIr::Enum(_) => true,
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
        | TypeIr::List(_)
        | TypeIr::Array { .. }
        | TypeIr::Optional(_) => false,
    }
}

fn validate_aggregation(
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

    if !types_compatible(&parent_key_field.ty, &child_key_field.ty) {
        return Err(SoraError::InvalidSchema(format!(
            "aggregation field `{}` joins `{}` and `{}` with incompatible key types `{}` and `{}`",
            field.name,
            aggregation.parent_key,
            aggregation.child_key,
            parent_key_field.ty,
            child_key_field.ty
        )));
    }

    validate_aggregation_result_type(field, source_table, structs)?;

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
        if !types_compatible(&struct_field.ty, &source_field.ty) {
            return Err(SoraError::InvalidSchema(format!(
                "aggregation field `{}` maps source field `{}` with incompatible type `{}` into `{}`",
                field.name, source_field.name, source_field.ty, struct_field.ty
            )));
        }
    }

    Ok(())
}

fn types_compatible(left: &TypeIr, right: &TypeIr) -> bool {
    left == right
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::normalize::normalize_schema;
    use sora_schema::model::SchemaFile;

    #[test]
    fn validates_valid_ir() {
        let ir = example_ir(
            r#"
[[structs]]
name = "Reward"

[[structs.fields]]
name = "item_id"
type = "ref<Item.id>"

[[tables]]
name = "Item"
mode = "map"
key = "id"

[[tables.fields]]
name = "id"
type = "i32"
key = true
required = true

[[tables.fields]]
name = "item_type"
type = "enum<ItemType>"

[[tables.fields]]
name = "reward"
type = "struct<Reward>"

[[tables.indexes]]
name = "by_type"
fields = ["item_type"]
"#,
        );

        validate_config_ir(&ir).unwrap();
    }

    #[test]
    fn rejects_duplicate_names_and_fields() {
        let duplicate_table = example_ir(
            r#"
[[tables]]
name = "Item"
mode = "list"

[[tables]]
name = "Item"
mode = "list"
"#,
        );
        assert!(matches!(
            validate_config_ir(&duplicate_table).unwrap_err(),
            SoraError::DuplicateSchemaName { kind: "table", name } if name == "Item"
        ));

        let duplicate_field = example_ir(
            r#"
[[tables]]
name = "Item"
mode = "list"

[[tables.fields]]
name = "id"
type = "i32"

[[tables.fields]]
name = "id"
type = "i64"
"#,
        );
        assert!(matches!(
            validate_config_ir(&duplicate_field).unwrap_err(),
            SoraError::DuplicateFieldName { owner_kind: "table", owner, field }
                if owner == "Item" && field == "id"
        ));
    }

    #[test]
    fn rejects_unknown_type_references() {
        let ir = example_ir(
            r#"
[[tables]]
name = "Item"
mode = "list"

[[tables.fields]]
name = "kind"
type = "enum<Missing>"
"#,
        );

        assert!(matches!(
            validate_config_ir(&ir).unwrap_err(),
            SoraError::UnknownTypeReference { kind: "enum", name, .. } if name == "Missing"
        ));
    }

    #[test]
    fn rejects_invalid_table_key_index_and_ref() {
        let missing_key = example_ir(
            r#"
[[tables]]
name = "Item"
mode = "map"
key = "id"
"#,
        );
        assert!(matches!(
            validate_config_ir(&missing_key).unwrap_err(),
            SoraError::MissingTableKey { table, field } if table == "Item" && field == "id"
        ));

        let map_without_key = example_ir(
            r#"
[[tables]]
name = "Item"
mode = "map"

[[tables.fields]]
name = "id"
type = "i32"
"#,
        );
        assert!(matches!(
            validate_config_ir(&map_without_key).unwrap_err(),
            SoraError::InvalidSchema(message) if message.contains("must declare `key`")
        ));

        let invalid_map_key_type = example_ir(
            r#"
[[tables]]
name = "Item"
mode = "map"
key = "weight"

[[tables.fields]]
name = "weight"
type = "f32"
"#,
        );
        assert!(matches!(
            validate_config_ir(&invalid_map_key_type).unwrap_err(),
            SoraError::InvalidSchema(message) if message.contains("unsupported key type")
        ));

        let bad_index = example_ir(
            r#"
[[tables]]
name = "Item"
mode = "list"

[[tables.fields]]
name = "id"
type = "i32"

[[tables.indexes]]
name = "bad"
fields = ["missing"]
"#,
        );
        assert!(matches!(
            validate_config_ir(&bad_index).unwrap_err(),
            SoraError::UnknownIndexField { table, index, field }
                if table == "Item" && index == "bad" && field == "missing"
        ));

        let bad_unique_index_type = example_ir(
            r#"
[[structs]]
name = "Tag"

[[structs.fields]]
name = "name"
type = "string"

[[tables]]
name = "Item"
mode = "map"
key = "id"

[[tables.fields]]
name = "id"
type = "i32"

[[tables.fields]]
name = "tag"
type = "struct<Tag>"

[[tables.indexes]]
name = "by_tag"
fields = ["tag"]
unique = true
"#,
        );
        assert!(matches!(
            validate_config_ir(&bad_unique_index_type).unwrap_err(),
            SoraError::InvalidSchema(message)
                if message.contains("unique index `by_tag`") && message.contains("unsupported key type")
        ));

        let bad_ref = example_ir(
            r#"
[[tables]]
name = "Item"
mode = "list"

[[tables.fields]]
name = "id"
type = "ref<Missing.id>"
"#,
        );
        assert!(matches!(
            validate_config_ir(&bad_ref).unwrap_err(),
            SoraError::UnknownRefTable { table, .. } if table == "Missing"
        ));
    }

    fn example_ir(extra: &str) -> ConfigIr {
        let source = format!(
            r#"
package = "game_config"

[[enums]]
name = "ItemType"
values = ["Weapon", "Armor"]

{extra}
"#
        );
        let schema: SchemaFile = toml::from_str(&source).unwrap();
        normalize_schema(schema).unwrap()
    }
}
