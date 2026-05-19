use std::path::Path;

pub use sora_codegen::CodegenTarget;
use sora_codegen::generator_for_target;
use sora_diagnostics::{Result, SoraError};
use sora_excel::ExcelTemplateGenerator;
pub use sora_export::ExportOutput;
use sora_export::{ExportRequest, ExporterRegistry};
use sora_ir::{ConfigIr, normalize_schema};
use sora_schema::load_schema_file;

pub fn check_schema(schema_path: &Path) -> Result<()> {
    let _ = load_ir(schema_path)?;
    Ok(())
}

pub fn generate_code(schema_path: &Path, target: CodegenTarget, out_dir: &Path) -> Result<()> {
    let ir = load_ir(schema_path)?;
    generator_for_target(target).generate(&ir, out_dir)
}

pub fn export_data(
    schema_path: &Path,
    data_root: &Path,
    format: &str,
    output: ExportOutput,
) -> Result<()> {
    let ir = load_ir(schema_path)?;
    let data = sora_data::load_config_data(&ir, data_root)?;
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

pub fn generate_excel_template(schema_path: &Path, out_dir: &Path) -> Result<()> {
    let ir = load_ir(schema_path)?;
    ExcelTemplateGenerator.generate(&ir, out_dir)
}

pub fn supported_export_formats() -> Vec<&'static str> {
    ExporterRegistry::with_builtin_exporters().supported_formats()
}

fn load_ir(schema_path: &Path) -> Result<ConfigIr> {
    normalize_schema(load_schema_file(schema_path)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        sync::atomic::{AtomicU64, Ordering},
    };

    static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn checks_schema_and_generates_outputs() {
        let base = temp_dir();
        let schema_path = write_example(&base);

        check_schema(&schema_path).unwrap();
        generate_code(&schema_path, CodegenTarget::Rust, &base.join("rust")).unwrap();
        generate_code(&schema_path, CodegenTarget::Kotlin, &base.join("kotlin")).unwrap();
        generate_excel_template(&schema_path, &base.join("excel")).unwrap();

        assert!(base.join("rust/item.rs").exists());
        assert!(base.join("kotlin/Item.kt").exists());
        assert!(base.join("excel/Item.csv").exists());

        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn exports_data_through_registry() {
        let base = temp_dir();
        let schema_path = write_example(&base);

        export_data(
            &schema_path,
            &base.join("data"),
            "binary",
            ExportOutput::File(base.join("config.sora")),
        )
        .unwrap();
        export_data(
            &schema_path,
            &base.join("data"),
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

        let error = export_data(
            &schema_path,
            &base.join("data"),
            "nope",
            ExportOutput::File(base.join("out.bin")),
        )
        .unwrap_err();

        assert!(matches!(
            error,
            SoraError::UnknownExportFormat { format, .. } if format == "nope"
        ));

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
}
