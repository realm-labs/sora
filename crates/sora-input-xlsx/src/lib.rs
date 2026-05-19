use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use calamine::{Data, Reader, open_workbook_auto};
use sora_data::{ConfigData, RowData, TableData, Value};
use sora_diagnostics::{Result, SoraError};
use sora_excel::schema_hash;
use sora_input::{DataInput, SchemaInput};
use sora_ir::{ConfigIr, TableIr, TypeIr};
use sora_schema::SchemaFile;

#[derive(Debug, Clone)]
pub struct XlsxProjectInput<S> {
    schema_input: S,
    data_root: PathBuf,
}

impl<S> XlsxProjectInput<S> {
    pub fn new(schema_input: S, data_root: impl Into<PathBuf>) -> Self {
        Self {
            schema_input,
            data_root: data_root.into(),
        }
    }

    pub fn data_root(&self) -> &Path {
        &self.data_root
    }
}

impl<S: SchemaInput> SchemaInput for XlsxProjectInput<S> {
    fn load_schema(&self) -> Result<SchemaFile> {
        self.schema_input.load_schema()
    }
}

impl<S: SchemaInput> DataInput for XlsxProjectInput<S> {
    fn load_data(&self, ir: &ConfigIr) -> Result<ConfigData> {
        load_xlsx_config_data(ir, &self.data_root)
    }
}

pub fn load_xlsx_config_data(ir: &ConfigIr, data_root: &Path) -> Result<ConfigData> {
    let mut tables = Vec::new();

    for table in &ir.tables {
        let source = table
            .source
            .as_ref()
            .ok_or_else(|| SoraError::MissingTableSource {
                table: table.name.clone(),
            })?;
        if source.format != "xlsx" {
            return Err(SoraError::InvalidSchema(format!(
                "table `{}` source format `{}` cannot be loaded by XLSX input adapter",
                table.name, source.format
            )));
        }

        let sheet = source.sheet.as_deref().unwrap_or(&table.name);
        tables.push(load_xlsx_table_data(
            table,
            &data_root.join(&source.file),
            sheet,
        )?);
    }

    Ok(ConfigData { tables })
}

pub fn load_xlsx_table_data(table: &TableIr, path: &Path, sheet: &str) -> Result<TableData> {
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

    verify_projection(table, path, sheet, &range)?;

    let mut rows = Vec::new();
    let field_names = table
        .fields
        .iter()
        .map(|field| field.name.as_str())
        .collect::<Vec<_>>();

    for row in range.rows().skip(10) {
        if row.iter().all(cell_is_empty) {
            continue;
        }

        let mut values = BTreeMap::new();
        for (column, field) in table.fields.iter().enumerate() {
            let cell = row.get(column).unwrap_or(&Data::Empty);
            if cell_is_empty(cell) {
                continue;
            }
            values.insert(
                field_names[column].to_owned(),
                cell_to_value(cell, &field.ty)?,
            );
        }
        rows.push(RowData { values });
    }

    Ok(TableData {
        name: table.name.clone(),
        rows,
    })
}

fn verify_projection(
    table: &TableIr,
    path: &Path,
    sheet: &str,
    range: &calamine::Range<Data>,
) -> Result<()> {
    expect_cell(path, sheet, range, 0, 0, "@table")?;
    expect_cell(path, sheet, range, 0, 1, &table.name)?;
    expect_cell(path, sheet, range, 3, 0, "@schema")?;
    expect_cell(path, sheet, range, 3, 1, &schema_hash(table))?;
    expect_cell(path, sheet, range, 6, 0, "#field")?;

    for (index, field) in table.fields.iter().enumerate() {
        expect_cell(path, sheet, range, 6, index + 1, &field.name)?;
    }

    Ok(())
}

fn expect_cell(
    path: &Path,
    sheet: &str,
    range: &calamine::Range<Data>,
    row: usize,
    column: usize,
    expected: &str,
) -> Result<()> {
    let actual = range
        .get((row, column))
        .map(cell_to_string)
        .unwrap_or_default();
    if actual == expected {
        Ok(())
    } else {
        Err(SoraError::InvalidSchema(format!(
            "worksheet `{}` in `{}` has `{}` at row {}, column {}; expected `{}`",
            sheet,
            path.display(),
            actual,
            row + 1,
            column + 1,
            expected
        )))
    }
}

fn cell_to_value(cell: &Data, ty: &TypeIr) -> Result<Value> {
    Ok(match ty {
        TypeIr::Bool => match cell {
            Data::Bool(value) => Value::Bool(*value),
            Data::String(value) => Value::Bool(value.eq_ignore_ascii_case("true")),
            Data::Int(value) => Value::Bool(*value != 0),
            Data::Float(value) => Value::Bool(*value != 0.0),
            _ => Value::String(cell_to_string(cell)),
        },
        TypeIr::I32 | TypeIr::I64 | TypeIr::Ref { .. } => match cell {
            Data::Int(value) => Value::Integer(*value),
            Data::Float(value) => Value::Integer(*value as i64),
            Data::String(value) => Value::Integer(value.parse::<i64>().map_err(|_| {
                SoraError::InvalidSchema(format!("failed to parse integer cell `{value}`"))
            })?),
            _ => Value::String(cell_to_string(cell)),
        },
        TypeIr::F32 | TypeIr::F64 => match cell {
            Data::Int(value) => Value::Float(*value as f64),
            Data::Float(value) => Value::Float(*value),
            Data::String(value) => Value::Float(value.parse::<f64>().map_err(|_| {
                SoraError::InvalidSchema(format!("failed to parse float cell `{value}`"))
            })?),
            _ => Value::String(cell_to_string(cell)),
        },
        TypeIr::String | TypeIr::Enum(_) | TypeIr::Struct(_) => Value::String(cell_to_string(cell)),
        TypeIr::Optional(inner) => cell_to_value(cell, inner)?,
        TypeIr::List(_) | TypeIr::Array { .. } => Value::String(cell_to_string(cell)),
    })
}

fn cell_is_empty(cell: &Data) -> bool {
    matches!(cell, Data::Empty) || matches!(cell, Data::String(value) if value.trim().is_empty())
}

fn cell_to_string(cell: &Data) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;
    use rust_xlsxwriter::Workbook;
    use sora_excel::table_template_rows;
    use sora_ir::{normalize_schema, validate_config_ir};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn loads_xlsx_rows_from_generated_projection() {
        let ir = example_ir();
        let base = temp_dir();
        let xlsx_path = base.join("Item.xlsx");
        write_item_workbook(&ir.tables[0], &xlsx_path);

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

    fn write_item_workbook(table: &TableIr, path: &Path) {
        let mut workbook = Workbook::new();
        let worksheet = workbook.add_worksheet();
        worksheet.set_name("Item").unwrap();

        for (row_index, row) in table_template_rows(table).iter().enumerate() {
            for (column_index, value) in row.iter().enumerate() {
                worksheet
                    .write_string(row_index as u32, column_index as u16, value)
                    .unwrap();
            }
        }

        let rows = [
            ["1001", "Iron Sword", "Weapon", "1"],
            ["1002", "Magic Stone", "Material", "999"],
        ];
        for (offset, row) in rows.iter().enumerate() {
            for (column, value) in row.iter().enumerate() {
                worksheet
                    .write_string((10 + offset) as u32, column as u16, *value)
                    .unwrap();
            }
        }

        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        workbook.save(path).unwrap();
    }

    fn temp_dir() -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("sora-input-xlsx-test-{unique}"))
    }
}
