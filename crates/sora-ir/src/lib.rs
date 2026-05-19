use std::fmt;

use serde::{Deserialize, Serialize};
use sora_diagnostics::{Result, SoraError};
use sora_schema::{FieldSchema, IndexSchema, SchemaFile, TableModeSchema, TableSchema};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfigIr {
    pub package: String,
    pub enums: Vec<EnumIr>,
    pub structs: Vec<StructIr>,
    pub tables: Vec<TableIr>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnumIr {
    pub name: String,
    pub values: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructIr {
    pub name: String,
    pub fields: Vec<FieldIr>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableIr {
    pub name: String,
    pub mode: TableModeIr,
    pub key: Option<String>,
    pub source: Option<String>,
    pub fields: Vec<FieldIr>,
    pub indexes: Vec<IndexIr>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TableModeIr {
    List,
    Map,
    Singleton,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexIr {
    pub name: String,
    pub fields: Vec<String>,
    pub unique: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldIr {
    pub name: String,
    pub ty: TypeIr,
    pub key: bool,
    pub comment: Option<String>,
    pub required: bool,
    pub parser: Option<String>,
    pub aggregation: Option<AggregationIr>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AggregationIr {
    pub source_table: String,
    pub parent_key: String,
    pub child_key: String,
    pub order_by: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TypeIr {
    Bool,
    I32,
    I64,
    F32,
    F64,
    String,
    Enum(String),
    Struct(String),
    List(Box<TypeIr>),
    Array { element: Box<TypeIr>, len: usize },
    Ref { table: String, field: String },
    Optional(Box<TypeIr>),
}

impl fmt::Display for TypeIr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeIr::Bool => f.write_str("bool"),
            TypeIr::I32 => f.write_str("i32"),
            TypeIr::I64 => f.write_str("i64"),
            TypeIr::F32 => f.write_str("f32"),
            TypeIr::F64 => f.write_str("f64"),
            TypeIr::String => f.write_str("string"),
            TypeIr::Enum(name) => write!(f, "enum<{name}>"),
            TypeIr::Struct(name) => write!(f, "struct<{name}>"),
            TypeIr::List(element) => write!(f, "list<{element}>"),
            TypeIr::Array { element, len } => write!(f, "array<{element},{len}>"),
            TypeIr::Ref { table, field } => write!(f, "ref<{table}.{field}>"),
            TypeIr::Optional(element) => write!(f, "optional<{element}>"),
        }
    }
}

pub fn normalize_schema(schema: SchemaFile) -> Result<ConfigIr> {
    ConfigIr::try_from(schema)
}

pub fn parse_type(input: &str) -> Result<TypeIr> {
    parse_type_inner(input.trim())
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
            source: table.source,
            fields: convert_fields(table.fields)?,
            indexes: table.indexes.into_iter().map(IndexIr::from).collect(),
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

fn convert_fields(fields: Vec<FieldSchema>) -> Result<Vec<FieldIr>> {
    fields.into_iter().map(FieldIr::try_from).collect()
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

        Ok(Self {
            name: field.name,
            ty: parse_type(&field.ty)?,
            key: field.key,
            comment: field.comment,
            required: field.required.unwrap_or(false),
            parser: field.parser,
            aggregation,
        })
    }
}

fn parse_type_inner(input: &str) -> Result<TypeIr> {
    if input.is_empty() {
        return Err(SoraError::InvalidType(input.to_owned()));
    }

    Ok(match input {
        "bool" => TypeIr::Bool,
        "i32" => TypeIr::I32,
        "i64" => TypeIr::I64,
        "f32" => TypeIr::F32,
        "f64" => TypeIr::F64,
        "string" => TypeIr::String,
        _ => {
            if let Some(inner) = generic_inner(input, "enum") {
                require_identifier(inner)?;
                TypeIr::Enum(inner.to_owned())
            } else if let Some(inner) = generic_inner(input, "struct") {
                require_identifier(inner)?;
                TypeIr::Struct(inner.to_owned())
            } else if let Some(inner) = generic_inner(input, "list") {
                TypeIr::List(Box::new(parse_nested_type(inner)?))
            } else if let Some(inner) = generic_inner(input, "optional") {
                TypeIr::Optional(Box::new(parse_nested_type(inner)?))
            } else if let Some(inner) = generic_inner(input, "array") {
                parse_array_type(input, inner)?
            } else if let Some(inner) = generic_inner(input, "ref") {
                parse_ref_type(input, inner)?
            } else if is_identifier(input) {
                TypeIr::Struct(input.to_owned())
            } else {
                return Err(SoraError::UnknownType(input.to_owned()));
            }
        }
    })
}

fn parse_nested_type(input: &str) -> Result<TypeIr> {
    parse_type_inner(input.trim())
}

fn generic_inner<'a>(input: &'a str, name: &str) -> Option<&'a str> {
    let prefix = format!("{name}<");
    input
        .strip_prefix(&prefix)
        .and_then(|rest| rest.strip_suffix('>'))
}

fn parse_array_type(original: &str, inner: &str) -> Result<TypeIr> {
    let (element, len) = inner
        .rsplit_once(',')
        .ok_or_else(|| SoraError::InvalidType(original.to_owned()))?;
    let len = len
        .trim()
        .parse::<usize>()
        .map_err(|_| SoraError::InvalidType(original.to_owned()))?;

    Ok(TypeIr::Array {
        element: Box::new(parse_nested_type(element)?),
        len,
    })
}

fn parse_ref_type(original: &str, inner: &str) -> Result<TypeIr> {
    let (table, field) = inner
        .split_once('.')
        .ok_or_else(|| SoraError::InvalidType(original.to_owned()))?;
    require_identifier(table)?;
    require_identifier(field)?;

    Ok(TypeIr::Ref {
        table: table.to_owned(),
        field: field.to_owned(),
    })
}

fn require_identifier(input: &str) -> Result<()> {
    if is_identifier(input) {
        Ok(())
    } else {
        Err(SoraError::InvalidType(input.to_owned()))
    }
}

fn is_identifier(input: &str) -> bool {
    let mut chars = input.chars();
    matches!(chars.next(), Some(first) if first == '_' || first.is_ascii_alphabetic())
        && chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_required_type_forms() {
        let cases = [
            ("bool", TypeIr::Bool),
            ("i32", TypeIr::I32),
            ("i64", TypeIr::I64),
            ("f32", TypeIr::F32),
            ("f64", TypeIr::F64),
            ("string", TypeIr::String),
            ("enum<ItemType>", TypeIr::Enum("ItemType".to_owned())),
            ("struct<Reward>", TypeIr::Struct("Reward".to_owned())),
            ("list<i32>", TypeIr::List(Box::new(TypeIr::I32))),
            (
                "list<Reward>",
                TypeIr::List(Box::new(TypeIr::Struct("Reward".to_owned()))),
            ),
            (
                "array<i32,3>",
                TypeIr::Array {
                    element: Box::new(TypeIr::I32),
                    len: 3,
                },
            ),
            (
                "ref<Item.id>",
                TypeIr::Ref {
                    table: "Item".to_owned(),
                    field: "id".to_owned(),
                },
            ),
            (
                "optional<string>",
                TypeIr::Optional(Box::new(TypeIr::String)),
            ),
        ];

        for (source, expected) in cases {
            assert_eq!(parse_type(source).unwrap(), expected);
        }
    }

    #[test]
    fn rejects_malformed_types() {
        for source in [
            "",
            "array<i32>",
            "array<i32,nope>",
            "ref<Item>",
            "enum<1Bad>",
        ] {
            assert!(parse_type(source).is_err(), "{source} should fail");
        }
    }

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
"#,
        )
        .unwrap();

        let ir = normalize_schema(schema).unwrap();
        assert_eq!(ir.package, "game_config");
        assert_eq!(ir.enums[0].name, "ItemType");
        assert_eq!(ir.tables[0].mode, TableModeIr::Map);
        assert!(ir.tables[0].fields[0].required);
        assert_eq!(ir.tables[0].fields[0].ty, TypeIr::I32);
    }
}
