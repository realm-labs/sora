use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;
use sora_diagnostics::{Result, SoraError};

pub fn decode_options<T>(target: &str, raw: &Value) -> Result<T>
where
    T: DeserializeOwned + Default,
{
    serde_json::from_value(raw.clone()).map_err(|source| {
        SoraError::InvalidSchema(format!("invalid `{target}` codegen options: {source}"))
    })
}

pub trait HasRuntimeFormat {
    fn runtime_format(&self) -> Option<RuntimeFormat>;
}

pub fn runtime_format_from_options<T>(target: &str, raw: &Value) -> Result<Option<RuntimeFormat>>
where
    T: DeserializeOwned + Default + HasRuntimeFormat,
{
    Ok(decode_options::<T>(target, raw)?.runtime_format())
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default)]
pub struct RustCodegenOptions {
    pub runtime_format: RuntimeFormat,
    pub map_type: RustMapType,
    pub string_storage: RustStringStorage,
    pub datetime_type: RustDateTimeType,
}

impl Default for RustCodegenOptions {
    fn default() -> Self {
        Self {
            runtime_format: RuntimeFormat::Sora,
            map_type: RustMapType::Std,
            string_storage: RustStringStorage::Owned,
            datetime_type: RustDateTimeType::SystemTime,
        }
    }
}

impl HasRuntimeFormat for RustCodegenOptions {
    fn runtime_format(&self) -> Option<RuntimeFormat> {
        Some(self.runtime_format)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default)]
pub struct LanguageCodegenOptions {
    pub runtime_format: RuntimeFormat,
}

impl Default for LanguageCodegenOptions {
    fn default() -> Self {
        Self {
            runtime_format: RuntimeFormat::Sora,
        }
    }
}

impl HasRuntimeFormat for LanguageCodegenOptions {
    fn runtime_format(&self) -> Option<RuntimeFormat> {
        Some(self.runtime_format)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default)]
pub struct CCodegenOptions {
    pub runtime_format: RuntimeFormat,
    pub c_standard: CStandard,
    pub prefix: Option<String>,
}

impl Default for CCodegenOptions {
    fn default() -> Self {
        Self {
            runtime_format: RuntimeFormat::Sora,
            c_standard: CStandard::C11,
            prefix: None,
        }
    }
}

impl HasRuntimeFormat for CCodegenOptions {
    fn runtime_format(&self) -> Option<RuntimeFormat> {
        Some(self.runtime_format)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default)]
pub struct CppCodegenOptions {
    pub runtime_format: RuntimeFormat,
    pub cpp_standard: CppStandard,
    pub namespace: Option<String>,
}

impl Default for CppCodegenOptions {
    fn default() -> Self {
        Self {
            runtime_format: RuntimeFormat::Sora,
            cpp_standard: CppStandard::Cpp17,
            namespace: None,
        }
    }
}

impl HasRuntimeFormat for CppCodegenOptions {
    fn runtime_format(&self) -> Option<RuntimeFormat> {
        Some(self.runtime_format)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default)]
pub struct ScalaCodegenOptions {
    pub runtime_format: RuntimeFormat,
    pub scala_version: ScalaVersion,
}

impl Default for ScalaCodegenOptions {
    fn default() -> Self {
        Self {
            runtime_format: RuntimeFormat::Sora,
            scala_version: ScalaVersion::Scala3,
        }
    }
}

impl HasRuntimeFormat for ScalaCodegenOptions {
    fn runtime_format(&self) -> Option<RuntimeFormat> {
        Some(self.runtime_format)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default)]
pub struct TypeScriptCodegenOptions {
    pub runtime_format: RuntimeFormat,
    pub enum_repr: EnumRepr,
}

impl Default for TypeScriptCodegenOptions {
    fn default() -> Self {
        Self {
            runtime_format: RuntimeFormat::Sora,
            enum_repr: EnumRepr::String,
        }
    }
}

impl HasRuntimeFormat for TypeScriptCodegenOptions {
    fn runtime_format(&self) -> Option<RuntimeFormat> {
        Some(self.runtime_format)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default)]
pub struct JavaScriptCodegenOptions {
    pub runtime_format: RuntimeFormat,
    pub enum_repr: EnumRepr,
    pub emit_dts: bool,
}

impl Default for JavaScriptCodegenOptions {
    fn default() -> Self {
        Self {
            runtime_format: RuntimeFormat::Sora,
            enum_repr: EnumRepr::String,
            emit_dts: true,
        }
    }
}

impl HasRuntimeFormat for JavaScriptCodegenOptions {
    fn runtime_format(&self) -> Option<RuntimeFormat> {
        Some(self.runtime_format)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default)]
pub struct ErlangCodegenOptions {
    pub runtime_format: RuntimeFormat,
    pub enum_repr: ErlangEnumRepr,
}

impl Default for ErlangCodegenOptions {
    fn default() -> Self {
        Self {
            runtime_format: RuntimeFormat::Sora,
            enum_repr: ErlangEnumRepr::Atom,
        }
    }
}

impl HasRuntimeFormat for ErlangCodegenOptions {
    fn runtime_format(&self) -> Option<RuntimeFormat> {
        Some(self.runtime_format)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default)]
pub struct LuaCodegenOptions {
    pub runtime_format: RuntimeFormat,
    pub module: Option<String>,
    pub lua_version: LuaVersion,
    pub enum_repr: LuaEnumRepr,
}

impl Default for LuaCodegenOptions {
    fn default() -> Self {
        Self {
            runtime_format: RuntimeFormat::Sora,
            module: None,
            lua_version: LuaVersion::Lua54,
            enum_repr: LuaEnumRepr::String,
        }
    }
}

impl HasRuntimeFormat for LuaCodegenOptions {
    fn runtime_format(&self) -> Option<RuntimeFormat> {
        Some(self.runtime_format)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeFormat {
    #[default]
    Sora,
    Json,
    #[serde(rename = "sora-protobuf")]
    SoraProtobuf,
    Cbor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum CStandard {
    C99,
    #[default]
    C11,
    C17,
    C23,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
pub enum CppStandard {
    #[serde(rename = "c++11")]
    Cpp11,
    #[serde(rename = "c++14")]
    Cpp14,
    #[default]
    #[serde(rename = "c++17")]
    Cpp17,
    #[serde(rename = "c++20")]
    Cpp20,
    #[serde(rename = "c++23")]
    Cpp23,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
pub enum ScalaVersion {
    #[serde(rename = "2.12")]
    Scala212,
    #[serde(rename = "2.13")]
    Scala213,
    #[default]
    #[serde(rename = "3")]
    Scala3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
pub enum LuaVersion {
    #[serde(rename = "5.1")]
    Lua51,
    #[serde(rename = "5.2")]
    Lua52,
    #[serde(rename = "5.3")]
    Lua53,
    #[default]
    #[serde(rename = "5.4")]
    Lua54,
    #[serde(rename = "luajit")]
    LuaJit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum LuaEnumRepr {
    Integer,
    #[default]
    String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum EnumRepr {
    Integer,
    #[default]
    String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ErlangEnumRepr {
    Integer,
    #[default]
    Atom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RustMapType {
    #[default]
    Std,
    FxHashMap,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RustStringStorage {
    #[default]
    Owned,
    Arc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RustDateTimeType {
    #[default]
    SystemTime,
    Chrono,
}
