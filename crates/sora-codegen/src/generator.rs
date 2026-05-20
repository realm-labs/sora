use std::path::Path;

use sora_diagnostics::Result;
use sora_ir::model::{ConfigIr, RuntimeFormatIr};

use crate::{
    csharp::CSharpCodeGenerator, go::GoCodeGenerator, java::JavaCodeGenerator,
    kotlin::KotlinCodeGenerator, lua::LuaCodeGenerator, rust::RustCodeGenerator,
    target::CodegenTarget,
};

pub trait CodeGenerator {
    fn generate(&self, ir: &ConfigIr, out_dir: &Path) -> Result<()>;
}

pub(crate) fn runtime_format_name(runtime_format: RuntimeFormatIr) -> &'static str {
    match runtime_format {
        RuntimeFormatIr::Sora => "sora",
        RuntimeFormatIr::Json => "json",
        RuntimeFormatIr::Protobuf => "protobuf",
        RuntimeFormatIr::Cbor => "cbor",
    }
}

pub fn generator_for_target(target: CodegenTarget) -> Box<dyn CodeGenerator> {
    match target {
        CodegenTarget::Rust => Box::new(RustCodeGenerator),
        CodegenTarget::Kotlin => Box::new(KotlinCodeGenerator),
        CodegenTarget::CSharp => Box::new(CSharpCodeGenerator),
        CodegenTarget::Java => Box::new(JavaCodeGenerator),
        CodegenTarget::Go => Box::new(GoCodeGenerator),
        CodegenTarget::Lua => Box::new(LuaCodeGenerator),
    }
}
