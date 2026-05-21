use std::{borrow::Cow, collections::BTreeMap, path::Path};

use serde_json::Value as JsonValue;
use sora_data::model::Value;
use sora_diagnostics::{Result, SoraError};
use sora_ir::model::{ConfigIr, ParserIr, TypeIr};

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
    fn error(&self, message: impl Into<String>) -> SoraError {
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
    Ok(match ty {
        TypeIr::Optional(_) if cell.is_empty() => Value::Null,
        TypeIr::Optional(inner) => cell_to_value(cell, inner, context)?,
        _ if parser_kind(context) == Some("json") => {
            json_cell_value(&cell.display_text(), ty, context)?
        }
        TypeIr::Bool => bool_cell(cell, context)?,
        TypeIr::I32 | TypeIr::I64 | TypeIr::Ref { .. } => integer_cell(cell, context)?,
        TypeIr::F32 | TypeIr::F64 => float_cell(cell, context)?,
        TypeIr::String | TypeIr::Enum(_) => Value::String(cell.display_text()),
        TypeIr::Struct(struct_name) if parser_kind(context) == Some("tuple") => {
            tuple_object_value(&cell.display_text(), struct_name, context)?
        }
        TypeIr::Struct(_) | TypeIr::Union(_) => json_object_value(&cell.display_text(), context)?,
        TypeIr::List(element) => separated_value(&cell.display_text(), element, None, context)?,
        TypeIr::Array { element, len } => {
            separated_value(&cell.display_text(), element, Some(*len), context)?
        }
    })
}

fn parser_kind<'a>(context: &'a CellContext<'_>) -> Option<&'a str> {
    context.parser.map(|parser| parser.kind.as_str())
}

fn parser_option<'a>(context: &'a CellContext<'_>, key: &str) -> Option<&'a str> {
    context
        .parser
        .and_then(|parser| parser.options.get(key))
        .map(String::as_str)
        .filter(|value| !value.is_empty())
}

fn bool_cell(cell: &CellValue<'_>, context: &CellContext<'_>) -> Result<Value> {
    match cell {
        CellValue::Bool(value) => Ok(Value::Bool(*value)),
        CellValue::Text(value) if value.eq_ignore_ascii_case("true") => Ok(Value::Bool(true)),
        CellValue::Text(value) if value.eq_ignore_ascii_case("false") => Ok(Value::Bool(false)),
        CellValue::Integer(value) => Ok(Value::Bool(*value != 0)),
        CellValue::Float(value) => Ok(Value::Bool(*value != 0.0)),
        _ => Err(context.error(format!("expected bool, got `{}`", cell.display_text()))),
    }
}

fn integer_cell(cell: &CellValue<'_>, context: &CellContext<'_>) -> Result<Value> {
    match cell {
        CellValue::Integer(value) => Ok(Value::Integer(*value)),
        CellValue::Float(value) if value.fract() == 0.0 => Ok(Value::Integer(*value as i64)),
        CellValue::Float(value) => {
            Err(context.error(format!("expected integer, got float `{value}`")))
        }
        CellValue::Text(value) => value
            .trim()
            .parse::<i64>()
            .map(Value::Integer)
            .map_err(|_| context.error(format!("expected integer, got `{value}`"))),
        _ => Err(context.error(format!("expected integer, got `{}`", cell.display_text()))),
    }
}

fn float_cell(cell: &CellValue<'_>, context: &CellContext<'_>) -> Result<Value> {
    match cell {
        CellValue::Integer(value) => Ok(Value::Float(*value as f64)),
        CellValue::Float(value) => Ok(Value::Float(*value)),
        CellValue::Text(value) => value
            .trim()
            .parse::<f64>()
            .map(Value::Float)
            .map_err(|_| context.error(format!("expected float, got `{value}`"))),
        _ => Err(context.error(format!("expected float, got `{}`", cell.display_text()))),
    }
}

fn separated_value(
    source: &str,
    element: &TypeIr,
    expected_len: Option<usize>,
    context: &CellContext<'_>,
) -> Result<Value> {
    if parser_kind(context) == Some("tuple_list") {
        return tuple_list_value(source, element, expected_len, context);
    }

    let separator = parser_option(context, "separator").unwrap_or(",");
    let source = split_source(source);
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

    Ok(Value::List(
        items
            .iter()
            .map(|item| separated_item_to_value(item, element, context))
            .collect::<Result<Vec<_>>>()?,
    ))
}

fn split_source(source: &str) -> &str {
    let source = source.trim();
    source
        .strip_prefix('[')
        .and_then(|inner| inner.strip_suffix(']'))
        .unwrap_or(source)
}

fn tuple_list_value(
    source: &str,
    element: &TypeIr,
    expected_len: Option<usize>,
    context: &CellContext<'_>,
) -> Result<Value> {
    let struct_name = tuple_list_struct_name(element)
        .ok_or_else(|| context.error("tuple_list parser requires list or array of struct"))?;
    let item_separator = parser_option(context, "item_separator").unwrap_or("|");
    let source = split_source(source);
    let items = if source.trim().is_empty() {
        Vec::new()
    } else {
        source
            .split(item_separator)
            .map(str::trim)
            .collect::<Vec<_>>()
    };
    if let Some(expected_len) = expected_len
        && items.len() != expected_len
    {
        return Err(context.error(format!(
            "expected {} tuple list items, got {}",
            expected_len,
            items.len()
        )));
    }

    Ok(Value::List(
        items
            .iter()
            .map(|item| tuple_object_value(item, struct_name, context))
            .collect::<Result<Vec<_>>>()?,
    ))
}

fn tuple_list_struct_name(ty: &TypeIr) -> Option<&str> {
    match ty {
        TypeIr::Optional(inner) => tuple_list_struct_name(inner),
        TypeIr::Struct(name) => Some(name),
        _ => None,
    }
}

fn tuple_object_value(source: &str, struct_name: &str, context: &CellContext<'_>) -> Result<Value> {
    let separator = parser_option(context, "separator").unwrap_or(",");
    let struct_ir = context
        .ir
        .structs
        .iter()
        .find(|candidate| candidate.name == struct_name)
        .ok_or_else(|| context.error(format!("unknown struct `{struct_name}`")))?;
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
                    location: context.location,
                    field: &field.name,
                    parser: field.parser.as_ref(),
                };
                Ok((
                    field.name.clone(),
                    source_to_value(item, &field.ty, &nested_context)?,
                ))
            })
            .collect::<Result<BTreeMap<_, _>>>()?,
    ))
}

fn json_cell_value(source: &str, ty: &TypeIr, context: &CellContext<'_>) -> Result<Value> {
    let parsed: JsonValue = serde_json::from_str(source)
        .map_err(|error| context.error(format!("failed to parse JSON cell `{source}`: {error}")))?;
    json_to_cell_value(&parsed, ty, context)
}

fn json_to_cell_value(
    value: &JsonValue,
    expected_type: &TypeIr,
    context: &CellContext<'_>,
) -> Result<Value> {
    Ok(match expected_type {
        TypeIr::Optional(_) if value.is_null() => Value::Null,
        TypeIr::Optional(inner) => json_to_cell_value(value, inner, context)?,
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
        TypeIr::Struct(_) | TypeIr::Union(_) => json_to_untyped_value(value, context)?,
        TypeIr::List(element) => {
            let JsonValue::Array(items) = value else {
                return Err(context.error("expected JSON array"));
            };
            Value::List(
                items
                    .iter()
                    .map(|item| json_to_cell_value(item, element, context))
                    .collect::<Result<Vec<_>>>()?,
            )
        }
        TypeIr::Array { element, len } => {
            let JsonValue::Array(items) = value else {
                return Err(context.error("expected JSON array"));
            };
            if items.len() != *len {
                return Err(context.error(format!(
                    "expected JSON array length {}, got {}",
                    len,
                    items.len()
                )));
            }
            Value::List(
                items
                    .iter()
                    .map(|item| json_to_cell_value(item, element, context))
                    .collect::<Result<Vec<_>>>()?,
            )
        }
    })
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
        TypeIr::Struct(_) | TypeIr::Union(_) => {
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

fn json_object_value(source: &str, context: &CellContext<'_>) -> Result<Value> {
    let parsed: JsonValue = serde_json::from_str(source)
        .map_err(|error| context.error(format!("failed to parse JSON cell `{source}`: {error}")))?;
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

fn source_to_value(source: &str, ty: &TypeIr, context: &CellContext<'_>) -> Result<Value> {
    let cell = CellValue::Text(Cow::Owned(source.trim().to_owned()));
    cell_to_value(&cell, ty, context)
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
