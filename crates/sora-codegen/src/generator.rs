use std::path::Path;

use sora_diagnostics::{Result, SoraError};
use sora_ir::model::{ConfigIr, RuntimeFormatIr};

use crate::{
    c::CCodeGenerator, cpp::CppCodeGenerator, csharp::CSharpCodeGenerator, dart::DartCodeGenerator,
    erlang::ErlangCodeGenerator, go::GoCodeGenerator, godot::GodotCodeGenerator,
    java::JavaCodeGenerator, javascript::JavaScriptCodeGenerator, kotlin::KotlinCodeGenerator,
    lua::LuaCodeGenerator, proto::ProtoCodeGenerator, python::PythonCodeGenerator,
    rust::RustCodeGenerator, scala::ScalaCodeGenerator, target::CodegenTarget,
    typescript::TypeScriptCodeGenerator,
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

pub fn runtime_format_for_target(ir: &ConfigIr, target: CodegenTarget) -> Option<RuntimeFormatIr> {
    Some(match target {
        CodegenTarget::Rust => ir.codegen.rust.runtime_format,
        CodegenTarget::Kotlin => ir.codegen.kotlin.runtime_format,
        CodegenTarget::CSharp => ir.codegen.csharp.runtime_format,
        CodegenTarget::Java => ir.codegen.java.runtime_format,
        CodegenTarget::Scala => ir.codegen.scala.runtime_format,
        CodegenTarget::Go => ir.codegen.go.runtime_format,
        CodegenTarget::Dart => ir.codegen.dart.runtime_format,
        CodegenTarget::Godot => ir.codegen.godot.runtime_format,
        CodegenTarget::C => ir.codegen.c.runtime_format,
        CodegenTarget::Cpp => ir.codegen.cpp.runtime_format,
        CodegenTarget::TypeScript => ir.codegen.typescript.runtime_format,
        CodegenTarget::JavaScript => ir.codegen.javascript.runtime_format,
        CodegenTarget::Erlang => ir.codegen.erlang.runtime_format,
        CodegenTarget::Lua => ir.codegen.lua.runtime_format,
        CodegenTarget::Python => ir.codegen.python.runtime_format,
        CodegenTarget::ProtoSchema => return None,
    })
}

pub fn supported_runtime_formats(target: CodegenTarget) -> &'static [RuntimeFormatIr] {
    use RuntimeFormatIr::{Cbor, Json, Sora, SoraProtobuf};

    match target {
        CodegenTarget::Rust
        | CodegenTarget::Kotlin
        | CodegenTarget::CSharp
        | CodegenTarget::Java
        | CodegenTarget::Go
        | CodegenTarget::TypeScript
        | CodegenTarget::JavaScript
        | CodegenTarget::Python => &[Sora, Json, Cbor, SoraProtobuf],
        CodegenTarget::Dart | CodegenTarget::Godot => &[Json],
        CodegenTarget::Scala
        | CodegenTarget::C
        | CodegenTarget::Cpp
        | CodegenTarget::Erlang
        | CodegenTarget::Lua => &[Sora],
        CodegenTarget::ProtoSchema => &[],
    }
}

pub fn runtime_format_supported(target: CodegenTarget, runtime_format: RuntimeFormatIr) -> bool {
    supported_runtime_formats(target).contains(&runtime_format)
}

pub fn runtime_format_name(runtime_format: RuntimeFormatIr) -> &'static str {
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
        CodegenTarget::Scala => Box::new(ScalaCodeGenerator),
        CodegenTarget::Go => Box::new(GoCodeGenerator),
        CodegenTarget::Dart => Box::new(DartCodeGenerator),
        CodegenTarget::Godot => Box::new(GodotCodeGenerator),
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
