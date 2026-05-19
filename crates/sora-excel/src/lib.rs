use std::{fs, path::Path};

use rust_xlsxwriter::{Format, Workbook};
use sora_diagnostics::{Result, SoraError};
use sora_ir::{ConfigIr, FieldIr, TableIr, TableModeIr};

pub struct ExcelTemplateGenerator;

impl ExcelTemplateGenerator {
    pub fn generate(&self, ir: &ConfigIr, out_dir: &Path) -> Result<()> {
        fs::create_dir_all(out_dir).map_err(|source| SoraError::CreateDir {
            path: out_dir.to_path_buf(),
            source,
        })?;

        for table in &ir.tables {
            let path = out_dir.join(format!("{}.xlsx", table.name));
            write_table_workbook(table, &path)?;
        }

        Ok(())
    }
}

pub fn table_template_rows(table: &TableIr) -> Vec<Vec<String>> {
    vec![
        vec!["@table".to_owned(), table.name.clone()],
        vec!["@mode".to_owned(), table_mode_name(table.mode).to_owned()],
        vec![
            "@key".to_owned(),
            table.key.as_deref().unwrap_or("").to_owned(),
        ],
        vec!["@schema".to_owned(), schema_hash(table)],
        Vec::new(),
        std::iter::once("#name".to_owned())
            .chain(
                table
                    .fields
                    .iter()
                    .map(|field| field_display_name(field).to_owned()),
            )
            .collect(),
        std::iter::once("#field".to_owned())
            .chain(table.fields.iter().map(|field| field.name.clone()))
            .collect(),
        std::iter::once("#type".to_owned())
            .chain(table.fields.iter().map(|field| field.ty.to_string()))
            .collect(),
        std::iter::once("#rule".to_owned())
            .chain(
                table
                    .fields
                    .iter()
                    .map(|field| field_rule(field).to_owned()),
            )
            .collect(),
        std::iter::once("#desc".to_owned())
            .chain(
                table
                    .fields
                    .iter()
                    .map(|field| field_display_name(field).to_owned()),
            )
            .collect(),
    ]
}

pub fn schema_hash(table: &TableIr) -> String {
    let mut hash = 0xcbf29ce484222325_u64;
    fn update(hash: &mut u64, value: &str) {
        for byte in value.as_bytes() {
            *hash ^= u64::from(*byte);
            *hash = hash.wrapping_mul(0x100000001b3);
        }
        *hash ^= 0xff;
        *hash = hash.wrapping_mul(0x100000001b3);
    }

    update(&mut hash, &table.name);
    update(&mut hash, table_mode_name(table.mode));
    update(&mut hash, table.key.as_deref().unwrap_or(""));
    for field in &table.fields {
        update(&mut hash, &field.name);
        update(&mut hash, &field.ty.to_string());
        update(
            &mut hash,
            if field.required {
                "required"
            } else {
                "optional"
            },
        );
        update(&mut hash, if field.key { "key" } else { "" });
    }

    format!("{hash:016x}")
}

fn table_mode_name(mode: TableModeIr) -> &'static str {
    match mode {
        TableModeIr::List => "list",
        TableModeIr::Map => "map",
        TableModeIr::Singleton => "singleton",
    }
}

fn field_display_name(field: &FieldIr) -> &str {
    field.comment.as_deref().unwrap_or(&field.name)
}

fn field_rule(field: &FieldIr) -> &'static str {
    if field.key {
        "key"
    } else if field.required {
        "required"
    } else {
        "optional"
    }
}

fn write_table_workbook(table: &TableIr, path: &Path) -> Result<()> {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    worksheet
        .set_name(&table.name)
        .map_err(|source| excel_error(path, source))?;

    let metadata_format = Format::new().set_bold();
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

#[cfg(test)]
mod tests {
    use super::*;
    use sora_ir::{ConfigIr, normalize_schema};
    use sora_schema::SchemaFile;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn builds_schema_projection_rows() {
        let ir = example_ir();
        let rows = table_template_rows(&ir.tables[0]);

        assert_eq!(rows[0], ["@table", "Item"]);
        assert_eq!(rows[1], ["@mode", "map"]);
        assert_eq!(rows[2], ["@key", "id"]);
        assert_eq!(rows[6], ["#field", "id", "name", "item_type", "max_stack"]);
        assert_eq!(rows[7], ["#type", "i32", "string", "enum<ItemType>", "i32"]);
        assert_eq!(
            rows[8],
            ["#rule", "key", "required", "required", "required"]
        );
        assert_eq!(
            rows[9],
            [
                "#desc",
                "Item id",
                "Display name",
                "Item type",
                "Max stack count"
            ]
        );
    }

    #[test]
    fn schema_hash_is_deterministic() {
        let ir = example_ir();

        assert_eq!(schema_hash(&ir.tables[0]), schema_hash(&ir.tables[0]));
    }

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
source = "items.toml"

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
