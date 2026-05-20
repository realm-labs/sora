use std::fmt;

use serde::{Deserialize, Serialize};

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
    pub source: Option<TableSourceIr>,
    pub fields: Vec<FieldIr>,
    pub indexes: Vec<IndexIr>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableSourceIr {
    pub format: String,
    pub file: String,
    pub sheet: Option<String>,
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
    pub default: Option<String>,
    pub range: Option<[i64; 2]>,
    pub parser: Option<String>,
    pub separator: Option<String>,
    pub prefix: Option<String>,
    pub suffix: Option<String>,
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
