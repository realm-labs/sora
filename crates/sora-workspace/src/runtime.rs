use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::Result;
use sora_codegen::type_mapping::TypeMappingRegistry;
use sora_execution::{ExecutionContext, ExecutionOptions};
use sora_input::parser::ParserRegistry as CellParserRegistry;
use sora_ir::parser::ParserRegistry as SchemaParserRegistry;

use crate::{load_parser_registries, load_type_mapping_registry};

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
    execution: ExecutionContext,
    schema_parsers: Arc<SchemaParserRegistry>,
    cell_parsers: Arc<CellParserRegistry>,
    type_mappings: Arc<TypeMappingRegistry>,
}

impl ProjectRuntime {
    /// Loads project-declared and explicitly supplied runtime extensions.
    pub fn load(project: Option<&Path>, options: RuntimeOptions) -> Result<Self> {
        let execution = ExecutionContext::new(options.execution)?;
        let parsers = load_parser_registries(project, &options.parser_scripts)?;
        let type_mappings = load_type_mapping_registry(project, &options.type_mapping_scripts)?;
        Ok(Self {
            execution,
            schema_parsers: Arc::new(parsers.schema),
            cell_parsers: Arc::new(parsers.cell),
            type_mappings: Arc::new(type_mappings),
        })
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
}
