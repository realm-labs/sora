use super::*;
use sora_data::model::{ConfigData, RowData, TableData, Value};
use sora_diagnostics::Result;
use sora_export::exporter::ExportOutput;
use sora_input::loaded::LoadedInput;
use sora_input_toml::input::{TomlProjectInput, TomlSchemaInput};
use sora_schema::model::SchemaFile;
use std::{
    collections::BTreeMap,
    fs,
    sync::atomic::{AtomicU64, Ordering},
};

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

#[test]
fn checks_schema_and_generates_outputs() {
    let base = temp_dir();
    let project_path = write_example(&base);
    let input = TomlSchemaInput::new(&project_path);

    check_schema(&input).unwrap();
    generate_schema_lock(&input, &base.join("schema.lock")).unwrap();
    check_schema_with_lock(&input, &base.join("schema.lock")).unwrap();
    generate_code(&input, "rust", &base.join("rust")).unwrap();
    generate_code(&input, "kotlin", &base.join("kotlin")).unwrap();
    generate_code(&input, "typescript", &base.join("typescript")).unwrap();
    generate_code(&input, "javascript", &base.join("javascript")).unwrap();
    generate_code(&input, "c", &base.join("c")).unwrap();
    generate_code(&input, "cpp", &base.join("cpp")).unwrap();
    generate_code(&input, "erlang", &base.join("erlang")).unwrap();
    generate_code(&input, "python", &base.join("python")).unwrap();
    generate_code(&input, "proto-schema", &base.join("proto")).unwrap();
    generate_excel_template(&input, &base.join("excel")).unwrap();

    assert!(base.join("rust/item.rs").exists());
    assert!(base.join("kotlin/game_config/Item.kt").exists());
    assert!(base.join("typescript/item.ts").exists());
    assert!(base.join("javascript/item.js").exists());
    assert!(base.join("c/item.h").exists());
    assert!(base.join("cpp/item.hpp").exists());
    assert!(base.join("erlang/item.erl").exists());
    assert!(base.join("python/item.py").exists());
    assert!(base.join("python/sora_config.py").exists());
    assert!(base.join("proto/sora_config.proto").exists());
    assert!(base.join("excel/Item.xlsx").exists());
    assert!(base.join("schema.lock").exists());

    let _ = fs::remove_dir_all(base);
}

#[test]
fn accepts_registered_codegen_target() {
    struct MarkerGenerator;

    impl sora_codegen::generator::CodeGenerator for MarkerGenerator {
        fn generate(
            &self,
            _context: sora_codegen::generator::CodegenContext<'_>,
            out_dir: &std::path::Path,
        ) -> Result<()> {
            fs::create_dir_all(out_dir).unwrap();
            fs::write(out_dir.join("marker.txt"), "generated").unwrap();
            Ok(())
        }
    }

    let base = temp_dir();
    let project_path = write_example(&base);
    let input = TomlSchemaInput::new(&project_path);
    let mut registry = sora_codegen::generator::CodegenRegistry::new();
    registry
        .register(sora_codegen::generator::CodegenRegistration {
            id: "marker",
            aliases: &[],
            display_name: "Marker",
            supported_runtime_formats: &[],
            runtime_format: |_, _| Ok(None),
            formatter: None,
            generator: Box::new(MarkerGenerator),
        })
        .unwrap();

    generate_code_with_registry_scope_and_format(
        &input,
        "marker",
        &base.join("marker"),
        sora_codegen::format::FormatMode::Never,
        None,
        &registry,
    )
    .unwrap();

    assert_eq!(
        fs::read_to_string(base.join("marker/marker.txt")).unwrap(),
        "generated"
    );

    let _ = fs::remove_dir_all(base);
}

#[test]
fn exports_data_through_registry() {
    let base = temp_dir();
    let project_path = write_example(&base);
    let input = TomlProjectInput::new(&project_path, base.join("data"));

    export_data(
        &input,
        "binary",
        ExportOutput::File(base.join("config.sora")),
    )
    .unwrap();
    export_data(&input, "json", ExportOutput::File(base.join("config.json"))).unwrap();
    export_data(
        &input,
        "sora-protobuf",
        ExportOutput::File(base.join("config.sora.pb")),
    )
    .unwrap();
    export_data(&input, "proto", ExportOutput::File(base.join("config.pb"))).unwrap();
    export_data(&input, "cbor", ExportOutput::File(base.join("config.cbor"))).unwrap();
    export_data(
        &input,
        "json-debug",
        ExportOutput::Directory(base.join("debug-json")),
    )
    .unwrap();

    assert!(base.join("config.sora").exists());
    assert!(base.join("config.json").exists());
    assert!(base.join("config.sora.pb").exists());
    assert!(base.join("config.pb").exists());
    assert!(base.join("config.cbor").exists());
    assert!(base.join("debug-json/Item.json").exists());

    let _ = fs::remove_dir_all(base);
}

#[test]
fn scoped_export_filters_schema_and_data() {
    let base = temp_dir();
    let input = LoadedInput::with_data(scoped_schema(), scoped_data());
    let output = base.join("client.json");

    export_data_with_scope(
        &input,
        "json",
        ExportOutput::File(output.clone()),
        Some("client"),
    )
    .unwrap();

    let value: serde_json::Value = serde_json::from_slice(&fs::read(output).unwrap()).unwrap();
    let fields = value["schema"]["tables"][0]["fields"]
        .as_array()
        .unwrap()
        .iter()
        .map(|field| field["name"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(fields, ["id", "name"]);
    assert!(value["data"]["tables"][0]["rows"][0]["server_formula"].is_null());

    let _ = fs::remove_dir_all(base);
}

#[test]
fn reports_unknown_export_format() {
    let base = temp_dir();
    let project_path = write_example(&base);
    let input = TomlProjectInput::new(&project_path, base.join("data"));

    let error = export_data(&input, "nope", ExportOutput::File(base.join("out.bin"))).unwrap_err();

    assert!(matches!(
        error,
        SoraError::UnknownExportFormat { format, .. } if format == "nope"
    ));

    let _ = fs::remove_dir_all(base);
}

#[test]
fn accepts_loaded_input_adapter() {
    let base = temp_dir();
    let input = LoadedInput::with_data(example_schema(), example_data());

    check_schema(&input).unwrap();
    generate_code(&input, "rust", &base.join("rust")).unwrap();
    export_data(
        &input,
        "json-debug",
        ExportOutput::Directory(base.join("debug-json")),
    )
    .unwrap();
    generate_excel_template(&input, &base.join("excel")).unwrap();

    assert!(base.join("rust/item.rs").exists());
    assert!(base.join("debug-json/Item.json").exists());
    assert!(base.join("excel/Item.xlsx").exists());

    let _ = fs::remove_dir_all(base);
}

#[test]
fn diffs_loaded_project_data() {
    let base = temp_dir();
    let left = LoadedInput::with_data(example_schema(), example_data());
    let right = LoadedInput::with_data(example_schema(), changed_example_data());

    let diff = diff_data(&left, &right, &base.join("diff/config.diff.json")).unwrap();

    assert!(diff.has_changes());
    assert!(base.join("diff/config.diff.json").exists());
    assert_eq!(diff.tables[0].name, "Item");
    assert_eq!(diff.tables[0].changed[0].fields[0].name, "max_stack");

    let _ = fs::remove_dir_all(base);
}

fn write_example(base: &Path) -> std::path::PathBuf {
    let data_dir = base.join("data");
    let schema_dir = base.join("schema");
    fs::create_dir_all(&data_dir).unwrap();
    fs::create_dir_all(&schema_dir).unwrap();
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
    fs::write(
        data_dir.join("items.toml"),
        r#"
[[rows]]
id = 1001
name = "Iron Sword"
item_type = "Weapon"
max_stack = 1
"#,
    )
    .unwrap();
    project_path
}

fn temp_dir() -> std::path::PathBuf {
    let unique = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("sora-core-test-{unique}"))
}

fn example_schema() -> SchemaFile {
    toml::from_str(
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
format = "toml"
file = "items.toml"

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
    .unwrap()
}

fn scoped_schema() -> SchemaFile {
    toml::from_str(
        r#"
package = "game_config"

[[tables]]
name = "Item"
mode = "map"
key = "id"

[[tables.fields]]
name = "id"
type = "i32"
key = true

[[tables.fields]]
name = "name"
type = "string"
required = true

[[tables.fields]]
name = "server_formula"
type = "string"
required = true
scope = "server"
"#,
    )
    .unwrap()
}

fn scoped_data() -> ConfigData {
    ConfigData {
        tables: vec![TableData {
            name: "Item".to_owned(),
            rows: vec![RowData {
                values: BTreeMap::from([
                    ("id".to_owned(), Value::Integer(1001)),
                    ("name".to_owned(), Value::String("Iron Sword".to_owned())),
                    (
                        "server_formula".to_owned(),
                        Value::String("internal".to_owned()),
                    ),
                ]),
            }],
        }],
    }
}

fn example_data() -> ConfigData {
    ConfigData {
        tables: vec![TableData {
            name: "Item".to_owned(),
            rows: vec![RowData {
                values: BTreeMap::from([
                    ("id".to_owned(), Value::Integer(1001)),
                    ("name".to_owned(), Value::String("Iron Sword".to_owned())),
                    ("item_type".to_owned(), Value::String("Weapon".to_owned())),
                    ("max_stack".to_owned(), Value::Integer(1)),
                ]),
            }],
        }],
    }
}

fn changed_example_data() -> ConfigData {
    ConfigData {
        tables: vec![TableData {
            name: "Item".to_owned(),
            rows: vec![RowData {
                values: BTreeMap::from([
                    ("id".to_owned(), Value::Integer(1001)),
                    ("name".to_owned(), Value::String("Iron Sword".to_owned())),
                    ("item_type".to_owned(), Value::String("Weapon".to_owned())),
                    ("max_stack".to_owned(), Value::Integer(99)),
                ]),
            }],
        }],
    }
}
