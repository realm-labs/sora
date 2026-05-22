use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use sora_export::exporter::ExportOutput;
use sora_input_toml::{input::TomlSchemaInput, schema::load_project_schema_file};
use sora_input_xlsx::input::XlsxProjectInput;
use sora_ir::{model::ConfigIr, normalize::normalize_schema, validate::validate_config_ir};

mod fs_util;
mod rows;
mod workbook;

use fs_util::{clean_dir, clean_file, clean_xlsx_files};
use workbook::write_workbooks;

fn main() -> Result<()> {
    let root = showcase_root();
    let project = root.join("project.toml");
    let data_root = root.join("data");
    let generated_root = root.join("generated");
    let rust_generated = root.join("rust/src/generated");
    let kotlin_generated = root.join("kotlin/src/generated/kotlin");
    let csharp_generated = root.join("csharp/src/generated/csharp");
    let java_generated = root.join("java/src/generated/java");
    let scala_generated = root.join("scala/src/generated/scala");
    let go_generated = root.join("go/internal/showcase");
    let dart_generated = root.join("dart/lib/src/generated");
    let godot_generated = root.join("godot/addons/sora_config/generated");
    let c_generated = root.join("c/generated");
    let cpp_generated = root.join("cpp/generated");
    let python_generated = root.join("python/generated");
    let proto_generated = generated_root.join("proto");

    fs::create_dir_all(&data_root)
        .with_context(|| format!("failed to create `{}`", data_root.display()))?;
    fs::create_dir_all(&generated_root)
        .with_context(|| format!("failed to create `{}`", generated_root.display()))?;

    let ir = load_ir(&project)?;
    clean_xlsx_files(&data_root)?;
    write_workbooks(&ir, &data_root)?;

    let schema_input = TomlSchemaInput::new(&project);
    let project_input = XlsxProjectInput::new(TomlSchemaInput::new(&project), &data_root);

    clean_dir(&rust_generated)?;
    clean_dir(&kotlin_generated)?;
    clean_dir(&csharp_generated)?;
    clean_dir(&java_generated)?;
    clean_dir(&scala_generated)?;
    clean_dir(&go_generated)?;
    clean_dir(&dart_generated)?;
    clean_dir(&godot_generated)?;
    clean_dir(&c_generated)?;
    clean_dir(&cpp_generated)?;
    clean_dir(&python_generated)?;
    clean_dir(&proto_generated)?;
    clean_dir(&generated_root.join("debug-json"))?;
    clean_dir(&generated_root.join("client"))?;
    clean_dir(&generated_root.join("server"))?;
    clean_dir(&root.join("godot/config"))?;
    clean_file(&generated_root.join("config.json"))?;
    clean_file(&generated_root.join("config.sora.pb"))?;
    clean_file(&generated_root.join("config.pb"))?;
    clean_file(&generated_root.join("config.cbor"))?;

    sora_core::pipeline::check_schema(&schema_input)?;
    sora_core::pipeline::generate_schema_lock(&schema_input, &generated_root.join("schema.lock"))?;
    sora_core::pipeline::generate_code(&schema_input, "rust", &rust_generated)?;
    sora_core::pipeline::generate_code(&schema_input, "kotlin", &kotlin_generated)?;
    sora_core::pipeline::generate_code(&schema_input, "csharp", &csharp_generated)?;
    sora_core::pipeline::generate_code(&schema_input, "java", &java_generated)?;
    sora_core::pipeline::generate_code(&schema_input, "scala", &scala_generated)?;
    sora_core::pipeline::generate_code(&schema_input, "go", &go_generated)?;
    sora_core::pipeline::generate_code_with_scope_and_format(
        &schema_input,
        "dart",
        &dart_generated,
        sora_codegen::format::FormatMode::Never,
        Some("client"),
    )?;
    sora_core::pipeline::generate_code_with_scope_and_format(
        &schema_input,
        "godot",
        &godot_generated,
        sora_codegen::format::FormatMode::Never,
        Some("client"),
    )?;
    sora_core::pipeline::generate_code(&schema_input, "c", &c_generated)?;
    sora_core::pipeline::generate_code(&schema_input, "cpp", &cpp_generated)?;
    sora_core::pipeline::generate_code(&schema_input, "python", &python_generated)?;
    sora_core::pipeline::generate_code(&schema_input, "proto-schema", &proto_generated)?;
    sora_core::pipeline::export_data(
        &project_input,
        "binary",
        ExportOutput::File(generated_root.join("config.sora")),
    )?;
    sora_core::pipeline::export_data(
        &project_input,
        "json",
        ExportOutput::File(generated_root.join("config.json")),
    )?;
    sora_core::pipeline::export_data_with_scope(
        &project_input,
        "json",
        ExportOutput::File(generated_root.join("client/config.json")),
        Some("client"),
    )?;
    sora_core::pipeline::export_data_with_scope(
        &project_input,
        "json",
        ExportOutput::File(root.join("godot/config/config.json")),
        Some("client"),
    )?;
    sora_core::pipeline::export_data_with_scope(
        &project_input,
        "json",
        ExportOutput::File(generated_root.join("server/config.json")),
        Some("server"),
    )?;
    sora_core::pipeline::export_data(
        &project_input,
        "sora-protobuf",
        ExportOutput::File(generated_root.join("config.sora.pb")),
    )?;
    sora_core::pipeline::export_data(
        &project_input,
        "proto",
        ExportOutput::File(generated_root.join("config.pb")),
    )?;
    sora_core::pipeline::export_data(
        &project_input,
        "cbor",
        ExportOutput::File(generated_root.join("config.cbor")),
    )?;
    sora_core::pipeline::export_data(
        &project_input,
        "json-debug",
        ExportOutput::Directory(generated_root.join("debug-json")),
    )?;

    println!("showcase generated under `{}`", root.display());
    Ok(())
}

fn load_ir(project: &Path) -> Result<ConfigIr> {
    let schema = load_project_schema_file(project)
        .with_context(|| format!("failed to load `{}`", project.display()))?;
    let ir = normalize_schema(schema)?;
    validate_config_ir(&ir)?;
    Ok(ir)
}

fn showcase_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("builder crate should live under examples/showcase/tools")
        .to_path_buf()
}
