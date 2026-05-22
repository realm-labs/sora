use std::{collections::BTreeMap, path::Path};

use serde::de::DeserializeOwned;
use serde_json::{Map, Value};
use sora_diagnostics::{Result, SoraError};
use sora_ir::model::ConfigIr;

use crate::{
    c::CCodeGenerator,
    cpp::CppCodeGenerator,
    csharp::CSharpCodeGenerator,
    dart::DartCodeGenerator,
    erlang::ErlangCodeGenerator,
    format::FormatterConfig,
    go::GoCodeGenerator,
    godot::GodotCodeGenerator,
    java::JavaCodeGenerator,
    javascript::JavaScriptCodeGenerator,
    kotlin::KotlinCodeGenerator,
    lua::LuaCodeGenerator,
    options::{
        CCodegenOptions, CppCodegenOptions, ErlangCodegenOptions, JavaScriptCodegenOptions,
        LanguageCodegenOptions, LuaCodegenOptions, RuntimeFormat, RustCodegenOptions,
        ScalaCodegenOptions, TypeScriptCodegenOptions, decode_options, runtime_format_from_options,
    },
    proto::ProtoCodeGenerator,
    python::PythonCodeGenerator,
    rust::RustCodeGenerator,
    scala::ScalaCodeGenerator,
    typescript::TypeScriptCodeGenerator,
};

pub trait CodeGenerator: Send + Sync {
    fn generate(&self, context: CodegenContext<'_>, out_dir: &Path) -> Result<()>;
}

#[derive(Debug, Clone, Copy)]
pub struct CodegenContext<'a> {
    pub target: &'a str,
    pub ir: &'a ConfigIr,
    pub options: &'a Value,
}

impl CodegenContext<'_> {
    pub fn options<T>(&self) -> Result<T>
    where
        T: DeserializeOwned + Default,
    {
        decode_options(self.target, self.options)
    }
}

pub struct CodegenRegistration {
    pub id: &'static str,
    pub aliases: &'static [&'static str],
    pub display_name: &'static str,
    pub runtime_capabilities: &'static [RuntimeCapability],
    pub runtime_format: fn(&str, &Value) -> Result<Option<RuntimeFormat>>,
    pub formatter: Option<FormatterConfig>,
    pub generator: Box<dyn CodeGenerator>,
}

impl CodegenRegistration {
    pub fn supports_runtime_format(&self, runtime_format: RuntimeFormat) -> bool {
        self.runtime_capability(runtime_format).is_some()
    }

    pub fn runtime_capability(&self, runtime_format: RuntimeFormat) -> Option<&RuntimeCapability> {
        self.runtime_capabilities
            .iter()
            .find(|capability| capability.format == runtime_format)
    }

    pub fn supported_runtime_formats(&self) -> Vec<RuntimeFormat> {
        self.runtime_capabilities
            .iter()
            .map(|capability| capability.format)
            .collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeCapability {
    pub format: RuntimeFormat,
    pub dependency: RuntimeDependency,
}

impl RuntimeCapability {
    pub const fn new(format: RuntimeFormat, dependency: RuntimeDependency) -> Self {
        Self { format, dependency }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeDependency {
    SelfContained,
    StandardLibrary,
    ManagedDependency,
    UserAdapter,
}

const SELF_CONTAINED: RuntimeDependency = RuntimeDependency::SelfContained;
const STANDARD_LIBRARY: RuntimeDependency = RuntimeDependency::StandardLibrary;
const MANAGED_DEPENDENCY: RuntimeDependency = RuntimeDependency::ManagedDependency;
const USER_ADAPTER: RuntimeDependency = RuntimeDependency::UserAdapter;

const fn capability(format: RuntimeFormat, dependency: RuntimeDependency) -> RuntimeCapability {
    RuntimeCapability::new(format, dependency)
}

const RUNTIME_SORA_ONLY: &[RuntimeCapability] = &[capability(RuntimeFormat::Sora, SELF_CONTAINED)];

const RUNTIME_JSON_ONLY: &[RuntimeCapability] =
    &[capability(RuntimeFormat::Json, STANDARD_LIBRARY)];

const RUNTIME_DART_EXPORTS: &[RuntimeCapability] = &[
    capability(RuntimeFormat::Json, STANDARD_LIBRARY),
    capability(RuntimeFormat::Cbor, USER_ADAPTER),
    capability(RuntimeFormat::SoraProtobuf, USER_ADAPTER),
];

const RUNTIME_MANAGED_EXPORTS: &[RuntimeCapability] = &[
    capability(RuntimeFormat::Sora, SELF_CONTAINED),
    capability(RuntimeFormat::Json, MANAGED_DEPENDENCY),
    capability(RuntimeFormat::Cbor, MANAGED_DEPENDENCY),
    capability(RuntimeFormat::SoraProtobuf, MANAGED_DEPENDENCY),
];

#[derive(Default)]
pub struct CodegenRegistry {
    generators: BTreeMap<String, CodegenRegistration>,
    aliases: BTreeMap<String, String>,
}

impl CodegenRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_builtin_generators() -> Self {
        let mut registry = Self::new();
        registry
            .register(CodegenRegistration {
                id: "rust",
                aliases: &["rs"],
                display_name: "Rust",
                runtime_capabilities: RUNTIME_MANAGED_EXPORTS,
                runtime_format: runtime_format_from_options::<RustCodegenOptions>,
                formatter: Some(FormatterConfig::new("Rust", "rustfmt", &[], &["rs"])),
                generator: Box::new(RustCodeGenerator),
            })
            .expect("built-in codegen target must be unique");
        registry
            .register(CodegenRegistration {
                id: "kotlin",
                aliases: &["kt"],
                display_name: "Kotlin",
                runtime_capabilities: RUNTIME_MANAGED_EXPORTS,
                runtime_format: runtime_format_from_options::<LanguageCodegenOptions>,
                formatter: None,
                generator: Box::new(KotlinCodeGenerator),
            })
            .expect("built-in codegen target must be unique");
        registry
            .register(CodegenRegistration {
                id: "csharp",
                aliases: &["cs"],
                display_name: "C#",
                runtime_capabilities: RUNTIME_MANAGED_EXPORTS,
                runtime_format: runtime_format_from_options::<LanguageCodegenOptions>,
                formatter: None,
                generator: Box::new(CSharpCodeGenerator),
            })
            .expect("built-in codegen target must be unique");
        registry
            .register(CodegenRegistration {
                id: "java",
                aliases: &[],
                display_name: "Java",
                runtime_capabilities: RUNTIME_MANAGED_EXPORTS,
                runtime_format: runtime_format_from_options::<LanguageCodegenOptions>,
                formatter: None,
                generator: Box::new(JavaCodeGenerator),
            })
            .expect("built-in codegen target must be unique");
        registry
            .register(CodegenRegistration {
                id: "scala",
                aliases: &[],
                display_name: "Scala",
                runtime_capabilities: RUNTIME_MANAGED_EXPORTS,
                runtime_format: runtime_format_from_options::<ScalaCodegenOptions>,
                formatter: Some(FormatterConfig::new("Scala", "scalafmt", &[], &["scala"])),
                generator: Box::new(ScalaCodeGenerator),
            })
            .expect("built-in codegen target must be unique");
        registry
            .register(CodegenRegistration {
                id: "go",
                aliases: &[],
                display_name: "Go",
                runtime_capabilities: RUNTIME_MANAGED_EXPORTS,
                runtime_format: runtime_format_from_options::<LanguageCodegenOptions>,
                formatter: Some(FormatterConfig::new("Go", "gofmt", &["-w"], &["go"])),
                generator: Box::new(GoCodeGenerator),
            })
            .expect("built-in codegen target must be unique");
        registry
            .register(CodegenRegistration {
                id: "dart",
                aliases: &[],
                display_name: "Dart",
                runtime_capabilities: RUNTIME_DART_EXPORTS,
                runtime_format: runtime_format_from_options::<LanguageCodegenOptions>,
                formatter: None,
                generator: Box::new(DartCodeGenerator),
            })
            .expect("built-in codegen target must be unique");
        registry
            .register(CodegenRegistration {
                id: "godot",
                aliases: &["gdscript"],
                display_name: "Godot",
                runtime_capabilities: RUNTIME_JSON_ONLY,
                runtime_format: runtime_format_from_options::<LanguageCodegenOptions>,
                formatter: None,
                generator: Box::new(GodotCodeGenerator),
            })
            .expect("built-in codegen target must be unique");
        registry
            .register(CodegenRegistration {
                id: "c",
                aliases: &[],
                display_name: "C",
                runtime_capabilities: RUNTIME_SORA_ONLY,
                runtime_format: runtime_format_from_options::<CCodegenOptions>,
                formatter: Some(FormatterConfig::new(
                    "C",
                    "clang-format",
                    &["-i"],
                    &["h", "c"],
                )),
                generator: Box::new(CCodeGenerator),
            })
            .expect("built-in codegen target must be unique");
        registry
            .register(CodegenRegistration {
                id: "cpp",
                aliases: &["c++"],
                display_name: "C++",
                runtime_capabilities: RUNTIME_SORA_ONLY,
                runtime_format: runtime_format_from_options::<CppCodegenOptions>,
                formatter: Some(FormatterConfig::new(
                    "C++",
                    "clang-format",
                    &["-i"],
                    &["hpp"],
                )),
                generator: Box::new(CppCodeGenerator),
            })
            .expect("built-in codegen target must be unique");
        registry
            .register(CodegenRegistration {
                id: "typescript",
                aliases: &["ts"],
                display_name: "TypeScript",
                runtime_capabilities: RUNTIME_MANAGED_EXPORTS,
                runtime_format: runtime_format_from_options::<TypeScriptCodegenOptions>,
                formatter: None,
                generator: Box::new(TypeScriptCodeGenerator),
            })
            .expect("built-in codegen target must be unique");
        registry
            .register(CodegenRegistration {
                id: "javascript",
                aliases: &["js"],
                display_name: "JavaScript",
                runtime_capabilities: RUNTIME_MANAGED_EXPORTS,
                runtime_format: runtime_format_from_options::<JavaScriptCodegenOptions>,
                formatter: None,
                generator: Box::new(JavaScriptCodeGenerator),
            })
            .expect("built-in codegen target must be unique");
        registry
            .register(CodegenRegistration {
                id: "erlang",
                aliases: &["erl"],
                display_name: "Erlang",
                runtime_capabilities: RUNTIME_SORA_ONLY,
                runtime_format: runtime_format_from_options::<ErlangCodegenOptions>,
                formatter: Some(FormatterConfig::new("Erlang", "erlfmt", &["-w"], &["erl"])),
                generator: Box::new(ErlangCodeGenerator),
            })
            .expect("built-in codegen target must be unique");
        registry
            .register(CodegenRegistration {
                id: "lua",
                aliases: &[],
                display_name: "Lua",
                runtime_capabilities: RUNTIME_SORA_ONLY,
                runtime_format: runtime_format_from_options::<LuaCodegenOptions>,
                formatter: None,
                generator: Box::new(LuaCodeGenerator),
            })
            .expect("built-in codegen target must be unique");
        registry
            .register(CodegenRegistration {
                id: "proto-schema",
                aliases: &["protobuf-schema"],
                display_name: "Proto schema",
                runtime_capabilities: &[],
                runtime_format: |_, _| Ok(None),
                formatter: None,
                generator: Box::new(ProtoCodeGenerator),
            })
            .expect("built-in codegen target must be unique");
        registry
            .register(CodegenRegistration {
                id: "python",
                aliases: &["py"],
                display_name: "Python",
                runtime_capabilities: RUNTIME_MANAGED_EXPORTS,
                runtime_format: runtime_format_from_options::<LanguageCodegenOptions>,
                formatter: Some(FormatterConfig::new(
                    "Python",
                    "black",
                    &["--quiet"],
                    &["py"],
                )),
                generator: Box::new(PythonCodeGenerator),
            })
            .expect("built-in codegen target must be unique");
        registry
    }

    pub fn register(&mut self, registration: CodegenRegistration) -> Result<()> {
        if registration.id.is_empty() {
            return Err(SoraError::InvalidSchema(
                "codegen target id cannot be empty".to_owned(),
            ));
        }
        if self.generators.contains_key(registration.id)
            || self.aliases.contains_key(registration.id)
        {
            return Err(SoraError::InvalidSchema(format!(
                "codegen target `{}` is already registered",
                registration.id
            )));
        }
        for alias in registration.aliases {
            if alias.is_empty() {
                return Err(SoraError::InvalidSchema(format!(
                    "codegen target `{}` declares an empty alias",
                    registration.id
                )));
            }
            if self.generators.contains_key(*alias) || self.aliases.contains_key(*alias) {
                return Err(SoraError::InvalidSchema(format!(
                    "codegen target alias `{}` is already registered",
                    alias
                )));
            }
        }

        let id = registration.id.to_owned();
        for alias in registration.aliases {
            self.aliases.insert((*alias).to_owned(), id.clone());
        }
        self.generators.insert(id, registration);
        Ok(())
    }

    pub fn get(&self, target: &str) -> Option<&CodegenRegistration> {
        self.canonical_id(target)
            .and_then(|canonical| self.generators.get(canonical))
    }

    pub fn canonical_id(&self, target: &str) -> Option<&str> {
        self.generators
            .get_key_value(target)
            .map(|(canonical, _)| canonical.as_str())
            .or_else(|| self.aliases.get(target).map(String::as_str))
    }

    pub fn supported_targets(&self) -> Vec<&str> {
        self.generators.keys().map(String::as_str).collect()
    }
}

pub(crate) fn ensure_sora_runtime_format(
    language: &'static str,
    runtime_format: RuntimeFormat,
) -> Result<()> {
    if runtime_format == RuntimeFormat::Sora {
        return Ok(());
    }

    Err(SoraError::InvalidSchema(format!(
        "{language} codegen runtime_format `{}` is not implemented yet; supported runtime_format: sora",
        runtime_format_name(runtime_format)
    )))
}

pub fn runtime_format_name(runtime_format: RuntimeFormat) -> &'static str {
    match runtime_format {
        RuntimeFormat::Sora => "sora",
        RuntimeFormat::Json => "json",
        RuntimeFormat::SoraProtobuf => "sora-protobuf",
        RuntimeFormat::Cbor => "cbor",
    }
}

pub fn empty_options() -> Value {
    Value::Object(Map::new())
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_test_codegen_generate {
    ($generator:ty, $target:expr) => {
        #[cfg(test)]
        #[allow(dead_code)]
        impl $generator {
            pub(crate) fn generate(
                &self,
                ir: &sora_ir::model::ConfigIr,
                out_dir: &std::path::Path,
            ) -> sora_diagnostics::Result<()> {
                let options = $crate::generator::empty_options();
                <$generator as $crate::generator::CodeGenerator>::generate(
                    self,
                    $crate::generator::CodegenContext {
                        target: $target,
                        ir,
                        options: &options,
                    },
                    out_dir,
                )
            }

            pub(crate) fn generate_with_options<T>(
                &self,
                ir: &sora_ir::model::ConfigIr,
                options: T,
                out_dir: &std::path::Path,
            ) -> sora_diagnostics::Result<()>
            where
                T: serde::Serialize,
            {
                let options = serde_json::to_value(options)
                    .map_err(sora_diagnostics::SoraError::SerializeData)?;
                <$generator as $crate::generator::CodeGenerator>::generate(
                    self,
                    $crate::generator::CodegenContext {
                        target: $target,
                        ir,
                        options: &options,
                    },
                    out_dir,
                )
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    struct NoopGenerator;

    impl CodeGenerator for NoopGenerator {
        fn generate(&self, _context: CodegenContext<'_>, _out_dir: &Path) -> Result<()> {
            Ok(())
        }
    }

    fn registration(id: &'static str, aliases: &'static [&'static str]) -> CodegenRegistration {
        CodegenRegistration {
            id,
            aliases,
            display_name: id,
            runtime_capabilities: &[],
            runtime_format: |_, _| Ok(None),
            formatter: None,
            generator: Box::new(NoopGenerator),
        }
    }

    #[test]
    fn resolves_builtin_aliases_to_canonical_targets() {
        let registry = CodegenRegistry::with_builtin_generators();

        assert_eq!(registry.canonical_id("rs"), Some("rust"));
        assert_eq!(registry.canonical_id("rust"), Some("rust"));
        assert!(registry.get("py").is_some());
    }

    #[test]
    fn rejects_duplicate_target_ids_and_aliases() {
        let mut registry = CodegenRegistry::new();
        registry
            .register(registration("custom", &["mine"]))
            .unwrap();

        assert!(registry.register(registration("custom", &[])).is_err());
        assert!(registry.register(registration("other", &["mine"])).is_err());
        assert!(registry.register(registration("mine", &[])).is_err());
    }
}
