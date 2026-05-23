use std::{collections::BTreeMap, fmt};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfigIr {
    pub package: String,
    pub enums: Vec<EnumIr>,
    pub structs: Vec<StructIr>,
    pub unions: Vec<UnionIr>,
    pub tables: Vec<TableIr>,
}

impl ConfigIr {
    pub fn data_schema(&self) -> Self {
        self.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnumIr {
    pub name: String,
    pub scope: ScopeIr,
    pub values: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aliases: Vec<EnumAliasIr>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnumAliasIr {
    pub name: String,
    pub alias: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructIr {
    pub name: String,
    pub scope: ScopeIr,
    pub fields: Vec<FieldIr>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnionIr {
    pub name: String,
    pub scope: ScopeIr,
    pub tag: String,
    pub variants: Vec<UnionVariantIr>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnionVariantIr {
    pub name: String,
    pub scope: ScopeIr,
    pub fields: Vec<FieldIr>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableIr {
    pub name: String,
    pub scope: ScopeIr,
    pub mode: TableModeIr,
    pub key: Option<String>,
    pub source: Option<TableSourceIr>,
    pub fields: Vec<FieldIr>,
    pub indexes: Vec<IndexIr>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableSourceIr {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    pub file: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
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
    pub scope: ScopeIr,
    pub key: bool,
    pub comment: Option<String>,
    pub default: Option<String>,
    pub range: Option<[i64; 2]>,
    pub length: Option<[usize; 2]>,
    pub parser: Option<ParserIr>,
    pub derived_from: Option<DerivedFieldIr>,
}

impl FieldIr {
    pub fn is_required(&self) -> bool {
        !self.ty.is_optional() && self.default.is_none()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParserIr {
    pub kind: String,
    pub options: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DerivedFieldIr {
    pub source_table: String,
    pub parent_key: String,
    pub child_key: String,
    pub value_field: Option<String>,
    pub order_by: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScopeIr {
    pub values: Vec<String>,
}

impl Default for ScopeIr {
    fn default() -> Self {
        Self {
            values: vec!["all".to_owned()],
        }
    }
}

impl ScopeIr {
    pub fn includes(&self, target: &str) -> bool {
        target == "all"
            || self.values.iter().any(|value| value == "all")
            || self.values.iter().any(|value| value == target)
    }

    pub fn display(&self) -> String {
        self.values.join(",")
    }
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
    Union(String),
    List(Box<TypeIr>),
    Set(Box<TypeIr>),
    Map {
        key: Box<TypeIr>,
        value: Box<TypeIr>,
    },
    Array {
        element: Box<TypeIr>,
        len: usize,
    },
    Ref {
        table: String,
        field: String,
    },
    Optional(Box<TypeIr>),
}

impl TypeIr {
    pub fn is_optional(&self) -> bool {
        matches!(self, Self::Optional(_))
    }
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
            TypeIr::Union(name) => write!(f, "union<{name}>"),
            TypeIr::List(element) => write!(f, "list<{element}>"),
            TypeIr::Set(element) => write!(f, "set<{element}>"),
            TypeIr::Map { key, value } => write!(f, "map<{key},{value}>"),
            TypeIr::Array { element, len } => write!(f, "array<{element},{len}>"),
            TypeIr::Ref { table, field } => write!(f, "ref<{table}.{field}>"),
            TypeIr::Optional(element) => write!(f, "optional<{element}>"),
        }
    }
}
