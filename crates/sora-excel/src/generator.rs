use std::{collections::BTreeMap, fs, path::Path};

use sora_diagnostics::{Result, SoraError};
use sora_input::source::{SourceFormat, resolve_table_source_format};
use sora_ir::model::{ConfigIr, TableIr};

use crate::{
    path::create_workbook_parent, sheets::ensure_single_table_definition,
    writer::write_workbook_with_rows,
};

pub struct ExcelTemplateGenerator;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExcelAdditionalSheet {
    pub file: String,
    pub sheet: String,
    pub rows: Vec<Vec<String>>,
}

impl ExcelTemplateGenerator {
    pub fn generate(&self, ir: &ConfigIr, out_dir: &Path) -> Result<()> {
        self.generate_with_rows(ir, out_dir, |_| Vec::new())
    }

    pub fn generate_with_rows(
        &self,
        ir: &ConfigIr,
        out_dir: &Path,
        rows_for_table: impl Fn(&TableIr) -> Vec<Vec<String>>,
    ) -> Result<()> {
        self.generate_with_rows_and_sheets(ir, out_dir, rows_for_table, &[])
    }

    pub fn generate_with_rows_and_sheets(
        &self,
        ir: &ConfigIr,
        out_dir: &Path,
        rows_for_table: impl Fn(&TableIr) -> Vec<Vec<String>>,
        additional_sheets: &[ExcelAdditionalSheet],
    ) -> Result<()> {
        fs::create_dir_all(out_dir).map_err(|source| SoraError::CreateDir {
            path: out_dir.to_path_buf(),
            source,
        })?;

        let mut workbooks = BTreeMap::<String, Vec<&TableIr>>::new();
        for table in &ir.tables {
            let file_name = table
                .source
                .as_ref()
                .filter(|_| {
                    matches!(
                        resolve_table_source_format(table, Some("xlsx")),
                        Ok(SourceFormat::Xlsx)
                    )
                })
                .map(|source| source.file.clone())
                .unwrap_or_else(|| format!("{}.xlsx", table.name));
            workbooks.entry(file_name).or_default().push(table);
        }
        for sheet in additional_sheets {
            workbooks.entry(sheet.file.clone()).or_default();
        }

        for (file_name, tables) in workbooks {
            ensure_single_table_definition(&tables, &file_name)?;
            let path = out_dir.join(file_name);
            create_workbook_parent(out_dir, &path)?;
            let sheets = additional_sheets
                .iter()
                .filter(|sheet| path.ends_with(&sheet.file))
                .collect::<Vec<_>>();
            write_workbook_with_rows(ir, &tables, &sheets, &path, &rows_for_table)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::projection::{DATA_START_ROW, METADATA_ROW, NAME_ROW};
    use calamine::Reader;
    use sora_ir::normalize::normalize_schema;
    use sora_schema::model::ProjectSchema;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn writes_xlsx_template_file() {
        let ir = example_ir();
        let out_dir = temp_dir();

        ExcelTemplateGenerator.generate(&ir, &out_dir).unwrap();

        let bytes = fs::read(out_dir.join("Item.xlsx")).unwrap();
        assert_eq!(&bytes[0..2], b"PK");
        assert!(bytes.len() > 1024);

        let _ = fs::remove_dir_all(out_dir);
    }

    #[test]
    fn writes_xlsx_template_file_in_nested_source_directory() {
        let mut ir = example_ir();
        ir.tables[0].source.as_mut().expect("table source").file =
            "core/InventoryProfile.xlsx".to_owned();
        let out_dir = temp_dir();

        ExcelTemplateGenerator.generate(&ir, &out_dir).unwrap();

        let bytes = fs::read(out_dir.join("core/InventoryProfile.xlsx")).unwrap();
        assert_eq!(&bytes[0..2], b"PK");

        let _ = fs::remove_dir_all(out_dir);
    }

    #[test]
    fn rejects_template_source_parent_directory_traversal() {
        let mut ir = example_ir();
        ir.tables[0].source.as_mut().expect("table source").file = "../escaped.xlsx".to_owned();
        let base = temp_dir();
        let out_dir = base.join("generated");

        let error = ExcelTemplateGenerator
            .generate(&ir, &out_dir)
            .expect_err("traversal must be rejected");

        assert!(error.to_string().contains("unsafe traversal"));
        assert!(!base.join("escaped.xlsx").exists());

        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn rejects_two_table_definitions_in_one_workbook() {
        let mut ir = example_ir();
        let mut second = ir.tables[0].clone();
        second.id = "second".to_owned();
        second.name = "Second".to_owned();
        second.canonical_name = "Second".to_owned();
        ir.tables.push(second);
        let out_dir = temp_dir();

        let error = ExcelTemplateGenerator.generate(&ir, &out_dir).unwrap_err();

        assert!(error.to_string().contains("exactly one table definition"));
        assert!(!out_dir.join("Item.xlsx").exists());
        let _ = fs::remove_dir_all(out_dir);
    }

    #[test]
    fn writes_xlsx_template_file_with_rows() {
        let ir = example_ir();
        let out_dir = temp_dir();

        ExcelTemplateGenerator
            .generate_with_rows(&ir, &out_dir, |_| {
                vec![vec![
                    "1001".to_owned(),
                    "Iron Sword".to_owned(),
                    "Weapon".to_owned(),
                    "1".to_owned(),
                ]]
            })
            .unwrap();

        let mut workbook: calamine::Xlsx<_> =
            calamine::open_workbook(out_dir.join("Item.xlsx")).unwrap();
        let range = workbook.worksheet_range("Item").unwrap();

        assert_eq!(
            range.get((NAME_ROW as usize, 0)).unwrap().to_string(),
            "#name"
        );
        assert_eq!(
            range.get((DATA_START_ROW as usize, 1)).unwrap().to_string(),
            "1001"
        );
        assert_eq!(
            range.get((DATA_START_ROW as usize, 2)).unwrap().to_string(),
            "Iron Sword"
        );

        let _ = fs::remove_dir_all(out_dir);
    }

    #[test]
    fn writes_namespaced_table_with_a_short_sheet_and_canonical_metadata() {
        let ir = namespaced_ir();
        let out_dir = temp_dir();

        ExcelTemplateGenerator.generate(&ir, &out_dir).unwrap();

        let mut workbook: calamine::Xlsx<_> =
            calamine::open_workbook(out_dir.join("VisualProfile.xlsx")).unwrap();
        assert_eq!(workbook.sheet_names(), ["VisualProfile"]);
        let range = workbook.worksheet_range("VisualProfile").unwrap();
        assert_eq!(
            range[(METADATA_ROW as usize, 1)].to_string(),
            "presentation.SurfaceResourceVisualProfile"
        );

        let _ = fs::remove_dir_all(out_dir);
    }

    #[test]
    fn writes_one_template_sheet_for_each_explicit_selector() {
        let ir = multi_sheet_ir(&["2026-02", "2026-01"]);
        let out_dir = temp_dir();

        ExcelTemplateGenerator.generate(&ir, &out_dir).unwrap();

        let workbook: calamine::Xlsx<_> =
            calamine::open_workbook(out_dir.join("Activity.xlsx")).unwrap();
        assert_eq!(workbook.sheet_names(), ["2026-02", "2026-01"]);

        let _ = fs::remove_dir_all(out_dir);
    }

    #[test]
    fn rejects_wildcard_selector_when_creating_a_new_template() {
        let ir = multi_sheet_ir(&["2026-*"]);
        let out_dir = temp_dir();

        let error = ExcelTemplateGenerator
            .generate(&ir, &out_dir)
            .expect_err("a new workbook has no sheet names for a wildcard to match");

        assert!(error.to_string().contains("matched no worksheets"));
        let _ = fs::remove_dir_all(out_dir);
    }

    #[test]
    fn rejects_rows_without_multi_sheet_ownership() {
        let ir = multi_sheet_ir(&["2026-01", "2026-02"]);
        let out_dir = temp_dir();

        let error = ExcelTemplateGenerator
            .generate_with_rows(&ir, &out_dir, |_| vec![vec!["1".to_owned()]])
            .expect_err("rows cannot be duplicated across sheets");

        assert!(error.to_string().contains("row-to-sheet ownership"));
        let _ = fs::remove_dir_all(out_dir);
    }

    fn example_ir() -> ConfigIr {
        let schema: ProjectSchema = toml::from_str(
            r#"
project = { id = "game_config" }
groups = { common = { default = true } }
views = { default = { contract = "game_config/default", groups = ["common"] } }

[[enums]]
name = "ItemType"
values = [{ id = 0, name = "Weapon" }, { id = 1, name = "Armor" }, { id = 2, name = "Material" }, { id = 3, name = "Consumable" }]

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
comment = "Item id"

[[tables.fields]]
name = "name"
type = "string"
comment = "Display name"

[[tables.fields]]
name = "item_type"
type = "enum<ItemType>"
comment = "Item type"

[[tables.fields]]
name = "max_stack"
type = "i32"
comment = "Max stack count"
"#,
        )
        .unwrap();
        normalize_schema(schema).unwrap()
    }

    fn multi_sheet_ir(sheets: &[&str]) -> ConfigIr {
        let sheet_values = sheets
            .iter()
            .map(|sheet| format!("\"{sheet}\""))
            .collect::<Vec<_>>()
            .join(", ");
        let schema: ProjectSchema = toml::from_str(&format!(
            r#"
project = {{ id = "game_config" }}
groups = {{ common = {{ default = true }} }}
views = {{ default = {{ contract = "game_config/default", groups = ["common"] }} }}

[[tables]]
id = "activity"
name = "Activity"
mode = "map"
key = "id"
source = {{ format = "xlsx", file = "Activity.xlsx", sheets = [{sheet_values}] }}

[[tables.fields]]
name = "id"
type = "i32"
"#
        ))
        .unwrap();
        normalize_schema(schema).unwrap()
    }

    fn namespaced_ir() -> ConfigIr {
        let schema: ProjectSchema = toml::from_str(
            r#"
project = { id = "game_config" }
groups = { common = { default = true } }
views = { default = { contract = "game_config/default", groups = ["common"] } }

[[enums]]
name = "worldgen.geography.TraversalBarrierKind"
values = [{ id = 0, name = "None" }, { id = 1, name = "Blocked" }]

[[tables]]
id = "visual_profile"
name = "presentation.SurfaceResourceVisualProfile"
mode = "map"
key = "id"
source = { format = "xlsx", file = "VisualProfile.xlsx" }

[[tables.fields]]
name = "id"
type = "i32"

[[tables.fields]]
name = "barrier"
type = "enum<worldgen.geography.TraversalBarrierKind>"
"#,
        )
        .unwrap();
        normalize_schema(schema).unwrap()
    }

    fn temp_dir() -> std::path::PathBuf {
        let unique = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("sora-excel-test-{}-{unique}", std::process::id()))
    }
}
