use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use serde::Deserialize;
use sora_diagnostics::{Result, SoraError};
use sora_schema::model::{EnumSchema, SchemaFile, StructSchema, TableSchema};

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
        includes: root.includes.clone(),
        enums: root.enums,
        structs: root.structs,
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

        merged.enums.extend(module.enums);
        merged.structs.extend(module.structs);
        merged.tables.extend(module.tables);
        merge_includes(&include_path, &module.includes, merged, visited)?;
    }

    Ok(())
}

fn load_schema_document(path: &Path) -> Result<TomlSchemaDocument> {
    let content = fs::read_to_string(path).map_err(|source| SoraError::ReadFile {
        path: path.to_path_buf(),
        source,
    })?;

    toml::from_str(&content).map_err(|source: toml::de::Error| SoraError::ParseSchema {
        path: path.to_path_buf(),
        message: source.to_string(),
    })
}

#[derive(Debug, Deserialize)]
struct TomlSchemaDocument {
    pub package: Option<String>,

    #[serde(default)]
    pub includes: Vec<String>,

    #[serde(default)]
    pub enums: Vec<EnumSchema>,

    #[serde(default)]
    pub structs: Vec<StructSchema>,

    #[serde(default)]
    pub tables: Vec<TableSchema>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn loads_project_schema_with_includes() {
        let base = temp_dir();
        let schema_dir = base.join("schema");
        fs::create_dir_all(&schema_dir).unwrap();
        let project_path = base.join("project.toml");
        fs::write(
            &project_path,
            r#"
package = "game_config"
includes = ["schema/items.toml"]
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
        assert_eq!(schema.includes, ["schema/items.toml"]);
        assert_eq!(schema.enums[0].name, "ItemType");
        assert_eq!(schema.tables[0].name, "Item");

        let _ = fs::remove_dir_all(base);
    }

    fn temp_dir() -> PathBuf {
        let unique = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("sora-input-toml-schema-test-{unique}"))
    }
}
