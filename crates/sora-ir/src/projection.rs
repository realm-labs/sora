use std::collections::{BTreeMap, BTreeSet};

use sora_diagnostics::{Result, SoraError};

use crate::{
    model::{
        ConfigIr, FieldIr, IndexIr, StructIr, TableIr, TypeIr, UnionIr, UnionVariantIr, ViewIr,
    },
    validate::validate_config_ir,
};

/// Resolves one declared consumer view into the exact schema consumed by every
/// generator and exporter.
pub fn project_config_ir(ir: &ConfigIr, view_name: &str) -> Result<ConfigIr> {
    let view = ir.views.get(view_name).ok_or_else(|| {
        SoraError::InvalidSchema(format!(
            "unknown view `{view_name}`; declared views: {}",
            ir.views.keys().cloned().collect::<Vec<_>>().join(", ")
        ))
    })?;
    validate_view_table_references(ir, view_name, view)?;

    let selected_tables = ir
        .tables
        .iter()
        .filter(|table| table.groups.intersects(&view.groups))
        .filter(|table| table_is_selected(table, view))
        .map(|table| table.canonical_name.as_str())
        .collect::<BTreeSet<_>>();

    let mut projected = ConfigIr {
        project_id: ir.project_id.clone(),
        contract_id: view.contract.clone(),
        view: Some(view_name.to_owned()),
        group_defaults: ir.group_defaults.clone(),
        views: ir.views.clone(),
        localization: ir.localization.clone(),
        enums: ir
            .enums
            .iter()
            .filter(|item| item.groups.intersects(&view.groups))
            .cloned()
            .collect(),
        structs: ir
            .structs
            .iter()
            .filter(|item| item.groups.intersects(&view.groups))
            .map(|item| project_struct(item, view))
            .collect(),
        unions: ir
            .unions
            .iter()
            .filter(|item| item.groups.intersects(&view.groups))
            .map(|item| project_union(item, view))
            .collect(),
        tables: ir
            .tables
            .iter()
            .filter(|table| selected_tables.contains(table.canonical_name.as_str()))
            .map(|table| project_table(table, view))
            .collect(),
    };

    lower_hidden_table_references(ir, &selected_tables, &mut projected)?;
    prune_unreachable_types(&mut projected);
    apply_table_names(view_name, view, &mut projected)?;
    validate_config_ir(&projected)?;
    Ok(projected)
}

/// Validates every declared external contract, including table selection,
/// aliases, hidden-reference lowering, and the resulting projected schema.
pub fn validate_config_views(ir: &ConfigIr) -> Result<()> {
    for view_name in ir.views.keys() {
        project_config_ir(ir, view_name)?;
    }
    Ok(())
}

pub fn view_binding<'a>(
    ir: &'a ConfigIr,
    view_name: &str,
    target: &str,
) -> Result<Option<&'a serde_json::Value>> {
    let view = ir
        .views
        .get(view_name)
        .ok_or_else(|| SoraError::InvalidSchema(format!("unknown view `{view_name}`")))?;
    Ok(view.bindings.get(target))
}

fn table_is_selected(table: &TableIr, view: &ViewIr) -> bool {
    let included = view.tables.include.is_empty()
        || view.tables.include.iter().any(|item| item == "*")
        || view.tables.include.iter().any(|item| item == &table.id);
    included && !view.tables.exclude.iter().any(|item| item == &table.id)
}

fn validate_view_table_references(ir: &ConfigIr, view_name: &str, view: &ViewIr) -> Result<()> {
    let table_ids = ir
        .tables
        .iter()
        .map(|table| table.id.as_str())
        .collect::<BTreeSet<_>>();
    for table_id in view
        .tables
        .include
        .iter()
        .chain(&view.tables.exclude)
        .filter(|table_id| table_id.as_str() != "*")
        .chain(view.table_names.keys())
    {
        if !table_ids.contains(table_id.as_str()) {
            return Err(SoraError::InvalidSchema(format!(
                "view `{view_name}` references unknown table id `{table_id}`"
            )));
        }
    }
    for (rule, values) in [
        ("include", &view.tables.include),
        ("exclude", &view.tables.exclude),
    ] {
        let mut seen = BTreeSet::new();
        for table_id in values {
            if !seen.insert(table_id) {
                return Err(SoraError::InvalidSchema(format!(
                    "view `{view_name}` repeats table id `{table_id}` in its {rule} list"
                )));
            }
        }
    }
    for table_id in &view.tables.include {
        if table_id != "*" && view.tables.exclude.contains(table_id) {
            return Err(SoraError::InvalidSchema(format!(
                "view `{view_name}` both includes and excludes table id `{table_id}`"
            )));
        }
    }
    if view.tables.exclude.iter().any(|item| item == "*") {
        return Err(SoraError::InvalidSchema(format!(
            "view `{view_name}` cannot exclude `*`; declare an explicit empty view instead"
        )));
    }
    Ok(())
}

fn project_struct(item: &StructIr, view: &ViewIr) -> StructIr {
    StructIr {
        name: item.name.clone(),
        groups: item.groups.clone(),
        fields: project_fields(&item.fields, view),
    }
}

fn project_union(item: &UnionIr, view: &ViewIr) -> UnionIr {
    UnionIr {
        name: item.name.clone(),
        groups: item.groups.clone(),
        tag: item.tag.clone(),
        variants: item
            .variants
            .iter()
            .filter(|variant| variant.groups.intersects(&view.groups))
            .map(|variant| project_union_variant(variant, view))
            .collect(),
    }
}

fn project_union_variant(item: &UnionVariantIr, view: &ViewIr) -> UnionVariantIr {
    UnionVariantIr {
        name: item.name.clone(),
        groups: item.groups.clone(),
        fields: project_fields(&item.fields, view),
    }
}

fn project_table(item: &TableIr, view: &ViewIr) -> TableIr {
    let fields = project_fields(&item.fields, view);
    let field_names = fields
        .iter()
        .map(|field| field.name.as_str())
        .collect::<BTreeSet<_>>();
    TableIr {
        id: item.id.clone(),
        name: item.name.clone(),
        canonical_name: item.canonical_name.clone(),
        groups: item.groups.clone(),
        mode: item.mode,
        key: item.key.clone(),
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

fn project_fields(fields: &[FieldIr], view: &ViewIr) -> Vec<FieldIr> {
    fields
        .iter()
        .filter(|field| field.groups.intersects(&view.groups))
        .cloned()
        .map(|mut field| {
            // Derived values are materialized against the canonical schema
            // before projection. Their source relationship is not part of the
            // consumer contract.
            field.derived_from = None;
            field
        })
        .collect()
}

fn lower_hidden_table_references(
    canonical: &ConfigIr,
    selected_tables: &BTreeSet<&str>,
    projected: &mut ConfigIr,
) -> Result<()> {
    for item in &mut projected.structs {
        lower_fields(canonical, selected_tables, &mut item.fields)?;
    }
    for item in &mut projected.unions {
        for variant in &mut item.variants {
            lower_fields(canonical, selected_tables, &mut variant.fields)?;
        }
    }
    for table in &mut projected.tables {
        lower_fields(canonical, selected_tables, &mut table.fields)?;
    }
    Ok(())
}

fn lower_fields(
    canonical: &ConfigIr,
    selected_tables: &BTreeSet<&str>,
    fields: &mut [FieldIr],
) -> Result<()> {
    for field in fields {
        field.ty = lower_type(canonical, selected_tables, &field.ty, &mut BTreeSet::new())?;
    }
    Ok(())
}

fn lower_type(
    canonical: &ConfigIr,
    selected_tables: &BTreeSet<&str>,
    ty: &TypeIr,
    resolving: &mut BTreeSet<(String, String)>,
) -> Result<TypeIr> {
    Ok(match ty {
        TypeIr::Ref { table, field } if !selected_tables.contains(table.as_str()) => {
            let key = (table.clone(), field.clone());
            if !resolving.insert(key.clone()) {
                return Err(SoraError::InvalidSchema(format!(
                    "reference cycle while projecting hidden table field `{table}.{field}`"
                )));
            }
            let target = canonical
                .tables
                .iter()
                .find(|candidate| candidate.canonical_name == *table)
                .and_then(|table| {
                    table
                        .fields
                        .iter()
                        .find(|candidate| candidate.name == *field)
                })
                .ok_or_else(|| {
                    SoraError::InvalidSchema(format!(
                        "reference targets unknown field `{table}.{field}`"
                    ))
                })?;
            let lowered = lower_type(canonical, selected_tables, &target.ty, resolving)?;
            resolving.remove(&key);
            lowered
        }
        TypeIr::List(element) => TypeIr::List(Box::new(lower_type(
            canonical,
            selected_tables,
            element,
            resolving,
        )?)),
        TypeIr::Set(element) => TypeIr::Set(Box::new(lower_type(
            canonical,
            selected_tables,
            element,
            resolving,
        )?)),
        TypeIr::Optional(element) => TypeIr::Optional(Box::new(lower_type(
            canonical,
            selected_tables,
            element,
            resolving,
        )?)),
        TypeIr::Array { element, len } => TypeIr::Array {
            element: Box::new(lower_type(canonical, selected_tables, element, resolving)?),
            len: *len,
        },
        TypeIr::Map { key, value } => TypeIr::Map {
            key: Box::new(lower_type(canonical, selected_tables, key, resolving)?),
            value: Box::new(lower_type(canonical, selected_tables, value, resolving)?),
        },
        other => other.clone(),
    })
}

fn apply_table_names(view_name: &str, view: &ViewIr, projected: &mut ConfigIr) -> Result<()> {
    let selected_ids = projected
        .tables
        .iter()
        .map(|table| table.id.as_str())
        .collect::<BTreeSet<_>>();
    for table_id in view.table_names.keys() {
        if !selected_ids.contains(table_id.as_str()) {
            return Err(SoraError::InvalidSchema(format!(
                "view `{view_name}` renames table `{table_id}` but does not select it"
            )));
        }
    }

    let mut canonical_to_output = BTreeMap::new();
    let mut output_names = BTreeSet::new();
    for table in &mut projected.tables {
        let output_name = view
            .table_names
            .get(&table.id)
            .cloned()
            .unwrap_or_else(|| table.canonical_name.clone());
        if output_name.is_empty() {
            return Err(SoraError::InvalidSchema(format!(
                "view `{view_name}` gives table `{}` an empty output name",
                table.id
            )));
        }
        if !output_names.insert(output_name.clone()) {
            return Err(SoraError::InvalidSchema(format!(
                "view `{view_name}` produces duplicate table name `{output_name}`"
            )));
        }
        canonical_to_output.insert(table.canonical_name.clone(), output_name.clone());
        table.name = output_name;
    }

    for item in &mut projected.structs {
        rename_field_types(&canonical_to_output, &mut item.fields);
    }
    for item in &mut projected.unions {
        for variant in &mut item.variants {
            rename_field_types(&canonical_to_output, &mut variant.fields);
        }
    }
    for table in &mut projected.tables {
        rename_field_types(&canonical_to_output, &mut table.fields);
    }
    Ok(())
}

fn rename_field_types(names: &BTreeMap<String, String>, fields: &mut [FieldIr]) {
    for field in fields {
        rename_type_tables(names, &mut field.ty);
    }
}

fn rename_type_tables(names: &BTreeMap<String, String>, ty: &mut TypeIr) {
    match ty {
        TypeIr::Ref { table, .. } => {
            if let Some(output) = names.get(table) {
                *table = output.clone();
            }
        }
        TypeIr::List(element)
        | TypeIr::Set(element)
        | TypeIr::Optional(element)
        | TypeIr::Array { element, .. } => rename_type_tables(names, element),
        TypeIr::Map { key, value } => {
            rename_type_tables(names, key);
            rename_type_tables(names, value);
        }
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
        | TypeIr::F32
        | TypeIr::F64
        | TypeIr::String
        | TypeIr::Text
        | TypeIr::Enum(_)
        | TypeIr::Struct(_)
        | TypeIr::Union(_) => {}
    }
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
        TypeIr::List(element)
        | TypeIr::Set(element)
        | TypeIr::Optional(element)
        | TypeIr::Array { element, .. } => {
            collect_type_names(element, enum_names, struct_names, union_names);
        }
        TypeIr::Map { key, value } => {
            collect_type_names(key, enum_names, struct_names, union_names);
            collect_type_names(value, enum_names, struct_names, union_names);
        }
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
        | TypeIr::F32
        | TypeIr::F64
        | TypeIr::String
        | TypeIr::Text
        | TypeIr::Ref { .. } => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{model::TypeIr, normalize::normalize_schema};
    use sora_schema::model::SchemaFile;

    #[test]
    fn projects_tables_by_stable_id_and_applies_external_names() {
        let ir = example_ir();
        let projected = project_config_ir(&ir, "client").unwrap();

        assert_eq!(projected.project_id, "game");
        assert_eq!(projected.contract_id, "game/client");
        assert_eq!(projected.view.as_deref(), Some("client"));
        assert_eq!(projected.tables.len(), 1);
        assert_eq!(projected.tables[0].id, "item");
        assert_eq!(projected.tables[0].canonical_name, "Item");
        assert_eq!(projected.tables[0].name, "Items");
        assert_eq!(projected.tables[0].fields[1].ty, TypeIr::String);
        assert_eq!(
            view_binding(&ir, "client", "csharp").unwrap().unwrap()["namespace"],
            "Game.Client"
        );
    }

    #[test]
    fn rejects_conflicting_table_selection_rules() {
        let mut ir = example_ir();
        ir.views
            .get_mut("client")
            .unwrap()
            .tables
            .exclude
            .push("item".to_owned());

        let error = project_config_ir(&ir, "client").unwrap_err();
        assert!(
            error
                .to_string()
                .contains("both includes and excludes table id `item`")
        );
    }

    fn example_ir() -> ConfigIr {
        let schema: SchemaFile = toml::from_str(
            r#"
project = { id = "game" }
groups = { common = { default = true }, server = { default = false } }

[views.client]
contract = "game/client"
groups = ["common"]
tables = { include = ["item"] }
names = { tables = { item = "Items" } }

[views.client.bindings.csharp]
namespace = "Game.Client"

[[tables]]
id = "secret"
name = "Secret"
groups = "server"
mode = "map"
key = "code"

[[tables.fields]]
name = "code"
type = "string"

[[tables]]
id = "item"
name = "Item"
mode = "map"
key = "id"

[[tables.fields]]
name = "id"
type = "i32"

[[tables.fields]]
name = "secret"
type = "ref<Secret.code>"
"#,
        )
        .unwrap();
        normalize_schema(schema).unwrap()
    }
}
