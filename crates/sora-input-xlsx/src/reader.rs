use std::{collections::BTreeMap, path::Path};

use calamine::{Data, Reader, open_workbook_auto};
use sora_data::model::{ConfigData, RowData, TableData};
use sora_diagnostics::{Result, SoraError};
use sora_excel::projection::{DATA_START_ROW, FIELD_START_COLUMN};
use sora_execution::ExecutionContext;
use sora_input::{
    cell::{CellContext, CellLocation},
    parser::{ParserRegistry, builtin_registry},
};
use sora_ir::model::{ConfigIr, TableIr, TypeIr};

use crate::{
    projection::verify_projection,
    value::{cell_is_empty, cell_to_value_with_registry},
    workbook::{group_xlsx_tables, load_grouped_ranges},
};

pub fn load_xlsx_config_data(ir: &ConfigIr, data_root: &Path) -> Result<ConfigData> {
    load_xlsx_config_data_with_context_and_parsers(
        ir,
        data_root,
        &ExecutionContext::default(),
        builtin_registry(),
    )
}

pub fn load_xlsx_config_data_with_context(
    ir: &ConfigIr,
    data_root: &Path,
    execution: &ExecutionContext,
) -> Result<ConfigData> {
    load_xlsx_config_data_with_context_and_parsers(ir, data_root, execution, builtin_registry())
}

pub fn load_xlsx_config_data_with_parsers(
    ir: &ConfigIr,
    data_root: &Path,
    parser_registry: &ParserRegistry,
) -> Result<ConfigData> {
    load_xlsx_config_data_with_context_and_parsers(
        ir,
        data_root,
        &ExecutionContext::default(),
        parser_registry,
    )
}

pub fn load_xlsx_config_data_with_context_and_parsers(
    ir: &ConfigIr,
    data_root: &Path,
    execution: &ExecutionContext,
    parser_registry: &ParserRegistry,
) -> Result<ConfigData> {
    let grouped_tables = group_xlsx_tables(ir, data_root)?;
    let tables = load_grouped_ranges(&grouped_tables, execution, |table, path, sheet, range| {
        load_xlsx_table_data_from_range(ir, table, path, sheet, range, parser_registry)
    })?;
    Ok(ConfigData { tables })
}

pub fn load_xlsx_table_data(table: &TableIr, path: &Path, sheet: &str) -> Result<TableData> {
    load_xlsx_table_data_with_parsers(table, path, sheet, builtin_registry())
}

pub fn load_xlsx_table_data_with_parsers(
    table: &TableIr,
    path: &Path,
    sheet: &str,
    parser_registry: &ParserRegistry,
) -> Result<TableData> {
    load_xlsx_table_data_with_ir_and_parsers(
        &ConfigIr {
            package: String::new(),
            codegen: Default::default(),
            enums: Vec::new(),
            structs: Vec::new(),
            unions: Vec::new(),
            tables: vec![table.clone()],
        },
        table,
        path,
        sheet,
        parser_registry,
    )
}

pub fn load_xlsx_table_data_with_ir(
    ir: &ConfigIr,
    table: &TableIr,
    path: &Path,
    sheet: &str,
) -> Result<TableData> {
    load_xlsx_table_data_with_ir_and_parsers(ir, table, path, sheet, builtin_registry())
}

pub fn load_xlsx_table_data_with_ir_and_context(
    ir: &ConfigIr,
    table: &TableIr,
    path: &Path,
    sheet: &str,
    _execution: &ExecutionContext,
) -> Result<TableData> {
    load_xlsx_table_data_with_ir_and_parsers(ir, table, path, sheet, builtin_registry())
}

pub fn load_xlsx_table_data_with_ir_and_parsers(
    ir: &ConfigIr,
    table: &TableIr,
    path: &Path,
    sheet: &str,
    parser_registry: &ParserRegistry,
) -> Result<TableData> {
    let mut workbook = open_workbook_auto(path).map_err(|source| SoraError::ParseData {
        path: path.to_path_buf(),
        message: source.to_string(),
    })?;
    let range = workbook
        .worksheet_range(sheet)
        .map_err(|source| SoraError::ParseData {
            path: path.to_path_buf(),
            message: source.to_string(),
        })?;

    load_xlsx_table_data_from_range(ir, table, path, sheet, range, parser_registry)
}

fn load_xlsx_table_data_from_range(
    ir: &ConfigIr,
    table: &TableIr,
    path: &Path,
    sheet: &str,
    range: calamine::Range<Data>,
    parser_registry: &ParserRegistry,
) -> Result<TableData> {
    verify_projection(ir, table, path, sheet, &range)?;
    let mut rows = Vec::new();
    let field_names = table
        .fields
        .iter()
        .map(|field| field.name.as_str())
        .collect::<Vec<_>>();

    for (row_index, row) in range.rows().enumerate().skip(DATA_START_ROW as usize) {
        if row.iter().all(cell_is_empty) {
            continue;
        }

        let mut values = BTreeMap::new();
        for (column, field) in table.fields.iter().enumerate() {
            let field_column = FIELD_START_COLUMN as usize + column;
            let cell = row.get(field_column).unwrap_or(&Data::Empty);
            if cell_is_empty(cell) && !matches!(field.ty, TypeIr::Optional(_)) {
                continue;
            }
            let context = CellContext {
                path,
                ir,
                location: CellLocation::Worksheet {
                    sheet,
                    row: row_index + 1,
                    column: field_column + 1,
                },
                field: &field.name,
                parser: field.parser.as_ref(),
            };
            values.insert(
                field_names[column].to_owned(),
                cell_to_value_with_registry(cell, &field.ty, &context, parser_registry)?,
            );
        }
        rows.push(RowData { values });
    }

    Ok(TableData {
        name: table.name.clone(),
        rows,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_xlsxwriter::Workbook;
    use sora_data::model::Value;
    use sora_excel::projection::table_template_rows;
    use sora_ir::{normalize::normalize_schema, validate::validate_config_ir};
    use std::{
        collections::BTreeMap,
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };

    static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn loads_xlsx_rows_from_generated_projection() {
        let ir = example_ir();
        let base = temp_dir();
        let xlsx_path = base.join("Item.xlsx");
        write_workbook_rows(
            &ir,
            &ir.tables[0],
            &xlsx_path,
            &[
                vec!["1001", "Iron Sword", "Weapon", "1"],
                vec!["1002", "Magic Stone", "Material", "999"],
            ],
        );

        let data = load_xlsx_config_data(&ir, &base).unwrap();

        assert_eq!(data.tables[0].name, "Item");
        assert_eq!(data.tables[0].rows.len(), 2);
        assert_eq!(data.tables[0].rows[0].values["id"], Value::Integer(1001));
        assert_eq!(
            data.tables[0].rows[1].values["name"],
            Value::String("Magic Stone".to_owned())
        );

        let _ = std::fs::remove_dir_all(base);
    }

    #[test]
    fn loads_complex_xlsx_cell_values() {
        let ir = complex_ir();
        let base = temp_dir();
        let xlsx_path = base.join("Item.xlsx");
        write_workbook_rows(
            &ir,
            &ir.tables[0],
            &xlsx_path,
            &[vec![
                "1001",
                "",
                "sharp,rare",
                "1,2",
                "{\"item_id\":1001,\"count\":2}",
            ]],
        );

        let data = load_xlsx_config_data(&ir, &base).unwrap();
        let values = &data.tables[0].rows[0].values;

        assert_eq!(values["optional_note"], Value::Null);
        assert_eq!(
            values["tags"],
            Value::List(vec![
                Value::String("sharp".to_owned()),
                Value::String("rare".to_owned())
            ])
        );
        assert_eq!(
            values["coords"],
            Value::List(vec![Value::Integer(1), Value::Integer(2)])
        );
        assert_eq!(
            values["reward"],
            Value::Object(BTreeMap::from([
                ("count".to_owned(), Value::Integer(2)),
                ("item_id".to_owned(), Value::Integer(1001))
            ]))
        );

        let _ = std::fs::remove_dir_all(base);
    }

    #[test]
    fn reports_cell_context_for_parse_errors() {
        let ir = example_ir();
        let base = temp_dir();
        let xlsx_path = base.join("Item.xlsx");
        write_workbook_rows(
            &ir,
            &ir.tables[0],
            &xlsx_path,
            &[vec!["not-an-int", "Iron Sword", "Weapon", "1"]],
        );

        let error = load_xlsx_config_data(&ir, &base).unwrap_err();
        let message = error.to_string();

        assert!(message.contains("Item.xlsx"));
        assert!(message.contains("worksheet `Item` row 8, column 2, field `id`"));
        assert!(message.contains("expected integer"));

        let _ = std::fs::remove_dir_all(base);
    }

    #[test]
    fn loads_tuple_list_xlsx_cell_values() {
        let ir = tuple_list_ir();
        let base = temp_dir();
        let xlsx_path = base.join("Recipe.xlsx");
        write_workbook_rows(
            &ir,
            &ir.tables[0],
            &xlsx_path,
            &[vec!["1001", "Item,2003,4|Gold,0,1000"]],
        );

        let data = load_xlsx_config_data(&ir, &base).unwrap();

        assert_eq!(
            data.tables[0].rows[0].values["materials"],
            Value::List(vec![
                Value::Object(BTreeMap::from([
                    ("count".to_owned(), Value::Integer(4)),
                    ("id".to_owned(), Value::Integer(2003)),
                    ("kind".to_owned(), Value::String("Item".to_owned())),
                ])),
                Value::Object(BTreeMap::from([
                    ("count".to_owned(), Value::Integer(1000)),
                    ("id".to_owned(), Value::Integer(0)),
                    ("kind".to_owned(), Value::String("Gold".to_owned())),
                ])),
            ])
        );

        let _ = std::fs::remove_dir_all(base);
    }

    #[test]
    fn loads_tuple_struct_cell_values() {
        let ir = tuple_ir();
        let base = temp_dir();
        let xlsx_path = base.join("Reward.xlsx");
        write_workbook_rows(&ir, &ir.tables[0], &xlsx_path, &[vec!["Item,2003,4"]]);

        let data = load_xlsx_config_data(&ir, &base).unwrap();

        assert_eq!(
            data.tables[0].rows[0].values["cost"],
            Value::Object(BTreeMap::from([
                ("count".to_owned(), Value::Integer(4)),
                ("id".to_owned(), Value::Integer(2003)),
                ("kind".to_owned(), Value::String("Item".to_owned())),
            ]))
        );

        let _ = std::fs::remove_dir_all(base);
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
format = "xlsx"
file = "Item.xlsx"
sheet = "Item"

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
"#,
        )
        .unwrap();
        let ir = normalize_schema(schema).unwrap();
        validate_config_ir(&ir).unwrap();
        ir
    }

    fn complex_ir() -> ConfigIr {
        let schema = toml::from_str(
            r#"
package = "game_config"

[[structs]]
name = "Reward"

[[structs.fields]]
name = "item_id"
type = "i32"

[[structs.fields]]
name = "count"
type = "i32"

[[tables]]
name = "Item"
mode = "map"
key = "id"

[tables.source]
format = "xlsx"
file = "Item.xlsx"
sheet = "Item"

[[tables.fields]]
name = "id"
type = "i32"
key = true
required = true

[[tables.fields]]
name = "optional_note"
type = "optional<string>"

[[tables.fields]]
name = "tags"
type = "list<string>"

[[tables.fields]]
name = "coords"
type = "array<i32,2>"

[[tables.fields]]
name = "reward"
type = "struct<Reward>"
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
format = "xlsx"
file = "Reward.xlsx"
sheet = "Reward"

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

    fn tuple_list_ir() -> ConfigIr {
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
name = "Recipe"
mode = "map"
key = "id"

[tables.source]
format = "xlsx"
file = "Recipe.xlsx"
sheet = "Recipe"

[[tables.fields]]
name = "id"
type = "i32"
key = true
required = true

[[tables.fields]]
name = "materials"
type = "list<ResourceCost>"
parser = { kind = "tuple_list" }
"#,
        )
        .unwrap();
        let ir = normalize_schema(schema).unwrap();
        validate_config_ir(&ir).unwrap();
        ir
    }

    fn write_workbook_rows(ir: &ConfigIr, table: &TableIr, path: &Path, rows: &[Vec<&str>]) {
        let mut workbook = Workbook::new();
        let worksheet = workbook.add_worksheet();
        worksheet.set_name(&table.name).unwrap();

        for (row_index, row) in table_template_rows(ir, table).iter().enumerate() {
            for (column_index, value) in row.iter().enumerate() {
                worksheet
                    .write_string(row_index as u32, column_index as u16, value)
                    .unwrap();
            }
        }

        for (offset, row) in rows.iter().enumerate() {
            for (column, value) in row.iter().enumerate() {
                worksheet
                    .write_string(
                        DATA_START_ROW + offset as u32,
                        FIELD_START_COLUMN + column as u16,
                        *value,
                    )
                    .unwrap();
            }
        }

        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        workbook.save(path).unwrap();
    }

    fn temp_dir() -> PathBuf {
        let unique = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("sora-input-xlsx-test-{unique}"))
    }
}
