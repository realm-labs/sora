use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use rust_xlsxwriter::Workbook;
use sora_codegen::target::CodegenTarget;
use sora_excel::projection::table_template_rows;
use sora_export::exporter::ExportOutput;
use sora_input_toml::{input::TomlSchemaInput, schema::load_project_schema_file};
use sora_input_xlsx::input::XlsxProjectInput;
use sora_ir::{model::ConfigIr, normalize::normalize_schema, validate::validate_config_ir};

fn main() -> Result<()> {
    let root = showcase_root();
    let project = root.join("project.toml");
    let data_root = root.join("data");
    let generated_root = root.join("generated");

    fs::create_dir_all(&data_root)
        .with_context(|| format!("failed to create `{}`", data_root.display()))?;
    fs::create_dir_all(&generated_root)
        .with_context(|| format!("failed to create `{}`", generated_root.display()))?;

    let ir = load_ir(&project)?;
    write_workbook(&ir, &data_root.join("GameConfig.xlsx"))?;

    let schema_input = TomlSchemaInput::new(&project);
    let project_input = XlsxProjectInput::new(TomlSchemaInput::new(&project), &data_root);

    clean_dir(&generated_root.join("rust"))?;
    clean_dir(&generated_root.join("kotlin"))?;
    clean_dir(&generated_root.join("debug-json"))?;

    sora_core::pipeline::check_schema(&schema_input)?;
    sora_core::pipeline::generate_schema_lock(&schema_input, &generated_root.join("schema.lock"))?;
    sora_core::pipeline::generate_code(
        &schema_input,
        CodegenTarget::Rust,
        &generated_root.join("rust"),
    )?;
    sora_core::pipeline::generate_code(
        &schema_input,
        CodegenTarget::Kotlin,
        &generated_root.join("kotlin"),
    )?;
    sora_core::pipeline::export_data(
        &project_input,
        "binary",
        ExportOutput::File(generated_root.join("config.sora")),
    )?;
    sora_core::pipeline::export_data(
        &project_input,
        "json-debug",
        ExportOutput::Directory(generated_root.join("debug-json")),
    )?;

    println!("showcase generated under `{}`", root.display());
    Ok(())
}

fn load_ir(project: &Path) -> Result<ConfigIr> {
    let schema = load_project_schema_file(project)
        .with_context(|| format!("failed to load `{}`", project.display()))?;
    let ir = normalize_schema(schema)?;
    validate_config_ir(&ir)?;
    Ok(ir)
}

fn write_workbook(ir: &ConfigIr, path: &Path) -> Result<()> {
    let mut workbook = Workbook::new();

    for table in &ir.tables {
        let worksheet = workbook.add_worksheet();
        let sheet = table
            .source
            .as_ref()
            .and_then(|source| source.sheet.as_deref())
            .unwrap_or(&table.name);
        worksheet.set_name(sheet)?;

        for (row_index, row) in table_template_rows(ir, table).iter().enumerate() {
            for (column_index, value) in row.iter().enumerate() {
                worksheet.write_string(row_index as u32, column_index as u16, value)?;
            }
        }

        for (row_offset, row) in showcase_rows(&table.name).iter().enumerate() {
            for (column, value) in row.iter().enumerate() {
                worksheet.write_string((10 + row_offset) as u32, column as u16, *value)?;
            }
        }
    }

    workbook.save(path)?;
    Ok(())
}

fn showcase_rows(table: &str) -> Vec<Vec<&'static str>> {
    match table {
        "Item" => vec![
            vec![
                "1001",
                "Iron Sword",
                "Weapon",
                "1",
                "Gold,0,120",
                "[\"starter\",\"melee\"]",
            ],
            vec![
                "1002",
                "Magic Crystal",
                "Material",
                "999",
                "Diamond,0,3",
                "[\"craft\",\"rare\"]",
            ],
            vec![
                "2001",
                "Health Potion",
                "Consumable",
                "50",
                "Gold,0,25",
                "[\"potion\",\"recover\"]",
            ],
            vec![
                "3001",
                "Training Medal",
                "Currency",
                "",
                "Gold,0,1",
                "[\"quest\",\"token\"]",
            ],
        ],
        "Skill" => vec![
            vec![
                "101",
                "Flame Slash",
                "Fire",
                "Gold,0,150",
                "{\"element\":\"Fire\",\"power\":120,\"radius\":2.5}",
                "3",
                "1001",
                "0,1.2,0",
            ],
            vec![
                "102",
                "Ice Lance",
                "Ice",
                "Item,1002,2",
                "{\"element\":\"Ice\",\"power\":95}",
                "",
                "",
                "0,1.5,3",
            ],
        ],
        "Quest" => vec![
            vec![
                "5001",
                "Main",
                "First Trial",
                "1001",
                "[101,102]",
                "12,0,5",
                "",
            ],
            vec![
                "5002",
                "Daily",
                "Crystal Supply",
                "1002",
                "[102]",
                "2,0,8",
                "",
            ],
        ],
        "QuestReward" => vec![
            vec!["5001", "1", "3001", "5"],
            vec!["5001", "2", "2001", "3"],
            vec!["5002", "1", "1002", "1"],
        ],
        "GameSettings" => vec![vec!["2026.05", "5", "", "0,0,0", "[1001,2001]"]],
        _ => Vec::new(),
    }
}

fn clean_dir(path: &Path) -> Result<()> {
    if path.exists() {
        fs::remove_dir_all(path)
            .with_context(|| format!("failed to remove `{}`", path.display()))?;
    }
    Ok(())
}

fn showcase_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("builder crate should live under examples/showcase/tools")
        .to_path_buf()
}
