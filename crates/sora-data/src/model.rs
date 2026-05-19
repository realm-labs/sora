use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfigData {
    pub tables: Vec<TableData>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TableData {
    pub name: String,
    pub rows: Vec<RowData>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RowData {
    pub values: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    Bool(bool),
    Integer(i64),
    Float(f64),
    String(String),
    List(Vec<Value>),
    Object(BTreeMap<String, Value>),
    Null,
}

impl Value {
    pub(crate) fn kind_name(&self) -> &'static str {
        match self {
            Self::Bool(_) => "bool",
            Self::Integer(_) => "integer",
            Self::Float(_) => "float",
            Self::String(_) => "string",
            Self::List(_) => "list",
            Self::Object(_) => "object",
            Self::Null => "null",
        }
    }
}
