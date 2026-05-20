use std::{collections::BTreeMap, path::Path};

use calamine::Data;
use serde_json::Value as JsonValue;
use sora_data::model::Value;
use sora_diagnostics::{Result, SoraError};
use sora_ir::model::TypeIr;

pub(crate) struct CellContext<'a> {
    pub path: &'a Path,
    pub sheet: &'a str,
    pub row: usize,
    pub column: usize,
    pub field: &'a str,
}

impl CellContext<'_> {
    fn error(&self, message: impl Into<String>) -> SoraError {
        SoraError::ParseData {
            path: self.path.to_path_buf(),
            message: format!(
                "worksheet `{}` row {}, column {}, field `{}`: {}",
                self.sheet,
                self.row + 1,
                self.column + 1,
                self.field,
                message.into()
            ),
        }
    }
}

pub(crate) fn cell_to_value(cell: &Data, ty: &TypeIr, context: &CellContext<'_>) -> Result<Value> {
    Ok(match ty {
        TypeIr::Optional(inner) if cell_is_empty(cell) => Value::Null,
        TypeIr::Optional(inner) => cell_to_value(cell, inner, context)?,
        TypeIr::Bool => bool_cell(cell, context)?,
        TypeIr::I32 | TypeIr::I64 | TypeIr::Ref { .. } => integer_cell(cell, context)?,
        TypeIr::F32 | TypeIr::F64 => float_cell(cell, context)?,
        TypeIr::String | TypeIr::Enum(_) => Value::String(cell_to_string(cell)),
        TypeIr::Struct(_) => json_object_cell(cell, context)?,
        TypeIr::List(element) => json_array_cell(cell, element, None, context)?,
        TypeIr::Array { element, len } => json_array_cell(cell, element, Some(*len), context)?,
    })
}

pub(crate) fn cell_is_empty(cell: &Data) -> bool {
    matches!(cell, Data::Empty) || matches!(cell, Data::String(value) if value.trim().is_empty())
}

pub(crate) fn cell_to_string(cell: &Data) -> String {
    match cell {
        Data::Empty => String::new(),
        Data::String(value) => value.clone(),
        Data::Float(value) => {
            if value.fract() == 0.0 {
                format!("{value:.0}")
            } else {
                value.to_string()
            }
        }
        Data::Int(value) => value.to_string(),
        Data::Bool(value) => value.to_string(),
        Data::DateTime(value) => value.to_string(),
        Data::DateTimeIso(value) => value.clone(),
        Data::DurationIso(value) => value.clone(),
        Data::Error(value) => value.to_string(),
    }
}

fn bool_cell(cell: &Data, context: &CellContext<'_>) -> Result<Value> {
    match cell {
        Data::Bool(value) => Ok(Value::Bool(*value)),
        Data::String(value) if value.eq_ignore_ascii_case("true") => Ok(Value::Bool(true)),
        Data::String(value) if value.eq_ignore_ascii_case("false") => Ok(Value::Bool(false)),
        Data::Int(value) => Ok(Value::Bool(*value != 0)),
        Data::Float(value) => Ok(Value::Bool(*value != 0.0)),
        _ => Err(context.error(format!("expected bool, got `{}`", cell_to_string(cell)))),
    }
}

fn integer_cell(cell: &Data, context: &CellContext<'_>) -> Result<Value> {
    match cell {
        Data::Int(value) => Ok(Value::Integer(*value)),
        Data::Float(value) if value.fract() == 0.0 => Ok(Value::Integer(*value as i64)),
        Data::Float(value) => Err(context.error(format!("expected integer, got float `{value}`"))),
        Data::String(value) => value
            .trim()
            .parse::<i64>()
            .map(Value::Integer)
            .map_err(|_| context.error(format!("expected integer, got `{value}`"))),
        _ => Err(context.error(format!("expected integer, got `{}`", cell_to_string(cell)))),
    }
}

fn float_cell(cell: &Data, context: &CellContext<'_>) -> Result<Value> {
    match cell {
        Data::Int(value) => Ok(Value::Float(*value as f64)),
        Data::Float(value) => Ok(Value::Float(*value)),
        Data::String(value) => value
            .trim()
            .parse::<f64>()
            .map(Value::Float)
            .map_err(|_| context.error(format!("expected float, got `{value}`"))),
        _ => Err(context.error(format!("expected float, got `{}`", cell_to_string(cell)))),
    }
}

fn json_array_cell(
    cell: &Data,
    element: &TypeIr,
    expected_len: Option<usize>,
    context: &CellContext<'_>,
) -> Result<Value> {
    let parsed = parse_json_cell(cell, context)?;
    let JsonValue::Array(items) = parsed else {
        return Err(context.error("expected JSON array"));
    };
    if let Some(expected_len) = expected_len {
        if items.len() != expected_len {
            return Err(context.error(format!(
                "expected JSON array length {}, got {}",
                expected_len,
                items.len()
            )));
        }
    }

    let values = items
        .iter()
        .map(|item| json_to_value(item, element, context))
        .collect::<Result<Vec<_>>>()?;
    Ok(Value::List(values))
}

fn json_object_cell(cell: &Data, context: &CellContext<'_>) -> Result<Value> {
    let parsed = parse_json_cell(cell, context)?;
    let JsonValue::Object(values) = parsed else {
        return Err(context.error("expected JSON object"));
    };

    Ok(Value::Object(
        values
            .iter()
            .map(|(key, value)| Ok((key.clone(), json_to_untyped_value(value, context)?)))
            .collect::<Result<BTreeMap<_, _>>>()?,
    ))
}

fn parse_json_cell(cell: &Data, context: &CellContext<'_>) -> Result<JsonValue> {
    let source = cell_to_string(cell);
    serde_json::from_str(&source).map_err(|error| {
        context.error(format!("failed to parse JSON cell `{}`: {}", source, error))
    })
}

fn json_to_value(
    value: &JsonValue,
    expected_type: &TypeIr,
    context: &CellContext<'_>,
) -> Result<Value> {
    Ok(match expected_type {
        TypeIr::Optional(_) if value.is_null() => Value::Null,
        TypeIr::Optional(inner) => json_to_value(value, inner, context)?,
        TypeIr::Bool => value
            .as_bool()
            .map(Value::Bool)
            .ok_or_else(|| context.error("expected JSON bool"))?,
        TypeIr::I32 | TypeIr::I64 | TypeIr::Ref { .. } => value
            .as_i64()
            .map(Value::Integer)
            .ok_or_else(|| context.error("expected JSON integer"))?,
        TypeIr::F32 | TypeIr::F64 => value
            .as_f64()
            .map(Value::Float)
            .ok_or_else(|| context.error("expected JSON number"))?,
        TypeIr::String | TypeIr::Enum(_) => value
            .as_str()
            .map(|value| Value::String(value.to_owned()))
            .ok_or_else(|| context.error("expected JSON string"))?,
        TypeIr::Struct(_) => json_to_untyped_value(value, context)?,
        TypeIr::List(element) => json_array_value(value, element, None, context)?,
        TypeIr::Array { element, len } => json_array_value(value, element, Some(*len), context)?,
    })
}

fn json_array_value(
    value: &JsonValue,
    element: &TypeIr,
    expected_len: Option<usize>,
    context: &CellContext<'_>,
) -> Result<Value> {
    let JsonValue::Array(items) = value else {
        return Err(context.error("expected JSON array"));
    };
    if let Some(expected_len) = expected_len {
        if items.len() != expected_len {
            return Err(context.error(format!(
                "expected JSON array length {}, got {}",
                expected_len,
                items.len()
            )));
        }
    }

    Ok(Value::List(
        items
            .iter()
            .map(|item| json_to_value(item, element, context))
            .collect::<Result<Vec<_>>>()?,
    ))
}

fn json_to_untyped_value(value: &JsonValue, context: &CellContext<'_>) -> Result<Value> {
    Ok(match value {
        JsonValue::Null => Value::Null,
        JsonValue::Bool(value) => Value::Bool(*value),
        JsonValue::Number(value) => {
            if let Some(value) = value.as_i64() {
                Value::Integer(value)
            } else if let Some(value) = value.as_f64() {
                Value::Float(value)
            } else {
                return Err(context.error("unsupported JSON number"));
            }
        }
        JsonValue::String(value) => Value::String(value.clone()),
        JsonValue::Array(values) => Value::List(
            values
                .iter()
                .map(|value| json_to_untyped_value(value, context))
                .collect::<Result<Vec<_>>>()?,
        ),
        JsonValue::Object(values) => Value::Object(
            values
                .iter()
                .map(|(key, value)| Ok((key.clone(), json_to_untyped_value(value, context)?)))
                .collect::<Result<BTreeMap<_, _>>>()?,
        ),
    })
}
