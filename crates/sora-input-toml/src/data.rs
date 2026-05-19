use std::{collections::BTreeMap, fs, path::Path};

use serde::Deserialize;
use sora_data::model::{ConfigData, RowData, TableData, Value};
use sora_diagnostics::{Result, SoraError};
use sora_ir::model::ConfigIr;

pub fn load_config_data(ir: &ConfigIr, data_root: &Path) -> Result<ConfigData> {
    let mut tables = Vec::new();

    for table in &ir.tables {
        let source = table
            .source
            .as_ref()
            .ok_or_else(|| SoraError::MissingTableSource {
                table: table.name.clone(),
            })?;
        if source.format != "toml" {
            return Err(SoraError::InvalidSchema(format!(
                "table `{}` source format `{}` cannot be loaded by TOML input adapter",
                table.name, source.format
            )));
        }
        tables.push(load_table_data_file(
            &table.name,
            &data_root.join(&source.file),
        )?);
    }

    Ok(ConfigData { tables })
}

pub fn load_table_data_file(table_name: &str, path: &Path) -> Result<TableData> {
    let content = fs::read_to_string(path).map_err(|source| SoraError::ReadFile {
        path: path.to_path_buf(),
        source,
    })?;
    let parsed: TomlRows =
        toml::from_str(&content).map_err(|source: toml::de::Error| SoraError::ParseData {
            path: path.to_path_buf(),
            message: source.to_string(),
        })?;

    Ok(TableData {
        name: table_name.to_owned(),
        rows: parsed
            .rows
            .into_iter()
            .map(|row| {
                Ok(RowData {
                    values: row
                        .into_iter()
                        .map(|(key, value)| Ok((key, convert_toml_value(value)?)))
                        .collect::<Result<BTreeMap<_, _>>>()?,
                })
            })
            .collect::<Result<Vec<_>>>()?,
    })
}

#[derive(Debug, Deserialize)]
struct TomlRows {
    #[serde(default)]
    rows: Vec<BTreeMap<String, toml::Value>>,
}

fn convert_toml_value(value: toml::Value) -> Result<Value> {
    Ok(match value {
        toml::Value::String(value) => Value::String(value),
        toml::Value::Integer(value) => Value::Integer(value),
        toml::Value::Float(value) => Value::Float(value),
        toml::Value::Boolean(value) => Value::Bool(value),
        toml::Value::Array(values) => Value::List(
            values
                .into_iter()
                .map(convert_toml_value)
                .collect::<Result<Vec<_>>>()?,
        ),
        toml::Value::Table(values) => Value::Object(
            values
                .into_iter()
                .map(|(key, value)| Ok((key, convert_toml_value(value)?)))
                .collect::<Result<BTreeMap<_, _>>>()?,
        ),
        toml::Value::Datetime(value) => Value::String(value.to_string()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::TomlProjectInput;
    use sora_input::traits::{DataInput, SchemaInput};
    use sora_ir::normalize::normalize_schema;
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    #[test]
    fn loads_toml_table_data_file() {
        let path = write_temp_file(
            "data",
            r#"
[[rows]]
id = 1001
name = "Iron Sword"
"#,
        );

        let table = load_table_data_file("Item", &path).unwrap();

        assert_eq!(table.name, "Item");
        assert_eq!(table.rows.len(), 1);
        assert_eq!(table.rows[0].values["id"], Value::Integer(1001));
        assert_eq!(
            table.rows[0].values["name"],
            Value::String("Iron Sword".to_owned())
        );

        let _ = fs::remove_file(path);
    }

    #[test]
    fn toml_project_input_loads_schema_and_data() {
        let base = temp_dir();
        let data_dir = base.join("data");
        fs::create_dir_all(&data_dir).unwrap();
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
required = true
"#,
        )
        .unwrap();
        fs::write(
            data_dir.join("items.toml"),
            r#"
[[rows]]
id = 1001
"#,
        )
        .unwrap();

        let input = TomlProjectInput::new(&project_path, &data_dir);
        let ir = normalize_schema(input.load_schema().unwrap()).unwrap();
        let data = input.load_data(&ir).unwrap();

        assert_eq!(data.tables[0].rows[0].values["id"], Value::Integer(1001));

        let _ = fs::remove_dir_all(base);
    }

    fn write_temp_file(prefix: &str, content: &str) -> PathBuf {
        let path = temp_dir().join(format!("{prefix}.toml"));
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, content).unwrap();
        path
    }

    fn temp_dir() -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("sora-input-toml-test-{unique}"))
    }
}
