use sora_ir::model::{ConfigIr, FieldIr, StructIr, TableIr, TableModeIr, TypeIr};

pub fn table_template_rows(ir: &ConfigIr, table: &TableIr) -> Vec<Vec<String>> {
    vec![
        vec!["@table".to_owned(), table.name.clone()],
        vec!["@mode".to_owned(), table_mode_name(table.mode).to_owned()],
        vec![
            "@key".to_owned(),
            table.key.as_deref().unwrap_or("").to_owned(),
        ],
        vec!["@scope".to_owned(), table.scope.display()],
        vec!["@schema".to_owned(), schema_hash(ir, table)],
        Vec::new(),
        std::iter::once("#name".to_owned())
            .chain(table.fields.iter().map(|field| field.name.clone()))
            .collect(),
        std::iter::once("#field".to_owned())
            .chain(table.fields.iter().map(|field| field.name.clone()))
            .collect(),
        std::iter::once("#type".to_owned())
            .chain(table.fields.iter().map(|field| field_type_hint(ir, field)))
            .collect(),
        std::iter::once("#scope".to_owned())
            .chain(table.fields.iter().map(|field| field.scope.display()))
            .collect(),
        std::iter::once("#rule".to_owned())
            .chain(table.fields.iter().map(field_rule))
            .collect(),
        std::iter::once("#desc".to_owned())
            .chain(
                table
                    .fields
                    .iter()
                    .map(|field| field.comment.clone().unwrap_or_default()),
            )
            .collect(),
    ]
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
            update(&mut hash, parser);
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
        update(&mut hash, field.separator.as_deref().unwrap_or(""));
        update(&mut hash, field.prefix.as_deref().unwrap_or(""));
        update(&mut hash, field.suffix.as_deref().unwrap_or(""));
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
    match struct_type_name(&field.ty)
        .and_then(|struct_name| struct_ir(ir, struct_name))
        .filter(|_| field.parser.as_deref() == Some("tuple"))
    {
        Some(struct_ir) => format!(
            "{}({})",
            field.ty,
            struct_ir
                .fields
                .iter()
                .map(|field| format!("{}: {}", field.name, field.ty))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        None => field.ty.to_string(),
    }
}

pub(crate) fn tuple_shape(ir: &ConfigIr, field: &FieldIr) -> Option<String> {
    if field.parser.as_deref() != Some("tuple") {
        return None;
    }
    let struct_name = struct_type_name(&field.ty)?;
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

fn field_rule(field: &FieldIr) -> String {
    let mut parts = Vec::new();
    if field.key {
        parts.push("key".to_owned());
    } else if field.required {
        parts.push("required".to_owned());
    } else {
        parts.push("optional".to_owned());
    }

    if let Some(separator) = &field.separator {
        parts.push(format!("separator={separator}"));
    }
    if let Some(parser) = &field.parser {
        parts.push(format!("parser={parser}"));
    }
    if let Some([min, max]) = field.range {
        parts.push(format!("range={min}..{max}"));
    }
    if let Some(prefix) = &field.prefix {
        parts.push(format!("prefix={prefix}"));
    }
    if let Some(suffix) = &field.suffix {
        parts.push(format!("suffix={suffix}"));
    }

    parts.join(";")
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

        assert_eq!(rows[0], ["@table", "Item"]);
        assert_eq!(rows[1], ["@mode", "map"]);
        assert_eq!(rows[2], ["@key", "id"]);
        assert_eq!(rows[3], ["@scope", "all"]);
        assert_eq!(rows[6], ["#name", "id", "name", "item_type", "max_stack"]);
        assert_eq!(rows[7], ["#field", "id", "name", "item_type", "max_stack"]);
        assert_eq!(rows[8], ["#type", "i32", "string", "enum<ItemType>", "i32"]);
        assert_eq!(rows[9], ["#scope", "all", "all", "all", "all"]);
        assert_eq!(
            rows[10],
            ["#rule", "key", "required", "required", "required"]
        );
        assert_eq!(
            rows[11],
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
    fn expands_tuple_struct_type_hints() {
        let ir = tuple_ir();
        let rows = table_template_rows(&ir, &ir.tables[0]);

        assert_eq!(
            rows[8],
            [
                "#type",
                "struct<ResourceCost>(kind: enum<ResourceType>, id: i32, count: i32)"
            ]
        );
        assert_eq!(rows[10], ["#rule", "required;separator=,;parser=tuple"]);
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
parser = "tuple"
separator = ","
"#,
        )
        .unwrap();
        normalize_schema(schema).unwrap()
    }
}
