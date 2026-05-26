use std::{fs, path::Path};

use sora_diagnostics::{Result, SoraError};
use sora_execution::ExecutionContext;
use sora_input::traits::{ProjectInput, SchemaInput};
use sora_ir::{
    model::ConfigIr, normalize::normalize_schema_with_parsers,
    parser::ParserRegistry as SchemaParserRegistry, scope::filter_config_ir_by_scope,
    validate::validate_config_ir,
};
use sora_schema::model::SchemaFile;

pub(super) struct ValidatedProjectData {
    pub config: sora_data::model::ConfigData,
    pub locale_catalog: Option<sora_data::localization::LocaleCatalog>,
}

pub(super) fn validate_schema_ir(schema: SchemaFile) -> Result<ConfigIr> {
    validate_schema_ir_with_parsers(schema, &SchemaParserRegistry::builtin())
}

pub(super) fn validate_schema_ir_with_parsers(
    schema: SchemaFile,
    parser_registry: &SchemaParserRegistry,
) -> Result<ConfigIr> {
    let ir = normalize_schema_with_parsers(schema, parser_registry)?;
    validate_config_ir(&ir)?;
    Ok(ir)
}

pub(super) fn load_ir(input: &impl SchemaInput) -> Result<ConfigIr> {
    load_ir_with_parsers(input, &SchemaParserRegistry::builtin())
}

pub(super) fn load_ir_with_parsers(
    input: &impl SchemaInput,
    parser_registry: &SchemaParserRegistry,
) -> Result<ConfigIr> {
    validate_schema_ir_with_parsers(input.load_schema()?, parser_registry)
}

pub(super) fn load_ir_with_scope(
    input: &impl SchemaInput,
    scope: Option<&str>,
) -> Result<ConfigIr> {
    load_ir_with_scope_and_parsers(input, scope, &SchemaParserRegistry::builtin())
}

pub(super) fn load_ir_with_scope_and_parsers(
    input: &impl SchemaInput,
    scope: Option<&str>,
    parser_registry: &SchemaParserRegistry,
) -> Result<ConfigIr> {
    let ir = load_ir_with_parsers(input, parser_registry)?;
    match scope {
        Some(scope) => filter_config_ir_by_scope(&ir, scope),
        None => Ok(ir),
    }
}

pub(super) fn filter_ir_and_data_by_scope(
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

pub(super) fn load_validated_data(
    input: &impl ProjectInput,
    ir: &ConfigIr,
    execution: &ExecutionContext,
) -> Result<sora_data::model::ConfigData> {
    Ok(load_validated_project_data_with_parsers(
        input,
        ir,
        execution,
        sora_input::parser::builtin_registry(),
    )?
    .config)
}

pub(super) fn load_validated_data_with_parsers(
    input: &impl ProjectInput,
    ir: &ConfigIr,
    execution: &ExecutionContext,
    parser_registry: &sora_input::parser::ParserRegistry,
) -> Result<sora_data::model::ConfigData> {
    Ok(load_validated_project_data_with_parsers(input, ir, execution, parser_registry)?.config)
}

pub(super) fn load_validated_project_data_with_parsers(
    input: &impl ProjectInput,
    ir: &ConfigIr,
    execution: &ExecutionContext,
    parser_registry: &sora_input::parser::ParserRegistry,
) -> Result<ValidatedProjectData> {
    let data = input.load_data_with_context(ir, execution)?;
    let data = sora_input::defaults::materialize_defaults_with_parsers(ir, &data, parser_registry)?;
    let data = sora_data::derived::materialize_derived_fields(ir, &data)?;
    sora_data::validate::validate_config_data(ir, &data)?;
    let localization_data = input.load_localization_data_with_context(ir, execution)?;
    let locale_catalog =
        sora_data::localization::build_locale_catalog(ir, &data, &localization_data)?;
    Ok(ValidatedProjectData {
        config: data,
        locale_catalog,
    })
}

pub(super) fn write_json_file<T: serde::Serialize>(path: &Path, value: &T) -> Result<()> {
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
