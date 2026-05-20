use std::{collections::BTreeMap, path::Path};

use csv::StringRecord;
use serde_json::Value as JsonValue;
use sora_data::model::{ConfigData, RowData, TableData, Value};
use sora_diagnostics::{Result, SoraError};
use sora_ir::model::{ConfigIr, TableIr, TypeIr};

pub fn load_csv_config_data(ir: &ConfigIr, data_root: &Path) -> Result<ConfigData> {
    let mut tables = Vec::new();

    for table in &ir.tables {
        let source = table
            .source
            .as_ref()
            .ok_or_else(|| SoraError::MissingTableSource {
                table: table.name.clone(),
            })?;
        if source.format != "csv" {
            return Err(SoraError::InvalidSchema(format!(
                "table `{}` source format `{}` cannot be loaded by CSV input adapter",
                table.name, source.format
            )));
        }
        tables.push(load_csv_table_data(
            ir,
            table,
            &data_root.join(&source.file),
        )?);
    }

    Ok(ConfigData { tables })
}

pub fn load_csv_table_data(ir: &ConfigIr, table: &TableIr, path: &Path) -> Result<TableData> {
    let mut reader = csv::Reader::from_path(path).map_err(|source| csv_error(path, source))?;
    let headers = reader
        .headers()
        .map_err(|source| csv_error(path, source))?
        .clone();
    let header_index = header_index(&headers);
    validate_headers(table, path, &headers, &header_index)?;
    let mut rows = Vec::new();

    for (record_index, record) in reader.records().enumerate() {
        let record = record.map_err(|source| csv_error(path, source))?;
        if record.iter().all(|cell| cell.trim().is_empty()) {
            continue;
        }

        let mut values = BTreeMap::new();
        for field in &table.fields {
            let Some(column) = header_index.get(&field.name).copied() else {
                continue;
            };
            let cell = record.get(column).unwrap_or_default();
            if cell.trim().is_empty() && !matches!(field.ty, TypeIr::Optional(_)) {
                continue;
            }

            let context = CsvCellContext {
                path,
                ir,
                row: record_index + 2,
                column: column + 1,
                field: &field.name,
                parser: field.parser.as_deref(),
                separator: field.separator.as_deref(),
                prefix: field.prefix.as_deref(),
                suffix: field.suffix.as_deref(),
            };
            values.insert(
                field.name.clone(),
                string_to_value(cell, &field.ty, &context)?,
            );
        }
        rows.push(RowData { values });
    }

    Ok(TableData {
        name: table.name.clone(),
        rows,
    })
}

fn header_index(headers: &StringRecord) -> BTreeMap<String, usize> {
    headers
        .iter()
        .enumerate()
        .map(|(index, header)| (header.trim().to_owned(), index))
        .collect()
}

fn validate_headers(
    table: &TableIr,
    path: &Path,
    headers: &StringRecord,
    header_index: &BTreeMap<String, usize>,
) -> Result<()> {
    for field in &table.fields {
        if !header_index.contains_key(&field.name) {
            return Err(SoraError::ParseData {
                path: path.to_path_buf(),
                message: format!(
                    "CSV table `{}` is missing header for field `{}`",
                    table.name, field.name
                ),
            });
        }
    }

    for header in headers
        .iter()
        .map(str::trim)
        .filter(|header| !header.is_empty())
    {
        if !table.fields.iter().any(|field| field.name == header) {
            return Err(SoraError::ParseData {
                path: path.to_path_buf(),
                message: format!("CSV table `{}` has unknown header `{header}`", table.name),
            });
        }
    }

    Ok(())
}

struct CsvCellContext<'a> {
    path: &'a Path,
    ir: &'a ConfigIr,
    row: usize,
    column: usize,
    field: &'a str,
    parser: Option<&'a str>,
    separator: Option<&'a str>,
    prefix: Option<&'a str>,
    suffix: Option<&'a str>,
}

impl CsvCellContext<'_> {
    fn error(&self, message: impl Into<String>) -> SoraError {
        SoraError::ParseData {
            path: self.path.to_path_buf(),
            message: format!(
                "CSV row {}, column {}, field `{}`: {}",
                self.row,
                self.column,
                self.field,
                message.into()
            ),
        }
    }
}

fn string_to_value(source: &str, ty: &TypeIr, context: &CsvCellContext<'_>) -> Result<Value> {
    let source = source.trim();
    Ok(match ty {
        TypeIr::Optional(_) if source.is_empty() => Value::Null,
        TypeIr::Optional(inner) => string_to_value(source, inner, context)?,
        TypeIr::Bool => bool_value(source, context)?,
        TypeIr::I32 | TypeIr::I64 | TypeIr::Ref { .. } => integer_value(source, context)?,
        TypeIr::F32 | TypeIr::F64 => float_value(source, context)?,
        TypeIr::String | TypeIr::Enum(_) => Value::String(source.to_owned()),
        TypeIr::Struct(struct_name) if context.parser == Some("tuple") => {
            tuple_object_value(source, struct_name, context)?
        }
        TypeIr::Struct(_) => json_object_value(source, context)?,
        TypeIr::List(element) => separated_value(source, element, None, context)?,
        TypeIr::Array { element, len } => separated_value(source, element, Some(*len), context)?,
    })
}

fn bool_value(source: &str, context: &CsvCellContext<'_>) -> Result<Value> {
    if source.eq_ignore_ascii_case("true") {
        Ok(Value::Bool(true))
    } else if source.eq_ignore_ascii_case("false") {
        Ok(Value::Bool(false))
    } else {
        Err(context.error(format!("expected bool, got `{source}`")))
    }
}

fn integer_value(source: &str, context: &CsvCellContext<'_>) -> Result<Value> {
    source
        .parse::<i64>()
        .map(Value::Integer)
        .map_err(|_| context.error(format!("expected integer, got `{source}`")))
}

fn float_value(source: &str, context: &CsvCellContext<'_>) -> Result<Value> {
    source
        .parse::<f64>()
        .map(Value::Float)
        .map_err(|_| context.error(format!("expected float, got `{source}`")))
}

fn separated_value(
    source: &str,
    element: &TypeIr,
    expected_len: Option<usize>,
    context: &CsvCellContext<'_>,
) -> Result<Value> {
    let separator = context
        .separator
        .filter(|separator| !separator.is_empty())
        .ok_or_else(|| context.error("list and array cells require schema `separator`"))?;
    if context.prefix == Some("[") && context.suffix == Some("]") {
        return json_array_value(source, element, expected_len, context);
    }

    let source = strip_bounds(source, context)?;
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

fn strip_bounds<'a>(source: &'a str, context: &CsvCellContext<'_>) -> Result<&'a str> {
    let mut inner = source.trim();
    if let Some(prefix) = context.prefix {
        inner = inner.strip_prefix(prefix).ok_or_else(|| {
            context.error(format!(
                "expected collection prefix `{prefix}`, got `{source}`"
            ))
        })?;
    }
    if let Some(suffix) = context.suffix {
        inner = inner.strip_suffix(suffix).ok_or_else(|| {
            context.error(format!(
                "expected collection suffix `{suffix}`, got `{source}`"
            ))
        })?;
    }

    Ok(inner)
}

fn json_array_value(
    source: &str,
    element: &TypeIr,
    expected_len: Option<usize>,
    context: &CsvCellContext<'_>,
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
    context: &CsvCellContext<'_>,
) -> Result<Value> {
    match expected_type {
        TypeIr::Optional(_) if item.is_empty() => Ok(Value::Null),
        TypeIr::Optional(inner) => separated_item_to_value(item, inner, context),
        TypeIr::Bool => bool_value(item, context),
        TypeIr::I32 | TypeIr::I64 | TypeIr::Ref { .. } => integer_value(item, context),
        TypeIr::F32 | TypeIr::F64 => float_value(item, context),
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

fn string_item_to_value(item: &str, context: &CsvCellContext<'_>) -> Result<Value> {
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

fn json_object_value(source: &str, context: &CsvCellContext<'_>) -> Result<Value> {
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

fn tuple_object_value(
    source: &str,
    struct_name: &str,
    context: &CsvCellContext<'_>,
) -> Result<Value> {
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
                let nested_context = CsvCellContext {
                    path: context.path,
                    ir: context.ir,
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
                    string_to_value(item, &field.ty, &nested_context)?,
                ))
            })
            .collect::<Result<BTreeMap<_, _>>>()?,
    ))
}

fn json_to_value(
    value: &JsonValue,
    expected_type: &TypeIr,
    context: &CsvCellContext<'_>,
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

fn json_to_untyped_value(value: &JsonValue, context: &CsvCellContext<'_>) -> Result<Value> {
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

fn csv_error(path: &Path, source: csv::Error) -> SoraError {
    SoraError::ParseData {
        path: path.to_path_buf(),
        message: source.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sora_ir::{normalize::normalize_schema, validate::validate_config_ir};
    use std::{
        fs,
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };

    static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn loads_csv_rows_from_headers() {
        let ir = example_ir();
        let base = temp_dir();
        fs::create_dir_all(&base).unwrap();
        fs::write(
            base.join("Item.csv"),
            "id,name,item_type,max_stack,tags\n1001,Iron Sword,Weapon,1,\"sharp,rare\"\n1002,Magic Stone,Material,999,\"craft\"\n",
        )
        .unwrap();

        let data = load_csv_config_data(&ir, &base).unwrap();

        assert_eq!(data.tables[0].name, "Item");
        assert_eq!(data.tables[0].rows.len(), 2);
        assert_eq!(data.tables[0].rows[0].values["id"], Value::Integer(1001));
        assert_eq!(
            data.tables[0].rows[0].values["tags"],
            Value::List(vec![
                Value::String("sharp".to_owned()),
                Value::String("rare".to_owned())
            ])
        );

        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn reports_csv_cell_context_for_parse_errors() {
        let ir = example_ir();
        let base = temp_dir();
        fs::create_dir_all(&base).unwrap();
        fs::write(
            base.join("Item.csv"),
            "id,name,item_type,max_stack,tags\nbad,Iron Sword,Weapon,1,sharp\n",
        )
        .unwrap();

        let error = load_csv_config_data(&ir, &base).unwrap_err();
        let message = error.to_string();

        assert!(message.contains("Item.csv"));
        assert!(message.contains("CSV row 2, column 1, field `id`"));
        assert!(message.contains("expected integer"));

        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn rejects_unknown_csv_headers() {
        let ir = example_ir();
        let base = temp_dir();
        fs::create_dir_all(&base).unwrap();
        fs::write(
            base.join("Item.csv"),
            "id,name,item_type,max_stack,tags,typo\n1001,Iron Sword,Weapon,1,sharp,value\n",
        )
        .unwrap();

        let error = load_csv_config_data(&ir, &base).unwrap_err();
        assert!(error.to_string().contains("unknown header `typo`"));

        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn rejects_missing_csv_headers() {
        let ir = example_ir();
        let base = temp_dir();
        fs::create_dir_all(&base).unwrap();
        fs::write(
            base.join("Item.csv"),
            "id,name,item_type,max_stack\n1001,Iron Sword,Weapon,1\n",
        )
        .unwrap();

        let error = load_csv_config_data(&ir, &base).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("missing header for field `tags`")
        );

        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn loads_tuple_struct_cells() {
        let ir = tuple_ir();
        let base = temp_dir();
        fs::create_dir_all(&base).unwrap();
        fs::write(base.join("Reward.csv"), "cost\n\"Item,2003,4\"\n").unwrap();

        let data = load_csv_config_data(&ir, &base).unwrap();

        assert_eq!(
            data.tables[0].rows[0].values["cost"],
            Value::Object(BTreeMap::from([
                ("count".to_owned(), Value::Integer(4)),
                ("id".to_owned(), Value::Integer(2003)),
                ("kind".to_owned(), Value::String("Item".to_owned())),
            ]))
        );

        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn rejects_tuple_struct_cells_with_wrong_arity() {
        let ir = tuple_ir();
        let base = temp_dir();
        fs::create_dir_all(&base).unwrap();
        fs::write(base.join("Reward.csv"), "cost\n\"Item,2003\"\n").unwrap();

        let error = load_csv_config_data(&ir, &base).unwrap_err();

        assert!(error.to_string().contains(
            "tuple `ResourceCost` expects 3 values (kind: enum<ResourceType>, id: i32, count: i32) but got 2"
        ));

        let _ = fs::remove_dir_all(base);
    }

    fn example_ir() -> ConfigIr {
        let schema = toml::from_str(
            r#"
package = "game_config"

[[enums]]
name = "ItemType"
values = ["Weapon", "Armor", "Material", "Consumable"]

[[tables]]
name = "Item"
mode = "map"
key = "id"

[tables.source]
format = "csv"
file = "Item.csv"

[[tables.fields]]
name = "id"
type = "i32"
key = true
required = true

[[tables.fields]]
name = "name"
type = "string"
required = true

[[tables.fields]]
name = "item_type"
type = "enum<ItemType>"
required = true

[[tables.fields]]
name = "max_stack"
type = "i32"
required = true

[[tables.fields]]
name = "tags"
type = "list<string>"
separator = ","
"#,
        )
        .unwrap();
        let ir = normalize_schema(schema).unwrap();
        validate_config_ir(&ir).unwrap();
        ir
    }

    fn tuple_ir() -> ConfigIr {
        let schema = toml::from_str(
            r#"
package = "game_config"

[[enums]]
name = "ResourceType"
values = ["Item", "Gold"]

[[structs]]
name = "ResourceCost"

[[structs.fields]]
name = "kind"
type = "enum<ResourceType>"

[[structs.fields]]
name = "id"
type = "i32"

[[structs.fields]]
name = "count"
type = "i32"

[[tables]]
name = "Reward"
mode = "list"

[tables.source]
format = "csv"
file = "Reward.csv"

[[tables.fields]]
name = "cost"
type = "struct<ResourceCost>"
parser = "tuple"
separator = ","
"#,
        )
        .unwrap();
        let ir = normalize_schema(schema).unwrap();
        validate_config_ir(&ir).unwrap();
        ir
    }

    fn temp_dir() -> PathBuf {
        let unique = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("sora-input-csv-test-{unique}"))
    }
}
