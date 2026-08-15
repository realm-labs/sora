use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use calamine::{Data, Range, Reader, open_workbook_auto};
use sora_diagnostics::{Result, SoraError};
use sora_excel::sheets::{
    ensure_single_table_definition, resolve_table_sheet_names,
    resolve_table_sheet_names_with_metadata,
};
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
    let mut tables: Vec<TableWorkbookSource<'a>> = Vec::new();

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

        let path = data_root.join(&source.file);
        if let Some(existing) = tables.iter().find(|candidate| candidate.path == path) {
            ensure_single_table_definition(&[existing.table, table], &source.file)?;
        }
        tables.push(TableWorkbookSource { index, table, path });
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
        let table_refs = table_sources
            .iter()
            .map(|source| source.table)
            .collect::<Vec<_>>();
        ensure_single_table_definition(&table_refs, &path.display().to_string())?;
        let mut workbook = open_workbook_auto(&path).map_err(|source| SoraError::ParseData {
            path: path.clone(),
            message: source.to_string(),
        })?;
        let available_sheets = workbook.sheet_names();
        let table_source = table_sources[0];
        let mut resolved_sheet_names =
            resolve_table_sheet_names(table_source.table, &available_sheets)?;
        let has_missing_sheet = resolved_sheet_names
            .iter()
            .any(|name| !available_sheets.contains(name));
        if has_missing_sheet {
            let mut table_metadata = BTreeMap::new();
            for sheet in &available_sheets {
                let range =
                    workbook
                        .worksheet_range(sheet)
                        .map_err(|source| SoraError::ParseData {
                            path: path.clone(),
                            message: format!(
                                "failed to inspect worksheet `{sheet}` metadata: {source}"
                            ),
                        })?;
                if matches!(range.get((0, 0)), Some(Data::String(value)) if value == "@table")
                    && let Some(Data::String(table)) = range.get((0, 1))
                    && !table.is_empty()
                {
                    table_metadata.insert(sheet.clone(), table.clone());
                }
            }
            resolved_sheet_names = resolve_table_sheet_names_with_metadata(
                table_source.table,
                &available_sheets,
                &table_metadata,
            )?;
        }

        let mut ranges = Vec::with_capacity(resolved_sheet_names.len());
        for sheet in resolved_sheet_names {
            let range =
                workbook
                    .worksheet_range(&sheet)
                    .map_err(|source| SoraError::ParseData {
                        path: path.clone(),
                        message: format!("failed to read worksheet `{sheet}`: {source}"),
                    })?;
            ranges.push((sheet, range));
        }
        Ok(vec![(
            table_source.index,
            load_table(table_source.table, &path, ranges)?,
        )])
    })?;

    let mut tables = table_groups.into_iter().flatten().collect::<Vec<_>>();
    tables.sort_by_key(|(index, _)| *index);
    Ok(tables.into_iter().map(|(_, table)| table).collect())
}
