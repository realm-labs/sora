use std::{collections::BTreeMap, fmt};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfigIr {
    pub package: String,
    #[serde(default, skip_serializing_if = "CodegenIr::is_default")]
    pub codegen: CodegenIr,
    pub enums: Vec<EnumIr>,
    pub structs: Vec<StructIr>,
    pub unions: Vec<UnionIr>,
    pub tables: Vec<TableIr>,
}

impl ConfigIr {
    pub fn data_schema(&self) -> Self {
        let mut schema = self.clone();
        schema.codegen = CodegenIr::default();
        schema
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CodegenIr {
    pub rust: RustCodegenIr,
    pub kotlin: LanguageCodegenIr,
    pub csharp: LanguageCodegenIr,
    pub java: LanguageCodegenIr,
    pub scala: ScalaCodegenIr,
    pub go: LanguageCodegenIr,
    pub dart: LanguageCodegenIr,
    pub godot: LanguageCodegenIr,
    pub c: CCodegenIr,
    pub cpp: CppCodegenIr,
    pub typescript: TypeScriptCodegenIr,
    pub javascript: JavaScriptCodegenIr,
    pub erlang: ErlangCodegenIr,
    pub lua: LuaCodegenIr,
    pub python: LanguageCodegenIr,
}

impl CodegenIr {
    pub fn is_default(&self) -> bool {
        self == &Self::default()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustCodegenIr {
    pub runtime_format: RuntimeFormatIr,
    pub map_type: RustMapTypeIr,
}

impl Default for RustCodegenIr {
    fn default() -> Self {
        Self {
            runtime_format: RuntimeFormatIr::Sora,
            map_type: RustMapTypeIr::Std,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CCodegenIr {
    pub runtime_format: RuntimeFormatIr,
    pub c_standard: CStandardIr,
    pub prefix: Option<String>,
}

impl Default for CCodegenIr {
    fn default() -> Self {
        Self {
            runtime_format: RuntimeFormatIr::Sora,
            c_standard: CStandardIr::C11,
            prefix: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CppCodegenIr {
    pub runtime_format: RuntimeFormatIr,
    pub cpp_standard: CppStandardIr,
    pub namespace: Option<String>,
}

impl Default for CppCodegenIr {
    fn default() -> Self {
        Self {
            runtime_format: RuntimeFormatIr::Sora,
            cpp_standard: CppStandardIr::Cpp17,
            namespace: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LanguageCodegenIr {
    pub runtime_format: RuntimeFormatIr,
}

impl Default for LanguageCodegenIr {
    fn default() -> Self {
        Self {
            runtime_format: RuntimeFormatIr::Sora,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScalaCodegenIr {
    pub runtime_format: RuntimeFormatIr,
    pub scala_version: ScalaVersionIr,
}

impl Default for ScalaCodegenIr {
    fn default() -> Self {
        Self {
            runtime_format: RuntimeFormatIr::Sora,
            scala_version: ScalaVersionIr::Scala3,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TypeScriptCodegenIr {
    pub runtime_format: RuntimeFormatIr,
    pub enum_repr: EnumReprIr,
}

impl Default for TypeScriptCodegenIr {
    fn default() -> Self {
        Self {
            runtime_format: RuntimeFormatIr::Sora,
            enum_repr: EnumReprIr::String,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JavaScriptCodegenIr {
    pub runtime_format: RuntimeFormatIr,
    pub enum_repr: EnumReprIr,
    pub emit_dts: bool,
}

impl Default for JavaScriptCodegenIr {
    fn default() -> Self {
        Self {
            runtime_format: RuntimeFormatIr::Sora,
            enum_repr: EnumReprIr::String,
            emit_dts: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErlangCodegenIr {
    pub runtime_format: RuntimeFormatIr,
    pub enum_repr: ErlangEnumReprIr,
}

impl Default for ErlangCodegenIr {
    fn default() -> Self {
        Self {
            runtime_format: RuntimeFormatIr::Sora,
            enum_repr: ErlangEnumReprIr::Atom,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LuaCodegenIr {
    pub runtime_format: RuntimeFormatIr,
    pub module: Option<String>,
    pub lua_version: LuaVersionIr,
    pub enum_repr: LuaEnumReprIr,
}

impl Default for LuaCodegenIr {
    fn default() -> Self {
        Self {
            runtime_format: RuntimeFormatIr::Sora,
            module: None,
            lua_version: LuaVersionIr::Lua54,
            enum_repr: LuaEnumReprIr::String,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum RuntimeFormatIr {
    #[default]
    Sora,
    Json,
    SoraProtobuf,
    Cbor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CStandardIr {
    C99,
    #[default]
    C11,
    C17,
    C23,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CppStandardIr {
    Cpp11,
    Cpp14,
    #[default]
    Cpp17,
    Cpp20,
    Cpp23,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ScalaVersionIr {
    Scala212,
    Scala213,
    #[default]
    Scala3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum LuaVersionIr {
    Lua51,
    Lua52,
    Lua53,
    #[default]
    Lua54,
    LuaJit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum LuaEnumReprIr {
    Integer,
    #[default]
    String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum EnumReprIr {
    Integer,
    #[default]
    String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ErlangEnumReprIr {
    Integer,
    #[default]
    Atom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum RustMapTypeIr {
    #[default]
    Std,
    FxHashMap,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnumIr {
    pub name: String,
    pub scope: ScopeIr,
    pub values: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructIr {
    pub name: String,
    pub scope: ScopeIr,
    pub fields: Vec<FieldIr>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnionIr {
    pub name: String,
    pub scope: ScopeIr,
    pub tag: String,
    pub variants: Vec<UnionVariantIr>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnionVariantIr {
    pub name: String,
    pub scope: ScopeIr,
    pub fields: Vec<FieldIr>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableIr {
    pub name: String,
    pub scope: ScopeIr,
    pub mode: TableModeIr,
    pub key: Option<String>,
    pub source: Option<TableSourceIr>,
    pub fields: Vec<FieldIr>,
    pub indexes: Vec<IndexIr>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableSourceIr {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    pub file: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sheet: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TableModeIr {
    List,
    Map,
    Singleton,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexIr {
    pub name: String,
    pub fields: Vec<String>,
    pub unique: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldIr {
    pub name: String,
    pub ty: TypeIr,
    pub scope: ScopeIr,
    pub key: bool,
    pub comment: Option<String>,
    pub required: bool,
    pub default: Option<String>,
    pub range: Option<[i64; 2]>,
    pub length: Option<[usize; 2]>,
    pub parser: Option<ParserIr>,
    pub aggregation: Option<AggregationIr>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParserIr {
    pub kind: String,
    pub options: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AggregationIr {
    pub source_table: String,
    pub parent_key: String,
    pub child_key: String,
    pub order_by: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScopeIr {
    pub values: Vec<String>,
}

impl Default for ScopeIr {
    fn default() -> Self {
        Self {
            values: vec!["all".to_owned()],
        }
    }
}

impl ScopeIr {
    pub fn includes(&self, target: &str) -> bool {
        target == "all"
            || self.values.iter().any(|value| value == "all")
            || self.values.iter().any(|value| value == target)
    }

    pub fn display(&self) -> String {
        self.values.join(",")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TypeIr {
    Bool,
    I32,
    I64,
    F32,
    F64,
    String,
    Enum(String),
    Struct(String),
    Union(String),
    List(Box<TypeIr>),
    Array { element: Box<TypeIr>, len: usize },
    Ref { table: String, field: String },
    Optional(Box<TypeIr>),
}

impl fmt::Display for TypeIr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeIr::Bool => f.write_str("bool"),
            TypeIr::I32 => f.write_str("i32"),
            TypeIr::I64 => f.write_str("i64"),
            TypeIr::F32 => f.write_str("f32"),
            TypeIr::F64 => f.write_str("f64"),
            TypeIr::String => f.write_str("string"),
            TypeIr::Enum(name) => write!(f, "enum<{name}>"),
            TypeIr::Struct(name) => write!(f, "struct<{name}>"),
            TypeIr::Union(name) => write!(f, "union<{name}>"),
            TypeIr::List(element) => write!(f, "list<{element}>"),
            TypeIr::Array { element, len } => write!(f, "array<{element},{len}>"),
            TypeIr::Ref { table, field } => write!(f, "ref<{table}.{field}>"),
            TypeIr::Optional(element) => write!(f, "optional<{element}>"),
        }
    }
}
