use std::{fs, path::Path};

use sora_diagnostics::{Result, SoraError};
use sora_execution::ExecutionContext;
use sora_input::traits::{ProjectInput, SchemaInput};
use sora_ir::{
    model::ConfigIr, normalize::normalize_schema, scope::filter_config_ir_by_scope,
    validate::validate_config_ir,
};
use sora_schema::model::SchemaFile;

pub(super) fn validate_schema_ir(schema: SchemaFile) -> Result<ConfigIr> {
    let ir = normalize_schema(schema)?;
    validate_config_ir(&ir)?;
    Ok(ir)
}

pub(super) fn load_ir(input: &impl SchemaInput) -> Result<ConfigIr> {
    validate_schema_ir(input.load_schema()?)
}

pub(super) fn load_ir_with_scope(
    input: &impl SchemaInput,
    scope: Option<&str>,
) -> Result<ConfigIr> {
    let ir = load_ir(input)?;
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
    let data = input.load_data_with_context(ir, execution)?;
    let data = sora_input::defaults::materialize_defaults(ir, &data)?;
    let data = sora_data::derived::materialize_derived_fields(ir, &data)?;
    sora_data::validate::validate_config_data(ir, &data)?;
    Ok(data)
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
