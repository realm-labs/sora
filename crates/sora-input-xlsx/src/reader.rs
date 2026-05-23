use std::{
    collections::{BTreeMap, HashMap},
    path::Path,
};

use calamine::{Data, Reader, open_workbook_auto};
use sora_data::model::{ConfigData, RowData, TableData, Value};
use sora_diagnostics::{Result, SoraError};
use sora_excel::projection::{DATA_START_ROW, FIELD_ROW, FIELD_START_COLUMN};
use sora_execution::ExecutionContext;
use sora_input::{
    cell::{CellContext, CellLocation},
    parser::{ParserRegistry, builtin_registry},
};
use sora_ir::{
    input_projection::{TaggedColumnKind, tagged_columns, tagged_columns_union},
    model::{ConfigIr, TableIr, TypeIr},
};

use crate::{
    projection::verify_projection,
    value::{cell_is_empty, cell_to_string, cell_to_value_with_registry},
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
    let field_columns = field_columns(table, ir, path, sheet, &range)?;

    for (row_index, row) in range.rows().enumerate().skip(DATA_START_ROW as usize) {
        if row_is_empty(row, &field_columns) {
            continue;
        }

        let mut values = BTreeMap::new();
        for (column, field) in table.fields.iter().enumerate() {
            match &field_columns[column] {
                FieldColumns::Single(field_column) => {
                    let cell = row.get(*field_column).unwrap_or(&Data::Empty);
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
                        field.name.clone(),
                        cell_to_value_with_registry(cell, &field.ty, &context, parser_registry)?,
                    );
                }
                FieldColumns::Tagged(columns) => {
                    if let Some(value) = tagged_columns_value(
                        ir,
                        field,
                        columns,
                        row,
                        TaggedRowContext {
                            path,
                            sheet,
                            row: row_index + 1,
                        },
                        parser_registry,
                    )? {
                        values.insert(field.name.clone(), value);
                    }
                }
            }
        }
        rows.push(RowData { values });
    }

    Ok(TableData {
        name: table.name.clone(),
        rows,
    })
}

#[derive(Debug, Clone)]
enum FieldColumns {
    Single(usize),
    Tagged(BTreeMap<String, usize>),
}

fn field_columns(
    table: &TableIr,
    ir: &ConfigIr,
    path: &Path,
    sheet: &str,
    range: &calamine::Range<Data>,
) -> Result<Vec<FieldColumns>> {
    let expected_columns = table
        .fields
        .iter()
        .enumerate()
        .flat_map(|(index, field)| {
            tagged_columns(ir, field)
                .map(|columns| {
                    columns
                        .into_iter()
                        .map(move |column| (column.name, index))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_else(|| vec![(field.name.clone(), index)])
        })
        .collect::<HashMap<_, _>>();
    let mut columns = table
        .fields
        .iter()
        .map(|field| {
            if tagged_columns(ir, field).is_some() {
                FieldColumnsBuilder::Tagged(BTreeMap::new())
            } else {
                FieldColumnsBuilder::Single(None)
            }
        })
        .collect::<Vec<_>>();
    let Some(field_row) = range.rows().nth(FIELD_ROW as usize) else {
        return Err(SoraError::InvalidSchema(format!(
            "worksheet `{}` in `{}` is missing #field row {}",
            sheet,
            path.display(),
            FIELD_ROW + 1
        )));
    };

    for (column, cell) in field_row
        .iter()
        .enumerate()
        .skip(FIELD_START_COLUMN as usize)
    {
        let field_name = crate::value::cell_to_string(cell);
        let field_name = field_name.trim();
        if field_name.is_empty() {
            continue;
        }
        let Some(&field_index) = expected_columns.get(field_name) else {
            continue;
        };
        match &mut columns[field_index] {
            FieldColumnsBuilder::Single(value) => {
                if value.replace(column).is_some() {
                    return Err(duplicate_field_column(path, sheet, field_name));
                }
            }
            FieldColumnsBuilder::Tagged(values) => {
                if values.insert(field_name.to_owned(), column).is_some() {
                    return Err(duplicate_field_column(path, sheet, field_name));
                }
            }
        }
    }

    columns
        .into_iter()
        .enumerate()
        .map(|(index, columns)| match columns {
            FieldColumnsBuilder::Single(column) => column
                .map(FieldColumns::Single)
                .ok_or_else(|| missing_field_column(path, sheet, &table.fields[index].name)),
            FieldColumnsBuilder::Tagged(values) => {
                let expected = tagged_columns(ir, &table.fields[index])
                    .expect("tagged builder should have tagged field");
                for column in expected {
                    if !values.contains_key(&column.name) {
                        return Err(missing_field_column(path, sheet, &column.name));
                    }
                }
                Ok(FieldColumns::Tagged(values))
            }
        })
        .collect()
}

enum FieldColumnsBuilder {
    Single(Option<usize>),
    Tagged(BTreeMap<String, usize>),
}

fn duplicate_field_column(path: &Path, sheet: &str, field_name: &str) -> SoraError {
    SoraError::InvalidSchema(format!(
        "worksheet `{}` in `{}` declares duplicate field `{}` in #field row",
        sheet,
        path.display(),
        field_name
    ))
}

fn missing_field_column(path: &Path, sheet: &str, field_name: &str) -> SoraError {
    SoraError::InvalidSchema(format!(
        "worksheet `{}` in `{}` is missing field `{}` in #field row",
        sheet,
        path.display(),
        field_name
    ))
}

struct TaggedRowContext<'a> {
    path: &'a Path,
    sheet: &'a str,
    row: usize,
}

fn tagged_columns_value(
    ir: &ConfigIr,
    field: &sora_ir::model::FieldIr,
    columns: &BTreeMap<String, usize>,
    row: &[Data],
    row_context: TaggedRowContext<'_>,
    parser_registry: &ParserRegistry,
) -> Result<Option<Value>> {
    let Some(union) = tagged_columns_union(ir, field) else {
        return Ok(None);
    };
    let projected = tagged_columns(ir, field).unwrap_or_default();
    if projected.iter().all(|column| {
        let index = columns[&column.name];
        row.get(index).is_none_or(cell_is_empty)
    }) {
        return Ok(None);
    }

    let tag_column = projected
        .iter()
        .find(|column| matches!(column.kind, TaggedColumnKind::Tag))
        .expect("tagged columns should include tag column");
    let tag_index = columns[&tag_column.name];
    let tag_cell = row.get(tag_index).unwrap_or(&Data::Empty);
    let tag = cell_to_string(tag_cell).trim().to_owned();
    if tag.is_empty() {
        return Err(CellContext {
            path: row_context.path,
            ir,
            location: CellLocation::Worksheet {
                sheet: row_context.sheet,
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
            location: CellLocation::Worksheet {
                sheet: row_context.sheet,
                row: row_context.row,
                column: tag_index + 1,
            },
            field: &field.name,
            parser: field.parser.as_ref(),
        }
        .error(format!("unknown union variant `{tag}`")));
    };

    let mut values = BTreeMap::from([(union.tag.clone(), Value::String(tag))]);
    for tagged_column in projected {
        let TaggedColumnKind::VariantField(variant_field) = tagged_column.kind else {
            continue;
        };
        let column_index = columns[&tagged_column.name];
        let cell = row.get(column_index).unwrap_or(&Data::Empty);
        let selected = variant
            .fields
            .iter()
            .any(|candidate| candidate.name == variant_field.name);
        if !selected {
            if !cell_is_empty(cell) {
                return Err(CellContext {
                    path: row_context.path,
                    ir,
                    location: CellLocation::Worksheet {
                        sheet: row_context.sheet,
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
        if cell_is_empty(cell) {
            continue;
        }
        let nested_field = format!("{}.{}", field.name, variant_field.name);
        let context = CellContext {
            path: row_context.path,
            ir,
            location: CellLocation::Worksheet {
                sheet: row_context.sheet,
                row: row_context.row,
                column: column_index + 1,
            },
            field: &nested_field,
            parser: variant_field.parser.as_ref(),
        };
        values.insert(
            variant_field.name.clone(),
            cell_to_value_with_registry(cell, &variant_field.ty, &context, parser_registry)?,
        );
    }

    Ok(Some(Value::Object(values)))
}

fn row_is_empty(row: &[Data], field_columns: &[FieldColumns]) -> bool {
    field_columns.iter().all(|columns| match columns {
        FieldColumns::Single(column) => row.get(*column).is_none_or(cell_is_empty),
        FieldColumns::Tagged(columns) => columns
            .values()
            .all(|column| row.get(*column).is_none_or(cell_is_empty)),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_xlsxwriter::{Format, Workbook};
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
    fn loads_xlsx_rows_by_field_names_and_ignores_unmapped_cells() {
        let ir = example_ir();
        let base = temp_dir();
        let xlsx_path = base.join("Item.xlsx");
        write_workbook_rows_with_field_columns(
            &ir,
            &ir.tables[0],
            &xlsx_path,
            &["name", "", "notes", "id", "max_stack", "item_type"],
            &[
                vec!["", "", "draft only", "", "", ""],
                vec!["Iron Sword", "", "designer note", "1001", "1", "Weapon"],
                vec!["Magic Stone", "", "", "1002", "999", "Material"],
            ],
        );

        let data = load_xlsx_config_data(&ir, &base).unwrap();

        assert_eq!(data.tables[0].rows.len(), 2);
        assert_eq!(data.tables[0].rows[0].values["id"], Value::Integer(1001));
        assert_eq!(
            data.tables[0].rows[0].values["name"],
            Value::String("Iron Sword".to_owned())
        );
        assert_eq!(
            data.tables[0].rows[1].values["item_type"],
            Value::String("Material".to_owned())
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

    #[test]
    fn loads_tagged_union_columns() {
        let ir = tagged_union_ir();
        let base = temp_dir();
        let xlsx_path = base.join("EventConditionEntry.xlsx");
        write_workbook_rows_with_field_columns(
            &ir,
            &ir.tables[0],
            &xlsx_path,
            &["id", "type", "quest_id", "item_id", "count"],
            &[
                vec!["1", "QuestCompleted", "5002", "", ""],
                vec!["2", "HasItem", "", "1001", "2"],
            ],
        );

        let data = load_xlsx_config_data(&ir, &base).unwrap();

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

        let _ = std::fs::remove_dir_all(base);
    }

    #[test]
    fn rejects_tagged_union_columns_for_other_variants() {
        let ir = tagged_union_ir();
        let base = temp_dir();
        let xlsx_path = base.join("EventConditionEntry.xlsx");
        write_workbook_rows_with_field_columns(
            &ir,
            &ir.tables[0],
            &xlsx_path,
            &["id", "type", "quest_id", "item_id", "count"],
            &[vec!["1", "QuestCompleted", "5002", "1001", ""]],
        );

        let error = load_xlsx_config_data(&ir, &base).unwrap_err();
        let message = error.to_string();

        assert!(message.contains("worksheet `EventConditionEntry` row 8, column 5, field `value`"));
        assert!(message.contains("is not part of union variant `QuestCompleted`"));

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

[[tables.fields]]
name = "name"
type = "string"

[[tables.fields]]
name = "item_type"
type = "enum<ItemType>"

[[tables.fields]]
name = "max_stack"
type = "i32"
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
format = "xlsx"
file = "EventConditionEntry.xlsx"
sheet = "EventConditionEntry"

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

    fn write_workbook_rows(ir: &ConfigIr, table: &TableIr, path: &Path, rows: &[Vec<&str>]) {
        let field_names = table
            .fields
            .iter()
            .map(|field| field.name.as_str())
            .collect::<Vec<_>>();
        write_workbook_rows_with_field_columns(ir, table, path, &field_names, rows);
    }

    fn write_workbook_rows_with_field_columns(
        ir: &ConfigIr,
        table: &TableIr,
        path: &Path,
        field_columns: &[&str],
        rows: &[Vec<&str>],
    ) {
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

        for (column, field_name) in field_columns.iter().enumerate() {
            let column = FIELD_START_COLUMN + column as u16;
            if field_name.is_empty() {
                worksheet
                    .write_blank(FIELD_ROW, column, &Format::new())
                    .unwrap();
            } else {
                worksheet
                    .write_string(FIELD_ROW, column, *field_name)
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
