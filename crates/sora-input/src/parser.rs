use std::{
    borrow::Cow,
    collections::{BTreeMap, HashMap},
    sync::OnceLock,
};

use serde_json::Value as JsonValue;
use sora_data::model::Value;
use sora_diagnostics::Result;
use sora_ir::model::TypeIr;

use crate::cell::{CellContext, CellValue};

pub trait CellParser: Send + Sync {
    fn kind(&self) -> &str;

    fn parse(
        &self,
        cell: &CellValue<'_>,
        ty: &TypeIr,
        context: &CellContext<'_>,
        registry: &ParserRegistry,
    ) -> Result<Value>;
}

pub struct ParserRegistry {
    parsers: HashMap<String, Box<dyn CellParser>>,
}

impl ParserRegistry {
    pub fn new() -> Self {
        Self {
            parsers: HashMap::new(),
        }
    }

    pub fn builtin() -> Self {
        let mut registry = Self::new();
        registry.register(SplitParser);
        registry.register(TupleParser);
        registry.register(TupleListParser);
        registry.register(MapParser);
        registry.register(JsonParser);
        registry
    }

    pub fn register(&mut self, parser: impl CellParser + 'static) {
        self.parsers
            .insert(parser.kind().to_owned(), Box::new(parser));
    }

    pub fn contains(&self, kind: &str) -> bool {
        self.parsers.contains_key(kind)
    }

    pub fn parse_cell(
        &self,
        cell: &CellValue<'_>,
        ty: &TypeIr,
        context: &CellContext<'_>,
    ) -> Result<Value> {
        if matches!(ty, TypeIr::Optional(_)) && cell.is_empty() {
            return Ok(Value::Null);
        }

        if let Some(parser) = context.parser {
            let Some(cell_parser) = self.parsers.get(&parser.kind) else {
                return Err(context.error(format!(
                    "unsupported parser `{}` at input parse time",
                    parser.kind
                )));
            };
            return cell_parser.parse(cell, ty, context, self);
        }

        default_cell_value(cell, ty, context)
    }
}

impl Default for ParserRegistry {
    fn default() -> Self {
        Self::builtin()
    }
}

pub fn builtin_registry() -> &'static ParserRegistry {
    static BUILTIN: OnceLock<ParserRegistry> = OnceLock::new();
    BUILTIN.get_or_init(ParserRegistry::builtin)
}

struct SplitParser;

impl CellParser for SplitParser {
    fn kind(&self) -> &str {
        "split"
    }

    fn parse(
        &self,
        cell: &CellValue<'_>,
        ty: &TypeIr,
        context: &CellContext<'_>,
        _registry: &ParserRegistry,
    ) -> Result<Value> {
        parse_collection_with_separator(&cell.display_text(), ty, context)
    }
}

struct TupleParser;

impl CellParser for TupleParser {
    fn kind(&self) -> &str {
        "tuple"
    }

    fn parse(
        &self,
        cell: &CellValue<'_>,
        ty: &TypeIr,
        context: &CellContext<'_>,
        registry: &ParserRegistry,
    ) -> Result<Value> {
        let struct_name = struct_type_name(ty)
            .ok_or_else(|| context.error("tuple parser requires struct type"))?;
        tuple_object_value(&cell.display_text(), struct_name, context, registry)
    }
}

struct TupleListParser;

impl CellParser for TupleListParser {
    fn kind(&self) -> &str {
        "tuple_list"
    }

    fn parse(
        &self,
        cell: &CellValue<'_>,
        ty: &TypeIr,
        context: &CellContext<'_>,
        registry: &ParserRegistry,
    ) -> Result<Value> {
        let source = cell.display_text();
        match ty {
            TypeIr::Optional(inner) => self.parse(cell, inner, context, registry),
            TypeIr::List(element) | TypeIr::Set(element) => {
                tuple_list_value(&source, element, None, context, registry)
            }
            TypeIr::Array { element, len } => {
                tuple_list_value(&source, element, Some(*len), context, registry)
            }
            _ => Err(context.error("tuple_list parser requires list or array type")),
        }
    }
}

struct MapParser;

impl CellParser for MapParser {
    fn kind(&self) -> &str {
        "map"
    }

    fn parse(
        &self,
        cell: &CellValue<'_>,
        ty: &TypeIr,
        context: &CellContext<'_>,
        _registry: &ParserRegistry,
    ) -> Result<Value> {
        map_value(&cell.display_text(), ty, context)
    }
}

struct JsonParser;

impl CellParser for JsonParser {
    fn kind(&self) -> &str {
        "json"
    }

    fn parse(
        &self,
        cell: &CellValue<'_>,
        ty: &TypeIr,
        context: &CellContext<'_>,
        _registry: &ParserRegistry,
    ) -> Result<Value> {
        json_cell_value(&cell.display_text(), ty, context)
    }
}

fn default_cell_value(
    cell: &CellValue<'_>,
    ty: &TypeIr,
    context: &CellContext<'_>,
) -> Result<Value> {
    Ok(match ty {
        TypeIr::Optional(_) if cell.is_empty() => Value::Null,
        TypeIr::Optional(inner) => default_cell_value(cell, inner, context)?,
        TypeIr::Bool => bool_cell(cell, context)?,
        TypeIr::I8
        | TypeIr::U8
        | TypeIr::I16
        | TypeIr::U16
        | TypeIr::I32
        | TypeIr::U32
        | TypeIr::I64
        | TypeIr::Ref { .. } => integer_cell(cell, context)?,
        TypeIr::Duration => duration_cell(cell, context)?,
        TypeIr::F32 | TypeIr::F64 => float_cell(cell, context)?,
        TypeIr::String | TypeIr::Text | TypeIr::Enum(_) => Value::String(cell.display_text()),
        TypeIr::Struct(_) | TypeIr::Union(_) => json_object_value(&cell.display_text(), context)?,
        TypeIr::List(_) | TypeIr::Set(_) | TypeIr::Array { .. } => {
            parse_collection_with_separator(&cell.display_text(), ty, context)?
        }
        TypeIr::Map { .. } => json_cell_value(&cell.display_text(), ty, context)?,
    })
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

fn duration_cell(cell: &CellValue<'_>, context: &CellContext<'_>) -> Result<Value> {
    match cell {
        CellValue::Text(value) => parse_duration_millis(value.trim())
            .map(Value::Integer)
            .map_err(|message| context.error(message)),
        _ => Err(context.error(format!(
            "expected duration literal, got `{}`",
            cell.display_text()
        ))),
    }
}

fn parse_duration_millis(source: &str) -> std::result::Result<i64, String> {
    let bytes = source.as_bytes();
    let mut index = 0usize;
    let mut total = 0i64;
    let mut parsed = false;
    let mut last_unit_rank = None;

    while index < bytes.len() {
        while bytes.get(index).is_some_and(u8::is_ascii_whitespace) {
            index += 1;
        }
        if index >= bytes.len() {
            break;
        }

        let number_start = index;
        while bytes.get(index).is_some_and(u8::is_ascii_digit) {
            index += 1;
        }
        if number_start == index {
            return Err(format!("expected duration number in `{source}`"));
        }
        let number = source[number_start..index]
            .parse::<i64>()
            .map_err(|_| format!("duration number is too large in `{source}`"))?;

        let unit_start = index;
        while bytes.get(index).is_some_and(u8::is_ascii_alphabetic) {
            index += 1;
        }
        if unit_start == index {
            return Err(format!("expected duration unit in `{source}`"));
        }
        let unit = &source[unit_start..index];
        let (rank, factor) = match unit {
            "d" => (0, 86_400_000),
            "h" => (1, 3_600_000),
            "m" => (2, 60_000),
            "s" => (3, 1_000),
            "ms" => (4, 1),
            _ => return Err(format!("unsupported duration unit `{unit}` in `{source}`")),
        };
        if last_unit_rank.is_some_and(|last_rank| rank <= last_rank) {
            return Err(format!(
                "duration units must be ordered as d, h, m, s, ms in `{source}`"
            ));
        }
        last_unit_rank = Some(rank);
        let millis = number
            .checked_mul(factor)
            .ok_or_else(|| format!("duration `{source}` is too large"))?;
        total = total
            .checked_add(millis)
            .ok_or_else(|| format!("duration `{source}` is too large"))?;
        parsed = true;
    }

    if parsed {
        Ok(total)
    } else {
        Err("expected duration literal".to_owned())
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

fn parse_collection_with_separator(
    source: &str,
    ty: &TypeIr,
    context: &CellContext<'_>,
) -> Result<Value> {
    match ty {
        TypeIr::Optional(inner) => parse_collection_with_separator(source, inner, context),
        TypeIr::List(element) | TypeIr::Set(element) => {
            separated_value(source, element, None, context)
        }
        TypeIr::Array { element, len } => separated_value(source, element, Some(*len), context),
        _ => Err(context.error("split parser requires list or array type")),
    }
}

fn separated_value(
    source: &str,
    element: &TypeIr,
    expected_len: Option<usize>,
    context: &CellContext<'_>,
) -> Result<Value> {
    let separator = parser_option(context, "separator").unwrap_or(",");
    let source = separated_source(source, context)?;
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

fn separated_source<'a>(source: &'a str, context: &CellContext<'_>) -> Result<&'a str> {
    let source = source.trim();
    if source.starts_with('[') && source.ends_with(']') {
        return Err(
            context.error("bracketed collection syntax is JSON; use parser `{ kind = \"json\" }`")
        );
    }
    Ok(source)
}

fn tuple_list_value(
    source: &str,
    element: &TypeIr,
    expected_len: Option<usize>,
    context: &CellContext<'_>,
    registry: &ParserRegistry,
) -> Result<Value> {
    let struct_name = struct_type_name(element)
        .ok_or_else(|| context.error("tuple_list parser requires list or array of struct"))?;
    let item_separator = parser_option(context, "item_separator").unwrap_or("|");
    let source = separated_source(source, context)?;
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
            .map(|item| tuple_object_value(item, struct_name, context, registry))
            .collect::<Result<Vec<_>>>()?,
    ))
}

fn map_value(source: &str, ty: &TypeIr, context: &CellContext<'_>) -> Result<Value> {
    match ty {
        TypeIr::Optional(inner) => map_value(source, inner, context),
        TypeIr::Map {
            key,
            value: element,
        } => separated_map_value(source, key, element, context),
        _ => Err(context.error("map parser requires map type")),
    }
}

fn separated_map_value(
    source: &str,
    key_ty: &TypeIr,
    value_ty: &TypeIr,
    context: &CellContext<'_>,
) -> Result<Value> {
    let separator = parser_option(context, "separator").unwrap_or(",");
    let item_separator = parser_option(context, "item_separator").unwrap_or("|");
    let source = separated_source(source, context)?;
    let items = if source.trim().is_empty() {
        Vec::new()
    } else {
        source
            .split(item_separator)
            .map(str::trim)
            .collect::<Vec<_>>()
    };

    Ok(Value::List(
        items
            .iter()
            .enumerate()
            .map(|(index, item)| {
                separated_map_pair(item, key_ty, value_ty, index, separator, context)
            })
            .collect::<Result<Vec<_>>>()?,
    ))
}

fn separated_map_pair(
    item: &str,
    key_ty: &TypeIr,
    value_ty: &TypeIr,
    index: usize,
    separator: &str,
    context: &CellContext<'_>,
) -> Result<Value> {
    let Some((key, value)) = item.split_once(separator) else {
        return Err(context.error(format!(
            "expected map item at index {index} to contain separator `{separator}`"
        )));
    };
    Ok(Value::List(vec![
        source_to_default_value(key, key_ty, context)?,
        source_to_default_value(value, value_ty, context)?,
    ]))
}

fn struct_type_name(ty: &TypeIr) -> Option<&str> {
    match ty {
        TypeIr::Optional(inner) => struct_type_name(inner),
        TypeIr::Struct(name) => Some(name),
        _ => None,
    }
}

fn tuple_object_value(
    source: &str,
    struct_name: &str,
    context: &CellContext<'_>,
    registry: &ParserRegistry,
) -> Result<Value> {
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
                    source_to_value(item, &field.ty, &nested_context, registry)?,
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
        TypeIr::I8
        | TypeIr::U8
        | TypeIr::I16
        | TypeIr::U16
        | TypeIr::I32
        | TypeIr::U32
        | TypeIr::I64
        | TypeIr::Ref { .. } => value
            .as_i64()
            .map(Value::Integer)
            .ok_or_else(|| context.error("expected JSON integer"))?,
        TypeIr::Duration => value
            .as_str()
            .ok_or_else(|| context.error("expected JSON duration string"))
            .and_then(|source| {
                parse_duration_millis(source)
                    .map(Value::Integer)
                    .map_err(|message| context.error(message))
            })?,
        TypeIr::F32 | TypeIr::F64 => value
            .as_f64()
            .map(Value::Float)
            .ok_or_else(|| context.error("expected JSON number"))?,
        TypeIr::String | TypeIr::Text | TypeIr::Enum(_) => value
            .as_str()
            .map(|value| Value::String(value.to_owned()))
            .ok_or_else(|| context.error("expected JSON string"))?,
        TypeIr::Struct(_) | TypeIr::Union(_) => json_to_untyped_value(value, context)?,
        TypeIr::List(element) | TypeIr::Set(element) => {
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
        TypeIr::Map {
            key,
            value: element,
        } => {
            let JsonValue::Array(items) = value else {
                return Err(context.error("expected JSON array"));
            };
            Value::List(
                items
                    .iter()
                    .enumerate()
                    .map(|(index, item)| json_to_map_pair(item, key, element, index, context))
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
        TypeIr::I8
        | TypeIr::U8
        | TypeIr::I16
        | TypeIr::U16
        | TypeIr::I32
        | TypeIr::U32
        | TypeIr::I64
        | TypeIr::Ref { .. } => item
            .parse::<i64>()
            .map(Value::Integer)
            .map_err(|_| context.error(format!("expected integer list item, got `{item}`"))),
        TypeIr::Duration => parse_duration_millis(item)
            .map(Value::Integer)
            .map_err(|message| context.error(message)),
        TypeIr::F32 | TypeIr::F64 => item
            .parse::<f64>()
            .map(Value::Float)
            .map_err(|_| context.error(format!("expected float list item, got `{item}`"))),
        TypeIr::String | TypeIr::Text | TypeIr::Enum(_) => string_item_to_value(item, context),
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
        TypeIr::List(_) | TypeIr::Set(_) | TypeIr::Map { .. } | TypeIr::Array { .. } => {
            Err(context.error(format!(
                "nested collection item `{item}` cannot be parsed with a single separator"
            )))
        }
    }
}

fn json_to_map_pair(
    item: &JsonValue,
    key_ty: &TypeIr,
    value_ty: &TypeIr,
    index: usize,
    context: &CellContext<'_>,
) -> Result<Value> {
    let JsonValue::Array(pair) = item else {
        return Err(context.error(format!("expected JSON map pair array at index {index}")));
    };
    let [key, value] = pair.as_slice() else {
        return Err(context.error(format!(
            "expected JSON map pair at index {index} to contain exactly two elements"
        )));
    };
    Ok(Value::List(vec![
        json_to_cell_value(key, key_ty, context)?,
        json_to_cell_value(value, value_ty, context)?,
    ]))
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

fn source_to_value(
    source: &str,
    ty: &TypeIr,
    context: &CellContext<'_>,
    registry: &ParserRegistry,
) -> Result<Value> {
    let cell = CellValue::Text(Cow::Owned(source.trim().to_owned()));
    registry.parse_cell(&cell, ty, context)
}

fn source_to_default_value(source: &str, ty: &TypeIr, context: &CellContext<'_>) -> Result<Value> {
    let cell = CellValue::Text(Cow::Owned(source.trim().to_owned()));
    default_cell_value(&cell, ty, context)
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

#[cfg(test)]
mod tests {
    use std::path::Path;

    use sora_ir::{
        model::{ConfigIr, ParserIr},
        parse::parse_type,
    };

    use super::*;
    use crate::cell::CellLocation;

    struct UpperParser;

    impl CellParser for UpperParser {
        fn kind(&self) -> &str {
            "upper"
        }

        fn parse(
            &self,
            cell: &CellValue<'_>,
            _ty: &TypeIr,
            _context: &CellContext<'_>,
            _registry: &ParserRegistry,
        ) -> Result<Value> {
            Ok(Value::String(cell.display_text().to_uppercase()))
        }
    }

    #[test]
    fn parses_duration_literals_as_milliseconds() {
        let registry = ParserRegistry::builtin();
        let ir = ConfigIr {
            package: "test".to_owned(),
            localization: None,
            enums: Vec::new(),
            structs: Vec::new(),
            unions: Vec::new(),
            tables: Vec::new(),
        };
        let context = CellContext {
            path: Path::new("<test>"),
            ir: &ir,
            location: CellLocation::Default,
            field: "duration",
            parser: None,
        };

        let value = registry
            .parse_cell(
                &CellValue::Text("1h30m5s250ms".into()),
                &parse_type("duration").unwrap(),
                &context,
            )
            .unwrap();

        assert_eq!(value, Value::Integer(5_405_250));
    }

    #[test]
    fn rejects_out_of_order_duration_units() {
        let registry = ParserRegistry::builtin();
        let ir = ConfigIr {
            package: "test".to_owned(),
            localization: None,
            enums: Vec::new(),
            structs: Vec::new(),
            unions: Vec::new(),
            tables: Vec::new(),
        };
        let context = CellContext {
            path: Path::new("<test>"),
            ir: &ir,
            location: CellLocation::Default,
            field: "duration",
            parser: None,
        };

        let error = registry
            .parse_cell(
                &CellValue::Text("30s1h".into()),
                &parse_type("duration").unwrap(),
                &context,
            )
            .unwrap_err();

        assert!(error.to_string().contains("must be ordered"));
    }

    #[test]
    fn rejects_numeric_duration_cells() {
        let registry = ParserRegistry::builtin();
        let ir = ConfigIr {
            package: "test".to_owned(),
            localization: None,
            enums: Vec::new(),
            structs: Vec::new(),
            unions: Vec::new(),
            tables: Vec::new(),
        };
        let context = CellContext {
            path: Path::new("<test>"),
            ir: &ir,
            location: CellLocation::Default,
            field: "duration",
            parser: None,
        };

        let error = registry
            .parse_cell(
                &CellValue::Integer(1000),
                &parse_type("duration").unwrap(),
                &context,
            )
            .unwrap_err();

        assert!(error.to_string().contains("expected duration literal"));
    }

    #[test]
    fn parses_map_cells_as_pairs() {
        let registry = ParserRegistry::builtin();
        let ir = ConfigIr {
            package: "test".to_owned(),
            localization: None,
            enums: Vec::new(),
            structs: Vec::new(),
            unions: Vec::new(),
            tables: Vec::new(),
        };
        let parser = ParserIr {
            kind: "map".to_owned(),
            options: BTreeMap::new(),
        };
        let context = CellContext {
            path: Path::new("<test>"),
            ir: &ir,
            location: CellLocation::Default,
            field: "attributes",
            parser: Some(&parser),
        };

        let value = registry
            .parse_cell(
                &CellValue::Text("tier,1|power,10".into()),
                &parse_type("map<string,i32>").unwrap(),
                &context,
            )
            .unwrap();

        assert_eq!(
            value,
            Value::List(vec![
                Value::List(vec![Value::String("tier".to_owned()), Value::Integer(1)]),
                Value::List(vec![Value::String("power".to_owned()), Value::Integer(10)]),
            ])
        );
    }

    #[test]
    fn parses_map_cells_with_custom_separators() {
        let registry = ParserRegistry::builtin();
        let ir = ConfigIr {
            package: "test".to_owned(),
            localization: None,
            enums: Vec::new(),
            structs: Vec::new(),
            unions: Vec::new(),
            tables: Vec::new(),
        };
        let parser = ParserIr {
            kind: "map".to_owned(),
            options: BTreeMap::from([
                ("item_separator".to_owned(), ";".to_owned()),
                ("separator".to_owned(), ":".to_owned()),
            ]),
        };
        let context = CellContext {
            path: Path::new("<test>"),
            ir: &ir,
            location: CellLocation::Default,
            field: "attributes",
            parser: Some(&parser),
        };

        let value = registry
            .parse_cell(
                &CellValue::Text("tier:1;power:10".into()),
                &parse_type("map<string,i32>").unwrap(),
                &context,
            )
            .unwrap();

        assert_eq!(
            value,
            Value::List(vec![
                Value::List(vec![Value::String("tier".to_owned()), Value::Integer(1)]),
                Value::List(vec![Value::String("power".to_owned()), Value::Integer(10)]),
            ])
        );
    }

    #[test]
    fn separated_parsers_reject_json_array_shape() {
        let registry = ParserRegistry::builtin();
        let ir = ConfigIr {
            package: "test".to_owned(),
            localization: None,
            enums: Vec::new(),
            structs: Vec::new(),
            unions: Vec::new(),
            tables: Vec::new(),
        };
        let parser = ParserIr {
            kind: "split".to_owned(),
            options: BTreeMap::new(),
        };
        let context = CellContext {
            path: Path::new("<test>"),
            ir: &ir,
            location: CellLocation::Default,
            field: "tags",
            parser: Some(&parser),
        };

        let error = registry
            .parse_cell(
                &CellValue::Text("[1,2,3]".into()),
                &parse_type("list<i32>").unwrap(),
                &context,
            )
            .unwrap_err();

        assert!(error.to_string().contains("bracketed collection syntax"));
    }

    #[test]
    fn registry_accepts_custom_cell_parsers() {
        let mut registry = ParserRegistry::builtin();
        registry.register(UpperParser);
        let ir = ConfigIr {
            package: "test".to_owned(),
            localization: None,
            enums: Vec::new(),
            structs: Vec::new(),
            unions: Vec::new(),
            tables: Vec::new(),
        };
        let parser = ParserIr {
            kind: "upper".to_owned(),
            options: BTreeMap::new(),
        };
        let context = CellContext {
            path: Path::new("<test>"),
            ir: &ir,
            location: CellLocation::Default,
            field: "name",
            parser: Some(&parser),
        };

        let value = registry
            .parse_cell(
                &CellValue::Text("iron sword".into()),
                &parse_type("string").unwrap(),
                &context,
            )
            .unwrap();

        assert_eq!(value, Value::String("IRON SWORD".to_owned()));
    }

    #[test]
    fn optional_empty_cells_skip_custom_parsers() {
        let mut registry = ParserRegistry::builtin();
        registry.register(UpperParser);
        let ir = ConfigIr {
            package: "test".to_owned(),
            localization: None,
            enums: Vec::new(),
            structs: Vec::new(),
            unions: Vec::new(),
            tables: Vec::new(),
        };
        let parser = ParserIr {
            kind: "upper".to_owned(),
            options: BTreeMap::new(),
        };
        let context = CellContext {
            path: Path::new("<test>"),
            ir: &ir,
            location: CellLocation::Default,
            field: "name",
            parser: Some(&parser),
        };

        let value = registry
            .parse_cell(
                &CellValue::Empty,
                &parse_type("optional<string>").unwrap(),
                &context,
            )
            .unwrap();

        assert_eq!(value, Value::Null);
    }
}
