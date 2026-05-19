use std::{collections::BTreeMap, fs, path::Path};

use sora_diagnostics::{Result, SoraError};
use sora_ir::model::{ConfigIr, TableIr};

use crate::writer::write_workbook;

pub struct ExcelTemplateGenerator;

impl ExcelTemplateGenerator {
    pub fn generate(&self, ir: &ConfigIr, out_dir: &Path) -> Result<()> {
        fs::create_dir_all(out_dir).map_err(|source| SoraError::CreateDir {
            path: out_dir.to_path_buf(),
            source,
        })?;

        let mut workbooks = BTreeMap::<String, Vec<&TableIr>>::new();
        for table in &ir.tables {
            let file_name = table
                .source
                .as_ref()
                .filter(|source| source.format == "xlsx")
                .map(|source| source.file.clone())
                .unwrap_or_else(|| format!("{}.xlsx", table.name));
            workbooks.entry(file_name).or_default().push(table);
        }

        for (file_name, tables) in workbooks {
            let path = out_dir.join(file_name);
            write_workbook(&tables, &path)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sora_ir::normalize::normalize_schema;
    use sora_schema::model::SchemaFile;
    use std::time::{SystemTime, UNIX_EPOCH};

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
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("sora-excel-test-{unique}"))
    }
}
