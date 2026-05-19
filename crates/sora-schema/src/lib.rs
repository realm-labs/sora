use std::{fs, path::Path};

use serde::Deserialize;
use sora_diagnostics::{Result, SoraError};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct SchemaFile {
    pub package: String,

    #[serde(default)]
    pub enums: Vec<EnumSchema>,

    #[serde(default)]
    pub structs: Vec<StructSchema>,

    #[serde(default)]
    pub tables: Vec<TableSchema>,
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
pub struct TableSchema {
    pub name: String,
    pub mode: TableModeSchema,
    pub key: Option<String>,
    pub source: Option<String>,

    #[serde(default)]
    pub fields: Vec<FieldSchema>,

    #[serde(default)]
    pub indexes: Vec<IndexSchema>,
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
    pub range: Option<[i64; 2]>,
    pub parser: Option<String>,
    pub source_table: Option<String>,
    pub parent_key: Option<String>,
    pub child_key: Option<String>,
    pub order_by: Option<String>,
}

pub fn load_schema_file(path: &Path) -> Result<SchemaFile> {
    let content = fs::read_to_string(path).map_err(|source| SoraError::ReadFile {
        path: path.to_path_buf(),
        source,
    })?;

    toml::from_str(&content).map_err(|source| SoraError::ParseSchema {
        path: path.to_path_buf(),
        source,
    })
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
source = "items.toml"

[[tables.fields]]
name = "id"
type = "i32"
key = true
required = true
comment = "Item id"
"#,
        )
        .expect("schema should parse");

        assert_eq!(schema.package, "game_config");
        assert_eq!(schema.enums[0].name, "ItemType");
        assert_eq!(schema.tables[0].mode, TableModeSchema::Map);
        assert_eq!(schema.tables[0].fields[0].name, "id");
        assert!(schema.tables[0].fields[0].key);
        assert_eq!(schema.tables[0].fields[0].required, Some(true));
    }

    #[test]
    fn defaults_optional_collections_and_field_flags() {
        let schema: SchemaFile = toml::from_str(
            r#"
package = "game_config"

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
        assert!(schema.structs.is_empty());
        assert!(schema.tables[0].indexes.is_empty());
        assert!(!schema.tables[0].fields[0].key);
        assert_eq!(schema.tables[0].fields[0].required, None);
    }
}
