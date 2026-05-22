use std::path::Path;

use calamine::Data;
use sora_diagnostics::{Result, SoraError};
use sora_excel::projection::{FIELD_ROW, METADATA_ROW, schema_hash};
use sora_ir::model::{ConfigIr, TableIr};

use crate::value::cell_to_string;

pub(crate) fn verify_projection(
    ir: &ConfigIr,
    table: &TableIr,
    path: &Path,
    sheet: &str,
    range: &calamine::Range<Data>,
) -> Result<()> {
    let metadata_row = METADATA_ROW as usize;
    let field_row = FIELD_ROW as usize;

    expect_cell(path, sheet, range, metadata_row, 0, "@table")?;
    expect_cell(path, sheet, range, metadata_row, 1, &table.name)?;
    expect_cell(path, sheet, range, metadata_row, 2, "@mode")?;
    expect_cell(path, sheet, range, metadata_row, 4, "@key")?;
    expect_cell(path, sheet, range, metadata_row, 6, "@scope")?;
    expect_cell(path, sheet, range, metadata_row, 8, "@schema")?;
    expect_cell(path, sheet, range, metadata_row, 9, &schema_hash(ir, table))?;
    expect_cell(path, sheet, range, field_row, 0, "#field")?;

    for (index, field) in table.fields.iter().enumerate() {
        expect_cell(path, sheet, range, field_row, index + 1, &field.name)?;
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
