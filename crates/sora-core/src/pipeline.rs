use std::{fs, path::Path};

use sora_codegen::{generator::generator_for_target, target::CodegenTarget};
use sora_diagnostics::{Result, SoraError};
use sora_excel::generator::ExcelTemplateGenerator;
use sora_export::{
    exporter::{ExportOutput, ExportRequest, OutputKind},
    registry::ExporterRegistry,
};
use sora_input::traits::{ProjectInput, SchemaInput};
use sora_ir::{model::ConfigIr, normalize::normalize_schema, validate::validate_config_ir};

use crate::diff::{ConfigDiff, diff_config_data};
use crate::schema_lock::{read_schema_lock_file, verify_schema_lock, write_schema_lock_file};

pub fn check_schema(input: &impl SchemaInput) -> Result<()> {
    let _ = load_ir(input)?;
    Ok(())
}

pub fn check_schema_with_lock(input: &impl SchemaInput, lock_path: &Path) -> Result<()> {
    let ir = load_ir(input)?;
    let lock = read_schema_lock_file(lock_path)?;
    verify_schema_lock(&ir, &lock)
}

pub fn generate_schema_lock(input: &impl SchemaInput, path: &Path) -> Result<()> {
    let ir = load_ir(input)?;
    write_schema_lock_file(&ir, path)
}

pub fn generate_code(
    input: &impl SchemaInput,
    target: CodegenTarget,
    out_dir: &Path,
) -> Result<()> {
    let ir = load_ir(input)?;
    generator_for_target(target).generate(&ir, out_dir)
}

pub fn export_data(input: &impl ProjectInput, format: &str, output: ExportOutput) -> Result<()> {
    let ir = load_ir(input)?;
    let data = load_validated_data(input, &ir)?;

    let registry = ExporterRegistry::with_builtin_exporters();
    let exporter = registry
        .get(format)
        .ok_or_else(|| SoraError::UnknownExportFormat {
            format: format.to_owned(),
            supported: registry.supported_formats().join(", "),
        })?;

    exporter.export(ExportRequest {
        ir: &ir,
        data: &data,
        output,
    })
}

pub fn diff_data(
    left: &impl ProjectInput,
    right: &impl ProjectInput,
    output_path: &Path,
) -> Result<ConfigDiff> {
    let ir = load_ir(left)?;
    let left_data = load_validated_data(left, &ir)?;
    let right_data = load_validated_data(right, &ir)?;
    let diff = diff_config_data(&ir, &left_data, &right_data)?;
    write_json_file(output_path, &diff)?;
    Ok(diff)
}

pub fn generate_excel_template(input: &impl SchemaInput, out_dir: &Path) -> Result<()> {
    let ir = load_ir(input)?;
    ExcelTemplateGenerator.generate(&ir, out_dir)
}

pub fn supported_export_formats() -> Vec<&'static str> {
    ExporterRegistry::with_builtin_exporters().supported_formats()
}

pub fn export_output_kind(format: &str) -> Option<OutputKind> {
    ExporterRegistry::with_builtin_exporters()
        .get(format)
        .map(|exporter| exporter.output_kind())
}

fn load_ir(input: &impl SchemaInput) -> Result<ConfigIr> {
    let ir = normalize_schema(input.load_schema()?)?;
    validate_config_ir(&ir)?;
    Ok(ir)
}

fn load_validated_data(
    input: &impl ProjectInput,
    ir: &ConfigIr,
) -> Result<sora_data::model::ConfigData> {
    let data = input.load_data(ir)?;
    let data = sora_input::defaults::materialize_defaults(ir, &data)?;
    let data = sora_data::aggregate::materialize_aggregations(ir, &data)?;
    sora_data::validate::validate_config_data(ir, &data)?;
    Ok(data)
}

fn write_json_file<T: serde::Serialize>(path: &Path, value: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| SoraError::CreateDir {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    let bytes = serde_json::to_vec_pretty(value).map_err(SoraError::SerializeData)?;
    fs::write(path, bytes).map_err(|source| SoraError::WriteFile {
        path: path.to_path_buf(),
        source,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use sora_data::model::{ConfigData, RowData, TableData, Value};
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
        generate_code(&input, CodegenTarget::Rust, &base.join("rust")).unwrap();
        generate_code(&input, CodegenTarget::Kotlin, &base.join("kotlin")).unwrap();
        generate_code(&input, CodegenTarget::TypeScript, &base.join("typescript")).unwrap();
        generate_code(&input, CodegenTarget::JavaScript, &base.join("javascript")).unwrap();
        generate_code(&input, CodegenTarget::Proto, &base.join("proto")).unwrap();
        generate_excel_template(&input, &base.join("excel")).unwrap();

        assert!(base.join("rust/item.rs").exists());
        assert!(base.join("kotlin/game_config/Item.kt").exists());
        assert!(base.join("typescript/item.ts").exists());
        assert!(base.join("javascript/item.js").exists());
        assert!(base.join("proto/sora_config.proto").exists());
        assert!(base.join("excel/Item.xlsx").exists());
        assert!(base.join("schema.lock").exists());

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
            "protobuf",
            ExportOutput::File(base.join("config.pb")),
        )
        .unwrap();
        export_data(
            &input,
            "typed-protobuf",
            ExportOutput::File(base.join("config.typed.pb")),
        )
        .unwrap();
        export_data(&input, "cbor", ExportOutput::File(base.join("config.cbor"))).unwrap();
        export_data(
            &input,
            "json-debug",
            ExportOutput::Directory(base.join("debug-json")),
        )
        .unwrap();

        assert!(base.join("config.sora").exists());
        assert!(base.join("config.json").exists());
        assert!(base.join("config.pb").exists());
        assert!(base.join("config.typed.pb").exists());
        assert!(base.join("config.cbor").exists());
        assert!(base.join("debug-json/Item.json").exists());

        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn reports_unknown_export_format() {
        let base = temp_dir();
        let project_path = write_example(&base);
        let input = TomlProjectInput::new(&project_path, base.join("data"));

        let error =
            export_data(&input, "nope", ExportOutput::File(base.join("out.bin"))).unwrap_err();

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
        generate_code(&input, CodegenTarget::Rust, &base.join("rust")).unwrap();
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
}
