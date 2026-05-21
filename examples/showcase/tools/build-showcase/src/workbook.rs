use std::{collections::BTreeMap, path::Path};

use anyhow::{Context, Result};
use rust_xlsxwriter::Workbook;
use sora_excel::projection::table_template_rows;
use sora_ir::model::ConfigIr;

use crate::rows::showcase_rows;
pub(crate) fn write_workbooks(ir: &ConfigIr, data_root: &Path) -> Result<()> {
    let mut tables_by_file = BTreeMap::<String, Vec<_>>::new();
    for table in &ir.tables {
        let Some(source) = &table.source else {
            continue;
        };
        tables_by_file
            .entry(source.file.clone())
            .or_default()
            .push(table);
    }

    for (file, tables) in tables_by_file {
        let mut workbook = Workbook::new();
        for table in tables {
            write_table_sheet(ir, &mut workbook, table)?;
        }
        let path = data_root.join(file);
        workbook
            .save(&path)
            .with_context(|| format!("failed to save `{}`", path.display()))?;
    }

    Ok(())
}

fn write_table_sheet(
    ir: &ConfigIr,
    workbook: &mut Workbook,
    table: &sora_ir::model::TableIr,
) -> Result<()> {
    let worksheet = workbook.add_worksheet();
    let sheet = table
        .source
        .as_ref()
        .and_then(|source| source.sheet.as_deref())
        .unwrap_or(&table.name);
    worksheet.set_name(sheet)?;

    for (row_index, row) in table_template_rows(ir, table).iter().enumerate() {
        for (column_index, value) in row.iter().enumerate() {
            worksheet.write_string(row_index as u32, column_index as u16, value)?;
        }
    }

    for (row_offset, row) in showcase_rows(&table.name).iter().enumerate() {
        for (column, value) in row.iter().enumerate() {
            worksheet.write_string((12 + row_offset) as u32, column as u16, value)?;
        }
    }

    Ok(())
}
