use std::{collections::BTreeMap, path::Path};

use csv::StringRecord;
use sora_data::model::{
    ConfigData, LocalizationRowData, LocalizationSourceData, RowData, TableData,
};
use sora_diagnostics::{Result, SoraError};
use sora_input::{
    cell::{CellContext, CellLocation, CellValue, cell_to_value_with_parsers},
    parser::{ParserRegistry, builtin_registry},
    source::{SourceFormat, resolve_table_source_format},
};
use sora_ir::{
    input_projection::{TaggedColumnKind, struct_columns, tagged_columns, tagged_columns_union},
    model::{ConfigIr, FieldIr, LocalizationSourceIr, TableIr, TypeIr},
};

pub fn load_csv_config_data(ir: &ConfigIr, data_root: &Path) -> Result<ConfigData> {
    load_csv_config_data_with_parsers(ir, data_root, builtin_registry())
}

pub fn load_csv_config_data_with_parsers(
    ir: &ConfigIr,
    data_root: &Path,
    parser_registry: &ParserRegistry,
) -> Result<ConfigData> {
    let mut tables = Vec::new();

    for table in &ir.tables {
        let source = table
            .source
            .as_ref()
            .ok_or_else(|| SoraError::MissingTableSource {
                table: table.name.clone(),
            })?;
        let format = resolve_table_source_format(table, Some("csv"))?;
        if format != SourceFormat::Csv {
            return Err(SoraError::InvalidSchema(format!(
                "table `{}` source format `{}` cannot be loaded by CSV input adapter",
                table.name,
                format.as_str()
            )));
        }
        tables.push(load_csv_table_data_with_parsers(
            ir,
            table,
            &data_root.join(&source.file),
            parser_registry,
        )?);
    }

    Ok(ConfigData { tables })
}

pub fn load_csv_table_data(ir: &ConfigIr, table: &TableIr, path: &Path) -> Result<TableData> {
    load_csv_table_data_with_parsers(ir, table, path, builtin_registry())
}

pub fn load_csv_table_data_with_parsers(
    ir: &ConfigIr,
    table: &TableIr,
    path: &Path,
    parser_registry: &ParserRegistry,
) -> Result<TableData> {
    let mut reader = csv::Reader::from_path(path).map_err(|source| csv_error(path, source))?;
    let headers = reader
        .headers()
        .map_err(|source| csv_error(path, source))?
        .clone();
    let header_index = header_index(&headers);
    validate_headers(ir, table, path, &headers, &header_index)?;
    let mut rows = Vec::new();

    for (record_index, record) in reader.records().enumerate() {
        let record = record.map_err(|source| csv_error(path, source))?;
        if record_is_empty(ir, table, &header_index, &record) {
            continue;
        }

        let mut values = BTreeMap::new();
        for field in &table.fields {
            if field.derived_from.is_some() {
                continue;
            }
            if tagged_columns(ir, field).is_some() {
                if let Some(value) = tagged_columns_value(
                    ir,
                    field,
                    &header_index,
                    &record,
                    CsvRowContext {
                        path,
                        row: record_index + 2,
                    },
                    parser_registry,
                )? {
                    values.insert(field.name.clone(), value);
                }
                continue;
            }
            if struct_columns(ir, field).is_some() {
                if let Some(value) = struct_columns_value(
                    ir,
                    field,
                    &header_index,
                    &record,
                    CsvRowContext {
                        path,
                        row: record_index + 2,
                    },
                    parser_registry,
                )? {
                    values.insert(field.name.clone(), value);
                }
                continue;
            }

            let column = header_index[&field.name];
            let cell = record.get(column).unwrap_or_default();
            if cell.trim().is_empty() && !matches!(field.ty, TypeIr::Optional(_)) {
                continue;
            }
            let context = CellContext {
                path,
                ir,
                location: CellLocation::Csv {
                    row: record_index + 2,
                    column: column + 1,
                },
                field: &field.name,
                parser: field.parser.as_ref(),
            };
            values.insert(
                field.name.clone(),
                cell_to_value_with_parsers(
                    &CellValue::Text(cell.trim().into()),
                    &field.ty,
                    &context,
                    parser_registry,
                )?,
            );
        }
        rows.push(RowData { values });
    }

    Ok(TableData {
        name: table.name.clone(),
        rows,
    })
}

pub fn load_csv_localization_source_data(
    source: &LocalizationSourceIr,
    path: &Path,
) -> Result<LocalizationSourceData> {
    let mut reader = csv::Reader::from_path(path).map_err(|source| csv_error(path, source))?;
    let headers = reader
        .headers()
        .map_err(|source| csv_error(path, source))?
        .clone();
    let columns = headers
        .iter()
        .map(|header| header.trim().to_owned())
        .filter(|header| !header.is_empty())
        .collect::<Vec<_>>();
    let mut rows = Vec::new();
    for record in reader.records() {
        let record = record.map_err(|source| csv_error(path, source))?;
        let mut values = BTreeMap::new();
        for (index, name) in columns.iter().enumerate() {
            let value = record.get(index).unwrap_or_default().trim();
            if !value.is_empty() {
                values.insert(name.clone(), value.to_owned());
            }
        }
        if !values.is_empty() {
            rows.push(LocalizationRowData { values });
        }
    }
    Ok(LocalizationSourceData {
        name: source.name.clone(),
        columns,
        rows,
    })
}

fn record_is_empty(
    ir: &ConfigIr,
    table: &TableIr,
    header_index: &BTreeMap<String, usize>,
    record: &StringRecord,
) -> bool {
    table
        .fields
        .iter()
        .filter(|field| field.derived_from.is_none())
        .all(|field| {
            if let Some(columns) = tagged_columns(ir, field) {
                columns.into_iter().all(|column| {
                    let index = header_index[&column.name];
                    record.get(index).unwrap_or_default().trim().is_empty()
                })
            } else if let Some(columns) = struct_columns(ir, field) {
                columns.into_iter().all(|column| {
                    let index = header_index[&column.name];
                    record.get(index).unwrap_or_default().trim().is_empty()
                })
            } else {
                let index = header_index[&field.name];
                record.get(index).unwrap_or_default().trim().is_empty()
            }
        })
}

struct CsvRowContext<'a> {
    path: &'a Path,
    row: usize,
}

fn tagged_columns_value(
    ir: &ConfigIr,
    field: &sora_ir::model::FieldIr,
    header_index: &BTreeMap<String, usize>,
    record: &StringRecord,
    row_context: CsvRowContext<'_>,
    parser_registry: &ParserRegistry,
) -> Result<Option<sora_data::model::Value>> {
    let Some(union) = tagged_columns_union(ir, field) else {
        return Ok(None);
    };
    let projected = tagged_columns(ir, field).unwrap_or_default();
    if projected.iter().all(|column| {
        let index = header_index[&column.name];
        record.get(index).unwrap_or_default().trim().is_empty()
    }) {
        return Ok(None);
    }

    let tag_column = projected
        .iter()
        .find(|column| matches!(column.kind, TaggedColumnKind::Tag))
        .expect("tagged columns should include tag column");
    let tag_index = header_index[&tag_column.name];
    let tag = record.get(tag_index).unwrap_or_default().trim().to_owned();
    if tag.is_empty() {
        return Err(CellContext {
            path: row_context.path,
            ir,
            location: CellLocation::Csv {
                row: row_context.row,
                column: tag_index + 1,
            },
            field: &field.name,
            parser: field.parser.as_ref(),
        }
        .error("tagged_columns requires a union tag"));
    }

    let Some(variant) = union.variants.iter().find(|variant| variant.name == tag) else {
        return Err(CellContext {
            path: row_context.path,
            ir,
            location: CellLocation::Csv {
                row: row_context.row,
                column: tag_index + 1,
            },
            field: &field.name,
            parser: field.parser.as_ref(),
        }
        .error(format!("unknown union variant `{tag}`")));
    };

    let mut values = BTreeMap::from([(union.tag.clone(), sora_data::model::Value::String(tag))]);
    for tagged_column in projected {
        let TaggedColumnKind::VariantField(variant_field) = tagged_column.kind else {
            continue;
        };
        let column_index = header_index[&tagged_column.name];
        let cell = record.get(column_index).unwrap_or_default().trim();
        let selected = variant
            .fields
            .iter()
            .any(|candidate| candidate.name == variant_field.name);
        if !selected {
            if !cell.is_empty() {
                return Err(CellContext {
                    path: row_context.path,
                    ir,
                    location: CellLocation::Csv {
                        row: row_context.row,
                        column: column_index + 1,
                    },
                    field: &field.name,
                    parser: field.parser.as_ref(),
                }
                .error(format!(
                    "field `{}` is not part of union variant `{}`",
                    tagged_column.name, variant.name
                )));
            }
            continue;
        }
        if cell.is_empty() {
            continue;
        }

        let nested_field = format!("{}.{}", field.name, variant_field.name);
        let context = CellContext {
            path: row_context.path,
            ir,
            location: CellLocation::Csv {
                row: row_context.row,
                column: column_index + 1,
            },
            field: &nested_field,
            parser: variant_field.parser.as_ref(),
        };
        values.insert(
            variant_field.name.clone(),
            cell_to_value_with_parsers(
                &CellValue::Text(cell.into()),
                &variant_field.ty,
                &context,
                parser_registry,
            )?,
        );
    }

    Ok(Some(sora_data::model::Value::Object(values)))
}

fn struct_columns_value(
    ir: &ConfigIr,
    field: &FieldIr,
    header_index: &BTreeMap<String, usize>,
    record: &StringRecord,
    row_context: CsvRowContext<'_>,
    parser_registry: &ParserRegistry,
) -> Result<Option<sora_data::model::Value>> {
    let projected = struct_columns(ir, field).unwrap_or_default();
    let all_empty = projected.iter().all(|column| {
        let index = header_index[&column.name];
        record.get(index).unwrap_or_default().trim().is_empty()
    });
    if all_empty {
        return Ok(matches!(field.ty, TypeIr::Optional(_)).then_some(sora_data::model::Value::Null));
    }

    let mut values = BTreeMap::new();
    for struct_column in projected {
        let column_index = header_index[&struct_column.name];
        let cell = record.get(column_index).unwrap_or_default().trim();
        if cell.is_empty() {
            continue;
        }

        let nested_field = format!("{}.{}", field.name, struct_column.field.name);
        let context = CellContext {
            path: row_context.path,
            ir,
            location: CellLocation::Csv {
                row: row_context.row,
                column: column_index + 1,
            },
            field: &nested_field,
            parser: struct_column.field.parser.as_ref(),
        };
        values.insert(
            struct_column.field.name.clone(),
            cell_to_value_with_parsers(
                &CellValue::Text(cell.into()),
                &struct_column.field.ty,
                &context,
                parser_registry,
            )?,
        );
    }

    Ok(Some(sora_data::model::Value::Object(values)))
}

fn header_index(headers: &StringRecord) -> BTreeMap<String, usize> {
    headers
        .iter()
        .enumerate()
        .map(|(index, header)| (header.trim().to_owned(), index))
        .collect()
}

fn validate_headers(
    ir: &ConfigIr,
    table: &TableIr,
    path: &Path,
    headers: &StringRecord,
    header_index: &BTreeMap<String, usize>,
) -> Result<()> {
    let expected_headers = expected_headers(ir, table);
    for header in &expected_headers {
        if !header_index.contains_key(header) {
            return Err(SoraError::ParseData {
                path: path.to_path_buf(),
                message: format!(
                    "CSV table `{}` is missing header for field `{}`",
                    table.name, header
                ),
            });
        }
    }

    for header in headers
        .iter()
        .map(str::trim)
        .filter(|header| !header.is_empty())
    {
        if !expected_headers.iter().any(|expected| expected == header) {
            return Err(SoraError::ParseData {
                path: path.to_path_buf(),
                message: format!("CSV table `{}` has unknown header `{header}`", table.name),
            });
        }
    }

    Ok(())
}

fn expected_headers(ir: &ConfigIr, table: &TableIr) -> Vec<String> {
    table
        .fields
        .iter()
        .flat_map(|field| {
            tagged_columns(ir, field)
                .map(|columns| {
                    columns
                        .into_iter()
                        .map(|column| column.name)
                        .collect::<Vec<_>>()
                })
                .or_else(|| {
                    struct_columns(ir, field).map(|columns| {
                        columns
                            .into_iter()
                            .map(|column| column.name)
                            .collect::<Vec<_>>()
                    })
                })
                .unwrap_or_else(|| vec![field.name.clone()])
        })
        .collect()
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
    use sora_data::model::Value;
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
    fn loads_struct_columns() {
        let ir = struct_columns_ir();
        let base = temp_dir();
        fs::create_dir_all(&base).unwrap();
        fs::write(
            base.join("Reward.csv"),
            "id,cost_kind,cost_id,cost_count\n1,Item,2003,4\n",
        )
        .unwrap();

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

    #[test]
    fn loads_tagged_union_columns() {
        let ir = tagged_union_ir();
        let base = temp_dir();
        fs::create_dir_all(&base).unwrap();
        fs::write(
            base.join("EventConditionEntry.csv"),
            "id,type,quest_id,item_id,count\n1,QuestCompleted,5002,,\n2,HasItem,,1001,2\n",
        )
        .unwrap();

        let data = load_csv_config_data(&ir, &base).unwrap();

        assert_eq!(data.tables[0].rows.len(), 2);
        assert_eq!(
            data.tables[0].rows[0].values["value"],
            Value::Object(BTreeMap::from([
                (
                    "type".to_owned(),
                    Value::String("QuestCompleted".to_owned())
                ),
                ("quest_id".to_owned(), Value::Integer(5002)),
            ]))
        );
        assert_eq!(
            data.tables[0].rows[1].values["value"],
            Value::Object(BTreeMap::from([
                ("type".to_owned(), Value::String("HasItem".to_owned())),
                ("item_id".to_owned(), Value::Integer(1001)),
                ("count".to_owned(), Value::Integer(2)),
            ]))
        );

        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn rejects_tagged_union_columns_for_other_variants() {
        let ir = tagged_union_ir();
        let base = temp_dir();
        fs::create_dir_all(&base).unwrap();
        fs::write(
            base.join("EventConditionEntry.csv"),
            "id,type,quest_id,item_id,count\n1,QuestCompleted,5002,1001,\n",
        )
        .unwrap();

        let error = load_csv_config_data(&ir, &base).unwrap_err();
        let message = error.to_string();

        assert!(message.contains("CSV row 2, column 4, field `value`"));
        assert!(message.contains("is not part of union variant `QuestCompleted`"));

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

[[tables.fields]]
name = "name"
type = "string"

[[tables.fields]]
name = "item_type"
type = "enum<ItemType>"

[[tables.fields]]
name = "max_stack"
type = "i32"

[[tables.fields]]
name = "tags"
type = "list<string>"
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
parser = { kind = "tuple" }
"#,
        )
        .unwrap();
        let ir = normalize_schema(schema).unwrap();
        validate_config_ir(&ir).unwrap();
        ir
    }

    fn tagged_union_ir() -> ConfigIr {
        let schema = toml::from_str(
            r#"
package = "game_config"

[[unions]]
name = "EventCondition"
tag = "type"

[[unions.variants]]
name = "QuestCompleted"

[[unions.variants.fields]]
name = "quest_id"
type = "i32"

[[unions.variants]]
name = "HasItem"

[[unions.variants.fields]]
name = "item_id"
type = "i32"

[[unions.variants.fields]]
name = "count"
type = "i32"

[[tables]]
name = "EventConditionEntry"
mode = "map"
key = "id"

[tables.source]
format = "csv"
file = "EventConditionEntry.csv"

[[tables.fields]]
name = "id"
type = "i32"

[[tables.fields]]
name = "value"
type = "union<EventCondition>"
parser = { kind = "tagged_columns", prefix = "" }
"#,
        )
        .unwrap();
        let ir = normalize_schema(schema).unwrap();
        validate_config_ir(&ir).unwrap();
        ir
    }

    fn struct_columns_ir() -> ConfigIr {
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
mode = "map"
key = "id"

[tables.source]
format = "csv"
file = "Reward.csv"

[[tables.fields]]
name = "id"
type = "i32"

[[tables.fields]]
name = "cost"
type = "struct<ResourceCost>"
parser = { kind = "columns", prefix = "cost_" }
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
