use sora_ir::{
    input_projection::{TaggedColumnKind, tagged_columns},
    model::{ConfigIr, FieldIr, StructIr, TableIr, TableModeIr, TypeIr},
};

pub const METADATA_ROW: u32 = 0;
pub const NAME_ROW: u32 = 1;
pub const FIELD_ROW: u32 = 2;
pub const TYPE_ROW: u32 = 3;
pub const SCOPE_ROW: u32 = 4;
pub const RULE_ROW: u32 = 5;
pub const DESC_ROW: u32 = 6;
pub const DATA_START_ROW: u32 = 7;
pub const FIELD_START_COLUMN: u16 = 1;

pub fn table_template_rows(ir: &ConfigIr, table: &TableIr) -> Vec<Vec<String>> {
    let columns = table_template_columns(ir, table);
    vec![
        vec![
            "@table".to_owned(),
            table.name.clone(),
            "@mode".to_owned(),
            table_mode_name(table.mode).to_owned(),
            "@key".to_owned(),
            table.key.as_deref().unwrap_or("").to_owned(),
            "@scope".to_owned(),
            table.scope.display(),
            "@schema".to_owned(),
            schema_hash(ir, table),
        ],
        std::iter::once("#name".to_owned())
            .chain(columns.iter().map(|column| column.name.clone()))
            .collect(),
        std::iter::once("#field".to_owned())
            .chain(columns.iter().map(|column| column.name.clone()))
            .collect(),
        std::iter::once("#type".to_owned())
            .chain(columns.iter().map(|column| column.type_hint.clone()))
            .collect(),
        std::iter::once("#scope".to_owned())
            .chain(columns.iter().map(|column| column.scope.clone()))
            .collect(),
        std::iter::once("#rule".to_owned())
            .chain(columns.iter().map(|column| column.rule.clone()))
            .collect(),
        std::iter::once("#desc".to_owned())
            .chain(columns.iter().map(|column| column.comment.clone()))
            .collect(),
    ]
}

#[derive(Debug, Clone)]
pub struct TemplateColumn {
    pub name: String,
    pub type_hint: String,
    pub scope: String,
    pub rule: String,
    pub comment: String,
}

pub fn table_template_columns(ir: &ConfigIr, table: &TableIr) -> Vec<TemplateColumn> {
    table
        .fields
        .iter()
        .flat_map(|field| {
            tagged_columns(ir, field)
                .map(|columns| {
                    columns
                        .into_iter()
                        .map(|column| match column.kind {
                            TaggedColumnKind::Tag => TemplateColumn {
                                name: column.name,
                                type_hint: format!("{} tag", field.ty),
                                scope: field.scope.display(),
                                rule: format!(
                                    "{};parser=tagged_columns;tag",
                                    field_presence(field)
                                ),
                                comment: format!("Tag for union field `{}`.", field.name),
                            },
                            TaggedColumnKind::VariantField(variant_field) => TemplateColumn {
                                name: column.name,
                                type_hint: field_type_hint(ir, variant_field),
                                scope: field.scope.display(),
                                rule: field_rule(variant_field),
                                comment: variant_field.comment.clone().unwrap_or_default(),
                            },
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_else(|| {
                    vec![TemplateColumn {
                        name: field.name.clone(),
                        type_hint: field_type_hint(ir, field),
                        scope: field.scope.display(),
                        rule: field_rule(field),
                        comment: field.comment.clone().unwrap_or_default(),
                    }]
                })
        })
        .collect()
}

pub fn schema_hash(ir: &ConfigIr, table: &TableIr) -> String {
    let mut hash = 0xcbf29ce484222325_u64;
    fn update(hash: &mut u64, value: &str) {
        for byte in value.as_bytes() {
            *hash ^= u64::from(*byte);
            *hash = hash.wrapping_mul(0x100000001b3);
        }
        *hash ^= 0xff;
        *hash = hash.wrapping_mul(0x100000001b3);
    }

    update(&mut hash, &table.name);
    update(&mut hash, table_mode_name(table.mode));
    update(&mut hash, table.key.as_deref().unwrap_or(""));
    update(&mut hash, &table.scope.display());
    for field in &table.fields {
        update(&mut hash, &field.name);
        update(&mut hash, &field.ty.to_string());
        update(&mut hash, &field.scope.display());
        if let Some(parser) = &field.parser {
            update(&mut hash, &parser.kind);
            for (key, value) in &parser.options {
                update(&mut hash, key);
                update(&mut hash, value);
            }
        }
        if let Some(columns) = tagged_columns(ir, field) {
            for column in columns {
                update(&mut hash, &column.name);
                match column.kind {
                    TaggedColumnKind::Tag => update(&mut hash, "tag"),
                    TaggedColumnKind::VariantField(variant_field) => {
                        update(&mut hash, "variant_field");
                        update(&mut hash, &variant_field.name);
                        update(&mut hash, &variant_field.ty.to_string());
                        if let Some(parser) = &variant_field.parser {
                            update(&mut hash, &parser.kind);
                            for (key, value) in &parser.options {
                                update(&mut hash, key);
                                update(&mut hash, value);
                            }
                        }
                    }
                }
            }
        }
        if let Some(tuple_shape) = tuple_shape(ir, field) {
            update(&mut hash, &tuple_shape);
        }
        update(
            &mut hash,
            &field
                .range
                .map(|[min, max]| format!("{min}..{max}"))
                .unwrap_or_default(),
        );
        update(
            &mut hash,
            if field.required {
                "required"
            } else {
                "optional"
            },
        );
        update(&mut hash, if field.key { "key" } else { "" });
    }

    format!("{hash:016x}")
}

pub(crate) fn table_mode_name(mode: TableModeIr) -> &'static str {
    match mode {
        TableModeIr::List => "list",
        TableModeIr::Map => "map",
        TableModeIr::Singleton => "singleton",
    }
}

pub(crate) fn field_type_hint(ir: &ConfigIr, field: &FieldIr) -> String {
    match tuple_shape(ir, field) {
        Some(tuple_shape) => format!("{}({})", field.ty, tuple_shape),
        None => field.ty.to_string(),
    }
}

pub(crate) fn tuple_shape(ir: &ConfigIr, field: &FieldIr) -> Option<String> {
    let struct_name = tuple_struct_type_name(field)?;
    let struct_ir = struct_ir(ir, struct_name)?;
    Some(
        struct_ir
            .fields
            .iter()
            .map(|field| format!("{}: {}", field.name, field.ty))
            .collect::<Vec<_>>()
            .join(", "),
    )
}

fn tuple_struct_type_name(field: &FieldIr) -> Option<&str> {
    match field.parser.as_ref()?.kind.as_str() {
        "tuple" => struct_type_name(&field.ty),
        "tuple_list" => list_struct_type_name(&field.ty),
        _ => None,
    }
}

pub(crate) fn struct_ir<'a>(ir: &'a ConfigIr, name: &str) -> Option<&'a StructIr> {
    ir.structs.iter().find(|item| item.name == name)
}

fn struct_type_name(ty: &TypeIr) -> Option<&str> {
    match ty {
        TypeIr::Struct(name) => Some(name),
        TypeIr::Optional(inner) => struct_type_name(inner),
        _ => None,
    }
}

fn list_struct_type_name(ty: &TypeIr) -> Option<&str> {
    match ty {
        TypeIr::List(element) => struct_type_name(element),
        TypeIr::Array { element, .. } => struct_type_name(element),
        TypeIr::Optional(inner) => list_struct_type_name(inner),
        _ => None,
    }
}

fn field_rule(field: &FieldIr) -> String {
    let mut parts = Vec::new();
    parts.push(field_presence(field));

    if let Some(parser) = &field.parser {
        parts.push(format!("parser={}", parser.kind));
        for (key, value) in &parser.options {
            parts.push(format!("{key}={value}"));
        }
    }
    if let Some([min, max]) = field.range {
        parts.push(format!("range={min}..{max}"));
    }
    parts.join(";")
}

fn field_presence(field: &FieldIr) -> String {
    if field.key {
        "key".to_owned()
    } else if field.required {
        "required".to_owned()
    } else {
        "optional".to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sora_ir::{model::ConfigIr, normalize::normalize_schema};
    use sora_schema::model::SchemaFile;

    #[test]
    fn builds_schema_projection_rows() {
        let ir = example_ir();
        let rows = table_template_rows(&ir, &ir.tables[0]);

        assert_eq!(
            rows[METADATA_ROW as usize][..9],
            [
                "@table", "Item", "@mode", "map", "@key", "id", "@scope", "all", "@schema"
            ]
        );
        assert_eq!(
            rows[NAME_ROW as usize],
            ["#name", "id", "name", "item_type", "max_stack"]
        );
        assert_eq!(
            rows[FIELD_ROW as usize],
            ["#field", "id", "name", "item_type", "max_stack"]
        );
        assert_eq!(
            rows[TYPE_ROW as usize],
            ["#type", "i32", "string", "enum<ItemType>", "i32"]
        );
        assert_eq!(
            rows[SCOPE_ROW as usize],
            ["#scope", "all", "all", "all", "all"]
        );
        assert_eq!(
            rows[RULE_ROW as usize],
            ["#rule", "key", "required", "required", "required"]
        );
        assert_eq!(
            rows[DESC_ROW as usize],
            [
                "#desc",
                "Item id",
                "Display name",
                "Item type",
                "Max stack count"
            ]
        );
    }

    #[test]
    fn schema_hash_is_deterministic() {
        let ir = example_ir();

        assert_eq!(
            schema_hash(&ir, &ir.tables[0]),
            schema_hash(&ir, &ir.tables[0])
        );
    }

    #[test]
    fn tagged_union_projection_affects_schema_hash() {
        let ir = tagged_union_ir();
        let mut changed = ir.clone();
        changed.unions[0].variants[0].fields[0].name = "changed_quest_id".to_owned();

        assert_ne!(
            schema_hash(&ir, &ir.tables[0]),
            schema_hash(&changed, &changed.tables[0])
        );
    }

    #[test]
    fn expands_tuple_struct_type_hints() {
        let ir = tuple_ir();
        let rows = table_template_rows(&ir, &ir.tables[0]);

        assert_eq!(
            rows[TYPE_ROW as usize],
            [
                "#type",
                "struct<ResourceCost>(kind: enum<ResourceType>, id: i32, count: i32)"
            ]
        );
        assert_eq!(rows[RULE_ROW as usize], ["#rule", "required;parser=tuple"]);
    }

    #[test]
    fn expands_tuple_list_struct_type_hints() {
        let ir = tuple_list_ir();
        let rows = table_template_rows(&ir, &ir.tables[0]);

        assert_eq!(
            rows[TYPE_ROW as usize],
            [
                "#type",
                "list<struct<ResourceCost>>(kind: enum<ResourceType>, id: i32, count: i32)"
            ]
        );
        assert_eq!(
            rows[RULE_ROW as usize],
            ["#rule", "required;parser=tuple_list"]
        );
    }

    #[test]
    fn expands_tagged_union_columns() {
        let ir = tagged_union_ir();
        let rows = table_template_rows(&ir, &ir.tables[0]);

        assert_eq!(
            rows[FIELD_ROW as usize],
            ["#field", "id", "type", "quest_id", "item_id", "count"]
        );
        assert_eq!(
            rows[TYPE_ROW as usize],
            [
                "#type",
                "i32",
                "union<EventCondition> tag",
                "i32",
                "i32",
                "i32"
            ]
        );
        assert_eq!(
            rows[RULE_ROW as usize],
            [
                "#rule",
                "key",
                "required;parser=tagged_columns;tag",
                "required",
                "required",
                "required"
            ]
        );
    }

    fn example_ir() -> ConfigIr {
        let schema: SchemaFile = toml::from_str(
            r#"
package = "game_config"

[[enums]]
name = "ItemType"
values = ["Weapon", "Armor", "Material", "Consumable"]

[[tables]]
name = "Item"
mode = "map"
key = "id"
[tables.source]
format = "xlsx"
file = "Item.xlsx"
sheet = "Item"

[[tables.fields]]
name = "id"
type = "i32"
key = true
required = true
comment = "Item id"

[[tables.fields]]
name = "name"
type = "string"
required = true
comment = "Display name"

[[tables.fields]]
name = "item_type"
type = "enum<ItemType>"
required = true
comment = "Item type"

[[tables.fields]]
name = "max_stack"
type = "i32"
required = true
comment = "Max stack count"
"#,
        )
        .unwrap();
        normalize_schema(schema).unwrap()
    }

    fn tuple_ir() -> ConfigIr {
        let schema: SchemaFile = toml::from_str(
            r#"
package = "game_config"

[[enums]]
name = "ResourceType"
values = ["Item"]

[[structs]]
name = "ResourceCost"

[[structs.fields]]
name = "kind"
type = "enum<ResourceType>"

[[structs.fields]]
name = "id"
type = "i32"

[[structs.fields]]
name = "count"
type = "i32"

[[tables]]
name = "Reward"
mode = "list"

[[tables.fields]]
name = "cost"
type = "struct<ResourceCost>"
required = true
parser = { kind = "tuple" }
"#,
        )
        .unwrap();
        normalize_schema(schema).unwrap()
    }

    fn tuple_list_ir() -> ConfigIr {
        let schema: SchemaFile = toml::from_str(
            r#"
package = "game_config"

[[enums]]
name = "ResourceType"
values = ["Item"]

[[structs]]
name = "ResourceCost"

[[structs.fields]]
name = "kind"
type = "enum<ResourceType>"

[[structs.fields]]
name = "id"
type = "i32"

[[structs.fields]]
name = "count"
type = "i32"

[[tables]]
name = "Recipe"
mode = "list"

[[tables.fields]]
name = "materials"
type = "list<ResourceCost>"
required = true
parser = { kind = "tuple_list" }
"#,
        )
        .unwrap();
        normalize_schema(schema).unwrap()
    }

    fn tagged_union_ir() -> ConfigIr {
        let schema: SchemaFile = toml::from_str(
            r#"
package = "game_config"

[[unions]]
name = "EventCondition"
tag = "type"

[[unions.variants]]
name = "QuestCompleted"

[[unions.variants.fields]]
name = "quest_id"
type = "i32"
required = true

[[unions.variants]]
name = "HasItem"

[[unions.variants.fields]]
name = "item_id"
type = "i32"
required = true

[[unions.variants.fields]]
name = "count"
type = "i32"
required = true

[[tables]]
name = "EventConditionEntry"
mode = "map"
key = "id"

[[tables.fields]]
name = "id"
type = "i32"
key = true
required = true

[[tables.fields]]
name = "value"
type = "union<EventCondition>"
required = true
parser = { kind = "tagged_columns", prefix = "" }
"#,
        )
        .unwrap();
        normalize_schema(schema).unwrap()
    }
}
