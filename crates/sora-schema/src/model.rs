use std::fmt;

use serde::{
    Deserialize, Deserializer,
    de::{SeqAccess, Visitor},
};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct SchemaFile {
    pub package: String,

    #[serde(default)]
    pub codegen: CodegenSchema,

    #[serde(default)]
    pub includes: Vec<String>,

    #[serde(default)]
    pub enums: Vec<EnumSchema>,

    #[serde(default)]
    pub structs: Vec<StructSchema>,

    #[serde(default)]
    pub unions: Vec<UnionSchema>,

    #[serde(default)]
    pub tables: Vec<TableSchema>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Default)]
pub struct CodegenSchema {
    #[serde(default)]
    pub rust: RustCodegenSchema,
    #[serde(default)]
    pub kotlin: LanguageCodegenSchema,
    #[serde(default)]
    pub csharp: LanguageCodegenSchema,
    #[serde(default)]
    pub java: LanguageCodegenSchema,
    #[serde(default)]
    pub go: LanguageCodegenSchema,
    #[serde(default)]
    pub dart: LanguageCodegenSchema,
    #[serde(default)]
    pub c: CCodegenSchema,
    #[serde(default)]
    pub cpp: CppCodegenSchema,
    #[serde(default)]
    pub typescript: TypeScriptCodegenSchema,
    #[serde(default)]
    pub javascript: JavaScriptCodegenSchema,
    #[serde(default)]
    pub erlang: ErlangCodegenSchema,
    #[serde(default)]
    pub lua: LuaCodegenSchema,
    #[serde(default)]
    pub python: LanguageCodegenSchema,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct RustCodegenSchema {
    #[serde(default)]
    pub runtime_format: RuntimeFormatSchema,

    #[serde(default)]
    pub map_type: RustMapTypeSchema,
}

impl Default for RustCodegenSchema {
    fn default() -> Self {
        Self {
            runtime_format: RuntimeFormatSchema::Sora,
            map_type: RustMapTypeSchema::Std,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CCodegenSchema {
    #[serde(default)]
    pub runtime_format: RuntimeFormatSchema,

    #[serde(default)]
    pub c_standard: CStandardSchema,

    pub prefix: Option<String>,
}

impl Default for CCodegenSchema {
    fn default() -> Self {
        Self {
            runtime_format: RuntimeFormatSchema::Sora,
            c_standard: CStandardSchema::C11,
            prefix: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CppCodegenSchema {
    #[serde(default)]
    pub runtime_format: RuntimeFormatSchema,

    #[serde(default)]
    pub cpp_standard: CppStandardSchema,

    pub namespace: Option<String>,
}

impl Default for CppCodegenSchema {
    fn default() -> Self {
        Self {
            runtime_format: RuntimeFormatSchema::Sora,
            cpp_standard: CppStandardSchema::Cpp17,
            namespace: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct LanguageCodegenSchema {
    #[serde(default)]
    pub runtime_format: RuntimeFormatSchema,
}

impl Default for LanguageCodegenSchema {
    fn default() -> Self {
        Self {
            runtime_format: RuntimeFormatSchema::Sora,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct TypeScriptCodegenSchema {
    #[serde(default)]
    pub runtime_format: RuntimeFormatSchema,

    #[serde(default)]
    pub enum_repr: EnumReprSchema,
}

impl Default for TypeScriptCodegenSchema {
    fn default() -> Self {
        Self {
            runtime_format: RuntimeFormatSchema::Sora,
            enum_repr: EnumReprSchema::String,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct JavaScriptCodegenSchema {
    #[serde(default)]
    pub runtime_format: RuntimeFormatSchema,

    #[serde(default)]
    pub enum_repr: EnumReprSchema,

    #[serde(default = "default_emit_dts")]
    pub emit_dts: bool,
}

impl Default for JavaScriptCodegenSchema {
    fn default() -> Self {
        Self {
            runtime_format: RuntimeFormatSchema::Sora,
            enum_repr: EnumReprSchema::String,
            emit_dts: default_emit_dts(),
        }
    }
}

fn default_emit_dts() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ErlangCodegenSchema {
    #[serde(default)]
    pub runtime_format: RuntimeFormatSchema,

    #[serde(default)]
    pub enum_repr: ErlangEnumReprSchema,
}

impl Default for ErlangCodegenSchema {
    fn default() -> Self {
        Self {
            runtime_format: RuntimeFormatSchema::Sora,
            enum_repr: ErlangEnumReprSchema::Atom,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct LuaCodegenSchema {
    #[serde(default)]
    pub runtime_format: RuntimeFormatSchema,

    pub module: Option<String>,

    #[serde(default)]
    pub lua_version: LuaVersionSchema,

    #[serde(default)]
    pub enum_repr: LuaEnumReprSchema,
}

impl Default for LuaCodegenSchema {
    fn default() -> Self {
        Self {
            runtime_format: RuntimeFormatSchema::Sora,
            module: None,
            lua_version: LuaVersionSchema::Lua54,
            enum_repr: LuaEnumReprSchema::String,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeFormatSchema {
    #[default]
    Sora,
    Json,
    #[serde(rename = "sora-protobuf")]
    SoraProtobuf,
    Cbor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum CStandardSchema {
    C99,
    #[default]
    C11,
    C17,
    C23,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
pub enum CppStandardSchema {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
pub enum LuaVersionSchema {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum LuaEnumReprSchema {
    Integer,
    #[default]
    String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum EnumReprSchema {
    Integer,
    #[default]
    String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ErlangEnumReprSchema {
    Integer,
    #[default]
    Atom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RustMapTypeSchema {
    #[default]
    Std,
    FxHashMap,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct EnumSchema {
    pub name: String,

    #[serde(default)]
    pub scope: ScopeSchema,

    #[serde(default)]
    pub values: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct StructSchema {
    pub name: String,

    #[serde(default)]
    pub scope: ScopeSchema,

    #[serde(default)]
    pub fields: Vec<FieldSchema>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct UnionSchema {
    pub name: String,

    #[serde(default)]
    pub scope: ScopeSchema,

    #[serde(default = "default_union_tag")]
    pub tag: String,

    #[serde(default)]
    pub variants: Vec<UnionVariantSchema>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct UnionVariantSchema {
    pub name: String,

    #[serde(default)]
    pub scope: ScopeSchema,

    #[serde(default)]
    pub fields: Vec<FieldSchema>,
}

fn default_union_tag() -> String {
    "type".to_owned()
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct TableSchema {
    pub name: String,
    #[serde(default)]
    pub scope: ScopeSchema,
    pub mode: TableModeSchema,
    pub key: Option<String>,
    pub source: Option<TableSourceSchema>,

    #[serde(default)]
    pub fields: Vec<FieldSchema>,

    #[serde(default)]
    pub indexes: Vec<IndexSchema>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct TableSourceSchema {
    pub format: String,
    pub file: String,
    pub sheet: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TableModeSchema {
    List,
    Map,
    Singleton,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct IndexSchema {
    pub name: String,

    #[serde(default)]
    pub fields: Vec<String>,

    #[serde(default)]
    pub unique: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct FieldSchema {
    pub name: String,

    #[serde(rename = "type")]
    pub ty: String,

    #[serde(default)]
    pub scope: ScopeSchema,

    #[serde(default)]
    pub key: bool,

    pub comment: Option<String>,
    pub required: Option<bool>,
    pub default: Option<String>,
    pub range: Option<[i64; 2]>,
    pub length: Option<[usize; 2]>,
    pub parser: Option<String>,
    pub separator: Option<String>,
    pub prefix: Option<String>,
    pub suffix: Option<String>,
    pub source_table: Option<String>,
    pub parent_key: Option<String>,
    pub child_key: Option<String>,
    pub order_by: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopeSchema {
    pub values: Vec<String>,
}

impl Default for ScopeSchema {
    fn default() -> Self {
        Self {
            values: vec!["all".to_owned()],
        }
    }
}

impl<'de> Deserialize<'de> for ScopeSchema {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ScopeVisitor;

        impl<'de> Visitor<'de> for ScopeVisitor {
            type Value = ScopeSchema;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a scope string or list of scope strings")
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(ScopeSchema {
                    values: vec![value.to_owned()],
                })
            }

            fn visit_seq<A>(self, mut seq: A) -> std::result::Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut values = Vec::new();
                while let Some(value) = seq.next_element::<String>()? {
                    values.push(value);
                }
                Ok(ScopeSchema { values })
            }
        }

        deserializer.deserialize_any(ScopeVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_toml_schema() {
        let schema: SchemaFile = toml::from_str(
            r#"
package = "game_config"

[[enums]]
name = "ItemType"
values = ["Weapon", "Armor"]

[[tables]]
name = "Item"
mode = "map"
key = "id"

[tables.source]
format = "toml"
file = "items.toml"

[[tables.fields]]
name = "id"
type = "i32"
key = true
required = true
comment = "Item id"

[[tables.fields]]
name = "tags"
type = "list<string>"
separator = "|"
prefix = "["
suffix = "]"
"#,
        )
        .expect("schema should parse");

        assert_eq!(schema.package, "game_config");
        assert_eq!(schema.codegen.rust.map_type, RustMapTypeSchema::Std);
        assert!(schema.includes.is_empty());
        assert_eq!(schema.enums[0].name, "ItemType");
        assert_eq!(schema.tables[0].mode, TableModeSchema::Map);
        assert_eq!(schema.tables[0].source.as_ref().unwrap().format, "toml");
        assert_eq!(schema.tables[0].fields[0].name, "id");
        assert!(schema.tables[0].fields[0].key);
        assert_eq!(schema.tables[0].fields[0].required, Some(true));
        assert_eq!(schema.tables[0].fields[1].separator.as_deref(), Some("|"));
        assert_eq!(schema.tables[0].fields[1].prefix.as_deref(), Some("["));
        assert_eq!(schema.tables[0].fields[1].suffix.as_deref(), Some("]"));
    }

    #[test]
    fn defaults_optional_collections_and_field_flags() {
        let schema: SchemaFile = toml::from_str(
            r#"
package = "game_config"
includes = ["items.toml"]

[[tables]]
name = "Item"
mode = "list"

[[tables.fields]]
name = "name"
type = "string"
"#,
        )
        .expect("schema should parse");

        assert!(schema.enums.is_empty());
        assert_eq!(schema.includes, ["items.toml"]);
        assert!(schema.structs.is_empty());
        assert!(schema.tables[0].indexes.is_empty());
        assert!(!schema.tables[0].fields[0].key);
        assert_eq!(schema.tables[0].fields[0].required, None);
    }

    #[test]
    fn loads_codegen_options() {
        let schema: SchemaFile = toml::from_str(
            r#"
package = "game_config"

[codegen.rust]
runtime_format = "sora"
map_type = "fx_hash_map"

[codegen.kotlin]
runtime_format = "sora"

[codegen.c]
runtime_format = "sora"
c_standard = "c17"
prefix = "game_config"

[codegen.cpp]
runtime_format = "sora"
cpp_standard = "c++20"
namespace = "sora::game_config"

[codegen.typescript]
runtime_format = "sora"
enum_repr = "string"

[codegen.javascript]
runtime_format = "sora"
enum_repr = "integer"
emit_dts = false

[codegen.erlang]
runtime_format = "sora"
enum_repr = "atom"

[codegen.lua]
runtime_format = "sora"
module = "generated.lua"
lua_version = "5.4"
enum_repr = "string"
"#,
        )
        .expect("schema should parse");

        assert_eq!(
            schema.codegen.rust.runtime_format,
            RuntimeFormatSchema::Sora
        );
        assert_eq!(schema.codegen.rust.map_type, RustMapTypeSchema::FxHashMap);
        assert_eq!(
            schema.codegen.kotlin.runtime_format,
            RuntimeFormatSchema::Sora
        );
        assert_eq!(schema.codegen.c.runtime_format, RuntimeFormatSchema::Sora);
        assert_eq!(schema.codegen.c.c_standard, CStandardSchema::C17);
        assert_eq!(schema.codegen.c.prefix.as_deref(), Some("game_config"));
        assert_eq!(schema.codegen.cpp.runtime_format, RuntimeFormatSchema::Sora);
        assert_eq!(schema.codegen.cpp.cpp_standard, CppStandardSchema::Cpp20);
        assert_eq!(
            schema.codegen.cpp.namespace.as_deref(),
            Some("sora::game_config")
        );
        assert_eq!(
            schema.codegen.typescript.runtime_format,
            RuntimeFormatSchema::Sora
        );
        assert_eq!(schema.codegen.typescript.enum_repr, EnumReprSchema::String);
        assert_eq!(
            schema.codegen.javascript.runtime_format,
            RuntimeFormatSchema::Sora
        );
        assert_eq!(schema.codegen.javascript.enum_repr, EnumReprSchema::Integer);
        assert!(!schema.codegen.javascript.emit_dts);
        assert_eq!(
            schema.codegen.erlang.runtime_format,
            RuntimeFormatSchema::Sora
        );
        assert_eq!(schema.codegen.erlang.enum_repr, ErlangEnumReprSchema::Atom);
        assert_eq!(schema.codegen.lua.runtime_format, RuntimeFormatSchema::Sora);
        assert_eq!(schema.codegen.lua.module.as_deref(), Some("generated.lua"));
        assert_eq!(schema.codegen.lua.lua_version, LuaVersionSchema::Lua54);
        assert_eq!(schema.codegen.lua.enum_repr, LuaEnumReprSchema::String);
    }
}
