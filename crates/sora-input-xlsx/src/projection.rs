use std::path::Path;

use calamine::Data;
use sora_diagnostics::{Result, SoraError};
use sora_excel::projection::schema_hash;
use sora_ir::model::TableIr;

use crate::value::cell_to_string;

pub(crate) fn verify_projection(
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
