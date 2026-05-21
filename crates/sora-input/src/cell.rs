use std::{borrow::Cow, path::Path};

use sora_data::model::Value;
use sora_diagnostics::{Result, SoraError};
use sora_ir::model::{ConfigIr, ParserIr, TypeIr};

use crate::parser::{ParserRegistry, builtin_registry};

#[derive(Debug, Clone)]
pub enum CellValue<'a> {
    Empty,
    Text(Cow<'a, str>),
    Integer(i64),
    Float(f64),
    Bool(bool),
    Error(Cow<'a, str>),
}

impl CellValue<'_> {
    pub fn is_empty(&self) -> bool {
        matches!(self, Self::Empty) || matches!(self, Self::Text(value) if value.trim().is_empty())
    }

    pub fn display_text(&self) -> String {
        match self {
            Self::Empty => String::new(),
            Self::Text(value) => value.to_string(),
            Self::Integer(value) => value.to_string(),
            Self::Float(value) => {
                if value.fract() == 0.0 {
                    format!("{value:.0}")
                } else {
                    value.to_string()
                }
            }
            Self::Bool(value) => value.to_string(),
            Self::Error(value) => value.to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CellLocation<'a> {
    Default,
    Csv {
        row: usize,
        column: usize,
    },
    Worksheet {
        sheet: &'a str,
        row: usize,
        column: usize,
    },
}

#[derive(Debug, Clone, Copy)]
pub struct CellContext<'a> {
    pub path: &'a Path,
    pub ir: &'a ConfigIr,
    pub location: CellLocation<'a>,
    pub field: &'a str,
    pub parser: Option<&'a ParserIr>,
}

impl CellContext<'_> {
    pub(crate) fn error(&self, message: impl Into<String>) -> SoraError {
        let location = match self.location {
            CellLocation::Default => "schema default".to_owned(),
            CellLocation::Csv { row, column } => format!("CSV row {row}, column {column}"),
            CellLocation::Worksheet { sheet, row, column } => {
                format!("worksheet `{sheet}` row {row}, column {column}")
            }
        };

        SoraError::ParseData {
            path: self.path.to_path_buf(),
            message: format!("{location}, field `{}`: {}", self.field, message.into()),
        }
    }
}

pub fn cell_to_value(
    cell: &CellValue<'_>,
    ty: &TypeIr,
    context: &CellContext<'_>,
) -> Result<Value> {
    cell_to_value_with_parsers(cell, ty, context, builtin_registry())
}

pub fn cell_to_value_with_parsers(
    cell: &CellValue<'_>,
    ty: &TypeIr,
    context: &CellContext<'_>,
    parser_registry: &ParserRegistry,
) -> Result<Value> {
    parser_registry.parse_cell(cell, ty, context)
}
