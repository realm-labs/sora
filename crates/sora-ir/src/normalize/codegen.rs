use sora_schema::model::{
    CStandardSchema, CodegenSchema, CppStandardSchema, EnumReprSchema, ErlangCodegenSchema,
    ErlangEnumReprSchema, JavaScriptCodegenSchema, LanguageCodegenSchema, LuaCodegenSchema,
    LuaEnumReprSchema, LuaVersionSchema, RuntimeFormatSchema, RustMapTypeSchema,
    RustStringStorageSchema, ScalaCodegenSchema, ScalaVersionSchema, TypeScriptCodegenSchema,
};

use crate::model::{
    CCodegenIr, CStandardIr, CodegenIr, CppCodegenIr, CppStandardIr, EnumReprIr, ErlangCodegenIr,
    ErlangEnumReprIr, JavaScriptCodegenIr, LanguageCodegenIr, LuaCodegenIr, LuaEnumReprIr,
    LuaVersionIr, RuntimeFormatIr, RustCodegenIr, RustMapTypeIr, RustStringStorageIr,
    ScalaCodegenIr, ScalaVersionIr, TypeScriptCodegenIr,
};
impl From<CodegenSchema> for CodegenIr {
    fn from(value: CodegenSchema) -> Self {
        Self {
            rust: RustCodegenIr {
                runtime_format: RuntimeFormatIr::from(value.rust.runtime_format),
                map_type: match value.rust.map_type {
                    RustMapTypeSchema::Std => RustMapTypeIr::Std,
                    RustMapTypeSchema::FxHashMap => RustMapTypeIr::FxHashMap,
                },
                string_storage: match value.rust.string_storage {
                    RustStringStorageSchema::Owned => RustStringStorageIr::Owned,
                    RustStringStorageSchema::Arc => RustStringStorageIr::Arc,
                },
            },
            kotlin: LanguageCodegenIr::from(value.kotlin),
            csharp: LanguageCodegenIr::from(value.csharp),
            java: LanguageCodegenIr::from(value.java),
            scala: ScalaCodegenIr::from(value.scala),
            go: LanguageCodegenIr::from(value.go),
            dart: LanguageCodegenIr::from(value.dart),
            godot: LanguageCodegenIr::from(value.godot),
            c: CCodegenIr {
                runtime_format: RuntimeFormatIr::from(value.c.runtime_format),
                c_standard: CStandardIr::from(value.c.c_standard),
                prefix: value.c.prefix,
            },
            cpp: CppCodegenIr {
                runtime_format: RuntimeFormatIr::from(value.cpp.runtime_format),
                cpp_standard: CppStandardIr::from(value.cpp.cpp_standard),
                namespace: value.cpp.namespace,
            },
            typescript: TypeScriptCodegenIr::from(value.typescript),
            javascript: JavaScriptCodegenIr::from(value.javascript),
            erlang: ErlangCodegenIr::from(value.erlang),
            lua: LuaCodegenIr::from(value.lua),
            python: LanguageCodegenIr::from(value.python),
        }
    }
}

impl From<ScalaCodegenSchema> for ScalaCodegenIr {
    fn from(value: ScalaCodegenSchema) -> Self {
        Self {
            runtime_format: RuntimeFormatIr::from(value.runtime_format),
            scala_version: ScalaVersionIr::from(value.scala_version),
        }
    }
}

impl From<LanguageCodegenSchema> for LanguageCodegenIr {
    fn from(value: LanguageCodegenSchema) -> Self {
        Self {
            runtime_format: RuntimeFormatIr::from(value.runtime_format),
        }
    }
}

impl From<TypeScriptCodegenSchema> for TypeScriptCodegenIr {
    fn from(value: TypeScriptCodegenSchema) -> Self {
        Self {
            runtime_format: RuntimeFormatIr::from(value.runtime_format),
            enum_repr: EnumReprIr::from(value.enum_repr),
        }
    }
}

impl From<JavaScriptCodegenSchema> for JavaScriptCodegenIr {
    fn from(value: JavaScriptCodegenSchema) -> Self {
        Self {
            runtime_format: RuntimeFormatIr::from(value.runtime_format),
            enum_repr: EnumReprIr::from(value.enum_repr),
            emit_dts: value.emit_dts,
        }
    }
}

impl From<ErlangCodegenSchema> for ErlangCodegenIr {
    fn from(value: ErlangCodegenSchema) -> Self {
        Self {
            runtime_format: RuntimeFormatIr::from(value.runtime_format),
            enum_repr: ErlangEnumReprIr::from(value.enum_repr),
        }
    }
}

impl From<LuaCodegenSchema> for LuaCodegenIr {
    fn from(value: LuaCodegenSchema) -> Self {
        Self {
            runtime_format: RuntimeFormatIr::from(value.runtime_format),
            module: value.module,
            lua_version: LuaVersionIr::from(value.lua_version),
            enum_repr: LuaEnumReprIr::from(value.enum_repr),
        }
    }
}

impl From<RuntimeFormatSchema> for RuntimeFormatIr {
    fn from(value: RuntimeFormatSchema) -> Self {
        match value {
            RuntimeFormatSchema::Sora => Self::Sora,
            RuntimeFormatSchema::Json => Self::Json,
            RuntimeFormatSchema::SoraProtobuf => Self::SoraProtobuf,
            RuntimeFormatSchema::Cbor => Self::Cbor,
        }
    }
}

impl From<CStandardSchema> for CStandardIr {
    fn from(value: CStandardSchema) -> Self {
        match value {
            CStandardSchema::C99 => Self::C99,
            CStandardSchema::C11 => Self::C11,
            CStandardSchema::C17 => Self::C17,
            CStandardSchema::C23 => Self::C23,
        }
    }
}

impl From<CppStandardSchema> for CppStandardIr {
    fn from(value: CppStandardSchema) -> Self {
        match value {
            CppStandardSchema::Cpp11 => Self::Cpp11,
            CppStandardSchema::Cpp14 => Self::Cpp14,
            CppStandardSchema::Cpp17 => Self::Cpp17,
            CppStandardSchema::Cpp20 => Self::Cpp20,
            CppStandardSchema::Cpp23 => Self::Cpp23,
        }
    }
}

impl From<ScalaVersionSchema> for ScalaVersionIr {
    fn from(value: ScalaVersionSchema) -> Self {
        match value {
            ScalaVersionSchema::Scala212 => Self::Scala212,
            ScalaVersionSchema::Scala213 => Self::Scala213,
            ScalaVersionSchema::Scala3 => Self::Scala3,
        }
    }
}

impl From<LuaVersionSchema> for LuaVersionIr {
    fn from(value: LuaVersionSchema) -> Self {
        match value {
            LuaVersionSchema::Lua51 => Self::Lua51,
            LuaVersionSchema::Lua52 => Self::Lua52,
            LuaVersionSchema::Lua53 => Self::Lua53,
            LuaVersionSchema::Lua54 => Self::Lua54,
            LuaVersionSchema::LuaJit => Self::LuaJit,
        }
    }
}

impl From<LuaEnumReprSchema> for LuaEnumReprIr {
    fn from(value: LuaEnumReprSchema) -> Self {
        match value {
            LuaEnumReprSchema::Integer => Self::Integer,
            LuaEnumReprSchema::String => Self::String,
        }
    }
}

impl From<EnumReprSchema> for EnumReprIr {
    fn from(value: EnumReprSchema) -> Self {
        match value {
            EnumReprSchema::Integer => Self::Integer,
            EnumReprSchema::String => Self::String,
        }
    }
}

impl From<ErlangEnumReprSchema> for ErlangEnumReprIr {
    fn from(value: ErlangEnumReprSchema) -> Self {
        match value {
            ErlangEnumReprSchema::Integer => Self::Integer,
            ErlangEnumReprSchema::Atom => Self::Atom,
        }
    }
}
