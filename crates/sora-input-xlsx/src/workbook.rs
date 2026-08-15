use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use calamine::{Data, Range, Reader, open_workbook_auto};
use sora_diagnostics::{Result, SoraError};
use sora_excel::sheets::resolve_table_sheet_names;
use sora_execution::ExecutionContext;
use sora_input::source::{SourceFormat, resolve_table_source_format};
use sora_ir::model::{ConfigIr, TableIr};

pub(crate) struct TableWorkbookSource<'a> {
    pub index: usize,
    pub table: &'a TableIr,
    pub path: PathBuf,
}

pub(crate) fn group_xlsx_tables<'a>(
    ir: &'a ConfigIr,
    data_root: &Path,
) -> Result<Vec<TableWorkbookSource<'a>>> {
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

        tables.push(TableWorkbookSource {
            index,
            table,
            path: data_root.join(&source.file),
        });
    }

    Ok(tables)
}

pub(crate) fn load_grouped_ranges<T>(
    grouped_tables: &[TableWorkbookSource<'_>],
    execution: &ExecutionContext,
    load_table: impl Fn(&TableIr, &Path, Vec<(String, Range<Data>)>) -> Result<T> + Sync,
) -> Result<Vec<T>>
where
    T: Send,
{
    let mut by_file = BTreeMap::<PathBuf, Vec<&TableWorkbookSource<'_>>>::new();
    for table_source in grouped_tables {
        by_file
            .entry(table_source.path.clone())
            .or_default()
            .push(table_source);
    }

    let grouped_files = by_file.into_iter().collect::<Vec<_>>();
    let table_groups = execution.map(grouped_files, |(path, table_sources)| {
        let mut workbook = open_workbook_auto(&path).map_err(|source| SoraError::ParseData {
            path: path.clone(),
            message: source.to_string(),
        })?;
        let available_sheets = workbook.sheet_names();

        let mut tables = Vec::with_capacity(table_sources.len());
        for table_source in table_sources {
            let sheet_names = resolve_table_sheet_names(table_source.table, &available_sheets)?;
            let mut ranges = Vec::with_capacity(sheet_names.len());
            for sheet in sheet_names {
                let range =
                    workbook
                        .worksheet_range(&sheet)
                        .map_err(|source| SoraError::ParseData {
                            path: path.clone(),
                            message: format!("failed to read worksheet `{sheet}`: {source}"),
                        })?;
                ranges.push((sheet, range));
            }
            tables.push((
                table_source.index,
                load_table(table_source.table, &path, ranges)?,
            ));
        }

        Ok(tables)
    })?;

    let mut tables = table_groups.into_iter().flatten().collect::<Vec<_>>();
    tables.sort_by_key(|(index, _)| *index);
    Ok(tables.into_iter().map(|(_, table)| table).collect())
}
