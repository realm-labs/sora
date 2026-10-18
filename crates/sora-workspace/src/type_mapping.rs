use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use mlua::{Lua, LuaOptions, StdLib, Table};
use sora_codegen::type_mapping::{
    StaticTypeMappingProvider, StaticTypeMappingRule, TypeMappingRegistry,
};

use crate::scripts::ScriptSource;

pub fn load_type_mapping_registry(paths: &[PathBuf]) -> Result<TypeMappingRegistry> {
    let scripts = paths
        .iter()
        .map(|path| {
            ScriptSource::read(path).with_context(|| {
                format!(
                    "failed to read Lua type mapping script `{}`",
                    path.display()
                )
            })
        })
        .collect::<Result<Vec<_>>>()?;
    load_type_mapping_registry_from_sources(&scripts)
}

pub(crate) fn load_type_mapping_registry_from_sources(
    scripts: &[ScriptSource],
) -> Result<TypeMappingRegistry> {
    if scripts.is_empty() {
        return Ok(TypeMappingRegistry::new());
    }

    let mut rules = Vec::new();
    let mut seen = BTreeSet::new();
    for source in scripts {
        for rule in discover_type_mappings(source.path(), source.source())? {
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
        nullable_type_name: optional_string(&table, "nullable_type_name")?,
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
    use std::{env::temp_dir as system_temp_dir, fs};

    use sora_codegen::type_mapping::TypeMappingContext;
    use sora_ir::model::{ConfigIr, TypeIr};
    use uuid::Uuid;

    use super::*;

    #[test]
    fn type_mapping_sources_ignore_replaced_or_missing_files() {
        let source = r#"
return { type_mappings = { {
  target = "csharp",
  schema_type = "Vec3",
  type_name = "MemoryVector3",
  imports = { "MemoryTypes" },
} } }
"#;
        let ir = ConfigIr {
            project_id: "snapshot".to_owned(),
            contract_id: "snapshot".to_owned(),
            view: None,
            group_defaults: Default::default(),
            views: Default::default(),
            localization: None,
            enums: Vec::new(),
            structs: Vec::new(),
            unions: Vec::new(),
            tables: Vec::new(),
        };
        for missing in [false, true] {
            let path =
                system_temp_dir().join(format!("sora-mapping-snapshot-{}.lua", Uuid::new_v4()));
            fs::write(&path, source).unwrap();
            let script = ScriptSource::new(path.clone(), source.to_owned());
            if missing {
                fs::remove_file(&path).unwrap();
            } else {
                fs::write(&path, source.replace("Memory", "Disk")).unwrap();
            }

            let registry = load_type_mapping_registry_from_sources(&[script]).unwrap();
            if !missing {
                fs::remove_file(&path).unwrap();
            }
            let mapping = registry
                .map_type(TypeMappingContext {
                    target: "csharp",
                    ir: &ir,
                    ty: &TypeIr::Struct("Vec3".to_owned()),
                })
                .unwrap();
            assert_eq!(mapping.type_name, "MemoryVector3");
            assert_eq!(mapping.imports, ["MemoryTypes"]);
        }
    }

    #[test]
    fn type_mapping_sources_reject_duplicate_rules() {
        let script = ScriptSource::new(
            PathBuf::from("missing.lua"),
            r#"return { type_mappings = { {
                target = "csharp", schema_type = "Vec3", type_name = "Vector3",
            } } }"#
                .to_owned(),
        );
        let error = load_type_mapping_registry_from_sources(&[script.clone(), script])
            .err()
            .expect("duplicate rules must fail");
        assert!(
            error
                .to_string()
                .contains("duplicate type mapping for target `csharp` and schema type `Vec3`")
        );
    }

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
      nullable_type_name = "Vector3?",
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
        assert_eq!(rules[0].nullable_type_name.as_deref(), Some("Vector3?"));
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
