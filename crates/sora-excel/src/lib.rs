use std::{fs, path::Path};

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
            let path = out_dir.join(format!("{}.csv", table.name));
            let content = render_table_template(table);
            fs::write(&path, content).map_err(|source| SoraError::WriteFile { path, source })?;
        }

        Ok(())
    }
}

pub fn render_table_template(table: &TableIr) -> String {
    let mut lines = vec![
        csv_row(["@table", table.name.as_str()]),
        csv_row(["@mode", table_mode_name(table.mode)]),
        csv_row(["@key", table.key.as_deref().unwrap_or("")]),
        csv_row(["@schema", &schema_hash(table)]),
        String::new(),
    ];

    lines.push(csv_row(
        std::iter::once("#name".to_owned()).chain(
            table
                .fields
                .iter()
                .map(|field| field_display_name(field).to_owned()),
        ),
    ));
    lines.push(csv_row(
        std::iter::once("#field".to_owned())
            .chain(table.fields.iter().map(|field| field.name.clone())),
    ));
    lines.push(csv_row(
        std::iter::once("#type".to_owned())
            .chain(table.fields.iter().map(|field| field.ty.to_string())),
    ));
    lines.push(csv_row(
        std::iter::once("#rule".to_owned()).chain(
            table
                .fields
                .iter()
                .map(|field| field_rule(field).to_owned()),
        ),
    ));
    lines.push(csv_row(
        std::iter::once("#desc".to_owned()).chain(
            table
                .fields
                .iter()
                .map(|field| field_display_name(field).to_owned()),
        ),
    ));
    lines.push(String::new());

    lines.join("\n")
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

fn csv_row(cells: impl IntoIterator<Item = impl AsRef<str>>) -> String {
    cells
        .into_iter()
        .map(|cell| csv_cell(cell.as_ref()))
        .collect::<Vec<_>>()
        .join(",")
}

fn csv_cell(value: &str) -> String {
    if value.contains([',', '"', '\n']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sora_ir::{ConfigIr, normalize_schema};
    use sora_schema::SchemaFile;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn renders_schema_projection_headers() {
        let ir = example_ir();
        let content = render_table_template(&ir.tables[0]);

        assert!(content.contains("@table,Item"));
        assert!(content.contains("@mode,map"));
        assert!(content.contains("@key,id"));
        assert!(content.contains("#field,id,name,item_type,max_stack"));
        assert!(content.contains("#type,i32,string,enum<ItemType>,i32"));
        assert!(content.contains("#rule,key,required,required,required"));
        assert!(content.contains("#desc,Item id,Display name,Item type,Max stack count"));
    }

    #[test]
    fn schema_hash_is_deterministic() {
        let ir = example_ir();

        assert_eq!(schema_hash(&ir.tables[0]), schema_hash(&ir.tables[0]));
    }

    #[test]
    fn writes_csv_template_file() {
        let ir = example_ir();
        let out_dir = temp_dir();

        ExcelTemplateGenerator.generate(&ir, &out_dir).unwrap();

        let content = fs::read_to_string(out_dir.join("Item.csv")).unwrap();
        assert!(content.contains("@schema,"));
        assert!(content.contains("#name,Item id,Display name,Item type,Max stack count"));

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
