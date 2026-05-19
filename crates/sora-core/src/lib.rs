use std::path::Path;

pub use sora_codegen::CodegenTarget;
use sora_codegen::generator_for_target;
use sora_diagnostics::{Result, SoraError};
use sora_excel::ExcelTemplateGenerator;
pub use sora_export::ExportOutput;
pub use sora_export::OutputKind;
use sora_export::{ExportRequest, ExporterRegistry};
pub use sora_input::{DataInput, LoadedInput, ProjectInput, SchemaInput};
use sora_ir::{ConfigIr, normalize_schema, validate_config_ir};

pub fn check_schema(input: &impl SchemaInput) -> Result<()> {
    let _ = load_ir(input)?;
    Ok(())
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
    let data = input.load_data(&ir)?;
    sora_data::validate_config_data(&ir, &data)?;

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

#[cfg(test)]
mod tests {
    use super::*;
    use sora_data::{ConfigData, RowData, TableData, Value};
    use sora_input_toml::{TomlProjectInput, TomlSchemaInput};
    use sora_schema::SchemaFile;
    use std::{
        collections::BTreeMap,
        fs,
        sync::atomic::{AtomicU64, Ordering},
    };

    static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn checks_schema_and_generates_outputs() {
        let base = temp_dir();
        let schema_path = write_example(&base);
        let input = TomlSchemaInput::new(&schema_path);

        check_schema(&input).unwrap();
        generate_code(&input, CodegenTarget::Rust, &base.join("rust")).unwrap();
        generate_code(&input, CodegenTarget::Kotlin, &base.join("kotlin")).unwrap();
        generate_excel_template(&input, &base.join("excel")).unwrap();

        assert!(base.join("rust/item.rs").exists());
        assert!(base.join("kotlin/Item.kt").exists());
        assert!(base.join("excel/Item.xlsx").exists());

        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn exports_data_through_registry() {
        let base = temp_dir();
        let schema_path = write_example(&base);
        let input = TomlProjectInput::new(&schema_path, base.join("data"));

        export_data(
            &input,
            "binary",
            ExportOutput::File(base.join("config.sora")),
        )
        .unwrap();
        export_data(
            &input,
            "json-debug",
            ExportOutput::Directory(base.join("debug-json")),
        )
        .unwrap();

        assert!(base.join("config.sora").exists());
        assert!(base.join("debug-json/Item.json").exists());

        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn reports_unknown_export_format() {
        let base = temp_dir();
        let schema_path = write_example(&base);
        let input = TomlProjectInput::new(&schema_path, base.join("data"));

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

    fn write_example(base: &Path) -> std::path::PathBuf {
        let data_dir = base.join("data");
        fs::create_dir_all(&data_dir).unwrap();
        let schema_path = base.join("schema.toml");
        fs::write(
            &schema_path,
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
        schema_path
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
}
