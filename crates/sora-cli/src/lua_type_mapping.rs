use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use mlua::{Lua, LuaOptions, StdLib, Table};
use serde::Deserialize;
use sora_codegen::type_mapping::{
    StaticTypeMappingProvider, StaticTypeMappingRule, TypeMappingRegistry,
};

pub fn load_type_mapping_registry(
    project: Option<&Path>,
    cli_paths: &[PathBuf],
) -> Result<TypeMappingRegistry> {
    let paths = type_mapping_script_paths(project, cli_paths)?;
    if paths.is_empty() {
        return Ok(TypeMappingRegistry::new());
    }

    let mut rules = Vec::new();
    let mut seen = BTreeSet::new();
    for path in paths {
        let source = fs::read_to_string(&path).with_context(|| {
            format!(
                "failed to read Lua type mapping script `{}`",
                path.display()
            )
        })?;
        for rule in discover_type_mappings(&path, &source)? {
            let key = (rule.target.clone(), rule.schema_type.clone());
            if !seen.insert(key.clone()) {
                bail!(
                    "duplicate type mapping for target `{}` and schema type `{}`",
                    key.0,
                    key.1
                );
            }
            rules.push(rule);
        }
    }

    if rules.is_empty() {
        bail!("Lua type mapping scripts did not register any type mappings");
    }

    let mut registry = TypeMappingRegistry::new();
    registry.register(StaticTypeMappingProvider::new(rules));
    Ok(registry)
}

fn type_mapping_script_paths(
    project: Option<&Path>,
    cli_paths: &[PathBuf],
) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    if let Some(project) = project {
        let config = ProjectTypeMappingDocument::load(project)?;
        let project_dir = project.parent().unwrap_or_else(|| Path::new("."));
        paths.extend(config.type_mappings.scripts.into_iter().map(|path| {
            if path.is_absolute() {
                path
            } else {
                project_dir.join(path)
            }
        }));
    }
    paths.extend(cli_paths.iter().cloned());
    Ok(paths)
}

#[derive(Debug, Default, Deserialize)]
struct ProjectTypeMappingDocument {
    #[serde(default)]
    type_mappings: ProjectTypeMappingConfig,
}

impl ProjectTypeMappingDocument {
    fn load(path: &Path) -> Result<Self> {
        sora_config_format::load_document(path).with_context(|| {
            format!(
                "failed to load type mapping config from project `{}`",
                path.display()
            )
        })
    }
}

#[derive(Debug, Default, Deserialize)]
struct ProjectTypeMappingConfig {
    #[serde(default)]
    scripts: Vec<PathBuf>,
}

fn discover_type_mappings(path: &Path, source: &str) -> Result<Vec<StaticTypeMappingRule>> {
    let lua = type_mapping_lua()?;
    let root: Table = lua
        .load(source)
        .set_name(path.display().to_string())
        .eval()
        .with_context(|| {
            format!(
                "failed to evaluate Lua type mapping script `{}`",
                path.display()
            )
        })?;
    let mappings: Table = root.get("type_mappings").with_context(|| {
        format!(
            "Lua type mapping script `{}` must return a table with `type_mappings`",
            path.display()
        )
    })?;

    let mut rules = Vec::new();
    for item in mappings.sequence_values::<Table>() {
        rules.push(lua_mapping_rule(path, item?)?);
    }
    Ok(rules)
}

fn lua_mapping_rule(path: &Path, table: Table) -> Result<StaticTypeMappingRule> {
    let target = required_string(&table, "target", path)?;
    let schema_type = required_string(&table, "schema_type", path)?;
    let type_name = required_string(&table, "type_name", path)?;
    Ok(StaticTypeMappingRule {
        target,
        schema_type,
        type_name,
        decode: optional_string(&table, "decode")?,
        value_decode: optional_string(&table, "value_decode")?,
        decode_into: optional_string(&table, "decode_into")?,
        free: optional_string(&table, "free")?,
        imports: optional_string_list(&table, "imports")?,
    })
}

fn required_string(table: &Table, key: &str, path: &Path) -> Result<String> {
    let value: String = table.get(key).with_context(|| {
        format!(
            "Lua type mapping in `{}` must define string field `{key}`",
            path.display()
        )
    })?;
    if value.trim().is_empty() {
        bail!(
            "Lua type mapping in `{}` declares empty `{key}`",
            path.display()
        );
    }
    Ok(value)
}

fn optional_string(table: &Table, key: &str) -> Result<Option<String>> {
    let Some(value) = table.get::<Option<String>>(key)? else {
        return Ok(None);
    };
    if value.trim().is_empty() {
        bail!("Lua type mapping declares empty `{key}`");
    }
    Ok(Some(value))
}

fn optional_string_list(table: &Table, key: &str) -> Result<Vec<String>> {
    let Some(values) = table.get::<Option<Table>>(key)? else {
        return Ok(Vec::new());
    };
    let mut out = Vec::new();
    for value in values.sequence_values::<String>() {
        let value = value?;
        if value.trim().is_empty() {
            bail!("Lua type mapping declares empty `{key}` entry");
        }
        out.push(value);
    }
    Ok(out)
}

fn type_mapping_lua() -> Result<Lua> {
    Lua::new_with(
        StdLib::TABLE | StdLib::STRING | StdLib::MATH | StdLib::UTF8,
        LuaOptions::default(),
    )
    .context("failed to create Lua type mapping runtime")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_lua_type_mapping_rules() {
        let rules = discover_type_mappings(
            Path::new("type_mappings.lua"),
            r#"
return {
  type_mappings = {
    {
      target = "csharp",
      schema_type = "Vec3",
      type_name = "Vector3",
      decode = "GameMappings.ToVector3({value})",
      value_decode = "GameMappings.ToVector3({value})",
      decode_into = "game_vector3_decode(reader, {target})",
      free = "game_vector3_free({target});",
      imports = { "UnityEngine" },
    },
  },
}
"#,
        )
        .unwrap();

        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].target, "csharp");
        assert_eq!(rules[0].schema_type, "Vec3");
        assert_eq!(rules[0].type_name, "Vector3");
        assert_eq!(
            rules[0].decode_into.as_deref(),
            Some("game_vector3_decode(reader, {target})")
        );
        assert_eq!(
            rules[0].free.as_deref(),
            Some("game_vector3_free({target});")
        );
        assert_eq!(rules[0].imports, ["UnityEngine"]);
    }
}
