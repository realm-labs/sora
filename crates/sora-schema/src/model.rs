use std::{collections::BTreeMap, fmt};

use serde::{
    Deserialize, Deserializer,
    de::{SeqAccess, Visitor},
};

#[derive(Debug, Clone, PartialEq, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Deserialize, Default)]
pub struct CodegenSchema {
    #[serde(flatten)]
    pub targets: BTreeMap<String, serde_json::Value>,
}

impl CodegenSchema {
    pub fn target_options(&self, target: &str) -> Option<&serde_json::Value> {
        self.targets.get(target)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct EnumSchema {
    pub name: String,

    #[serde(default)]
    pub scope: ScopeSchema,

    #[serde(default)]
    pub values: Vec<String>,

    #[serde(default)]
    pub aliases: Vec<EnumAliasSchema>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct EnumAliasSchema {
    pub name: String,

    pub alias: String,
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
    pub fields: Vec<TableFieldSchema>,

    #[serde(default)]
    pub indexes: Vec<IndexSchema>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct TableSourceSchema {
    pub format: Option<String>,
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
#[serde(deny_unknown_fields)]
pub struct FieldSchema {
    pub name: String,

    #[serde(rename = "type")]
    pub ty: String,

    #[serde(default)]
    pub scope: ScopeSchema,

    pub comment: Option<String>,
    pub required: Option<bool>,
    pub default: Option<String>,
    pub range: Option<[i64; 2]>,
    pub length: Option<[usize; 2]>,
    pub parser: Option<ParserSchema>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TableFieldSchema {
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
    pub parser: Option<ParserSchema>,
    pub from: Option<TableFieldFromSchema>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TableFieldFromSchema {
    pub table: String,
    pub parent_key: Option<String>,
    pub child_key: Option<String>,
    #[serde(rename = "field")]
    pub value_field: Option<String>,
    pub order_by: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ParserSchema {
    pub kind: String,

    #[serde(flatten)]
    pub options: BTreeMap<String, String>,
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
parser = { kind = "split", separator = "|" }
"#,
        )
        .expect("schema should parse");

        assert_eq!(schema.package, "game_config");
        assert!(schema.codegen.targets.is_empty());
        assert!(schema.includes.is_empty());
        assert_eq!(schema.enums[0].name, "ItemType");
        assert_eq!(schema.tables[0].mode, TableModeSchema::Map);
        assert_eq!(schema.tables[0].source.as_ref().unwrap().format, None);
        assert_eq!(schema.tables[0].fields[0].name, "id");
        assert!(schema.tables[0].fields[0].key);
        assert_eq!(schema.tables[0].fields[0].required, Some(true));
        let parser = schema.tables[0].fields[1].parser.as_ref().unwrap();
        assert_eq!(parser.kind, "split");
        assert_eq!(parser.options["separator"], "|");
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
    fn rejects_table_only_properties_on_struct_fields() {
        let error = toml::from_str::<SchemaFile>(
            r#"
package = "game_config"

[[structs]]
name = "Reward"

[[structs.fields]]
name = "item_id"
type = "i32"
key = true
"#,
        )
        .unwrap_err();

        assert!(error.to_string().contains("unknown field `key`"));
    }

    #[test]
    fn loads_codegen_options() {
        let schema: SchemaFile = toml::from_str(
            r#"
package = "game_config"

[codegen.rust]
runtime_format = "sora"
map_type = "fx_hash_map"
string_storage = "arc"

[codegen.kotlin]
runtime_format = "sora"

[codegen.godot]
runtime_format = "json"

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
            schema.codegen.targets["rust"]["map_type"],
            serde_json::Value::String("fx_hash_map".to_owned())
        );
        assert_eq!(
            schema.codegen.targets["rust"]["string_storage"],
            serde_json::Value::String("arc".to_owned())
        );
        assert_eq!(
            schema.codegen.targets["godot"]["runtime_format"],
            serde_json::Value::String("json".to_owned())
        );
        assert_eq!(
            schema.codegen.targets["cpp"]["namespace"],
            serde_json::Value::String("sora::game_config".to_owned())
        );
        assert_eq!(
            schema.codegen.targets["javascript"]["emit_dts"],
            serde_json::Value::Bool(false)
        );
    }
}
