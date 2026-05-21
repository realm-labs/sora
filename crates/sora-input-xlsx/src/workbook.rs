use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use calamine::{Reader, open_workbook_auto};
use sora_diagnostics::{Result, SoraError};
use sora_execution::ExecutionContext;
use sora_input::source::{SourceFormat, resolve_table_source_format};
use sora_ir::model::{ConfigIr, TableIr};

pub(crate) struct TableSheet<'a> {
    pub index: usize,
    pub table: &'a TableIr,
    pub path: PathBuf,
    pub sheet: String,
}

pub(crate) fn group_xlsx_tables<'a>(
    ir: &'a ConfigIr,
    data_root: &Path,
) -> Result<Vec<TableSheet<'a>>> {
    let mut tables = Vec::new();

    for (index, table) in ir.tables.iter().enumerate() {
        let source = table
            .source
            .as_ref()
            .ok_or_else(|| SoraError::MissingTableSource {
                table: table.name.clone(),
            })?;
        let format = resolve_table_source_format(table, Some("xlsx"))?;
        if format != SourceFormat::Xlsx {
            return Err(SoraError::InvalidSchema(format!(
                "table `{}` source format `{}` cannot be loaded by XLSX input adapter",
                table.name,
                format.as_str()
            )));
        }

        tables.push(TableSheet {
            index,
            table,
            path: data_root.join(&source.file),
            sheet: source.sheet.clone().unwrap_or_else(|| table.name.clone()),
        });
    }

    Ok(tables)
}

pub(crate) fn load_grouped_ranges<T>(
    grouped_tables: &[TableSheet<'_>],
    execution: &ExecutionContext,
    load_table: impl Fn(&TableIr, &Path, &str, calamine::Range<calamine::Data>) -> Result<T> + Sync,
) -> Result<Vec<T>>
where
    T: Send,
{
    let mut by_file = BTreeMap::<PathBuf, Vec<&TableSheet<'_>>>::new();
    for table_sheet in grouped_tables {
        by_file
            .entry(table_sheet.path.clone())
            .or_default()
            .push(table_sheet);
    }

    let grouped_files = by_file.into_iter().collect::<Vec<_>>();
    let table_groups = execution.map(grouped_files, |(path, table_sheets)| {
        let mut workbook = open_workbook_auto(&path).map_err(|source| SoraError::ParseData {
            path: path.clone(),
            message: source.to_string(),
        })?;

        let mut tables = Vec::with_capacity(table_sheets.len());
        for table_sheet in table_sheets {
            let range = workbook
                .worksheet_range(&table_sheet.sheet)
                .map_err(|source| SoraError::ParseData {
                    path: path.clone(),
                    message: format!(
                        "failed to read worksheet `{}`: {}",
                        table_sheet.sheet, source
                    ),
                })?;
            tables.push((
                table_sheet.index,
                load_table(table_sheet.table, &path, &table_sheet.sheet, range)?,
            ));
        }

        Ok(tables)
    })?;

    let mut tables = table_groups.into_iter().flatten().collect::<Vec<_>>();
    tables.sort_by_key(|(index, _)| *index);
    Ok(tables.into_iter().map(|(_, table)| table).collect())
}
