use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::{Result, bail};
use sora_codegen::type_mapping::TypeMappingRegistry;
use sora_execution::{ExecutionContext, ExecutionOptions};
use sora_input::parser::ParserRegistry as CellParserRegistry;
use sora_input::source::{DataSourceDependency, DataSourceRegistry};
use sora_ir::parser::ParserRegistry as SchemaParserRegistry;

use crate::{
    ProjectManifest, load_parser_registries, load_type_mapping_registry,
    lua_source_loader::register_lua_source_loaders, source::builtin_source_registry,
};

/// Runtime extensions and execution policy shared by all project adapters.
#[derive(Debug, Clone, Default)]
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
}

impl ProjectRuntime {
    /// Loads project-declared and explicitly supplied runtime extensions.
    pub fn load(project: Option<&Path>, options: RuntimeOptions) -> Result<Self> {
        let manifest = project.map(ProjectManifest::load).transpose()?;
        Self::load_with_manifest(project, manifest, options)
    }

    pub(crate) fn load_with_manifest(
        project: Option<&Path>,
        manifest: Option<ProjectManifest>,
        options: RuntimeOptions,
    ) -> Result<Self> {
        let execution = ExecutionContext::new(options.execution)?;
        let project_dir = project
            .and_then(Path::parent)
            .unwrap_or_else(|| Path::new("."));
        let mut parser_scripts = manifest
            .as_ref()
            .map(|manifest| resolve_scripts(project_dir, &manifest.parsers.scripts))
            .unwrap_or_default();
        parser_scripts.extend(options.parser_scripts);
        let mut type_mapping_scripts = manifest
            .as_ref()
            .map(|manifest| resolve_scripts(project_dir, &manifest.type_mappings.scripts))
            .unwrap_or_default();
        type_mapping_scripts.extend(options.type_mapping_scripts);
        let parsers = load_parser_registries(&parser_scripts)?;
        let type_mappings = load_type_mapping_registry(&type_mapping_scripts)?;
        let source_loader_scripts = manifest
            .as_ref()
            .map(|manifest| {
                resolve_source_loader_scripts(project_dir, &manifest.source_loaders.scripts)
            })
            .transpose()?
            .unwrap_or_default();
        let mut source_registry = builtin_source_registry();
        register_lua_source_loaders(&mut source_registry, &source_loader_scripts)?;
        Ok(Self {
            manifest: manifest.map(Arc::new),
            execution,
            schema_parsers: Arc::new(parsers.schema),
            cell_parsers: Arc::new(parsers.cell),
            type_mappings: Arc::new(type_mappings),
            source_registry: Arc::new(source_registry),
        })
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

fn resolve_scripts(project_dir: &Path, scripts: &[PathBuf]) -> Vec<PathBuf> {
    scripts
        .iter()
        .map(|path| {
            if path.is_absolute() {
                path.clone()
            } else {
                project_dir.join(path)
            }
        })
        .collect()
}

fn resolve_source_loader_scripts(
    project_dir: &Path,
    scripts: &[PathBuf],
) -> Result<Vec<(PathBuf, PathBuf)>> {
    scripts
        .iter()
        .map(|declared| {
            if declared.is_absolute()
                || declared.components().any(|component| {
                    !matches!(
                        component,
                        std::path::Component::Normal(_) | std::path::Component::CurDir
                    )
                })
            {
                bail!(
                    "Lua source loader script paths must stay relative to the project: `{}`",
                    declared.display()
                );
            }
            Ok((declared.clone(), project_dir.join(declared)))
        })
        .collect()
}
