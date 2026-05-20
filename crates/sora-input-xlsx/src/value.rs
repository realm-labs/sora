use std::{collections::BTreeMap, path::Path};

use calamine::Data;
use serde_json::Value as JsonValue;
use sora_data::model::Value;
use sora_diagnostics::{Result, SoraError};
use sora_ir::model::{ConfigIr, TypeIr};

pub(crate) struct CellContext<'a> {
    pub path: &'a Path,
    pub ir: &'a ConfigIr,
    pub sheet: &'a str,
    pub row: usize,
    pub column: usize,
    pub field: &'a str,
    pub parser: Option<&'a str>,
    pub separator: Option<&'a str>,
    pub prefix: Option<&'a str>,
    pub suffix: Option<&'a str>,
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
        TypeIr::Struct(struct_name) if context.parser == Some("tuple") => {
            tuple_object_cell(cell, struct_name, context)?
        }
        TypeIr::Struct(_) => json_object_cell(cell, context)?,
        TypeIr::List(element) => separated_cell(cell, element, None, context)?,
        TypeIr::Array { element, len } => separated_cell(cell, element, Some(*len), context)?,
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

fn separated_cell(
    cell: &Data,
    element: &TypeIr,
    expected_len: Option<usize>,
    context: &CellContext<'_>,
) -> Result<Value> {
    let separator = context
        .separator
        .filter(|separator| !separator.is_empty())
        .ok_or_else(|| context.error("list and array cells require schema `separator`"))?;
    let source = cell_to_string(cell);
    if context.prefix == Some("[") && context.suffix == Some("]") {
        return json_array_cell(&source, element, expected_len, context);
    }

    let source = strip_bounds(&source, context)?;
    let items = source.split(separator).map(str::trim).collect::<Vec<_>>();

    if let Some(expected_len) = expected_len
        && items.len() != expected_len
    {
        return Err(context.error(format!(
            "expected {} separated values, got {}",
            expected_len,
            items.len()
        )));
    }

    let values = items
        .iter()
        .map(|item| separated_item_to_value(item, element, context))
        .collect::<Result<Vec<_>>>()?;
    Ok(Value::List(values))
}

fn tuple_object_cell(cell: &Data, struct_name: &str, context: &CellContext<'_>) -> Result<Value> {
    let source = cell_to_string(cell);
    tuple_object_value(&source, struct_name, context)
}

fn tuple_object_value(source: &str, struct_name: &str, context: &CellContext<'_>) -> Result<Value> {
    let separator = context
        .separator
        .filter(|separator| !separator.is_empty())
        .ok_or_else(|| context.error("tuple parser requires schema `separator`"))?;
    let struct_ir = context
        .ir
        .structs
        .iter()
        .find(|candidate| candidate.name == struct_name)
        .ok_or_else(|| context.error(format!("unknown struct `{struct_name}`")))?;
    let source = strip_bounds(source, context)?;
    let items = source.split(separator).map(str::trim).collect::<Vec<_>>();
    if items.len() != struct_ir.fields.len() {
        return Err(context.error(format!(
            "tuple `{struct_name}` expects {} values ({}) but got {}",
            struct_ir.fields.len(),
            struct_ir
                .fields
                .iter()
                .map(|field| format!("{}: {}", field.name, field.ty))
                .collect::<Vec<_>>()
                .join(", "),
            items.len()
        )));
    }

    Ok(Value::Object(
        struct_ir
            .fields
            .iter()
            .zip(items)
            .map(|(field, item)| {
                let nested_context = CellContext {
                    path: context.path,
                    ir: context.ir,
                    sheet: context.sheet,
                    row: context.row,
                    column: context.column,
                    field: &field.name,
                    parser: field.parser.as_deref(),
                    separator: field.separator.as_deref(),
                    prefix: field.prefix.as_deref(),
                    suffix: field.suffix.as_deref(),
                };
                Ok((
                    field.name.clone(),
                    source_to_value(item, &field.ty, &nested_context)?,
                ))
            })
            .collect::<Result<BTreeMap<_, _>>>()?,
    ))
}

fn source_to_value(source: &str, ty: &TypeIr, context: &CellContext<'_>) -> Result<Value> {
    let cell = Data::String(source.trim().to_owned());
    cell_to_value(&cell, ty, context)
}

fn strip_bounds<'a>(source: &'a str, context: &CellContext<'_>) -> Result<&'a str> {
    let mut inner = source.trim();
    if let Some(prefix) = context.prefix {
        inner = inner.strip_prefix(prefix).ok_or_else(|| {
            context.error(format!(
                "expected collection prefix `{prefix}`, got `{}`",
                source
            ))
        })?;
    }
    if let Some(suffix) = context.suffix {
        inner = inner.strip_suffix(suffix).ok_or_else(|| {
            context.error(format!(
                "expected collection suffix `{suffix}`, got `{}`",
                source
            ))
        })?;
    }

    Ok(inner)
}

fn json_array_cell(
    source: &str,
    element: &TypeIr,
    expected_len: Option<usize>,
    context: &CellContext<'_>,
) -> Result<Value> {
    let parsed: JsonValue = serde_json::from_str(source).map_err(|error| {
        context.error(format!("failed to parse JSON array `{source}`: {error}"))
    })?;
    let JsonValue::Array(items) = parsed else {
        return Err(context.error("expected JSON array"));
    };
    if let Some(expected_len) = expected_len
        && items.len() != expected_len
    {
        return Err(context.error(format!(
            "expected JSON array length {}, got {}",
            expected_len,
            items.len()
        )));
    }

    Ok(Value::List(
        items
            .iter()
            .map(|item| json_to_value(item, element, context))
            .collect::<Result<Vec<_>>>()?,
    ))
}

fn separated_item_to_value(
    item: &str,
    expected_type: &TypeIr,
    context: &CellContext<'_>,
) -> Result<Value> {
    match expected_type {
        TypeIr::Optional(_) if item.is_empty() => Ok(Value::Null),
        TypeIr::Optional(inner) => separated_item_to_value(item, inner, context),
        TypeIr::Bool => match item {
            value if value.eq_ignore_ascii_case("true") => Ok(Value::Bool(true)),
            value if value.eq_ignore_ascii_case("false") => Ok(Value::Bool(false)),
            _ => Err(context.error(format!("expected bool list item, got `{item}`"))),
        },
        TypeIr::I32 | TypeIr::I64 | TypeIr::Ref { .. } => item
            .parse::<i64>()
            .map(Value::Integer)
            .map_err(|_| context.error(format!("expected integer list item, got `{item}`"))),
        TypeIr::F32 | TypeIr::F64 => item
            .parse::<f64>()
            .map(Value::Float)
            .map_err(|_| context.error(format!("expected float list item, got `{item}`"))),
        TypeIr::String | TypeIr::Enum(_) => string_item_to_value(item, context),
        TypeIr::Struct(_) => {
            let parsed: JsonValue = serde_json::from_str(item).map_err(|error| {
                context.error(format!(
                    "failed to parse JSON object list item `{item}`: {error}"
                ))
            })?;
            let JsonValue::Object(values) = parsed else {
                return Err(context.error("expected JSON object list item"));
            };
            Ok(Value::Object(
                values
                    .iter()
                    .map(|(key, value)| Ok((key.clone(), json_to_untyped_value(value, context)?)))
                    .collect::<Result<BTreeMap<_, _>>>()?,
            ))
        }
        TypeIr::List(_) | TypeIr::Array { .. } => Err(context.error(format!(
            "nested list or array item `{item}` cannot be parsed with a single separator"
        ))),
    }
}

fn string_item_to_value(item: &str, context: &CellContext<'_>) -> Result<Value> {
    if item.starts_with('"') || item.ends_with('"') {
        serde_json::from_str::<String>(item)
            .map(Value::String)
            .map_err(|error| {
                context.error(format!(
                    "failed to parse JSON string item `{item}`: {error}"
                ))
            })
    } else {
        Ok(Value::String(item.to_owned()))
    }
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
        TypeIr::List(_) | TypeIr::Array { .. } => {
            return Err(context.error("nested JSON arrays are not supported in separated cells"));
        }
    })
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
