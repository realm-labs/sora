use std::path::Path;

use rust_xlsxwriter::{Format, Workbook};
use sora_diagnostics::{Result, SoraError};
use sora_ir::model::TableIr;

use crate::projection::table_template_rows;

pub(crate) fn write_workbook(tables: &[&TableIr], path: &Path) -> Result<()> {
    let mut workbook = Workbook::new();
    let metadata_format = Format::new().set_bold();

    for table in tables {
        let worksheet = workbook.add_worksheet();
        let sheet_name = table
            .source
            .as_ref()
            .and_then(|source| source.sheet.as_deref())
            .unwrap_or(&table.name);
        worksheet
            .set_name(sheet_name)
            .map_err(|source| excel_error(path, source))?;

        for (row_index, row) in table_template_rows(table).iter().enumerate() {
            for (column_index, value) in row.iter().enumerate() {
                if column_index == 0 && !value.is_empty() {
                    worksheet
                        .write_with_format(
                            row_index as u32,
                            column_index as u16,
                            value,
                            &metadata_format,
                        )
                        .map_err(|source| excel_error(path, source))?;
                } else {
                    worksheet
                        .write_string(row_index as u32, column_index as u16, value)
                        .map_err(|source| excel_error(path, source))?;
                }
            }
        }

        worksheet
            .set_freeze_panes(6, 0)
            .map_err(|source| excel_error(path, source))?;
        worksheet.autofit();
    }

    workbook
        .save(path)
        .map_err(|source| excel_error(path, source))
}

fn excel_error(path: &Path, source: impl std::fmt::Display) -> SoraError {
    SoraError::ExcelTemplate {
        path: path.to_path_buf(),
        message: source.to_string(),
    }
}
