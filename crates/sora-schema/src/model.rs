use std::{collections::BTreeMap, fmt};

use serde::{
    Deserialize, Deserializer, Serialize,
    de::{SeqAccess, Visitor},
};

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ProjectSchema {
    pub project: ProjectMetadataSchema,

    #[serde(default)]
    pub groups: BTreeMap<String, GroupSchema>,

    #[serde(default)]
    pub views: BTreeMap<String, ViewSchema>,

    #[serde(default)]
    pub codegen: CodegenSchema,

    #[serde(default)]
    pub localization: Option<LocalizationSchema>,

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

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectMetadataSchema {
    pub id: String,

    #[serde(default)]
    pub views: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SchemaModule {
    pub namespace: String,
    pub imports: BTreeMap<String, String>,
    pub project: Option<ProjectMetadataSchema>,
    pub groups: BTreeMap<String, GroupSchema>,
    pub views: BTreeMap<String, ViewSchema>,
    pub codegen: Option<CodegenSchema>,
    pub localization: Option<LocalizationSchema>,
    pub includes: Vec<String>,
    pub enums: Vec<EnumSchema>,
    pub structs: Vec<StructSchema>,
    pub unions: Vec<UnionSchema>,
    pub tables: Vec<TableSchema>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GroupSchema {
    #[serde(default)]
    pub default: bool,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ViewSchema {
    pub contract: String,

    #[serde(default)]
    pub groups: Vec<String>,

    #[serde(default)]
    pub tables: ViewTableSelectionSchema,

    #[serde(default)]
    pub names: ViewNamesSchema,

    #[serde(default)]
    pub bindings: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ViewTableSelectionSchema {
    #[serde(default)]
    pub include: Vec<String>,

    #[serde(default)]
    pub exclude: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ViewNamesSchema {
    #[serde(default)]
    pub tables: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct LocalizationSchema {
    #[serde(default)]
    pub locales: Vec<String>,
    pub default_locale: Option<String>,
    pub fallback_locale: Option<String>,
    #[serde(default)]
    pub sources: Vec<LocalizationSourceSchema>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct LocalizationSourceSchema {
    pub name: String,
    pub file: String,
    pub sheet: Option<String>,
    pub format: Option<String>,
    #[serde(default = "default_localization_key")]
    pub key: String,
}

fn default_localization_key() -> String {
    "key".to_owned()
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
    pub comment: Option<String>,

    #[serde(default)]
    pub groups: GroupSetSchema,

    #[serde(default)]
    pub values: Vec<EnumValueSchema>,

    #[serde(default)]
    pub aliases: Vec<EnumAliasSchema>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct EnumValueSchema {
    pub id: u32,

    pub name: String,

    #[serde(default)]
    pub comment: Option<String>,
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
    pub groups: GroupSetSchema,

    #[serde(default)]
    pub fields: Vec<FieldSchema>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct UnionSchema {
    pub name: String,

    #[serde(default)]
    pub groups: GroupSetSchema,

    #[serde(default = "default_union_tag")]
    pub tag: String,

    #[serde(default)]
    pub variants: Vec<UnionVariantSchema>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct UnionVariantSchema {
    pub name: String,

    #[serde(default)]
    pub groups: GroupSetSchema,

    #[serde(default)]
    pub fields: Vec<FieldSchema>,
}

fn default_union_tag() -> String {
    "type".to_owned()
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct TableSchema {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub groups: GroupSetSchema,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
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
    pub groups: GroupSetSchema,

    pub comment: Option<String>,
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
    pub groups: GroupSetSchema,

    pub comment: Option<String>,
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

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GroupSetSchema {
    pub values: Vec<String>,
}

impl<'de> Deserialize<'de> for GroupSetSchema {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct GroupSetVisitor;

        impl<'de> Visitor<'de> for GroupSetVisitor {
            type Value = GroupSetSchema;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a group string or list of group strings")
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(GroupSetSchema {
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
                Ok(GroupSetSchema { values })
            }
        }

        deserializer.deserialize_any(GroupSetVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_toml_schema() {
        let schema: ProjectSchema = toml::from_str(
            r#"
project = { id = "game_config" }
groups = { common = { default = true } }
views = { default = { contract = "game_config/default", groups = ["common"] } }

[[enums]]
name = "ItemType"
values = [{ id = 0, name = "Weapon" }, { id = 1, name = "Armor" }]

[[tables]]
id = "item"
name = "Item"
mode = "map"
key = "id"

[tables.source]
file = "items.toml"

[[tables.fields]]
name = "id"
type = "i32"
comment = "Item id"

[[tables.fields]]
name = "tags"
type = "list<string>"
parser = { kind = "split", separator = "|" }
"#,
        )
        .expect("schema should parse");

        assert_eq!(schema.project.id, "game_config");
        assert!(schema.codegen.targets.is_empty());
        assert!(schema.includes.is_empty());
        assert_eq!(schema.enums[0].name, "ItemType");
        assert_eq!(schema.tables[0].mode, TableModeSchema::Map);
        assert_eq!(schema.tables[0].source.as_ref().unwrap().format, None);
        assert_eq!(schema.tables[0].fields[0].name, "id");
        let parser = schema.tables[0].fields[1].parser.as_ref().unwrap();
        assert_eq!(parser.kind, "split");
        assert_eq!(parser.options["separator"], "|");
    }

    #[test]
    fn defaults_optional_collections_and_field_flags() {
        let schema: ProjectSchema = toml::from_str(
            r#"
project = { id = "game_config" }
groups = { common = { default = true } }
views = { default = { contract = "game_config/default", groups = ["common"] } }
includes = ["items.toml"]

[[tables]]
id = "item"
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
    }

    #[test]
    fn rejects_table_only_properties_on_struct_fields() {
        let error = toml::from_str::<ProjectSchema>(
            r#"
project = { id = "game_config" }
groups = { common = { default = true } }
views = { default = { contract = "game_config/default", groups = ["common"] } }

[[structs]]
name = "Reward"

[[structs.fields]]
name = "item_id"
type = "i32"
from = { table = "RewardRow", parent_key = "id", child_key = "reward_id" }
"#,
        )
        .unwrap_err();

        assert!(error.to_string().contains("unknown field `from`"));
    }

    #[test]
    fn loads_codegen_options() {
        let schema: ProjectSchema = toml::from_str(
            r#"
project = { id = "game_config" }
groups = { common = { default = true } }
views = { default = { contract = "game_config/default", groups = ["common"] } }

[codegen.rust]
runtime_format = "sora"
map_type = "fx_hash_map"
string_storage = "arc"

[codegen.kotlin]
runtime_format = "sora"

[codegen.godot]
runtime_format = "json"
godot_version = "4.4"

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
            schema.codegen.targets["godot"]["godot_version"],
            serde_json::Value::String("4.4".to_owned())
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
