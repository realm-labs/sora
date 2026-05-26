use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

use serde::Deserialize;
use sora_config_format::{DocumentError, load_document};
use sora_diagnostics::{Result, SoraError};
use sora_schema::model::{
    CodegenSchema, EnumSchema, LocalizationSchema, SchemaFile, StructSchema, TableSchema,
    UnionSchema,
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
        localization: root.localization,
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
        if module.localization.is_some() {
            return Err(SoraError::InvalidSchema(format!(
                "included schema module `{}` must not declare `localization`",
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
    load_document(path).map_err(schema_document_error)
}

fn schema_document_error(error: DocumentError) -> SoraError {
    match error {
        DocumentError::Read { path, source } => SoraError::ReadFile { path, source },
        DocumentError::Parse { path, message } => SoraError::ParseSchema { path, message },
        DocumentError::UnsupportedExtension { path, extension } => {
            SoraError::InvalidSchema(format!(
                "schema file `{}` has unsupported extension `{extension}`",
                path.display()
            ))
        }
        DocumentError::MissingExtension { path } => SoraError::InvalidSchema(format!(
            "schema file `{}` must have an extension",
            path.display()
        )),
    }
}

#[derive(Debug, Deserialize)]
struct SchemaDocument {
    pub package: Option<String>,
    pub codegen: Option<CodegenSchema>,
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        sync::atomic::{AtomicU64, Ordering},
    };

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
