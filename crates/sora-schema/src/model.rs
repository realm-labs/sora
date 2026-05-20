use serde::Deserialize;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeFormatSchema {
    #[default]
    Sora,
    Json,
    Protobuf,
    Cbor,
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
    pub values: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct StructSchema {
    pub name: String,

    #[serde(default)]
    pub fields: Vec<FieldSchema>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct UnionSchema {
    pub name: String,

    #[serde(default = "default_union_tag")]
    pub tag: String,

    #[serde(default)]
    pub variants: Vec<UnionVariantSchema>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct UnionVariantSchema {
    pub name: String,

    #[serde(default)]
    pub fields: Vec<FieldSchema>,
}

fn default_union_tag() -> String {
    "type".to_owned()
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct TableSchema {
    pub name: String,
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
    }
}
