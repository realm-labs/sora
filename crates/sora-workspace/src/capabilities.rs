use sora_codegen::generator::{CodegenRegistry, runtime_format_name};

use crate::WorkspaceService;

impl WorkspaceService {
    /// Returns canonical built-in code generation target identifiers in stable order.
    pub fn supported_codegen_targets(&self) -> Vec<String> {
        CodegenRegistry::with_builtin_generators()
            .supported_targets()
            .into_iter()
            .map(str::to_owned)
            .collect()
    }

    /// Returns the runtime formats supported by a canonical target or one of its aliases.
    pub fn supported_runtime_formats(&self, target: &str) -> Vec<String> {
        CodegenRegistry::with_builtin_generators()
            .get(target)
            .map(|generator| {
                generator
                    .supported_runtime_formats()
                    .into_iter()
                    .map(runtime_format_name)
                    .map(str::to_owned)
                    .collect()
            })
            .unwrap_or_default()
    }
}
