use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::Result;
use serde_json::to_value;
use sora_codegen::type_mapping::TypeMappingRegistry;
use sora_execution::{ExecutionContext, ExecutionOptions};
use sora_input::{
    parser::ParserRegistry as CellParserRegistry,
    source::{DataSourceDependency, DataSourceRegistry},
};
use sora_ir::parser::ParserRegistry as SchemaParserRegistry;

use crate::{
    ProjectManifest,
    lua_source_loader::register_lua_source_loaders_from_sources,
    parser::load_parser_registries_from_sources,
    project::capture_manifest,
    scripts::{ScriptSources, capture_project_scripts},
    source::builtin_source_registry,
    type_mapping::load_type_mapping_registry_from_sources,
};

/// Runtime extensions and execution policy shared by all project adapters.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RuntimeOptions {
    pub execution: ExecutionOptions,
    pub parser_scripts: Vec<PathBuf>,
    pub type_mapping_scripts: Vec<PathBuf>,
}

/// Immutable registries and execution context for one project invocation.
#[derive(Clone)]
pub struct ProjectRuntime {
    manifest: Option<Arc<ProjectManifest>>,
    execution: ExecutionContext,
    schema_parsers: Arc<SchemaParserRegistry>,
    cell_parsers: Arc<CellParserRegistry>,
    type_mappings: Arc<TypeMappingRegistry>,
    source_registry: Arc<DataSourceRegistry>,
    scripts: Arc<ScriptSources>,
    options: RuntimeOptions,
}

impl ProjectRuntime {
    /// Loads project-declared and explicitly supplied runtime extensions.
    pub fn load(project: Option<&Path>, options: RuntimeOptions) -> Result<Self> {
        let (manifest, scripts) = match project {
            Some(path) => {
                let (manifest, _) = capture_manifest(path)?;
                let scripts = capture_project_scripts(path, &manifest, None)?.0;
                (Some(manifest), scripts)
            }
            None => (None, ScriptSources::default()),
        };
        let scripts =
            scripts.with_extras(&options.parser_scripts, &options.type_mapping_scripts)?;
        Self::load_from_sources(manifest, scripts, options)
    }

    pub(crate) fn load_from_sources(
        manifest: Option<ProjectManifest>,
        scripts: ScriptSources,
        options: RuntimeOptions,
    ) -> Result<Self> {
        let execution = ExecutionContext::new(options.execution)?;
        let parsers = load_parser_registries_from_sources(&scripts.parsers)?;
        let type_mappings = load_type_mapping_registry_from_sources(&scripts.type_mappings)?;
        let mut source_registry = builtin_source_registry();
        register_lua_source_loaders_from_sources(&mut source_registry, &scripts.source_loaders)?;
        Ok(Self {
            manifest: manifest.map(Arc::new),
            execution,
            schema_parsers: Arc::new(parsers.schema),
            cell_parsers: Arc::new(parsers.cell),
            type_mappings: Arc::new(type_mappings),
            source_registry: Arc::new(source_registry),
            scripts: Arc::new(scripts),
            options,
        })
    }

    /// Compares actual loaded inputs, never a fresh digest of the session's paths.
    pub(crate) fn matches_snapshot(
        &self,
        manifest: &ProjectManifest,
        scripts: &ScriptSources,
        options: &RuntimeOptions,
    ) -> Result<bool> {
        Ok(self.scripts.as_ref() == scripts
            && &self.options == options
            && self.manifest.as_deref().map(to_value).transpose()? == Some(to_value(manifest)?))
    }

    pub(crate) fn scripts(&self) -> &ScriptSources {
        &self.scripts
    }

    pub fn manifest(&self) -> Option<&ProjectManifest> {
        self.manifest.as_deref()
    }

    pub fn execution(&self) -> &ExecutionContext {
        &self.execution
    }

    pub fn schema_parsers(&self) -> &Arc<SchemaParserRegistry> {
        &self.schema_parsers
    }

    pub fn cell_parsers(&self) -> &Arc<CellParserRegistry> {
        &self.cell_parsers
    }

    pub fn type_mappings(&self) -> &Arc<TypeMappingRegistry> {
        &self.type_mappings
    }

    pub fn source_registry(&self) -> &Arc<DataSourceRegistry> {
        &self.source_registry
    }

    pub fn source_dependencies(&self) -> Vec<DataSourceDependency> {
        self.source_registry.dependencies()
    }
}
