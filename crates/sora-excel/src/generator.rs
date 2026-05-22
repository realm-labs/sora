use std::{collections::BTreeMap, fs, path::Path};

use sora_diagnostics::{Result, SoraError};
use sora_input::source::{SourceFormat, resolve_table_source_format};
use sora_ir::model::{ConfigIr, TableIr};

use crate::writer::write_workbook_with_rows;

pub struct ExcelTemplateGenerator;

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

        for (file_name, tables) in workbooks {
            let path = out_dir.join(file_name);
            write_workbook_with_rows(ir, &tables, &path, &rows_for_table)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use calamine::Reader;
    use sora_ir::normalize::normalize_schema;
    use sora_schema::model::SchemaFile;
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

        assert_eq!(range.get((6, 0)).unwrap().to_string(), "#name");
        assert_eq!(range.get((12, 1)).unwrap().to_string(), "1001");
        assert_eq!(range.get((12, 2)).unwrap().to_string(), "Iron Sword");

        let _ = fs::remove_dir_all(out_dir);
    }

    fn example_ir() -> ConfigIr {
        let schema: SchemaFile = toml::from_str(
            r#"
package = "game_config"

[[enums]]
name = "ItemType"
values = ["Weapon", "Armor", "Material", "Consumable"]

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
key = true
required = true
comment = "Item id"

[[tables.fields]]
name = "name"
type = "string"
required = true
comment = "Display name"

[[tables.fields]]
name = "item_type"
type = "enum<ItemType>"
required = true
comment = "Item type"

[[tables.fields]]
name = "max_stack"
type = "i32"
required = true
comment = "Max stack count"
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
