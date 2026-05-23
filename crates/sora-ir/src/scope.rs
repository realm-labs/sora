use std::collections::BTreeSet;

use sora_diagnostics::{Result, SoraError};

use crate::{
    model::{
        ConfigIr, FieldIr, IndexIr, ScopeIr, StructIr, TableIr, TypeIr, UnionIr, UnionVariantIr,
    },
    validate::validate_config_ir,
};

pub fn filter_config_ir_by_scope(ir: &ConfigIr, target: &str) -> Result<ConfigIr> {
    validate_scope_target(target)?;
    if target == "all" {
        return Ok(ir.clone());
    }

    let mut filtered = ConfigIr {
        package: ir.package.clone(),
        enums: ir
            .enums
            .iter()
            .filter(|item| includes(&item.scope, target))
            .cloned()
            .collect(),
        structs: ir
            .structs
            .iter()
            .filter(|item| includes(&item.scope, target))
            .map(|item| filter_struct(item, target))
            .collect(),
        unions: ir
            .unions
            .iter()
            .filter(|item| includes(&item.scope, target))
            .map(|item| filter_union(item, target))
            .collect(),
        tables: ir
            .tables
            .iter()
            .filter(|item| includes(&item.scope, target))
            .map(|item| filter_table(item, target))
            .collect(),
    };

    prune_unreachable_types(&mut filtered);
    validate_config_ir(&filtered)?;
    Ok(filtered)
}

fn validate_scope_target(target: &str) -> Result<()> {
    if target.is_empty()
        || !target
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
    {
        return Err(SoraError::InvalidSchema(format!(
            "scope `{target}` must contain only ASCII letters, digits, `_`, or `-`"
        )));
    }
    Ok(())
}

fn filter_struct(item: &StructIr, target: &str) -> StructIr {
    StructIr {
        name: item.name.clone(),
        scope: item.scope.clone(),
        fields: filter_fields(&item.fields, target),
    }
}

fn filter_union(item: &UnionIr, target: &str) -> UnionIr {
    UnionIr {
        name: item.name.clone(),
        scope: item.scope.clone(),
        tag: item.tag.clone(),
        variants: item
            .variants
            .iter()
            .filter(|variant| includes(&variant.scope, target))
            .map(|variant| filter_union_variant(variant, target))
            .collect(),
    }
}

fn filter_union_variant(item: &UnionVariantIr, target: &str) -> UnionVariantIr {
    UnionVariantIr {
        name: item.name.clone(),
        scope: item.scope.clone(),
        fields: filter_fields(&item.fields, target),
    }
}

fn filter_table(item: &TableIr, target: &str) -> TableIr {
    let fields = filter_fields(&item.fields, target);
    let field_names = fields
        .iter()
        .map(|field| field.name.as_str())
        .collect::<BTreeSet<_>>();
    TableIr {
        name: item.name.clone(),
        scope: item.scope.clone(),
        mode: item.mode,
        key: item
            .key
            .as_ref()
            .filter(|key| field_names.contains(key.as_str()))
            .cloned(),
        source: item.source.clone(),
        indexes: item
            .indexes
            .iter()
            .filter(|index| {
                index
                    .fields
                    .iter()
                    .all(|field| field_names.contains(field.as_str()))
            })
            .cloned()
            .collect::<Vec<IndexIr>>(),
        fields,
    }
}

fn filter_fields(fields: &[FieldIr], target: &str) -> Vec<FieldIr> {
    fields
        .iter()
        .filter(|field| includes(&field.scope, target))
        .cloned()
        .collect()
}

fn includes(scope: &ScopeIr, target: &str) -> bool {
    scope.includes(target)
}

fn prune_unreachable_types(ir: &mut ConfigIr) {
    let mut enum_names = BTreeSet::new();
    let mut struct_names = BTreeSet::new();
    let mut union_names = BTreeSet::new();

    for table in &ir.tables {
        for field in &table.fields {
            collect_type_names(
                &field.ty,
                &mut enum_names,
                &mut struct_names,
                &mut union_names,
            );
        }
    }

    let mut changed = true;
    while changed {
        changed = false;
        for item in &ir.structs {
            if struct_names.contains(item.name.as_str()) {
                for field in &item.fields {
                    let before = (enum_names.len(), struct_names.len(), union_names.len());
                    collect_type_names(
                        &field.ty,
                        &mut enum_names,
                        &mut struct_names,
                        &mut union_names,
                    );
                    changed |= before != (enum_names.len(), struct_names.len(), union_names.len());
                }
            }
        }
        for item in &ir.unions {
            if union_names.contains(item.name.as_str()) {
                for variant in &item.variants {
                    for field in &variant.fields {
                        let before = (enum_names.len(), struct_names.len(), union_names.len());
                        collect_type_names(
                            &field.ty,
                            &mut enum_names,
                            &mut struct_names,
                            &mut union_names,
                        );
                        changed |=
                            before != (enum_names.len(), struct_names.len(), union_names.len());
                    }
                }
            }
        }
    }

    ir.enums
        .retain(|item| enum_names.contains(item.name.as_str()));
    ir.structs
        .retain(|item| struct_names.contains(item.name.as_str()));
    ir.unions
        .retain(|item| union_names.contains(item.name.as_str()));
}

fn collect_type_names(
    ty: &TypeIr,
    enum_names: &mut BTreeSet<String>,
    struct_names: &mut BTreeSet<String>,
    union_names: &mut BTreeSet<String>,
) {
    match ty {
        TypeIr::Enum(name) => {
            enum_names.insert(name.clone());
        }
        TypeIr::Struct(name) => {
            struct_names.insert(name.clone());
        }
        TypeIr::Union(name) => {
            union_names.insert(name.clone());
        }
        TypeIr::List(element) | TypeIr::Set(element) | TypeIr::Optional(element) => {
            collect_type_names(element, enum_names, struct_names, union_names);
        }
        TypeIr::Map { key, value } => {
            collect_type_names(key, enum_names, struct_names, union_names);
            collect_type_names(value, enum_names, struct_names, union_names);
        }
        TypeIr::Array { element, .. } => {
            collect_type_names(element, enum_names, struct_names, union_names);
        }
        TypeIr::Bool
        | TypeIr::I32
        | TypeIr::I64
        | TypeIr::F32
        | TypeIr::F64
        | TypeIr::String
        | TypeIr::Ref { .. } => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::normalize::normalize_schema;
    use sora_schema::model::SchemaFile;

    #[test]
    fn filters_tables_fields_and_nested_types_by_scope() {
        let schema: SchemaFile = toml::from_str(
            r#"
package = "game_config"

[[enums]]
name = "ItemType"
values = ["Weapon"]

[[structs]]
name = "Reward"

[[structs.fields]]
name = "item_id"
type = "i32"

[[structs.fields]]
name = "debug_note"
type = "string"
scope = "server"

[[tables]]
name = "Item"
mode = "map"
key = "id"

[[tables.fields]]
name = "id"
type = "i32"

[[tables.fields]]
name = "icon"
type = "string"
scope = "client"

[[tables.fields]]
name = "drop"
type = "struct<Reward>"

[[tables.fields]]
name = "formula"
type = "string"
scope = "server"

[[tables]]
name = "ServerOnly"
mode = "list"
scope = "server"

[[tables.fields]]
name = "value"
type = "i32"
"#,
        )
        .unwrap();
        let ir = normalize_schema(schema).unwrap();

        let client = filter_config_ir_by_scope(&ir, "client").unwrap();

        assert_eq!(client.tables.len(), 1);
        assert_eq!(client.tables[0].name, "Item");
        assert_eq!(
            client.tables[0]
                .fields
                .iter()
                .map(|field| field.name.as_str())
                .collect::<Vec<_>>(),
            ["id", "icon", "drop"]
        );
        assert_eq!(
            client.structs[0]
                .fields
                .iter()
                .map(|field| field.name.as_str())
                .collect::<Vec<_>>(),
            ["item_id"]
        );
    }
}
