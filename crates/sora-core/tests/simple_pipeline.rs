use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use rust_xlsxwriter::Workbook;
use sora_excel::projection::{DATA_START_ROW, FIELD_START_COLUMN, table_template_rows};
use sora_export::exporter::ExportOutput;
use sora_input::project::SplitProjectInput;
use sora_input_schema::{input::ProjectSchemaInput, schema::load_project_schema};
use sora_input_toml::input::TomlDataInput;
use sora_input_xlsx::input::XlsxProjectInput;
use sora_ir::{normalize::normalize_schema, validate::validate_config_ir};

#[test]
fn simple_example_pipeline_generates_all_artifacts() {
    let root = workspace_root();
    let project = root.join("examples/simple/project.scon");
    let schema_input = ProjectSchemaInput::new(&project);
    let out_dir = temp_dir();
    let data_root = out_dir.join("excel-data");
    write_item_workbook(&project, &data_root);
    let project_input = XlsxProjectInput::new(ProjectSchemaInput::new(&project), &data_root);

    sora_core::pipeline::check_schema(&schema_input).unwrap();

    sora_core::pipeline::generate_code(&schema_input, "rust", &out_dir.join("rust")).unwrap();
    sora_core::pipeline::generate_code(&schema_input, "kotlin", &out_dir.join("kotlin")).unwrap();
    sora_core::pipeline::generate_code(&schema_input, "proto-schema", &out_dir.join("proto"))
        .unwrap();
    sora_core::pipeline::export_data(
        &project_input,
        "binary",
        ExportOutput::File(out_dir.join("config.sora")),
    )
    .unwrap();
    sora_core::pipeline::export_data(
        &project_input,
        "json",
        ExportOutput::File(out_dir.join("config.json")),
    )
    .unwrap();
    sora_core::pipeline::export_data(
        &project_input,
        "sora-protobuf",
        ExportOutput::File(out_dir.join("config.sora.pb")),
    )
    .unwrap();
    sora_core::pipeline::export_data(
        &project_input,
        "proto",
        ExportOutput::File(out_dir.join("config.pb")),
    )
    .unwrap();
    sora_core::pipeline::export_data(
        &project_input,
        "cbor",
        ExportOutput::File(out_dir.join("config.cbor")),
    )
    .unwrap();
    sora_core::pipeline::export_data(
        &project_input,
        "json-debug",
        ExportOutput::Directory(out_dir.join("debug-json")),
    )
    .unwrap();
    sora_core::pipeline::generate_excel_template(&schema_input, &out_dir.join("excel")).unwrap();

    assert!(
        fs::read_to_string(out_dir.join("rust/item.rs"))
            .unwrap()
            .contains("pub struct Item")
    );
    assert!(
        fs::read_to_string(out_dir.join("rust/runtime.rs"))
            .unwrap()
            .contains("pub struct SoraBundle")
    );
    assert!(
        fs::read_to_string(out_dir.join("rust/mod.rs"))
            .unwrap()
            .contains("pub struct SoraConfig")
    );
    assert!(
        fs::read_to_string(out_dir.join("kotlin/game_config/Item.kt"))
            .unwrap()
            .contains("data class Item")
    );
    assert_eq!(
        &fs::read(out_dir.join("config.sora")).unwrap()[0..4],
        b"SORA"
    );
    assert!(
        fs::read_to_string(out_dir.join("config.json"))
            .unwrap()
            .contains("\"format\": \"json\"")
    );
    assert!(!fs::read(out_dir.join("config.sora.pb")).unwrap().is_empty());
    assert!(!fs::read(out_dir.join("config.pb")).unwrap().is_empty());
    assert!(!fs::read(out_dir.join("config.cbor")).unwrap().is_empty());
    assert!(
        fs::read_to_string(out_dir.join("debug-json/Item.json"))
            .unwrap()
            .contains("Magic Stone")
    );
    assert_eq!(
        &fs::read(out_dir.join("excel/Item.xlsx")).unwrap()[0..2],
        b"PK"
    );

    let _ = fs::remove_dir_all(out_dir);
}

#[test]
fn namespaced_project_builds_code_and_data_end_to_end() {
    let root = temp_dir();
    let schema_dir = root.join("schema");
    let data_dir = root.join("data");
    fs::create_dir_all(&schema_dir).unwrap();
    fs::create_dir_all(&data_dir).unwrap();
    let project = root.join("project.scon");
    fs::write(
        &project,
        r#"project { id = "game" }
groups { common { default = true } }
views {
  default {
    contract = "game/default"
    groups = ["common"]
  }
}
includes = ["schema/items.scon"]
"#,
    )
    .unwrap();
    fs::write(
        schema_dir.join("items.scon"),
        r#"namespace = "game.items"
tables {
  Item {
    mode = "map"
    key = "id"
    source {
      format = "toml"
      file = "items.toml"
    }
    fields {
      id = "string"
      name = "string"
    }
  }
}
"#,
    )
    .unwrap();
    fs::write(
        data_dir.join("items.toml"),
        "[[rows]]\nid = \"iron_sword\"\nname = \"Iron Sword\"\n",
    )
    .unwrap();

    let schema_input = ProjectSchemaInput::new(&project);
    let project_input = SplitProjectInput::new(
        ProjectSchemaInput::new(&project),
        TomlDataInput::new(&data_dir),
    );
    let out = root.join("generated");
    sora_core::pipeline::check_schema(&schema_input).unwrap();
    sora_core::pipeline::generate_code(&schema_input, "rust", &out.join("rust")).unwrap();
    sora_core::pipeline::export_data(
        &project_input,
        "binary",
        ExportOutput::File(out.join("config.sora")),
    )
    .unwrap();
    sora_core::pipeline::export_data(
        &project_input,
        "json-debug",
        ExportOutput::Directory(out.join("debug")),
    )
    .unwrap();

    let item = fs::read_to_string(out.join("rust/game/items/item.rs")).unwrap();
    assert!(item.contains("pub const NAME: &'static str = \"game.items.Item\""));
    assert!(out.join("debug/game/items/Item.json").is_file());
    assert_eq!(&fs::read(out.join("config.sora")).unwrap()[0..4], b"SORA");

    let _ = fs::remove_dir_all(root);
}

fn write_item_workbook(project: &Path, out_dir: &Path) {
    let schema = load_project_schema(project).unwrap();
    let ir = normalize_schema(schema).unwrap();
    validate_config_ir(&ir).unwrap();
    let table = &ir.tables[0];

    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    worksheet.set_name("Item").unwrap();
    for (row_index, row) in table_template_rows(&ir, table).iter().enumerate() {
        for (column_index, value) in row.iter().enumerate() {
            worksheet
                .write_string(row_index as u32, column_index as u16, value)
                .unwrap();
        }
    }

    let rows = [
        ["1001", "Iron Sword", "Weapon", "1"],
        ["1002", "Magic Stone", "Material", "999"],
    ];
    for (offset, row) in rows.iter().enumerate() {
        for (column, value) in row.iter().enumerate() {
            worksheet
                .write_string(
                    DATA_START_ROW + offset as u32,
                    FIELD_START_COLUMN + column as u16,
                    *value,
                )
                .unwrap();
        }
    }

    fs::create_dir_all(out_dir).unwrap();
    workbook.save(out_dir.join("Item.xlsx")).unwrap();
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
        .to_path_buf()
}

fn temp_dir() -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("sora-simple-pipeline-test-{unique}"))
}
