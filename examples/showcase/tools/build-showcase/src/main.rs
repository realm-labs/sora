use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use sora_execution::ExecutionContext;
use sora_export::exporter::{ExportOptions, ExportOutput};
use sora_input_schema::{input::ProjectSchemaInput, schema::load_project_schema};
use sora_input_xlsx::input::XlsxProjectInput;
use sora_ir::{model::ConfigIr, normalize::normalize_schema, validate::validate_config_ir};

mod fs_util;
mod rows;
mod workbook;

use fs_util::{clean_dir, clean_file, clean_xlsx_files};
use workbook::write_workbooks;

fn main() -> Result<()> {
    let root = showcase_root();
    let project = root.join("project.scon");
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
    let typescript_generated = root.join("typescript/generated");
    let javascript_generated = root.join("javascript/generated");
    let erlang_generated = root.join("erlang/generated");
    let lua_generated = root.join("lua/generated");
    let python_generated = root.join("python/generated");
    let proto_generated = generated_root.join("proto");

    fs::create_dir_all(&data_root)
        .with_context(|| format!("failed to create `{}`", data_root.display()))?;
    fs::create_dir_all(&generated_root)
        .with_context(|| format!("failed to create `{}`", generated_root.display()))?;

    let ir = load_ir(&project)?;
    clean_xlsx_files(&data_root)?;
    write_workbooks(&ir, &data_root)?;

    let schema_input = ProjectSchemaInput::new(&project);
    let project_input = XlsxProjectInput::new(ProjectSchemaInput::new(&project), &data_root);

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
    clean_dir(&typescript_generated)?;
    clean_dir(&javascript_generated)?;
    clean_dir(&erlang_generated)?;
    clean_dir(&lua_generated)?;
    clean_dir(&python_generated)?;
    clean_dir(&proto_generated)?;
    clean_dir(&generated_root.join("debug-json"))?;
    clean_dir(&generated_root.join("i18n"))?;
    clean_dir(&generated_root.join("client"))?;
    clean_dir(&generated_root.join("server"))?;
    clean_dir(&root.join("godot/config"))?;
    clean_file(&generated_root.join("config.json"))?;
    clean_file(&generated_root.join("config.sora.pb"))?;
    clean_file(&generated_root.join("config.pb"))?;
    clean_file(&generated_root.join("config.cbor"))?;

    sora_core::pipeline::check_schema(&schema_input)?;
    sora_core::pipeline::generate_schema_lock_with_view(
        &schema_input,
        &generated_root.join("schema.lock"),
        Some("full"),
    )?;
    generate_code(&schema_input, "rust", &rust_generated, "full")?;
    generate_code(&schema_input, "kotlin", &kotlin_generated, "full")?;
    generate_code(&schema_input, "csharp", &csharp_generated, "full")?;
    generate_code(&schema_input, "java", &java_generated, "full")?;
    generate_code(&schema_input, "scala", &scala_generated, "full")?;
    generate_code(&schema_input, "go", &go_generated, "full")?;
    sora_core::pipeline::generate_code_with_view_and_format(
        &schema_input,
        "dart",
        &dart_generated,
        sora_codegen::format::FormatMode::Never,
        Some("client"),
    )?;
    sora_core::pipeline::generate_code_with_view_and_format(
        &schema_input,
        "godot",
        &godot_generated,
        sora_codegen::format::FormatMode::Never,
        Some("client"),
    )?;
    generate_code(&schema_input, "c", &c_generated, "full")?;
    generate_code(&schema_input, "cpp", &cpp_generated, "full")?;
    generate_code(&schema_input, "typescript", &typescript_generated, "full")?;
    generate_code(&schema_input, "javascript", &javascript_generated, "full")?;
    generate_code(&schema_input, "erlang", &erlang_generated, "full")?;
    generate_code(&schema_input, "lua", &lua_generated, "full")?;
    generate_code(&schema_input, "python", &python_generated, "full")?;
    generate_code(&schema_input, "proto-schema", &proto_generated, "full")?;
    sora_core::pipeline::export_data_with_view(
        &project_input,
        "binary",
        ExportOutput::File(generated_root.join("config.sora")),
        Some("full"),
    )?;
    sora_core::pipeline::export_data_with_view(
        &project_input,
        "json",
        ExportOutput::File(generated_root.join("config.json")),
        Some("full"),
    )?;
    sora_core::pipeline::export_data_with_view(
        &project_input,
        "json",
        ExportOutput::File(generated_root.join("client/config.json")),
        Some("client"),
    )?;
    sora_core::pipeline::export_data_with_view(
        &project_input,
        "json",
        ExportOutput::File(root.join("godot/config/config.json")),
        Some("client"),
    )?;
    sora_core::pipeline::export_data_with_view(
        &project_input,
        "json",
        ExportOutput::File(generated_root.join("server/config.json")),
        Some("server"),
    )?;
    sora_core::pipeline::export_data_with_view(
        &project_input,
        "sora-protobuf",
        ExportOutput::File(generated_root.join("config.sora.pb")),
        Some("full"),
    )?;
    sora_core::pipeline::export_data_with_view(
        &project_input,
        "proto",
        ExportOutput::File(generated_root.join("config.pb")),
        Some("full"),
    )?;
    sora_core::pipeline::export_data_with_view(
        &project_input,
        "cbor",
        ExportOutput::File(generated_root.join("config.cbor")),
        Some("full"),
    )?;
    sora_core::pipeline::export_data_with_view(
        &project_input,
        "json-debug",
        ExportOutput::Directory(generated_root.join("debug-json")),
        Some("full"),
    )?;
    generate_i18n_exports(&project_input, &generated_root)?;

    println!("showcase generated under `{}`", root.display());
    Ok(())
}

fn generate_i18n_exports(
    input: &XlsxProjectInput<ProjectSchemaInput>,
    generated_root: &Path,
) -> Result<()> {
    let execution = ExecutionContext::default();
    let schema_parsers = sora_ir::parser::ParserRegistry::default();
    let (ir, data, locale_catalog) =
        sora_core::pipeline::load_project_data_and_catalog_with_context_and_parsers(
            input,
            &execution,
            &schema_parsers,
            sora_input::parser::builtin_registry(),
        )?;
    for locale in ["zh_cn", "en_us"] {
        for (format, extension) in [("i18n-binary", "sora-i18n"), ("i18n-json", "json")] {
            sora_core::pipeline::export_loaded_data(
                sora_core::pipeline::LoadedDataExportRequest {
                    ir: &ir,
                    data: &data,
                    locale_catalog: locale_catalog.as_ref(),
                    format,
                    output: ExportOutput::File(
                        generated_root
                            .join("i18n")
                            .join(format!("{locale}.{extension}")),
                    ),
                    view: Some("full"),
                    execution: &execution,
                    options: ExportOptions {
                        locale: Some(locale.to_owned()),
                        ..ExportOptions::default()
                    },
                },
            )?;
        }
    }
    Ok(())
}

fn generate_code(input: &ProjectSchemaInput, target: &str, out: &Path, view: &str) -> Result<()> {
    let format = if matches!(target, "rust" | "go") {
        sora_codegen::format::FormatMode::Auto
    } else {
        sora_codegen::format::FormatMode::Never
    };
    sora_core::pipeline::generate_code_with_view_and_format(
        input,
        target,
        out,
        format,
        Some(view),
    )?;
    Ok(())
}

fn load_ir(project: &Path) -> Result<ConfigIr> {
    let schema = load_project_schema(project)
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
