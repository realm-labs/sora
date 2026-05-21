use std::{fs, path::Path};

use sora_codegen::{
    format::{FormatMode, format_generated_code},
    generator::generator_for_target,
    target::CodegenTarget,
};
use sora_diagnostics::{Result, SoraError};
use sora_excel::generator::ExcelTemplateGenerator;
use sora_execution::ExecutionContext;
use sora_export::{
    exporter::{ExportOutput, ExportRequest, OutputKind},
    registry::ExporterRegistry,
};
use sora_input::traits::{ProjectInput, SchemaInput};
use sora_ir::{
    model::ConfigIr, normalize::normalize_schema, scope::filter_config_ir_by_scope,
    validate::validate_config_ir,
};

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
    generate_schema_lock_with_scope(input, path, None)
}

pub fn generate_schema_lock_with_scope(
    input: &impl SchemaInput,
    path: &Path,
    scope: Option<&str>,
) -> Result<()> {
    let ir = load_ir_with_scope(input, scope)?;
    write_schema_lock_file(&ir, path)
}

pub fn generate_code(
    input: &impl SchemaInput,
    target: CodegenTarget,
    out_dir: &Path,
) -> Result<()> {
    generate_code_with_format(input, target, out_dir, FormatMode::Never)
}

pub fn generate_code_with_format(
    input: &impl SchemaInput,
    target: CodegenTarget,
    out_dir: &Path,
    format_mode: FormatMode,
) -> Result<()> {
    generate_code_with_scope_and_format(input, target, out_dir, format_mode, None)
}

pub fn generate_code_with_scope_and_format(
    input: &impl SchemaInput,
    target: CodegenTarget,
    out_dir: &Path,
    format_mode: FormatMode,
    scope: Option<&str>,
) -> Result<()> {
    let ir = load_ir_with_scope(input, scope)?;
    generator_for_target(target).generate(&ir, out_dir)?;
    format_generated_code(target, out_dir, format_mode)
}

pub fn export_data(input: &impl ProjectInput, format: &str, output: ExportOutput) -> Result<()> {
    export_data_with_scope_and_context(input, format, output, None, &ExecutionContext::default())
}

pub fn export_data_with_scope(
    input: &impl ProjectInput,
    format: &str,
    output: ExportOutput,
    scope: Option<&str>,
) -> Result<()> {
    export_data_with_scope_and_context(input, format, output, scope, &ExecutionContext::default())
}

pub fn export_data_with_context(
    input: &impl ProjectInput,
    format: &str,
    output: ExportOutput,
    execution: &ExecutionContext,
) -> Result<()> {
    export_data_with_scope_and_context(input, format, output, None, execution)
}

pub fn export_data_with_scope_and_context(
    input: &impl ProjectInput,
    format: &str,
    output: ExportOutput,
    scope: Option<&str>,
    execution: &ExecutionContext,
) -> Result<()> {
    let ir = load_ir(input)?;
    let data = load_validated_data(input, &ir, execution)?;
    let (ir, data) = filter_ir_and_data_by_scope(&ir, &data, scope)?;

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
        execution,
        output,
    })
}

pub fn diff_data(
    left: &impl ProjectInput,
    right: &impl ProjectInput,
    output_path: &Path,
) -> Result<ConfigDiff> {
    diff_data_with_scope(left, right, output_path, None)
}

pub fn diff_data_with_context(
    left: &impl ProjectInput,
    right: &impl ProjectInput,
    output_path: &Path,
    execution: &ExecutionContext,
) -> Result<ConfigDiff> {
    diff_data_with_scope_and_context(left, right, output_path, None, execution)
}

pub fn diff_data_with_scope(
    left: &impl ProjectInput,
    right: &impl ProjectInput,
    output_path: &Path,
    scope: Option<&str>,
) -> Result<ConfigDiff> {
    diff_data_with_scope_and_context(
        left,
        right,
        output_path,
        scope,
        &ExecutionContext::default(),
    )
}

pub fn diff_data_with_scope_and_context(
    left: &impl ProjectInput,
    right: &impl ProjectInput,
    output_path: &Path,
    scope: Option<&str>,
    execution: &ExecutionContext,
) -> Result<ConfigDiff> {
    let ir = load_ir(left)?;
    let left_data = load_validated_data(left, &ir, execution)?;
    let right_data = load_validated_data(right, &ir, execution)?;
    let (ir, left_data) = filter_ir_and_data_by_scope(&ir, &left_data, scope)?;
    let right_data = match scope {
        Some(_) => {
            let scoped = sora_data::scope::filter_config_data_by_ir(&ir, &right_data);
            sora_data::validate::validate_config_data(&ir, &scoped)?;
            scoped
        }
        None => right_data,
    };
    let diff = diff_config_data(&ir, &left_data, &right_data)?;
    write_json_file(output_path, &diff)?;
    Ok(diff)
}

pub fn generate_excel_template(input: &impl SchemaInput, out_dir: &Path) -> Result<()> {
    generate_excel_template_with_scope(input, out_dir, None)
}

pub fn generate_excel_template_with_scope(
    input: &impl SchemaInput,
    out_dir: &Path,
    scope: Option<&str>,
) -> Result<()> {
    let ir = load_ir_with_scope(input, scope)?;
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

fn load_ir_with_scope(input: &impl SchemaInput, scope: Option<&str>) -> Result<ConfigIr> {
    let ir = load_ir(input)?;
    match scope {
        Some(scope) => filter_config_ir_by_scope(&ir, scope),
        None => Ok(ir),
    }
}

fn filter_ir_and_data_by_scope(
    ir: &ConfigIr,
    data: &sora_data::model::ConfigData,
    scope: Option<&str>,
) -> Result<(ConfigIr, sora_data::model::ConfigData)> {
    let Some(scope) = scope else {
        return Ok((ir.clone(), data.clone()));
    };
    let scoped_ir = filter_config_ir_by_scope(ir, scope)?;
    let scoped_data = sora_data::scope::filter_config_data_by_ir(&scoped_ir, data);
    sora_data::validate::validate_config_data(&scoped_ir, &scoped_data)?;
    Ok((scoped_ir, scoped_data))
}

fn load_validated_data(
    input: &impl ProjectInput,
    ir: &ConfigIr,
    execution: &ExecutionContext,
) -> Result<sora_data::model::ConfigData> {
    let data = input.load_data_with_context(ir, execution)?;
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
        generate_code(&input, CodegenTarget::C, &base.join("c")).unwrap();
        generate_code(&input, CodegenTarget::Cpp, &base.join("cpp")).unwrap();
        generate_code(&input, CodegenTarget::Erlang, &base.join("erlang")).unwrap();
        generate_code(&input, CodegenTarget::Python, &base.join("python")).unwrap();
        generate_code(&input, CodegenTarget::ProtoSchema, &base.join("proto")).unwrap();
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
}
