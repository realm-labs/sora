use std::{collections::BTreeMap, path::Path};

use csv::StringRecord;
use sora_data::model::{ConfigData, RowData, TableData};
use sora_diagnostics::{Result, SoraError};
use sora_input::{
    cell::{CellContext, CellLocation, CellValue, cell_to_value},
    source::{SourceFormat, resolve_table_source_format},
};
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
        let format = resolve_table_source_format(table, Some("csv"))?;
        if format != SourceFormat::Csv {
            return Err(SoraError::InvalidSchema(format!(
                "table `{}` source format `{}` cannot be loaded by CSV input adapter",
                table.name,
                format.as_str()
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
                cell_to_value(&CellValue::Text(cell.trim().into()), &field.ty, &context)?,
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

    fn temp_dir() -> PathBuf {
        let unique = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("sora-input-csv-test-{unique}"))
    }
}
