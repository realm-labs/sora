use std::path::Path;

use crate::diff::{ConfigDiff, diff_config_data};
use crate::schema_lock::{read_schema_lock_file, verify_schema_lock, write_schema_lock_file};
use sora_codegen::{
    format::{FormatMode, format_generated_code},
    generator::{CodegenContext, CodegenRegistry, empty_options},
};
use sora_diagnostics::{Result, SoraError};
use sora_excel::generator::ExcelTemplateGenerator;
use sora_excel::sync::{ExcelSyncReport, ExcelTemplateSync};
use sora_execution::ExecutionContext;
use sora_export::{
    exporter::{ExportOptions, ExportOutput, ExportRequest, OutputKind},
    registry::ExporterRegistry,
};
use sora_input::parser::ParserRegistry as CellParserRegistry;
use sora_input::traits::{ProjectInput, SchemaInput};
use sora_ir::parser::ParserRegistry as SchemaParserRegistry;

mod support;

use support::{
    filter_ir_and_data_by_scope, load_ir, load_ir_with_scope, load_validated_data,
    validate_schema_ir, write_json_file,
};

pub fn load_schema_ir(input: &impl SchemaInput) -> Result<sora_ir::model::ConfigIr> {
    load_ir(input)
}

pub fn load_schema_ir_with_parsers(
    input: &impl SchemaInput,
    parser_registry: &SchemaParserRegistry,
) -> Result<sora_ir::model::ConfigIr> {
    support::load_ir_with_parsers(input, parser_registry)
}

pub fn load_schema_ir_with_scope(
    input: &impl SchemaInput,
    scope: Option<&str>,
) -> Result<sora_ir::model::ConfigIr> {
    load_ir_with_scope(input, scope)
}

pub fn load_schema_ir_with_scope_and_parsers(
    input: &impl SchemaInput,
    scope: Option<&str>,
    parser_registry: &SchemaParserRegistry,
) -> Result<sora_ir::model::ConfigIr> {
    support::load_ir_with_scope_and_parsers(input, scope, parser_registry)
}

pub fn load_project_data_with_context(
    input: &impl ProjectInput,
    execution: &ExecutionContext,
) -> Result<(sora_ir::model::ConfigIr, sora_data::model::ConfigData)> {
    let ir = load_ir(input)?;
    let data = load_validated_data(input, &ir, execution)?;
    Ok((ir, data))
}

pub fn load_project_data_with_context_and_parsers(
    input: &impl ProjectInput,
    execution: &ExecutionContext,
    schema_parser_registry: &SchemaParserRegistry,
    cell_parser_registry: &CellParserRegistry,
) -> Result<(sora_ir::model::ConfigIr, sora_data::model::ConfigData)> {
    let ir = support::load_ir_with_parsers(input, schema_parser_registry)?;
    let data =
        support::load_validated_data_with_parsers(input, &ir, execution, cell_parser_registry)?;
    Ok((ir, data))
}

pub fn load_project_data_and_catalog_with_context_and_parsers(
    input: &impl ProjectInput,
    execution: &ExecutionContext,
    schema_parser_registry: &SchemaParserRegistry,
    cell_parser_registry: &CellParserRegistry,
) -> Result<(
    sora_ir::model::ConfigIr,
    sora_data::model::ConfigData,
    Option<sora_data::localization::LocaleCatalog>,
)> {
    let ir = support::load_ir_with_parsers(input, schema_parser_registry)?;
    let project_data = support::load_validated_project_data_with_parsers(
        input,
        &ir,
        execution,
        cell_parser_registry,
    )?;
    Ok((ir, project_data.config, project_data.locale_catalog))
}

pub fn check_schema(input: &impl SchemaInput) -> Result<()> {
    let _ = load_ir(input)?;
    Ok(())
}

pub fn check_schema_with_parsers(
    input: &impl SchemaInput,
    parser_registry: &SchemaParserRegistry,
) -> Result<()> {
    let _ = support::load_ir_with_parsers(input, parser_registry)?;
    Ok(())
}

pub fn check_schema_with_lock(input: &impl SchemaInput, lock_path: &Path) -> Result<()> {
    let ir = load_ir(input)?;
    let lock = read_schema_lock_file(lock_path)?;
    verify_schema_lock(&ir, &lock)
}

pub fn check_schema_with_lock_and_parsers(
    input: &impl SchemaInput,
    lock_path: &Path,
    parser_registry: &SchemaParserRegistry,
) -> Result<()> {
    let ir = support::load_ir_with_parsers(input, parser_registry)?;
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

pub fn generate_schema_lock_with_scope_and_parsers(
    input: &impl SchemaInput,
    path: &Path,
    scope: Option<&str>,
    parser_registry: &SchemaParserRegistry,
) -> Result<()> {
    let ir = support::load_ir_with_scope_and_parsers(input, scope, parser_registry)?;
    write_schema_lock_file(&ir, path)
}

pub fn generate_code(input: &impl SchemaInput, target: &str, out_dir: &Path) -> Result<()> {
    generate_code_with_format(input, target, out_dir, FormatMode::Never)
}

pub fn generate_code_with_format(
    input: &impl SchemaInput,
    target: &str,
    out_dir: &Path,
    format_mode: FormatMode,
) -> Result<()> {
    generate_code_with_scope_and_format(input, target, out_dir, format_mode, None)
}

pub fn generate_code_with_scope_and_format(
    input: &impl SchemaInput,
    target: &str,
    out_dir: &Path,
    format_mode: FormatMode,
    scope: Option<&str>,
) -> Result<()> {
    let registry = CodegenRegistry::with_builtin_generators();
    generate_code_with_registry_scope_and_format(
        input,
        target,
        out_dir,
        format_mode,
        scope,
        &registry,
    )
}

pub fn generate_code_with_scope_format_and_parsers(
    input: &impl SchemaInput,
    target: &str,
    out_dir: &Path,
    format_mode: FormatMode,
    scope: Option<&str>,
    parser_registry: &SchemaParserRegistry,
) -> Result<()> {
    let registry = CodegenRegistry::with_builtin_generators();
    generate_code_with_registry_scope_format_and_parsers(
        input,
        target,
        out_dir,
        format_mode,
        scope,
        &registry,
        parser_registry,
    )
}

pub fn generate_code_with_registry_scope_and_format(
    input: &impl SchemaInput,
    target: &str,
    out_dir: &Path,
    format_mode: FormatMode,
    scope: Option<&str>,
    registry: &CodegenRegistry,
) -> Result<()> {
    let schema = input.load_schema()?;
    let codegen_options = schema.codegen.clone();
    let ir = validate_schema_ir(schema)?;
    let ir = match scope {
        Some(scope) => sora_ir::scope::filter_config_ir_by_scope(&ir, scope)?,
        None => ir,
    };
    let generator = registry.get(target).ok_or_else(|| {
        SoraError::InvalidSchema(format!(
            "unknown codegen target `{}`; supported targets: {}",
            target,
            registry.supported_targets().join(", ")
        ))
    })?;
    let canonical_target = registry.canonical_id(target).unwrap_or(target);
    let empty = empty_options();
    let options = codegen_options
        .target_options(canonical_target)
        .or_else(|| codegen_options.target_options(target))
        .unwrap_or(&empty);
    generator.generator.generate(
        CodegenContext {
            target: canonical_target,
            ir: &ir,
            options,
        },
        out_dir,
    )?;
    format_generated_code(
        generator.display_name,
        generator.formatter,
        out_dir,
        format_mode,
    )
}

pub fn generate_code_with_registry_scope_format_and_parsers(
    input: &impl SchemaInput,
    target: &str,
    out_dir: &Path,
    format_mode: FormatMode,
    scope: Option<&str>,
    registry: &CodegenRegistry,
    parser_registry: &SchemaParserRegistry,
) -> Result<()> {
    let schema = input.load_schema()?;
    let codegen_options = schema.codegen.clone();
    let ir = support::validate_schema_ir_with_parsers(schema, parser_registry)?;
    let ir = match scope {
        Some(scope) => sora_ir::scope::filter_config_ir_by_scope(&ir, scope)?,
        None => ir,
    };
    let generator = registry.get(target).ok_or_else(|| {
        SoraError::InvalidSchema(format!(
            "unknown codegen target `{}`; supported targets: {}",
            target,
            registry.supported_targets().join(", ")
        ))
    })?;
    let canonical_target = registry.canonical_id(target).unwrap_or(target);
    let empty = empty_options();
    let options = codegen_options
        .target_options(canonical_target)
        .or_else(|| codegen_options.target_options(target))
        .unwrap_or(&empty);
    generator.generator.generate(
        CodegenContext {
            target: canonical_target,
            ir: &ir,
            options,
        },
        out_dir,
    )?;
    format_generated_code(
        generator.display_name,
        generator.formatter,
        out_dir,
        format_mode,
    )
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
    let project_data = support::load_validated_project_data_with_parsers(
        input,
        &ir,
        execution,
        sora_input::parser::builtin_registry(),
    )?;
    export_loaded_data(LoadedDataExportRequest {
        ir: &ir,
        data: &project_data.config,
        locale_catalog: project_data.locale_catalog.as_ref(),
        format,
        output,
        scope,
        execution,
        options: ExportOptions::default(),
    })
}

pub fn export_loaded_data_with_scope_and_context(
    ir: &sora_ir::model::ConfigIr,
    data: &sora_data::model::ConfigData,
    format: &str,
    output: ExportOutput,
    scope: Option<&str>,
    execution: &ExecutionContext,
) -> Result<()> {
    export_loaded_data_with_scope_context_and_options(
        ir,
        data,
        format,
        output,
        scope,
        execution,
        ExportOptions::default(),
    )
}

pub fn export_loaded_data_with_scope_context_and_options(
    ir: &sora_ir::model::ConfigIr,
    data: &sora_data::model::ConfigData,
    format: &str,
    output: ExportOutput,
    scope: Option<&str>,
    execution: &ExecutionContext,
    options: ExportOptions,
) -> Result<()> {
    export_loaded_data(LoadedDataExportRequest {
        ir,
        data,
        locale_catalog: None,
        format,
        output,
        scope,
        execution,
        options,
    })
}

pub struct LoadedDataExportRequest<'a> {
    pub ir: &'a sora_ir::model::ConfigIr,
    pub data: &'a sora_data::model::ConfigData,
    pub locale_catalog: Option<&'a sora_data::localization::LocaleCatalog>,
    pub format: &'a str,
    pub output: ExportOutput,
    pub scope: Option<&'a str>,
    pub execution: &'a ExecutionContext,
    pub options: ExportOptions,
}

pub fn export_loaded_data(request: LoadedDataExportRequest<'_>) -> Result<()> {
    let (ir, data) = filter_ir_and_data_by_scope(request.ir, request.data, request.scope)?;
    let locale_catalog = if request.format.starts_with("i18n-") {
        request.locale_catalog
    } else {
        None
    };
    let registry = ExporterRegistry::with_builtin_exporters();
    let exporter = registry
        .get(request.format)
        .ok_or_else(|| SoraError::UnknownExportFormat {
            format: request.format.to_owned(),
            supported: registry.supported_formats().join(", "),
        })?;

    exporter.export(ExportRequest {
        ir: &ir,
        data: &data,
        locale_catalog,
        execution: request.execution,
        options: request.options,
        output: request.output,
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

pub fn diff_data_with_scope_context_and_parsers(
    left: &impl ProjectInput,
    right: &impl ProjectInput,
    output_path: &Path,
    scope: Option<&str>,
    execution: &ExecutionContext,
    schema_parser_registry: &SchemaParserRegistry,
    cell_parser_registry: &CellParserRegistry,
) -> Result<ConfigDiff> {
    let ir = support::load_ir_with_parsers(left, schema_parser_registry)?;
    let left_data =
        support::load_validated_data_with_parsers(left, &ir, execution, cell_parser_registry)?;
    let right_data =
        support::load_validated_data_with_parsers(right, &ir, execution, cell_parser_registry)?;
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

pub fn generate_excel_template_with_scope_and_parsers(
    input: &impl SchemaInput,
    out_dir: &Path,
    scope: Option<&str>,
    parser_registry: &SchemaParserRegistry,
) -> Result<()> {
    let ir = support::load_ir_with_scope_and_parsers(input, scope, parser_registry)?;
    ExcelTemplateGenerator.generate(&ir, out_dir)
}

pub fn preview_excel_sync(
    input: &impl SchemaInput,
    data_root: &Path,
    scope: Option<&str>,
) -> Result<ExcelSyncReport> {
    let ir = load_ir_with_scope(input, scope)?;
    ExcelTemplateSync.preview(&ir, data_root)
}

pub fn preview_excel_sync_with_parsers(
    input: &impl SchemaInput,
    data_root: &Path,
    scope: Option<&str>,
    parser_registry: &SchemaParserRegistry,
) -> Result<ExcelSyncReport> {
    let ir = support::load_ir_with_scope_and_parsers(input, scope, parser_registry)?;
    ExcelTemplateSync.preview(&ir, data_root)
}

pub fn write_excel_sync(
    input: &impl SchemaInput,
    data_root: &Path,
    scope: Option<&str>,
) -> Result<ExcelSyncReport> {
    let ir = load_ir_with_scope(input, scope)?;
    ExcelTemplateSync.write(&ir, data_root)
}

pub fn write_excel_sync_with_parsers(
    input: &impl SchemaInput,
    data_root: &Path,
    scope: Option<&str>,
    parser_registry: &SchemaParserRegistry,
) -> Result<ExcelSyncReport> {
    let ir = support::load_ir_with_scope_and_parsers(input, scope, parser_registry)?;
    ExcelTemplateSync.write(&ir, data_root)
}

pub fn supported_export_formats() -> Vec<&'static str> {
    ExporterRegistry::with_builtin_exporters().supported_formats()
}

pub fn export_output_kind(format: &str) -> Option<OutputKind> {
    ExporterRegistry::with_builtin_exporters()
        .get(format)
        .map(|exporter| exporter.output_kind())
}

#[cfg(test)]
mod tests;
