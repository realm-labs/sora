use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use serde::Deserialize;
use sora_data::{ConfigData, RowData, TableData, Value};
use sora_diagnostics::{Result, SoraError};
use sora_input::{DataInput, SchemaInput};
use sora_ir::ConfigIr;
use sora_schema::SchemaFile;

#[derive(Debug, Clone)]
pub struct TomlSchemaInput {
    schema_path: PathBuf,
}

impl TomlSchemaInput {
    pub fn new(schema_path: impl Into<PathBuf>) -> Self {
        Self {
            schema_path: schema_path.into(),
        }
    }

    pub fn schema_path(&self) -> &Path {
        &self.schema_path
    }
}

impl SchemaInput for TomlSchemaInput {
    fn load_schema(&self) -> Result<SchemaFile> {
        load_schema_file(&self.schema_path)
    }
}

#[derive(Debug, Clone)]
pub struct TomlProjectInput {
    schema_path: PathBuf,
    data_root: PathBuf,
}

impl TomlProjectInput {
    pub fn new(schema_path: impl Into<PathBuf>, data_root: impl Into<PathBuf>) -> Self {
        Self {
            schema_path: schema_path.into(),
            data_root: data_root.into(),
        }
    }

    pub fn schema_path(&self) -> &Path {
        &self.schema_path
    }

    pub fn data_root(&self) -> &Path {
        &self.data_root
    }
}

impl SchemaInput for TomlProjectInput {
    fn load_schema(&self) -> Result<SchemaFile> {
        load_schema_file(&self.schema_path)
    }
}

impl DataInput for TomlProjectInput {
    fn load_data(&self, ir: &ConfigIr) -> Result<ConfigData> {
        load_config_data(ir, &self.data_root)
    }
}

pub fn load_schema_file(path: &Path) -> Result<SchemaFile> {
    let content = fs::read_to_string(path).map_err(|source| SoraError::ReadFile {
        path: path.to_path_buf(),
        source,
    })?;

    toml::from_str(&content).map_err(|source: toml::de::Error| SoraError::ParseSchema {
        path: path.to_path_buf(),
        message: source.to_string(),
    })
}

pub fn load_config_data(ir: &ConfigIr, data_root: &Path) -> Result<ConfigData> {
    let mut tables = Vec::new();

    for table in &ir.tables {
        let source = table
            .source
            .as_ref()
            .ok_or_else(|| SoraError::MissingTableSource {
                table: table.name.clone(),
            })?;
        tables.push(load_table_data_file(&table.name, &data_root.join(source))?);
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
    use sora_ir::normalize_schema;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn loads_toml_schema_file() {
        let path = write_temp_file(
            "schema",
            r#"
package = "game_config"

[[tables]]
name = "Item"
mode = "map"
key = "id"
"#,
        );

        let schema = load_schema_file(&path).unwrap();

        assert_eq!(schema.package, "game_config");
        assert_eq!(schema.tables[0].name, "Item");

        let _ = fs::remove_file(path);
    }

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
        let schema_path = base.join("schema.toml");
        fs::write(
            &schema_path,
            r#"
package = "game_config"

[[tables]]
name = "Item"
mode = "map"
key = "id"
source = "items.toml"

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

        let input = TomlProjectInput::new(&schema_path, &data_dir);
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
