use sora_diagnostics::{Result, SoraError};
use sora_schema::model::{
    FieldSchema, IndexSchema, SchemaFile, TableModeSchema, TableSchema, TableSourceSchema,
};

use crate::{
    model::{
        AggregationIr, ConfigIr, EnumIr, FieldIr, IndexIr, StructIr, TableIr, TableModeIr,
        TableSourceIr, TypeIr,
    },
    parse::parse_type,
};

pub fn normalize_schema(schema: SchemaFile) -> Result<ConfigIr> {
    ConfigIr::try_from(schema)
}

impl TryFrom<SchemaFile> for ConfigIr {
    type Error = SoraError;

    fn try_from(schema: SchemaFile) -> Result<Self> {
        Ok(Self {
            package: schema.package,
            enums: schema
                .enums
                .into_iter()
                .map(|item| EnumIr {
                    name: item.name,
                    values: item.values,
                })
                .collect(),
            structs: schema
                .structs
                .into_iter()
                .map(|item| {
                    Ok(StructIr {
                        name: item.name,
                        fields: convert_fields(item.fields)?,
                    })
                })
                .collect::<Result<Vec<_>>>()?,
            tables: schema
                .tables
                .into_iter()
                .map(TableIr::try_from)
                .collect::<Result<Vec<_>>>()?,
        })
    }
}

impl TryFrom<TableSchema> for TableIr {
    type Error = SoraError;

    fn try_from(table: TableSchema) -> Result<Self> {
        Ok(Self {
            name: table.name,
            mode: table.mode.into(),
            key: table.key,
            source: table.source.map(Into::into),
            fields: convert_fields(table.fields)?,
            indexes: table.indexes.into_iter().map(IndexIr::from).collect(),
        })
    }
}

impl TryFrom<FieldSchema> for FieldIr {
    type Error = SoraError;

    fn try_from(field: FieldSchema) -> Result<Self> {
        let aggregation = match (field.source_table, field.parent_key, field.child_key) {
            (None, None, None) => None,
            (Some(source_table), Some(parent_key), Some(child_key)) => Some(AggregationIr {
                source_table,
                parent_key,
                child_key,
                order_by: field.order_by,
            }),
            _ => {
                return Err(SoraError::InvalidSchema(format!(
                    "field `{}` has incomplete aggregation metadata",
                    field.name
                )));
            }
        };

        let ty = parse_type(&field.ty)?;
        if field.default.is_some() && aggregation.is_some() {
            return Err(SoraError::InvalidSchema(format!(
                "field `{}` declares both `default` and aggregation metadata",
                field.name
            )));
        }
        if aggregation.is_none() {
            validate_parser_format(
                &field.name,
                &ty,
                field.parser.as_deref(),
                field.separator.as_deref(),
                field.prefix.as_deref(),
                field.suffix.as_deref(),
            )?;
            if field.parser.is_none() {
                validate_collection_format(
                    &field.name,
                    &ty,
                    field.separator.as_deref(),
                    field.prefix.as_deref(),
                    field.suffix.as_deref(),
                )?;
            }
        } else {
            validate_optional_non_empty(&field.name, "parser", field.parser.as_deref())?;
            validate_optional_non_empty(&field.name, "separator", field.separator.as_deref())?;
            validate_optional_non_empty(&field.name, "prefix", field.prefix.as_deref())?;
            validate_optional_non_empty(&field.name, "suffix", field.suffix.as_deref())?;
        }

        Ok(Self {
            name: field.name,
            ty,
            key: field.key,
            comment: field.comment,
            required: field.required.unwrap_or(false),
            default: field.default,
            range: field.range,
            parser: field.parser,
            separator: field.separator,
            prefix: field.prefix,
            suffix: field.suffix,
            aggregation,
        })
    }
}

impl From<TableModeSchema> for TableModeIr {
    fn from(mode: TableModeSchema) -> Self {
        match mode {
            TableModeSchema::List => Self::List,
            TableModeSchema::Map => Self::Map,
            TableModeSchema::Singleton => Self::Singleton,
        }
    }
}

impl From<IndexSchema> for IndexIr {
    fn from(index: IndexSchema) -> Self {
        Self {
            name: index.name,
            fields: index.fields,
            unique: index.unique,
        }
    }
}

impl From<TableSourceSchema> for TableSourceIr {
    fn from(source: TableSourceSchema) -> Self {
        Self {
            format: source.format,
            file: source.file,
            sheet: source.sheet,
        }
    }
}

fn convert_fields(fields: Vec<FieldSchema>) -> Result<Vec<FieldIr>> {
    fields.into_iter().map(FieldIr::try_from).collect()
}

fn validate_parser_format(
    field_name: &str,
    ty: &TypeIr,
    parser: Option<&str>,
    separator: Option<&str>,
    prefix: Option<&str>,
    suffix: Option<&str>,
) -> Result<()> {
    let Some(parser) = parser else {
        return Ok(());
    };

    match parser {
        "tuple" => {
            validate_tuple_target(field_name, ty)?;
            validate_required_non_empty(field_name, ty, "separator", separator)?;
            validate_optional_non_empty(field_name, "prefix", prefix)?;
            validate_optional_non_empty(field_name, "suffix", suffix)
        }
        _ => Err(SoraError::InvalidSchema(format!(
            "field `{field_name}` declares unsupported parser `{parser}`"
        ))),
    }
}

fn validate_tuple_target(field_name: &str, ty: &TypeIr) -> Result<()> {
    match ty {
        TypeIr::Struct(_) => Ok(()),
        TypeIr::Optional(inner) => validate_tuple_target(field_name, inner),
        _ => Err(SoraError::InvalidSchema(format!(
            "field `{field_name}` declares parser `tuple` but type `{ty}` is not struct"
        ))),
    }
}

fn validate_collection_format(
    field_name: &str,
    ty: &TypeIr,
    separator: Option<&str>,
    prefix: Option<&str>,
    suffix: Option<&str>,
) -> Result<()> {
    match ty {
        TypeIr::List(_) | TypeIr::Array { .. } => {
            validate_required_non_empty(field_name, ty, "separator", separator)?;
            validate_optional_non_empty(field_name, "prefix", prefix)?;
            validate_optional_non_empty(field_name, "suffix", suffix)?;
            Ok(())
        }
        TypeIr::Optional(inner) => {
            validate_collection_format(field_name, inner, separator, prefix, suffix)
        }
        _ if separator.is_some() || prefix.is_some() || suffix.is_some() => {
            Err(SoraError::InvalidSchema(format!(
                "field `{field_name}` declares collection format metadata but type `{ty}` is not list or array"
            )))
        }
        _ => Ok(()),
    }
}

fn validate_required_non_empty(
    field_name: &str,
    ty: &TypeIr,
    key: &'static str,
    value: Option<&str>,
) -> Result<()> {
    match value {
        Some(value) if !value.is_empty() => Ok(()),
        _ => Err(SoraError::InvalidSchema(format!(
            "field `{field_name}` with type `{ty}` must declare non-empty `{key}`"
        ))),
    }
}

fn validate_optional_non_empty(
    field_name: &str,
    key: &'static str,
    value: Option<&str>,
) -> Result<()> {
    match value {
        Some("") => Err(SoraError::InvalidSchema(format!(
            "field `{field_name}` declares empty `{key}`"
        ))),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{TableModeIr, TypeIr};

    #[test]
    fn normalizes_schema() {
        let schema: SchemaFile = toml::from_str(
            r#"
package = "game_config"

[[enums]]
name = "ItemType"
values = ["Weapon"]

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
name = "tags"
type = "list<string>"
separator = "|"
prefix = "["
suffix = "]"
default = "[\"starter\"]"
"#,
        )
        .unwrap();

        let ir = normalize_schema(schema).unwrap();
        assert_eq!(ir.package, "game_config");
        assert_eq!(ir.enums[0].name, "ItemType");
        assert_eq!(ir.tables[0].mode, TableModeIr::Map);
        assert!(ir.tables[0].fields[0].required);
        assert_eq!(ir.tables[0].fields[0].ty, TypeIr::I32);
        assert_eq!(ir.tables[0].fields[1].separator.as_deref(), Some("|"));
        assert_eq!(ir.tables[0].fields[1].prefix.as_deref(), Some("["));
        assert_eq!(ir.tables[0].fields[1].suffix.as_deref(), Some("]"));
        assert_eq!(
            ir.tables[0].fields[1].default.as_deref(),
            Some("[\"starter\"]")
        );
    }

    #[test]
    fn normalizes_tuple_struct_parser() {
        let schema: SchemaFile = toml::from_str(
            r#"
package = "game_config"

[[structs]]
name = "Vec3"

[[structs.fields]]
name = "x"
type = "f32"

[[structs.fields]]
name = "y"
type = "f32"

[[structs.fields]]
name = "z"
type = "f32"

[[tables]]
name = "Spawn"
mode = "list"

[[tables.fields]]
name = "pos"
type = "struct<Vec3>"
parser = "tuple"
separator = ","
"#,
        )
        .unwrap();

        let ir = normalize_schema(schema).unwrap();
        assert_eq!(ir.tables[0].fields[0].parser.as_deref(), Some("tuple"));
        assert_eq!(ir.tables[0].fields[0].separator.as_deref(), Some(","));
    }

    #[test]
    fn validates_collection_separator_metadata() {
        let missing_separator: SchemaFile = toml::from_str(
            r#"
package = "game_config"

[[tables]]
name = "Item"
mode = "list"

[[tables.fields]]
name = "tags"
type = "list<string>"
"#,
        )
        .unwrap();
        assert!(matches!(
            normalize_schema(missing_separator).unwrap_err(),
            SoraError::InvalidSchema(message) if message.contains("must declare non-empty `separator`")
        ));

        let invalid_separator: SchemaFile = toml::from_str(
            r#"
package = "game_config"

[[tables]]
name = "Item"
mode = "list"

[[tables.fields]]
name = "name"
type = "string"
separator = "|"
"#,
        )
        .unwrap();
        assert!(matches!(
            normalize_schema(invalid_separator).unwrap_err(),
            SoraError::InvalidSchema(message) if message.contains("collection format metadata")
        ));

        let scalar_prefix: SchemaFile = toml::from_str(
            r#"
package = "game_config"

[[tables]]
name = "Item"
mode = "list"

[[tables.fields]]
name = "name"
type = "string"
prefix = "["
"#,
        )
        .unwrap();
        assert!(matches!(
            normalize_schema(scalar_prefix).unwrap_err(),
            SoraError::InvalidSchema(message) if message.contains("collection format metadata")
        ));
    }

    #[test]
    fn rejects_invalid_tuple_parser_metadata() {
        let missing_separator: SchemaFile = toml::from_str(
            r#"
package = "game_config"

[[structs]]
name = "Vec2"

[[tables]]
name = "Spawn"
mode = "list"

[[tables.fields]]
name = "pos"
type = "struct<Vec2>"
parser = "tuple"
"#,
        )
        .unwrap();
        assert!(matches!(
            normalize_schema(missing_separator).unwrap_err(),
            SoraError::InvalidSchema(message) if message.contains("must declare non-empty `separator`")
        ));

        let scalar_parser: SchemaFile = toml::from_str(
            r#"
package = "game_config"

[[tables]]
name = "Spawn"
mode = "list"

[[tables.fields]]
name = "pos"
type = "string"
parser = "tuple"
separator = ","
"#,
        )
        .unwrap();
        assert!(matches!(
            normalize_schema(scalar_parser).unwrap_err(),
            SoraError::InvalidSchema(message) if message.contains("is not struct")
        ));
    }

    #[test]
    fn aggregation_list_fields_do_not_need_separator_metadata() {
        let schema: SchemaFile = toml::from_str(
            r#"
package = "game_config"

[[structs]]
name = "Reward"

[[structs.fields]]
name = "count"
type = "i32"

[[tables]]
name = "Item"
mode = "map"
key = "id"

[[tables.fields]]
name = "id"
type = "i32"

[[tables.fields]]
name = "rewards"
type = "list<Reward>"
source_table = "ItemReward"
parent_key = "id"
child_key = "item_id"

[[tables]]
name = "ItemReward"
mode = "list"
"#,
        )
        .unwrap();

        let ir = normalize_schema(schema).unwrap();
        assert!(ir.tables[0].fields[1].aggregation.is_some());
        assert_eq!(ir.tables[0].fields[1].separator, None);
    }

    #[test]
    fn rejects_default_on_aggregation_fields() {
        let schema: SchemaFile = toml::from_str(
            r#"
package = "game_config"

[[structs]]
name = "Reward"

[[tables]]
name = "Item"
mode = "map"
key = "id"

[[tables.fields]]
name = "id"
type = "i32"

[[tables.fields]]
name = "rewards"
type = "list<Reward>"
source_table = "ItemReward"
parent_key = "id"
child_key = "item_id"
default = "[]"

[[tables]]
name = "ItemReward"
mode = "list"
"#,
        )
        .unwrap();

        assert!(matches!(
            normalize_schema(schema).unwrap_err(),
            SoraError::InvalidSchema(message)
                if message.contains("declares both `default` and aggregation")
        ));
    }
}
