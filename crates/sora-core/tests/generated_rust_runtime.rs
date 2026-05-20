use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use sora_codegen::target::CodegenTarget;
use sora_export::exporter::ExportOutput;
use sora_input_toml::input::{TomlProjectInput, TomlSchemaInput};

#[test]
fn generated_rust_runtime_compiles_and_loads_binary_bundle() {
    let base = temp_dir();
    let project_path = write_project(&base);
    let generated_dir = base.join("generated-crate");
    let generated_src = generated_dir.join("src/generated");

    let schema_input = TomlSchemaInput::new(&project_path);
    sora_core::pipeline::generate_code(&schema_input, CodegenTarget::Rust, &generated_src).unwrap();

    let project_input = TomlProjectInput::new(&project_path, base.join("data"));
    sora_core::pipeline::export_data(
        &project_input,
        "binary",
        ExportOutput::File(generated_dir.join("config.sora")),
    )
    .unwrap();

    write_generated_crate(&generated_dir);
    assert_generated_crate_tests_pass(&generated_dir);

    let _ = fs::remove_dir_all(base);
}

fn write_project(base: &Path) -> PathBuf {
    let schema_dir = base.join("schema");
    let data_dir = base.join("data");
    fs::create_dir_all(&schema_dir).unwrap();
    fs::create_dir_all(&data_dir).unwrap();

    let project_path = base.join("project.toml");
    fs::write(
        &project_path,
        r#"
package = "game_config"
includes = ["schema/items.toml"]
"#,
    )
    .unwrap();
    fs::write(
        schema_dir.join("items.toml"),
        r#"
[[enums]]
name = "ItemType"
values = ["Weapon", "Armor", "Material", "Consumable"]

[[tables]]
name = "Item"
mode = "map"
key = "id"

[tables.source]
format = "toml"
file = "items.toml"

[[tables.fields]]
name = "id"
type = "i32"
required = true

[[tables.fields]]
name = "name"
type = "string"
required = true

[[tables.fields]]
name = "item_type"
type = "enum<ItemType>"
required = true

[[tables.fields]]
name = "max_stack"
type = "i32"
required = true
"#,
    )
    .unwrap();
    fs::write(
        data_dir.join("items.toml"),
        r#"
[[rows]]
id = 1001
name = "Iron Sword"
item_type = "Weapon"
max_stack = 1

[[rows]]
id = 1002
name = "Magic Stone"
item_type = "Material"
max_stack = 999
"#,
    )
    .unwrap();

    project_path
}

fn write_generated_crate(crate_dir: &Path) {
    fs::write(
        crate_dir.join("Cargo.toml"),
        r#"
[package]
name = "generated-sora-config-test"
version = "0.1.0"
edition = "2024"

[dependencies]
serde = { version = "1", features = ["derive"] }
"#,
    )
    .unwrap();
    fs::write(
        crate_dir.join("src/lib.rs"),
        r#"
pub mod generated;

#[cfg(test)]
mod tests {
    use super::generated::{item_type::ItemType, SoraConfig};

    #[test]
    fn loads_sora_bundle() {
        let config = SoraConfig::from_bytes(include_bytes!("../config.sora")).unwrap();
        let item = config.item.get(&1002).unwrap();

        assert_eq!(item.name, "Magic Stone");
        assert_eq!(item.item_type, ItemType::Material);
        assert_eq!(item.max_stack, 999);
    }
}
"#,
    )
    .unwrap();
}

fn assert_generated_crate_tests_pass(crate_dir: &Path) {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned());
    let status = Command::new(cargo)
        .arg("test")
        .arg("--manifest-path")
        .arg(crate_dir.join("Cargo.toml"))
        .arg("--target-dir")
        .arg(crate_dir.join("target"))
        .status()
        .expect("generated crate test command should start");

    assert!(status.success(), "generated crate tests should pass");
}

fn temp_dir() -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("sora-generated-runtime-test-{unique}"))
}
