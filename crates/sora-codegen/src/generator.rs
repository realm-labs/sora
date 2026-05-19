use std::path::Path;

use sora_diagnostics::Result;
use sora_ir::model::ConfigIr;

use crate::{kotlin::KotlinCodeGenerator, rust::RustCodeGenerator, target::CodegenTarget};

pub trait CodeGenerator {
    fn generate(&self, ir: &ConfigIr, out_dir: &Path) -> Result<()>;
}

pub fn generator_for_target(target: CodegenTarget) -> Box<dyn CodeGenerator> {
    match target {
        CodegenTarget::Rust => Box::new(RustCodeGenerator),
        CodegenTarget::Kotlin => Box::new(KotlinCodeGenerator),
    }
}
