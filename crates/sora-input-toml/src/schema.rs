use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use mlua::{Lua, LuaOptions, LuaSerdeExt, StdLib};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use sora_diagnostics::{Result, SoraError};
use sora_schema::model::{
    CodegenSchema, EnumSchema, SchemaFile, StructSchema, TableSchema, UnionSchema,
};

pub fn load_project_schema_file(path: &Path) -> Result<SchemaFile> {
    let mut visited = BTreeSet::new();
    let root = load_schema_document(path)?;
    let package = root.package.clone().ok_or_else(|| {
        SoraError::InvalidSchema(format!(
            "project schema `{}` must declare `package`",
            path.display()
        ))
    })?;
    let mut merged = SchemaFile {
        package,
        codegen: root.codegen.unwrap_or_default(),
        includes: root.includes.clone(),
        enums: root.enums,
        structs: root.structs,
        unions: root.unions,
        tables: root.tables,
    };

    merge_includes(path, &root.includes, &mut merged, &mut visited)?;
    Ok(merged)
}

fn merge_includes(
    parent_path: &Path,
    includes: &[String],
    merged: &mut SchemaFile,
    visited: &mut BTreeSet<PathBuf>,
) -> Result<()> {
    let base_dir = parent_path.parent().unwrap_or_else(|| Path::new("."));

    for include in includes {
        let include_path = base_dir.join(include);
        let canonical_key = include_path
            .canonicalize()
            .unwrap_or_else(|_| include_path.clone());
        if !visited.insert(canonical_key.clone()) {
            return Err(SoraError::InvalidSchema(format!(
                "schema include cycle or duplicate include `{}`",
                include_path.display()
            )));
        }

        let module = load_schema_document(&include_path)?;
        if module.package.is_some() {
            return Err(SoraError::InvalidSchema(format!(
                "included schema module `{}` must not declare `package`",
                include_path.display()
            )));
        }
        if module.codegen.is_some() {
            return Err(SoraError::InvalidSchema(format!(
                "included schema module `{}` must not declare `codegen`",
                include_path.display()
            )));
        }

        merged.enums.extend(module.enums);
        merged.structs.extend(module.structs);
        merged.unions.extend(module.unions);
        merged.tables.extend(module.tables);
        merge_includes(&include_path, &module.includes, merged, visited)?;
    }

    Ok(())
}

fn load_schema_document(path: &Path) -> Result<SchemaDocument> {
    let content = fs::read_to_string(path).map_err(|source| SoraError::ReadFile {
        path: path.to_path_buf(),
        source,
    })?;

    match schema_format(path)? {
        SchemaFormat::Toml => {
            toml::from_str(&content).map_err(|source: toml::de::Error| SoraError::ParseSchema {
                path: path.to_path_buf(),
                message: source.to_string(),
            })
        }
        SchemaFormat::Yaml => {
            serde_yaml::from_str(&content).map_err(|source| SoraError::ParseSchema {
                path: path.to_path_buf(),
                message: source.to_string(),
            })
        }
        SchemaFormat::Json => {
            serde_json::from_str(&content).map_err(|source| SoraError::ParseSchema {
                path: path.to_path_buf(),
                message: source.to_string(),
            })
        }
        SchemaFormat::Lua => parse_lua_document(path, &content),
    }
}

fn parse_lua_document<T>(path: &Path, content: &str) -> Result<T>
where
    T: DeserializeOwned,
{
    let lua = Lua::new_with(
        StdLib::TABLE | StdLib::STRING | StdLib::MATH | StdLib::UTF8,
        LuaOptions::default(),
    )
    .map_err(|source| SoraError::ParseSchema {
        path: path.to_path_buf(),
        message: source.to_string(),
    })?;
    let value = lua
        .load(content)
        .eval()
        .map_err(|source| SoraError::ParseSchema {
            path: path.to_path_buf(),
            message: source.to_string(),
        })?;

    lua.from_value(value)
        .map_err(|source| SoraError::ParseSchema {
            path: path.to_path_buf(),
            message: source.to_string(),
        })
}

fn schema_format(path: &Path) -> Result<SchemaFormat> {
    match path.extension().and_then(|extension| extension.to_str()) {
        Some("toml") => Ok(SchemaFormat::Toml),
        Some("yaml" | "yml") => Ok(SchemaFormat::Yaml),
        Some("json") => Ok(SchemaFormat::Json),
        Some("lua") => Ok(SchemaFormat::Lua),
        Some(extension) => Err(SoraError::InvalidSchema(format!(
            "schema file `{}` has unsupported extension `{extension}`",
            path.display()
        ))),
        None => Err(SoraError::InvalidSchema(format!(
            "schema file `{}` must have an extension",
            path.display()
        ))),
    }
}

#[derive(Debug, Clone, Copy)]
enum SchemaFormat {
    Toml,
    Yaml,
    Json,
    Lua,
}

#[derive(Debug, Deserialize)]
struct SchemaDocument {
    pub package: Option<String>,
    pub codegen: Option<CodegenSchema>,

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn loads_project_schema_with_toml_includes() {
        let base = temp_dir();
        let schema_dir = base.join("schema");
        fs::create_dir_all(&schema_dir).unwrap();
        let project_path = base.join("project.toml");
        fs::write(
            &project_path,
            r#"
package = "game_config"
includes = ["schema/items.toml"]

[codegen.rust]
map_type = "fx_hash_map"
"#,
        )
        .unwrap();
        fs::write(
            schema_dir.join("items.toml"),
            r#"
[[enums]]
name = "ItemType"
values = ["Weapon", "Armor"]

[[tables]]
name = "Item"
mode = "map"
key = "id"
"#,
        )
        .unwrap();

        let schema = load_project_schema_file(&project_path).unwrap();

        assert_eq!(schema.package, "game_config");
        assert_eq!(
            schema.codegen.targets["rust"]["map_type"].as_str(),
            Some("fx_hash_map")
        );
        assert_eq!(schema.includes, ["schema/items.toml"]);
        assert_eq!(schema.enums[0].name, "ItemType");
        assert_eq!(schema.tables[0].name, "Item");

        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn loads_project_schema_with_yaml_includes() {
        let base = temp_dir();
        let schema_dir = base.join("schema");
        fs::create_dir_all(&schema_dir).unwrap();
        let project_path = base.join("project.yaml");
        fs::write(
            &project_path,
            r#"
package: game_config
includes:
  - schema/items.yml
codegen:
  rust:
    map_type: fx_hash_map
"#,
        )
        .unwrap();
        fs::write(
            schema_dir.join("items.yml"),
            r#"
enums:
  - name: ItemType
    values: [Weapon, Armor]
tables:
  - name: Item
    mode: map
    key: id
"#,
        )
        .unwrap();

        let schema = load_project_schema_file(&project_path).unwrap();

        assert_eq!(schema.package, "game_config");
        assert_eq!(
            schema.codegen.targets["rust"]["map_type"].as_str(),
            Some("fx_hash_map")
        );
        assert_eq!(schema.includes, ["schema/items.yml"]);
        assert_eq!(schema.enums[0].name, "ItemType");
        assert_eq!(schema.tables[0].name, "Item");

        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn loads_project_schema_with_json_includes() {
        let base = temp_dir();
        let schema_dir = base.join("schema");
        fs::create_dir_all(&schema_dir).unwrap();
        let project_path = base.join("project.json");
        fs::write(
            &project_path,
            r#"
{
  "package": "game_config",
  "includes": ["schema/items.json"],
  "codegen": {
    "rust": {
      "map_type": "fx_hash_map"
    }
  }
}
"#,
        )
        .unwrap();
        fs::write(
            schema_dir.join("items.json"),
            r#"
{
  "enums": [
    { "name": "ItemType", "values": ["Weapon", "Armor"] }
  ],
  "tables": [
    { "name": "Item", "mode": "map", "key": "id" }
  ]
}
"#,
        )
        .unwrap();

        let schema = load_project_schema_file(&project_path).unwrap();

        assert_eq!(schema.package, "game_config");
        assert_eq!(
            schema.codegen.targets["rust"]["map_type"].as_str(),
            Some("fx_hash_map")
        );
        assert_eq!(schema.includes, ["schema/items.json"]);
        assert_eq!(schema.enums[0].name, "ItemType");
        assert_eq!(schema.tables[0].name, "Item");

        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn loads_project_schema_with_lua_includes() {
        let base = temp_dir();
        let schema_dir = base.join("schema");
        fs::create_dir_all(&schema_dir).unwrap();
        let project_path = base.join("project.lua");
        fs::write(
            &project_path,
            r#"
return {
  package = "game_config",
  includes = { "schema/items.lua" },
  codegen = {
    rust = {
      map_type = "fx_hash_map",
    },
  },
}
"#,
        )
        .unwrap();
        fs::write(
            schema_dir.join("items.lua"),
            r#"
return {
  enums = {
    { name = "ItemType", values = { "Weapon", "Armor" } },
  },
  tables = {
    { name = "Item", mode = "map", key = "id" },
  },
}
"#,
        )
        .unwrap();

        let schema = load_project_schema_file(&project_path).unwrap();

        assert_eq!(schema.package, "game_config");
        assert_eq!(
            schema.codegen.targets["rust"]["map_type"].as_str(),
            Some("fx_hash_map")
        );
        assert_eq!(schema.includes, ["schema/items.lua"]);
        assert_eq!(schema.enums[0].name, "ItemType");
        assert_eq!(schema.tables[0].name, "Item");

        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn allows_mixed_schema_include_formats() {
        let base = temp_dir();
        let schema_dir = base.join("schema");
        fs::create_dir_all(&schema_dir).unwrap();
        let project_path = base.join("project.yaml");
        fs::write(
            &project_path,
            r#"
package: game_config
includes:
  - schema/items.toml
"#,
        )
        .unwrap();
        fs::write(
            schema_dir.join("items.toml"),
            r#"
[[tables]]
name = "Item"
mode = "map"
key = "id"
"#,
        )
        .unwrap();

        let schema = load_project_schema_file(&project_path).unwrap();

        assert_eq!(schema.tables[0].name, "Item");

        let _ = fs::remove_dir_all(base);
    }

    fn temp_dir() -> PathBuf {
        let unique = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("sora-input-schema-test-{unique}"))
    }
}
