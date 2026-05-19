use std::collections::BTreeSet;

use sora_diagnostics::{Result, SoraError};

use crate::model::{AggregationIr, ConfigIr, FieldIr, TableIr, TypeIr};

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
            &enum_names,
            &struct_names,
            &table_names,
            &ir.tables,
        )?;
    }

    for table in &ir.tables {
        let field_names = validate_fields(
            "table",
            &table.name,
            &table.fields,
            &enum_names,
            &struct_names,
            &table_names,
            &ir.tables,
        )?;

        if let Some(key) = &table.key {
            if !field_names.contains(key.as_str()) {
                return Err(SoraError::MissingTableKey {
                    table: table.name.clone(),
                    field: key.clone(),
                });
            }
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
    enum_names: &BTreeSet<&str>,
    struct_names: &BTreeSet<&str>,
    table_names: &BTreeSet<&str>,
    tables: &[TableIr],
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
            enum_names,
            struct_names,
            table_names,
            tables,
        )?;

        if let Some(aggregation) = &field.aggregation {
            validate_aggregation(owner_kind, owner, &field.name, aggregation, tables)?;
        }
    }

    Ok(field_names)
}

fn validate_type_references(
    owner_kind: &'static str,
    owner: &str,
    field_name: &str,
    ty: &TypeIr,
    enum_names: &BTreeSet<&str>,
    struct_names: &BTreeSet<&str>,
    table_names: &BTreeSet<&str>,
    tables: &[TableIr],
) -> Result<()> {
    match ty {
        TypeIr::Enum(name) if !enum_names.contains(name.as_str()) => {
            Err(SoraError::UnknownTypeReference {
                kind: "enum",
                name: name.clone(),
                owner_kind,
                owner: owner.to_owned(),
                field: field_name.to_owned(),
            })
        }
        TypeIr::Struct(name) if !struct_names.contains(name.as_str()) => {
            Err(SoraError::UnknownTypeReference {
                kind: "struct",
                name: name.clone(),
                owner_kind,
                owner: owner.to_owned(),
                field: field_name.to_owned(),
            })
        }
        TypeIr::List(element) | TypeIr::Optional(element) => validate_type_references(
            owner_kind,
            owner,
            field_name,
            element,
            enum_names,
            struct_names,
            table_names,
            tables,
        ),
        TypeIr::Array { element, .. } => validate_type_references(
            owner_kind,
            owner,
            field_name,
            element,
            enum_names,
            struct_names,
            table_names,
            tables,
        ),
        TypeIr::Ref { table, field } => {
            if !table_names.contains(table.as_str()) {
                return Err(SoraError::UnknownRefTable {
                    owner_kind,
                    owner: owner.to_owned(),
                    field: field_name.to_owned(),
                    table: table.clone(),
                });
            }

            let table_ir = tables
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

fn validate_aggregation(
    owner_kind: &'static str,
    owner: &str,
    field_name: &str,
    aggregation: &AggregationIr,
    tables: &[TableIr],
) -> Result<()> {
    let Some(source_table) = tables
        .iter()
        .find(|table| table.name == aggregation.source_table)
    else {
        return Err(SoraError::UnknownRefTable {
            owner_kind,
            owner: owner.to_owned(),
            field: field_name.to_owned(),
            table: aggregation.source_table.clone(),
        });
    };

    if !source_table
        .fields
        .iter()
        .any(|field| field.name == aggregation.child_key)
    {
        return Err(SoraError::UnknownRefField {
            owner_kind,
            owner: owner.to_owned(),
            field: field_name.to_owned(),
            table: aggregation.source_table.clone(),
            ref_field: aggregation.child_key.clone(),
        });
    }

    if let Some(order_by) = &aggregation.order_by {
        if !source_table
            .fields
            .iter()
            .any(|field| field.name == *order_by)
        {
            return Err(SoraError::UnknownRefField {
                owner_kind,
                owner: owner.to_owned(),
                field: field_name.to_owned(),
                table: aggregation.source_table.clone(),
                ref_field: order_by.clone(),
            });
        }
    }

    Ok(())
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
