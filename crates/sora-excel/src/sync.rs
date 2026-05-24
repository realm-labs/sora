use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use calamine::{Data, Reader, open_workbook_auto};
use sora_diagnostics::{Result, SoraError};
use sora_input::source::{SourceFormat, resolve_table_source_format};
use sora_ir::model::{ConfigIr, TableIr};

use crate::{
    projection::{DATA_START_ROW, FIELD_ROW, FIELD_START_COLUMN, table_template_columns},
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
            let existing = ExistingWorkbook::load(&path)?;
            let mut table_sheets = Vec::new();
            let mut sheet_reports = Vec::new();
            let mut handled_sheets = BTreeSet::new();

            for table in tables {
                let sheet_name = table_sheet_name(table);
                handled_sheets.insert(sheet_name.clone());
                let existing_sheet = existing.sheets.get(&sheet_name).map(Vec::as_slice);
                let synced = sync_table_sheet(ir, table, &sheet_name, existing_sheet);
                sheet_reports.push(synced.report);
                table_sheets.push(synced.sheet);
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

            let backup_path = if write && existing.exists {
                Some(backup_existing_workbook(data_root, &path)?)
            } else {
                None
            };

            if write {
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent).map_err(|source| SoraError::CreateDir {
                        path: parent.to_path_buf(),
                        source,
                    })?;
                }
                write_synced_workbook(ir, &table_sheets, &preserved_sheets, &path)?;
            }

            report.workbooks.push(ExcelSyncWorkbookReport {
                path,
                created: !existing.exists,
                written: write,
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

fn backup_existing_workbook(data_root: &Path, path: &Path) -> Result<PathBuf> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default();
    let relative = path.strip_prefix(data_root).unwrap_or(path);
    let backup_path = data_root
        .join(".sora-backup")
        .join(timestamp.to_string())
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
    Ok(backup_path)
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

    SyncedSheet {
        sheet: SyncedTableSheet {
            table,
            sheet_name: sheet_name.to_owned(),
            rows,
            legacy_columns,
        },
        report: ExcelSyncSheetReport {
            sheet: sheet_name.to_owned(),
            created: existing_rows.is_none(),
            rows: data_row_count,
            added_columns,
            legacy_columns: legacy_names,
        },
    }
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

fn table_sheet_name(table: &TableIr) -> String {
    table
        .source
        .as_ref()
        .and_then(|source| source.sheet.clone())
        .unwrap_or_else(|| table.name.clone())
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU64, Ordering};

    use calamine::Reader;
    use rust_xlsxwriter::{Format, Workbook};
    use sora_ir::normalize::normalize_schema;
    use sora_schema::model::SchemaFile;

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

    fn write_existing_workbook(path: &Path, sheet: &str, fields: &[&str], values: &[&str]) {
        let mut workbook = Workbook::new();
        let worksheet = workbook.add_worksheet();
        worksheet.set_name(sheet).unwrap();
        worksheet.write_string(0, 0, "@table").unwrap();
        worksheet.write_string(1, 0, "#name").unwrap();
        worksheet.write_string(FIELD_ROW, 0, "#field").unwrap();
        worksheet.write_string(3, 0, "#type").unwrap();
        worksheet.write_string(4, 0, "#scope").unwrap();
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

    fn example_ir() -> ConfigIr {
        let schema: SchemaFile = toml::from_str(
            r#"
package = "game_config"

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
name = "rarity"
type = "string"
"#,
        )
        .unwrap();
        normalize_schema(schema).unwrap()
    }

    fn struct_columns_ir() -> ConfigIr {
        let schema: SchemaFile = toml::from_str(
            r#"
package = "game_config"

[[enums]]
name = "ResourceType"
values = ["Item"]

[[structs]]
name = "ResourceCost"

[[structs.fields]]
name = "kind"
type = "enum<ResourceType>"

[[structs.fields]]
name = "id"
type = "i32"

[[tables]]
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
        let schema: SchemaFile = toml::from_str(
            r#"
package = "game_config"

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
