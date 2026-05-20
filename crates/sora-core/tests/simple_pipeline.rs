use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use rust_xlsxwriter::Workbook;
use sora_codegen::target::CodegenTarget;
use sora_excel::projection::table_template_rows;
use sora_export::exporter::ExportOutput;
use sora_input_toml::{input::TomlSchemaInput, schema::load_project_schema_file};
use sora_input_xlsx::input::XlsxProjectInput;
use sora_ir::{normalize::normalize_schema, validate::validate_config_ir};

#[test]
fn simple_example_pipeline_generates_all_artifacts() {
    let root = workspace_root();
    let project = root.join("examples/simple/project.toml");
    let schema_input = TomlSchemaInput::new(&project);
    let out_dir = temp_dir();
    let data_root = out_dir.join("excel-data");
    write_item_workbook(&project, &data_root);
    let project_input = XlsxProjectInput::new(TomlSchemaInput::new(&project), &data_root);

    sora_core::pipeline::check_schema(&schema_input).unwrap();

    sora_core::pipeline::generate_code(&schema_input, CodegenTarget::Rust, &out_dir.join("rust"))
        .unwrap();
    sora_core::pipeline::generate_code(
        &schema_input,
        CodegenTarget::Kotlin,
        &out_dir.join("kotlin"),
    )
    .unwrap();
    sora_core::pipeline::generate_code(&schema_input, CodegenTarget::Proto, &out_dir.join("proto"))
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
        "protobuf",
        ExportOutput::File(out_dir.join("config.pb")),
    )
    .unwrap();
    sora_core::pipeline::export_data(
        &project_input,
        "typed-protobuf",
        ExportOutput::File(out_dir.join("config.typed.pb")),
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
    assert!(!fs::read(out_dir.join("config.pb")).unwrap().is_empty());
    assert!(
        !fs::read(out_dir.join("config.typed.pb"))
            .unwrap()
            .is_empty()
    );
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

fn write_item_workbook(project: &Path, out_dir: &Path) {
    let schema = load_project_schema_file(project).unwrap();
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
                .write_string((10 + offset) as u32, column as u16, *value)
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
