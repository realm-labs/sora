use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    fs, io,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use calamine::{Data, Reader, open_workbook_auto};
use sora_diagnostics::{Result, SoraError};
use sora_input::source::{SourceFormat, resolve_table_source_format};
use sora_ir::model::{ConfigIr, TableIr};

use crate::{
    projection::{
        DATA_START_ROW, FIELD_ROW, FIELD_START_COLUMN, table_template_columns, table_template_rows,
    },
    sheets::resolve_table_sheet_names,
    writer::{LegacyColumn, PreservedSheet, SyncedTableSheet, write_synced_workbook},
};

#[derive(Debug, Default)]
pub struct ExcelSyncReport {
    pub workbooks: Vec<ExcelSyncWorkbookReport>,
}

impl ExcelSyncReport {
    pub fn is_empty(&self) -> bool {
        self.workbooks.is_empty()
    }
}

#[derive(Debug)]
pub struct ExcelSyncWorkbookReport {
    pub path: PathBuf,
    pub created: bool,
    pub written: bool,
    pub backup_path: Option<PathBuf>,
    pub sheets: Vec<ExcelSyncSheetReport>,
    pub preserved_sheets: Vec<String>,
}

#[derive(Debug)]
pub struct ExcelSyncSheetReport {
    pub sheet: String,
    pub created: bool,
    pub changed: bool,
    pub rows: usize,
    pub added_columns: Vec<String>,
    pub legacy_columns: Vec<String>,
}

pub struct ExcelTemplateSync;

impl ExcelTemplateSync {
    pub fn preview(&self, ir: &ConfigIr, data_root: &Path) -> Result<ExcelSyncReport> {
        self.sync(ir, data_root, false)
    }

    pub fn write(&self, ir: &ConfigIr, data_root: &Path) -> Result<ExcelSyncReport> {
        self.sync(ir, data_root, true)
    }

    fn sync(&self, ir: &ConfigIr, data_root: &Path, write: bool) -> Result<ExcelSyncReport> {
        let mut report = ExcelSyncReport::default();
        for (file_name, tables) in group_xlsx_tables(ir)? {
            let path = data_root.join(file_name);
            ensure_workbook_path_is_bounded(data_root, &path)?;
            let existing = ExistingWorkbook::load(&path)?;
            let mut table_sheets = Vec::new();
            let mut sheet_reports = Vec::new();
            let mut handled_sheets = BTreeSet::new();

            for table in tables {
                let sheet_names = resolve_table_sheet_names(table, &existing.sheet_order)?;
                for sheet_name in sheet_names {
                    if !handled_sheets.insert(sheet_name.clone()) {
                        return Err(SoraError::InvalidSchema(format!(
                            "worksheet `{sheet_name}` in `{}` is selected by more than one table",
                            path.display()
                        )));
                    }
                    let existing_sheet = existing.sheets.get(&sheet_name).map(Vec::as_slice);
                    let synced = sync_table_sheet(ir, table, &sheet_name, existing_sheet);
                    sheet_reports.push(synced.report);
                    table_sheets.push(synced.sheet);
                }
            }

            let preserved_sheets = existing
                .sheet_order
                .iter()
                .filter(|name| !handled_sheets.contains(*name))
                .filter_map(|name| {
                    existing.sheets.get(name).map(|rows| PreservedSheet {
                        sheet_name: name.clone(),
                        rows: rows.clone(),
                    })
                })
                .collect::<Vec<_>>();

            let has_changes = sheet_reports.iter().any(|sheet| sheet.changed);

            let backup_path = if write && existing.exists && has_changes {
                Some(backup_existing_workbook(data_root, &path)?)
            } else {
                None
            };

            if write && has_changes {
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent).map_err(|source| SoraError::CreateDir {
                        path: parent.to_path_buf(),
                        source,
                    })?;
                }
                write_synced_workbook_transactionally(
                    ir,
                    &table_sheets,
                    &preserved_sheets,
                    &path,
                    backup_path.as_deref(),
                )?;
            }

            report.workbooks.push(ExcelSyncWorkbookReport {
                path,
                created: !existing.exists,
                written: write && has_changes,
                backup_path,
                sheets: sheet_reports,
                preserved_sheets: preserved_sheets
                    .into_iter()
                    .map(|sheet| sheet.sheet_name)
                    .collect(),
            });
        }

        Ok(report)
    }
}

fn ensure_workbook_path_is_bounded(data_root: &Path, path: &Path) -> Result<()> {
    let relative = path
        .strip_prefix(data_root)
        .map_err(|_| SoraError::ExcelTemplate {
            path: path.to_path_buf(),
            message: "workbook path is outside the configured data root".to_owned(),
        })?;
    if relative.as_os_str().is_empty()
        || relative.components().any(|component| {
            matches!(
                component,
                std::path::Component::ParentDir
                    | std::path::Component::RootDir
                    | std::path::Component::Prefix(_)
            )
        })
    {
        return Err(SoraError::ExcelTemplate {
            path: path.to_path_buf(),
            message: "workbook path contains unsafe traversal".to_owned(),
        });
    }

    let resolved_root = resolve_through_existing_ancestor(data_root)?;
    let resolved_path = resolve_through_existing_ancestor(path)?;
    if !resolved_path.starts_with(&resolved_root) {
        return Err(SoraError::ExcelTemplate {
            path: path.to_path_buf(),
            message: "workbook path resolves outside the configured data root".to_owned(),
        });
    }
    Ok(())
}

fn resolve_through_existing_ancestor(path: &Path) -> Result<PathBuf> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|source| SoraError::ReadFile {
                path: path.to_path_buf(),
                source,
            })?
            .join(path)
    };
    let existing = absolute
        .ancestors()
        .find(|ancestor| ancestor.exists())
        .ok_or_else(|| SoraError::ExcelTemplate {
            path: path.to_path_buf(),
            message: "workbook path has no existing ancestor".to_owned(),
        })?;
    let mut resolved = fs::canonicalize(existing).map_err(|source| SoraError::ReadFile {
        path: existing.to_path_buf(),
        source,
    })?;
    let unresolved = absolute
        .strip_prefix(existing)
        .expect("an ancestor must be a path prefix");
    for component in unresolved.components() {
        match component {
            std::path::Component::Normal(segment) => resolved.push(segment),
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir
            | std::path::Component::RootDir
            | std::path::Component::Prefix(_) => {
                return Err(SoraError::ExcelTemplate {
                    path: path.to_path_buf(),
                    message: "workbook path contains unsafe unresolved traversal".to_owned(),
                });
            }
        }
    }
    Ok(resolved)
}

static WORKBOOK_WRITE_COUNTER: AtomicU64 = AtomicU64::new(0);
static BACKUP_COUNTER: AtomicU64 = AtomicU64::new(0);
const MAX_BACKUP_BATCHES: usize = 20;
const BACKUP_GITIGNORE: &[u8] = b"*\n";

fn write_synced_workbook_transactionally(
    ir: &ConfigIr,
    table_sheets: &[SyncedTableSheet<'_>],
    preserved_sheets: &[PreservedSheet],
    path: &Path,
    backup_path: Option<&Path>,
) -> Result<()> {
    let temp_path = sibling_temp_path(path);
    write_synced_workbook(ir, table_sheets, preserved_sheets, &temp_path)?;
    if let Err(source) = replace_file(&temp_path, path) {
        let _ = fs::remove_file(&temp_path);
        if let Some(backup_path) = backup_path {
            let _ = fs::copy(backup_path, path);
        }
        return Err(SoraError::WriteFile {
            path: path.to_path_buf(),
            source,
        });
    }
    Ok(())
}

fn replace_file(source: &Path, target: &Path) -> io::Result<()> {
    match fs::rename(source, target) {
        Ok(()) => Ok(()),
        Err(error) if target.exists() => {
            fs::remove_file(target)?;
            fs::rename(source, target).map_err(|second_error| {
                io::Error::new(
                    second_error.kind(),
                    format!("failed after initial replace error `{error}`: {second_error}"),
                )
            })
        }
        Err(error) => Err(error),
    }
}

fn sibling_temp_path(target: &Path) -> PathBuf {
    let id = WORKBOOK_WRITE_COUNTER.fetch_add(1, Ordering::Relaxed);
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    let name = target
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("workbook.xlsx");
    target.with_file_name(format!(
        ".{name}.sora-excel-write-{}-{timestamp}-{id}.tmp",
        std::process::id()
    ))
}

fn backup_existing_workbook(data_root: &Path, path: &Path) -> Result<PathBuf> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    let sequence = BACKUP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let relative = path.strip_prefix(data_root).unwrap_or(path);
    let backup_root = data_root.join(".sora-backup");
    ensure_backup_root(&backup_root)?;
    let backup_path = backup_root
        .join(format!("{timestamp}-{}-{sequence}", std::process::id()))
        .join(relative);
    if let Some(parent) = backup_path.parent() {
        fs::create_dir_all(parent).map_err(|source| SoraError::CreateDir {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    fs::copy(path, &backup_path).map_err(|source| SoraError::WriteFile {
        path: backup_path.clone(),
        source,
    })?;
    prune_backup_batches(&backup_root);
    Ok(backup_path)
}

fn ensure_backup_root(path: &Path) -> Result<()> {
    fs::create_dir_all(path).map_err(|source| SoraError::CreateDir {
        path: path.to_path_buf(),
        source,
    })?;
    let ignore = path.join(".gitignore");
    if !ignore.exists() {
        fs::write(&ignore, BACKUP_GITIGNORE).map_err(|source| SoraError::WriteFile {
            path: ignore,
            source,
        })?;
    }
    Ok(())
}

fn prune_backup_batches(backup_root: &Path) {
    let Ok(entries) = fs::read_dir(backup_root) else {
        return;
    };
    let mut batches = entries
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()))
        .map(|entry| {
            let modified = entry
                .metadata()
                .and_then(|metadata| metadata.modified())
                .unwrap_or(SystemTime::UNIX_EPOCH);
            (modified, entry.file_name(), entry.path())
        })
        .collect::<Vec<_>>();
    batches.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)));
    let remove_count = batches.len().saturating_sub(MAX_BACKUP_BATCHES);
    for (_, _, path) in batches.into_iter().take(remove_count) {
        let _ = fs::remove_dir_all(path);
    }
}

struct SyncedSheet<'a> {
    sheet: SyncedTableSheet<'a>,
    report: ExcelSyncSheetReport,
}

fn sync_table_sheet<'a>(
    ir: &ConfigIr,
    table: &'a TableIr,
    sheet_name: &str,
    existing_rows: Option<&[Vec<String>]>,
) -> SyncedSheet<'a> {
    let columns = table_template_columns(ir, table);
    let schema_names = columns
        .iter()
        .map(|column| column.name.clone())
        .collect::<BTreeSet<_>>();
    let old_columns = existing_rows
        .and_then(|rows| rows.get(FIELD_ROW as usize))
        .map(|row| old_field_columns(row))
        .unwrap_or_default();
    let old_by_name = old_columns
        .iter()
        .map(|column| (column.name.clone(), column.index))
        .collect::<HashMap<_, _>>();

    let data_row_count = existing_rows
        .map(|rows| rows.len().saturating_sub(DATA_START_ROW as usize))
        .unwrap_or_default();
    let rows = (0..data_row_count)
        .map(|row_offset| {
            columns
                .iter()
                .map(|column| {
                    old_by_name
                        .get(&column.name)
                        .and_then(|index| {
                            cell(existing_rows, DATA_START_ROW as usize + row_offset, *index)
                        })
                        .unwrap_or_default()
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    let mut legacy_columns = Vec::new();
    let mut legacy_names = Vec::new();
    for old_column in &old_columns {
        if schema_names.contains(&old_column.name) {
            continue;
        }
        legacy_names.push(old_column.name.clone());
        legacy_columns.push(LegacyColumn {
            headers: (0..DATA_START_ROW as usize)
                .map(|row| cell(existing_rows, row, old_column.index).unwrap_or_default())
                .collect(),
            values: (0..data_row_count)
                .map(|row_offset| {
                    cell(
                        existing_rows,
                        DATA_START_ROW as usize + row_offset,
                        old_column.index,
                    )
                    .unwrap_or_default()
                })
                .collect(),
        });
    }

    let added_columns = columns
        .iter()
        .filter(|column| !old_by_name.contains_key(&column.name))
        .map(|column| column.name.clone())
        .collect::<Vec<_>>();
    let sheet = SyncedTableSheet {
        table,
        sheet_name: sheet_name.to_owned(),
        rows,
        legacy_columns,
    };
    let changed = match existing_rows {
        Some(existing_rows) => {
            !rows_equal_ignoring_trailing_blanks(existing_rows, &synced_sheet_rows(ir, &sheet))
        }
        None => true,
    };

    SyncedSheet {
        sheet,
        report: ExcelSyncSheetReport {
            sheet: sheet_name.to_owned(),
            created: existing_rows.is_none(),
            changed,
            rows: data_row_count,
            added_columns,
            legacy_columns: legacy_names,
        },
    }
}

fn synced_sheet_rows(ir: &ConfigIr, sheet: &SyncedTableSheet<'_>) -> Vec<Vec<String>> {
    let columns = table_template_columns(ir, sheet.table);
    let mut rows = table_template_rows(ir, sheet.table);
    let legacy_start = FIELD_START_COLUMN as usize + columns.len();
    for (legacy_index, legacy) in sheet.legacy_columns.iter().enumerate() {
        let column = legacy_start + legacy_index;
        for row_index in 0..DATA_START_ROW as usize {
            ensure_row_column(&mut rows, row_index, column);
            rows[row_index][column] = legacy.headers.get(row_index).cloned().unwrap_or_default();
        }
    }

    let legacy_row_count = sheet
        .legacy_columns
        .iter()
        .map(|column| column.values.len())
        .max()
        .unwrap_or_default();
    let row_count = sheet.rows.len().max(legacy_row_count);
    for row_offset in 0..row_count {
        let row_index = DATA_START_ROW as usize + row_offset;
        ensure_row_column(
            &mut rows,
            row_index,
            FIELD_START_COLUMN as usize + columns.len(),
        );
        if let Some(row) = sheet.rows.get(row_offset) {
            for (column_index, value) in row.iter().enumerate() {
                let column_info = &columns[column_index];
                rows[row_index][FIELD_START_COLUMN as usize + column_index] =
                    if column_info.derived && value.is_empty() {
                        derived_placeholder(column_info)
                    } else {
                        value.clone()
                    };
            }
        }
        for (legacy_index, legacy) in sheet.legacy_columns.iter().enumerate() {
            let column = legacy_start + legacy_index;
            ensure_row_column(&mut rows, row_index, column);
            rows[row_index][column] = legacy.values.get(row_offset).cloned().unwrap_or_default();
        }
    }
    rows
}

fn derived_placeholder(column: &crate::projection::TemplateColumn) -> String {
    if column.input.is_empty() {
        "generated".to_owned()
    } else {
        column.input.clone()
    }
}

fn ensure_row_column(rows: &mut Vec<Vec<String>>, row: usize, column: usize) {
    while rows.len() <= row {
        rows.push(Vec::new());
    }
    if rows[row].len() <= column {
        rows[row].resize(column + 1, String::new());
    }
}

fn rows_equal_ignoring_trailing_blanks(left: &[Vec<String>], right: &[Vec<String>]) -> bool {
    normalized_rows(left) == normalized_rows(right)
}

fn normalized_rows(rows: &[Vec<String>]) -> Vec<Vec<String>> {
    let mut rows = rows
        .iter()
        .map(|row| {
            let mut row = row.clone();
            while row.last().is_some_and(|value| value.is_empty()) {
                row.pop();
            }
            row
        })
        .collect::<Vec<_>>();
    while rows.last().is_some_and(Vec::is_empty) {
        rows.pop();
    }
    rows
}

#[derive(Debug)]
struct OldColumn {
    name: String,
    index: usize,
}

fn old_field_columns(row: &[String]) -> Vec<OldColumn> {
    row.iter()
        .enumerate()
        .skip(FIELD_START_COLUMN as usize)
        .filter_map(|(index, value)| {
            let name = value.trim();
            (!name.is_empty()).then(|| OldColumn {
                name: name.to_owned(),
                index,
            })
        })
        .collect()
}

fn cell(rows: Option<&[Vec<String>]>, row: usize, column: usize) -> Option<String> {
    rows.and_then(|rows| rows.get(row))
        .and_then(|row| row.get(column))
        .cloned()
}

struct ExistingWorkbook {
    exists: bool,
    sheet_order: Vec<String>,
    sheets: HashMap<String, Vec<Vec<String>>>,
}

impl ExistingWorkbook {
    fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self {
                exists: false,
                sheet_order: Vec::new(),
                sheets: HashMap::new(),
            });
        }

        let mut workbook = open_workbook_auto(path).map_err(|source| SoraError::ExcelTemplate {
            path: path.to_path_buf(),
            message: source.to_string(),
        })?;
        let sheet_order = workbook.sheet_names().to_vec();
        let mut sheets = HashMap::new();
        for sheet in &sheet_order {
            let range =
                workbook
                    .worksheet_range(sheet)
                    .map_err(|source| SoraError::ExcelTemplate {
                        path: path.to_path_buf(),
                        message: format!("failed to read worksheet `{sheet}`: {source}"),
                    })?;
            sheets.insert(sheet.clone(), range_to_rows(&range));
        }

        Ok(Self {
            exists: true,
            sheet_order,
            sheets,
        })
    }
}

fn range_to_rows(range: &calamine::Range<Data>) -> Vec<Vec<String>> {
    range
        .rows()
        .map(|row| row.iter().map(cell_to_string).collect())
        .collect()
}

fn cell_to_string(cell: &Data) -> String {
    match cell {
        Data::Empty => String::new(),
        Data::String(value) => value.clone(),
        Data::Float(value) => value.to_string(),
        Data::Int(value) => value.to_string(),
        Data::Bool(value) => value.to_string(),
        Data::DateTime(value) => value.to_string(),
        Data::DateTimeIso(value) => value.clone(),
        Data::DurationIso(value) => value.clone(),
        Data::Error(value) => value.to_string(),
    }
}

fn group_xlsx_tables(ir: &ConfigIr) -> Result<BTreeMap<String, Vec<&TableIr>>> {
    let mut workbooks = BTreeMap::<String, Vec<&TableIr>>::new();
    for table in &ir.tables {
        let format = resolve_table_source_format(table, Some("xlsx"))?;
        if format != SourceFormat::Xlsx {
            continue;
        }
        let file_name = table
            .source
            .as_ref()
            .map(|source| source.file.clone())
            .unwrap_or_else(|| format!("{}.xlsx", table.name));
        workbooks.entry(file_name).or_default().push(table);
    }
    Ok(workbooks)
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU64, Ordering};

    use calamine::Reader;
    use rust_xlsxwriter::{Format, Workbook};
    use sora_ir::normalize::normalize_schema;
    use sora_schema::model::ProjectSchema;

    use super::*;

    static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn sync_preserves_deleted_columns_as_legacy_columns() {
        let ir = example_ir();
        let base = temp_dir();
        fs::create_dir_all(&base).unwrap();
        write_existing_workbook(
            &base.join("Item.xlsx"),
            "Item",
            &["id", "name", "old_category"],
            &["1001", "Iron Sword", "deprecated"],
        );

        let report = ExcelTemplateSync.write(&ir, &base).unwrap();

        assert_eq!(report.workbooks.len(), 1);
        assert_eq!(report.workbooks[0].sheets[0].added_columns, ["rarity"]);
        assert_eq!(
            report.workbooks[0].sheets[0].legacy_columns,
            ["old_category"]
        );

        let mut workbook: calamine::Xlsx<_> =
            calamine::open_workbook(base.join("Item.xlsx")).unwrap();
        let range = workbook.worksheet_range("Item").unwrap();
        let field_row = range.rows().nth(FIELD_ROW as usize).unwrap();
        assert_eq!(cell_to_string(&field_row[1]), "id");
        assert_eq!(cell_to_string(&field_row[2]), "name");
        assert_eq!(cell_to_string(&field_row[3]), "rarity");
        assert_eq!(cell_to_string(&field_row[4]), "old_category");

        let data_row = range.rows().nth(DATA_START_ROW as usize).unwrap();
        assert_eq!(cell_to_string(&data_row[1]), "1001");
        assert_eq!(cell_to_string(&data_row[2]), "Iron Sword");
        assert_eq!(cell_to_string(&data_row[3]), "");
        assert_eq!(cell_to_string(&data_row[4]), "deprecated");

        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn sync_rejects_parent_directory_traversal() {
        let mut ir = example_ir();
        ir.tables[0].source.as_mut().expect("table source").file = "../outside.xlsx".to_owned();
        let base = temp_dir();
        fs::create_dir_all(&base).unwrap();

        let error = ExcelTemplateSync
            .preview(&ir, &base)
            .expect_err("traversal must be rejected");
        assert!(error.to_string().contains("unsafe traversal"));

        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn preview_accepts_a_missing_data_root_without_creating_it() {
        let ir = example_ir();
        let base = temp_dir();
        fs::create_dir_all(&base).unwrap();
        let data_root = base.join("data");

        let report = ExcelTemplateSync.preview(&ir, &data_root).unwrap();

        assert_eq!(report.workbooks.len(), 1);
        assert!(!data_root.exists());

        let _ = fs::remove_dir_all(base);
    }

    #[cfg(unix)]
    #[test]
    fn sync_rejects_workbook_symlink_outside_data_root() {
        let ir = example_ir();
        let base = temp_dir();
        let outside = temp_dir();
        fs::create_dir_all(&base).unwrap();
        fs::create_dir_all(&outside).unwrap();
        write_existing_workbook(
            &outside.join("Item.xlsx"),
            "Item",
            &["id", "name"],
            &["1001", "Iron Sword"],
        );
        std::os::unix::fs::symlink(outside.join("Item.xlsx"), base.join("Item.xlsx")).unwrap();

        let error = ExcelTemplateSync
            .preview(&ir, &base)
            .expect_err("external symlink must be rejected");
        assert!(error.to_string().contains("resolves outside"));

        let _ = fs::remove_dir_all(base);
        let _ = fs::remove_dir_all(outside);
    }

    #[test]
    fn sync_preserves_struct_columns_by_projected_field_name() {
        let ir = struct_columns_ir();
        let base = temp_dir();
        fs::create_dir_all(&base).unwrap();
        write_existing_workbook(
            &base.join("Reward.xlsx"),
            "Reward",
            &["id", "cost_kind", "cost_id", "cost_count"],
            &["1", "Item", "1001", "3"],
        );

        let report = ExcelTemplateSync.write(&ir, &base).unwrap();

        assert!(report.workbooks[0].sheets[0].added_columns.is_empty());
        assert_eq!(report.workbooks[0].sheets[0].legacy_columns, ["cost_count"]);

        let mut workbook: calamine::Xlsx<_> =
            calamine::open_workbook(base.join("Reward.xlsx")).unwrap();
        let range = workbook.worksheet_range("Reward").unwrap();
        let field_row = range.rows().nth(FIELD_ROW as usize).unwrap();
        assert_eq!(cell_to_string(&field_row[1]), "id");
        assert_eq!(cell_to_string(&field_row[2]), "cost_kind");
        assert_eq!(cell_to_string(&field_row[3]), "cost_id");
        assert_eq!(cell_to_string(&field_row[4]), "cost_count");

        let data_row = range.rows().nth(DATA_START_ROW as usize).unwrap();
        assert_eq!(cell_to_string(&data_row[1]), "1");
        assert_eq!(cell_to_string(&data_row[2]), "Item");
        assert_eq!(cell_to_string(&data_row[3]), "1001");
        assert_eq!(cell_to_string(&data_row[4]), "3");

        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn sync_preserves_tagged_union_columns_by_projected_field_name() {
        let ir = tagged_columns_ir();
        let base = temp_dir();
        fs::create_dir_all(&base).unwrap();
        write_existing_workbook(
            &base.join("EventConditionEntry.xlsx"),
            "EventConditionEntry",
            &["id", "type", "quest_id", "item_id", "count"],
            &["1", "HasItem", "", "1001", "5"],
        );

        let report = ExcelTemplateSync.write(&ir, &base).unwrap();

        assert!(report.workbooks[0].sheets[0].added_columns.is_empty());
        assert_eq!(report.workbooks[0].sheets[0].legacy_columns, ["count"]);

        let mut workbook: calamine::Xlsx<_> =
            calamine::open_workbook(base.join("EventConditionEntry.xlsx")).unwrap();
        let range = workbook.worksheet_range("EventConditionEntry").unwrap();
        let field_row = range.rows().nth(FIELD_ROW as usize).unwrap();
        assert_eq!(cell_to_string(&field_row[1]), "id");
        assert_eq!(cell_to_string(&field_row[2]), "type");
        assert_eq!(cell_to_string(&field_row[3]), "quest_id");
        assert_eq!(cell_to_string(&field_row[4]), "item_id");
        assert_eq!(cell_to_string(&field_row[5]), "count");

        let data_row = range.rows().nth(DATA_START_ROW as usize).unwrap();
        assert_eq!(cell_to_string(&data_row[1]), "1");
        assert_eq!(cell_to_string(&data_row[2]), "HasItem");
        assert_eq!(cell_to_string(&data_row[3]), "");
        assert_eq!(cell_to_string(&data_row[4]), "1001");
        assert_eq!(cell_to_string(&data_row[5]), "5");

        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn sync_skips_writing_unchanged_workbook() {
        let ir = example_ir();
        let base = temp_dir();
        fs::create_dir_all(&base).unwrap();

        let first_report = ExcelTemplateSync.write(&ir, &base).unwrap();
        assert!(first_report.workbooks[0].written);
        assert!(first_report.workbooks[0].sheets[0].changed);
        let path = base.join("Item.xlsx");
        let before = fs::read(&path).unwrap();

        let second_report = ExcelTemplateSync.write(&ir, &base).unwrap();

        assert!(!second_report.workbooks[0].written);
        assert!(second_report.workbooks[0].backup_path.is_none());
        assert!(!second_report.workbooks[0].sheets[0].changed);
        assert_eq!(fs::read(&path).unwrap(), before);

        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn syncs_each_selected_sheet_without_merging_their_rows() {
        let mut ir = example_ir();
        let source = ir.tables[0].source.as_mut().unwrap();
        source.sheet = None;
        source.sheets = vec!["2026-01".to_owned(), "2026-02".to_owned()];
        let base = temp_dir();
        fs::create_dir_all(&base).unwrap();
        write_existing_workbook_sheets(
            &base.join("Item.xlsx"),
            &[
                ("2026-01", ["1001", "January"]),
                ("2026-02", ["2001", "February"]),
            ],
        );

        let report = ExcelTemplateSync.write(&ir, &base).unwrap();

        assert_eq!(
            report.workbooks[0]
                .sheets
                .iter()
                .map(|sheet| sheet.sheet.as_str())
                .collect::<Vec<_>>(),
            ["2026-01", "2026-02"]
        );
        let mut workbook: calamine::Xlsx<_> =
            calamine::open_workbook(base.join("Item.xlsx")).unwrap();
        let january = workbook.worksheet_range("2026-01").unwrap();
        let february = workbook.worksheet_range("2026-02").unwrap();
        assert_eq!(
            cell_to_string(&january[(DATA_START_ROW as usize, 1)]),
            "1001"
        );
        assert_eq!(
            cell_to_string(&january[(DATA_START_ROW as usize, 2)]),
            "January"
        );
        assert_eq!(
            cell_to_string(&february[(DATA_START_ROW as usize, 1)]),
            "2001"
        );
        assert_eq!(
            cell_to_string(&february[(DATA_START_ROW as usize, 2)]),
            "February"
        );

        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn workbook_backups_are_unique_ignored_and_bounded() {
        let base = temp_dir();
        fs::create_dir_all(&base).unwrap();
        let workbook = base.join("Item.xlsx");
        fs::write(&workbook, b"workbook").unwrap();

        let mut paths = BTreeSet::new();
        for _ in 0..=MAX_BACKUP_BATCHES {
            paths.insert(backup_existing_workbook(&base, &workbook).unwrap());
        }

        assert_eq!(paths.len(), MAX_BACKUP_BATCHES + 1);
        let backup_root = base.join(".sora-backup");
        assert_eq!(
            fs::read_to_string(backup_root.join(".gitignore")).unwrap(),
            "*\n"
        );
        let backup_count = fs::read_dir(backup_root)
            .unwrap()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()))
            .count();
        assert_eq!(backup_count, MAX_BACKUP_BATCHES);

        let _ = fs::remove_dir_all(base);
    }

    fn write_existing_workbook(path: &Path, sheet: &str, fields: &[&str], values: &[&str]) {
        let mut workbook = Workbook::new();
        let worksheet = workbook.add_worksheet();
        worksheet.set_name(sheet).unwrap();
        worksheet.write_string(0, 0, "@table").unwrap();
        worksheet.write_string(1, 0, "#name").unwrap();
        worksheet.write_string(FIELD_ROW, 0, "#field").unwrap();
        worksheet.write_string(3, 0, "#type").unwrap();
        worksheet.write_string(4, 0, "#groups").unwrap();
        worksheet.write_string(5, 0, "#input").unwrap();
        worksheet.write_string(6, 0, "#desc").unwrap();
        for (index, field) in fields.iter().enumerate() {
            worksheet
                .write_string(FIELD_ROW, FIELD_START_COLUMN + index as u16, *field)
                .unwrap();
        }
        for (index, value) in values.iter().enumerate() {
            worksheet
                .write_string_with_format(
                    DATA_START_ROW,
                    FIELD_START_COLUMN + index as u16,
                    *value,
                    &Format::new(),
                )
                .unwrap();
        }
        workbook.save(path).unwrap();
    }

    fn write_existing_workbook_sheets(path: &Path, sheets: &[(&str, [&str; 2])]) {
        let mut workbook = Workbook::new();
        for (sheet, values) in sheets {
            let worksheet = workbook.add_worksheet();
            worksheet.set_name(*sheet).unwrap();
            worksheet.write_string(0, 0, "@table").unwrap();
            worksheet.write_string(1, 0, "#name").unwrap();
            worksheet.write_string(FIELD_ROW, 0, "#field").unwrap();
            worksheet.write_string(3, 0, "#type").unwrap();
            worksheet.write_string(4, 0, "#groups").unwrap();
            worksheet.write_string(5, 0, "#input").unwrap();
            worksheet.write_string(6, 0, "#desc").unwrap();
            worksheet.write_string(FIELD_ROW, 1, "id").unwrap();
            worksheet.write_string(FIELD_ROW, 2, "name").unwrap();
            worksheet
                .write_string(DATA_START_ROW, 1, values[0])
                .unwrap();
            worksheet
                .write_string(DATA_START_ROW, 2, values[1])
                .unwrap();
        }
        workbook.save(path).unwrap();
    }

    fn example_ir() -> ConfigIr {
        let schema: ProjectSchema = toml::from_str(
            r#"
project = { id = "game_config" }
groups = { common = { default = true } }
views = { default = { contract = "game_config/default", groups = ["common"] } }

[[tables]]
id = "item"
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
name = "rarity"
type = "string"
"#,
        )
        .unwrap();
        normalize_schema(schema).unwrap()
    }

    fn struct_columns_ir() -> ConfigIr {
        let schema: ProjectSchema = toml::from_str(
            r#"
project = { id = "game_config" }
groups = { common = { default = true } }
views = { default = { contract = "game_config/default", groups = ["common"] } }

[[enums]]
name = "ResourceType"
values = [{ id = 0, name = "Item" }]

[[structs]]
name = "ResourceCost"

[[structs.fields]]
name = "kind"
type = "enum<ResourceType>"

[[structs.fields]]
name = "id"
type = "i32"

[[tables]]
id = "reward"
name = "Reward"
mode = "map"
key = "id"

[tables.source]
format = "xlsx"
file = "Reward.xlsx"
sheet = "Reward"

[[tables.fields]]
name = "id"
type = "i32"

[[tables.fields]]
name = "cost"
type = "struct<ResourceCost>"
parser = { kind = "columns", prefix = "cost_" }
"#,
        )
        .unwrap();
        normalize_schema(schema).unwrap()
    }

    fn tagged_columns_ir() -> ConfigIr {
        let schema: ProjectSchema = toml::from_str(
            r#"
project = { id = "game_config" }
groups = { common = { default = true } }
views = { default = { contract = "game_config/default", groups = ["common"] } }

[[unions]]
name = "EventCondition"

[[unions.variants]]
name = "HasQuest"

[[unions.variants.fields]]
name = "quest_id"
type = "i32"

[[unions.variants]]
name = "HasItem"

[[unions.variants.fields]]
name = "item_id"
type = "i32"

[[tables]]
id = "event_condition_entry"
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
        normalize_schema(schema).unwrap()
    }

    fn temp_dir() -> PathBuf {
        let id = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("sora-excel-sync-test-{}-{id}", std::process::id()))
    }
}
