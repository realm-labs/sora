use std::path::Path;

use sora_diagnostics::{Result, SoraError};
use sora_ir::model::{ConfigIr, RuntimeFormatIr};

use crate::{
    c::CCodeGenerator, cpp::CppCodeGenerator, csharp::CSharpCodeGenerator, dart::DartCodeGenerator,
    erlang::ErlangCodeGenerator, go::GoCodeGenerator, java::JavaCodeGenerator,
    javascript::JavaScriptCodeGenerator, kotlin::KotlinCodeGenerator, lua::LuaCodeGenerator,
    proto::ProtoCodeGenerator, python::PythonCodeGenerator, rust::RustCodeGenerator,
    target::CodegenTarget, typescript::TypeScriptCodeGenerator,
};

pub trait CodeGenerator {
    fn generate(&self, ir: &ConfigIr, out_dir: &Path) -> Result<()>;
}

pub(crate) fn ensure_sora_runtime_format(
    language: &'static str,
    runtime_format: RuntimeFormatIr,
) -> Result<()> {
    if runtime_format == RuntimeFormatIr::Sora {
        return Ok(());
    }

    Err(SoraError::InvalidSchema(format!(
        "{language} codegen runtime_format `{}` is not implemented yet; supported runtime_format: sora",
        runtime_format_name(runtime_format)
    )))
}

pub(crate) fn runtime_format_name(runtime_format: RuntimeFormatIr) -> &'static str {
    match runtime_format {
        RuntimeFormatIr::Sora => "sora",
        RuntimeFormatIr::Json => "json",
        RuntimeFormatIr::SoraProtobuf => "sora-protobuf",
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
        CodegenTarget::Dart => Box::new(DartCodeGenerator),
        CodegenTarget::C => Box::new(CCodeGenerator),
        CodegenTarget::Cpp => Box::new(CppCodeGenerator),
        CodegenTarget::TypeScript => Box::new(TypeScriptCodeGenerator),
        CodegenTarget::JavaScript => Box::new(JavaScriptCodeGenerator),
        CodegenTarget::Erlang => Box::new(ErlangCodeGenerator),
        CodegenTarget::Lua => Box::new(LuaCodeGenerator),
        CodegenTarget::ProtoSchema => Box::new(ProtoCodeGenerator),
        CodegenTarget::Python => Box::new(PythonCodeGenerator),
    }
}
