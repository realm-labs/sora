use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use sora_export::exporter::ExportOutput;
use sora_input::project::SplitProjectInput;
use sora_input_schema::input::ProjectSchemaInput;
use sora_input_toml::input::TomlDataInput;

#[test]
fn generated_rust_runtime_compiles_and_loads_config_bundles() {
    for case in [
        RuntimeCase {
            runtime_format: "sora",
            export_format: "binary",
            file_name: "config.sora",
        },
        RuntimeCase {
            runtime_format: "json",
            export_format: "json",
            file_name: "config.json",
        },
        RuntimeCase {
            runtime_format: "cbor",
            export_format: "cbor",
            file_name: "config.cbor",
        },
        RuntimeCase {
            runtime_format: "sora-protobuf",
            export_format: "sora-protobuf",
            file_name: "config.sora.pb",
        },
    ] {
        let base = temp_dir();
        let project_path = write_project(&base, case.runtime_format);
        let generated_dir = base.join("generated-crate");

        let schema_input = ProjectSchemaInput::new(&project_path);
        sora_core::pipeline::generate_code(&schema_input, "rust", &generated_dir).unwrap();

        let project_input = SplitProjectInput::new(
            ProjectSchemaInput::new(&project_path),
            TomlDataInput::new(base.join("data")),
        );
        sora_core::pipeline::export_data(
            &project_input,
            case.export_format,
            ExportOutput::File(generated_dir.join(case.file_name)),
        )
        .unwrap();

        write_generated_crate_test(&generated_dir, case.runtime_format, case.file_name);
        assert_generated_crate_tests_pass(&generated_dir);

        let _ = fs::remove_dir_all(base);
    }
}

#[derive(Clone, Copy)]
struct RuntimeCase {
    runtime_format: &'static str,
    export_format: &'static str,
    file_name: &'static str,
}

fn write_project(base: &Path, runtime_format: &str) -> PathBuf {
    let schema_dir = base.join("schema");
    let data_dir = base.join("data");
    fs::create_dir_all(&schema_dir).unwrap();
    fs::create_dir_all(&data_dir).unwrap();

    let project_path = base.join("project.toml");
    fs::write(
        &project_path,
        r#"
project = { id = "game_config" }
groups = { common = { default = true } }
views = { default = { contract = "game_config/default", groups = ["common"] } }
includes = ["schema/items.toml"]

[codegen.rust]
runtime_format = "__RUNTIME_FORMAT__"
crate = { name = "generated-sora-config-test" }
"#
        .replace("__RUNTIME_FORMAT__", runtime_format),
    )
    .unwrap();
    fs::write(
        schema_dir.join("items.toml"),
        r#"
[enums.ItemType]
values = [{ id = 10, name = "Weapon" }, { id = 20, name = "Armor" }, { id = 30, name = "Material" }, { id = 40, name = "Consumable" }]

[structs.Reward.fields]
reward_item_id = "i32"
count = "i32"

[tables.Item]
id = "item"
mode = "map"
key = "id"

[tables.Item.source]
format = "toml"
file = "items.toml"

[tables.Item.fields]
id = "i32"
name = "string"
item_type = "enum<ItemType>"
max_stack = "i32"
signed_byte = "i8"
unsigned_byte = "u8"
signed_short = "i16"
unsigned_short = "u16"
unsigned_int = "u32"

[tables.Item.fields.rewards]
type = "list<Reward>"
from = { table = "ItemReward", parent_key = "id", child_key = "item_id", order_by = "seq" }

[tables.ItemReward]
id = "item_reward"
mode = "list"

[tables.ItemReward.source]
format = "toml"
file = "item_rewards.toml"

[tables.ItemReward.fields]
item_id = "i32"
seq = "i32"
reward_item_id = "i32"
count = "i32"
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
signed_byte = -128
unsigned_byte = 255
signed_short = -32768
unsigned_short = 65535
unsigned_int = 4294967295

[[rows]]
id = 1002
name = "Magic Stone"
item_type = "Material"
max_stack = 999
signed_byte = 127
unsigned_byte = 0
signed_short = 32767
unsigned_short = 0
unsigned_int = 0
"#,
    )
    .unwrap();
    fs::write(
        data_dir.join("item_rewards.toml"),
        r#"
[[rows]]
item_id = 1002
seq = 2
reward_item_id = 3002
count = 5

[[rows]]
item_id = 1002
seq = 1
reward_item_id = 3001
count = 2
"#,
    )
    .unwrap();

    project_path
}

fn write_generated_crate_test(crate_dir: &Path, runtime_format: &str, file_name: &str) {
    let bundle_type = match runtime_format {
        "sora" => "SoraBundle",
        "json" => "JsonBundle",
        "cbor" => "CborBundle",
        "sora-protobuf" => "ProtobufBundle",
        other => panic!("unsupported Rust runtime format `{other}`"),
    };
    fs::create_dir_all(crate_dir.join("tests")).unwrap();
    fs::write(
        crate_dir.join("tests/runtime.rs"),
        r#"
use generated_sora_config_test::{
    item_type::ItemType,
    runtime::{__BUNDLE_TYPE__, SoraDecode, SoraReadError, SoraTableSource},
    SoraConfig,
};

#[test]
fn loads_sora_bundle() {
    let bundle = __BUNDLE_TYPE__::parse(include_bytes!("../__CONFIG_FILE__")).unwrap();
    let config = SoraConfig::from_source(&bundle).unwrap();
    let item = config.item().get(&1002).unwrap();

    assert_eq!(item.name, "Magic Stone");
    assert_eq!(item.item_type, ItemType::Material);
    assert_eq!(item.max_stack, 999);
    assert_eq!(item.signed_byte, 127);
    assert_eq!(item.unsigned_byte, 0);
    assert_eq!(item.signed_short, 32767);
    assert_eq!(item.unsigned_short, 0);
    assert_eq!(item.unsigned_int, 0);
    assert_eq!(item.rewards.len(), 2);
    assert_eq!(item.rewards[0].reward_item_id, 3001);
    assert_eq!(item.rewards[0].count, 2);
    assert_eq!(item.rewards[1].reward_item_id, 3002);
    assert_eq!(config.item().values().count(), 2);
    assert_eq!(config.item_reward().len(), 2);

    let boundary_item = config.item().get(&1001).unwrap();
    assert_eq!(boundary_item.signed_byte, -128);
    assert_eq!(boundary_item.unsigned_byte, 255);
    assert_eq!(boundary_item.signed_short, -32768);
    assert_eq!(boundary_item.unsigned_short, 65535);
    assert_eq!(boundary_item.unsigned_int, 4294967295);
}

#[test]
fn rejects_schema_fingerprint_mismatch() {
    let error = SoraConfig::from_source(&BadSource).unwrap_err();

    assert!(error.to_string().contains("schema fingerprint mismatch"));
}

struct BadSource;

impl SoraTableSource for BadSource {
    fn schema_fingerprint(&self) -> Result<&str, SoraReadError> {
        Ok("bad-schema")
    }

    fn decode_table<T>(&self, _name: &str) -> Result<Vec<T>, SoraReadError>
    where
        T: SoraDecode + serde::de::DeserializeOwned,
    {
        panic!("schema mismatch should be reported before decoding tables")
    }
}
"#
        .replace("__BUNDLE_TYPE__", bundle_type)
        .replace("__CONFIG_FILE__", file_name),
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
