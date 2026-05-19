use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use sora_core::{CodegenTarget, ExportOutput};

#[test]
fn simple_example_pipeline_generates_all_artifacts() {
    let root = workspace_root();
    let schema = root.join("examples/simple/schema.toml");
    let data_root = root.join("examples/simple/data");
    let out_dir = temp_dir();

    sora_core::check_schema(&schema).unwrap();

    sora_core::generate_code(&schema, CodegenTarget::Rust, &out_dir.join("rust")).unwrap();
    sora_core::generate_code(&schema, CodegenTarget::Kotlin, &out_dir.join("kotlin")).unwrap();
    sora_core::export_data(
        &schema,
        &data_root,
        "binary",
        ExportOutput::File(out_dir.join("config.sora")),
    )
    .unwrap();
    sora_core::export_data(
        &schema,
        &data_root,
        "json-debug",
        ExportOutput::Directory(out_dir.join("debug-json")),
    )
    .unwrap();
    sora_core::generate_excel_template(&schema, &out_dir.join("excel")).unwrap();

    assert!(
        fs::read_to_string(out_dir.join("rust/item.rs"))
            .unwrap()
            .contains("pub struct Item")
    );
    assert!(
        fs::read_to_string(out_dir.join("kotlin/Item.kt"))
            .unwrap()
            .contains("data class Item")
    );
    assert_eq!(
        &fs::read(out_dir.join("config.sora")).unwrap()[0..4],
        b"SORA"
    );
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
