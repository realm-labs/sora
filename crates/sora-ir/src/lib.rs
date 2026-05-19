use std::{collections::BTreeSet, fmt};

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
        normalize_schema(toml::from_str(&source).unwrap()).unwrap()
    }
}
