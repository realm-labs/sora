use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use sora_data::model::{RowData, TableData, Value};
use sora_diagnostics::{Result, SoraError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructuredFormat {
    Json,
    Yaml,
}

impl StructuredFormat {
    pub fn extensions(self) -> &'static [&'static str] {
        match self {
            Self::Json => &["json"],
            Self::Yaml => &["yaml", "yml"],
        }
    }

    fn parse_rows(self, path: &Path, content: &str) -> Result<Vec<RowData>> {
        match self {
            Self::Json => {
                let value =
                    serde_json::from_str(content).map_err(|source| SoraError::ParseData {
                        path: path.to_path_buf(),
                        message: source.to_string(),
                    })?;
                rows_from_json_value(path, value)
            }
            Self::Yaml => {
                let value =
                    serde_yaml::from_str(content).map_err(|source| SoraError::ParseData {
                        path: path.to_path_buf(),
                        message: source.to_string(),
                    })?;
                rows_from_yaml_value(path, value)
            }
        }
    }

    fn parse_row(self, path: &Path, content: &str) -> Result<RowData> {
        match self {
            Self::Json => {
                let value =
                    serde_json::from_str(content).map_err(|source| SoraError::ParseData {
                        path: path.to_path_buf(),
                        message: source.to_string(),
                    })?;
                row_from_json_value(path, value)
            }
            Self::Yaml => {
                let value =
                    serde_yaml::from_str(content).map_err(|source| SoraError::ParseData {
                        path: path.to_path_buf(),
                        message: source.to_string(),
                    })?;
                row_from_yaml_value(path, value)
            }
        }
    }
}

pub fn load_json_table_data(table_name: &str, path: &Path) -> Result<TableData> {
    load_structured_table_data(table_name, path, StructuredFormat::Json)
}

pub fn load_yaml_table_data(table_name: &str, path: &Path) -> Result<TableData> {
    load_structured_table_data(table_name, path, StructuredFormat::Yaml)
}

pub fn load_structured_table_data(
    table_name: &str,
    path: &Path,
    format: StructuredFormat,
) -> Result<TableData> {
    let rows = if path.is_dir() {
        load_structured_table_data_dir(path, format)?
    } else {
        let content = read_to_string(path)?;
        format.parse_rows(path, &content)?
    };

    Ok(TableData {
        name: table_name.to_owned(),
        rows,
    })
}

fn load_structured_table_data_dir(path: &Path, format: StructuredFormat) -> Result<Vec<RowData>> {
    let mut files = Vec::new();
    collect_files(path, format.extensions(), &mut files)?;
    files.sort();

    files
        .into_iter()
        .map(|file| {
            let content = read_to_string(&file)?;
            format.parse_row(&file, &content)
        })
        .collect()
}

fn collect_files(path: &Path, extensions: &[&str], out: &mut Vec<PathBuf>) -> Result<()> {
    let entries = fs::read_dir(path).map_err(|source| SoraError::ReadFile {
        path: path.to_path_buf(),
        source,
    })?;

    for entry in entries {
        let entry = entry.map_err(|source| SoraError::ReadFile {
            path: path.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        if path.is_dir() {
            collect_files(&path, extensions, out)?;
        } else if has_extension(&path, extensions) {
            out.push(path);
        }
    }

    Ok(())
}

fn has_extension(path: &Path, extensions: &[&str]) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extensions.contains(&extension))
}

fn read_to_string(path: &Path) -> Result<String> {
    fs::read_to_string(path).map_err(|source| SoraError::ReadFile {
        path: path.to_path_buf(),
        source,
    })
}

fn rows_from_json_value(path: &Path, value: serde_json::Value) -> Result<Vec<RowData>> {
    let serde_json::Value::Array(rows) = value else {
        return Err(SoraError::ParseData {
            path: path.to_path_buf(),
            message: "JSON table data must be an array of row objects".to_owned(),
        });
    };
    rows.into_iter()
        .map(|value| row_from_json_value(path, value))
        .collect()
}

fn row_from_json_value(path: &Path, value: serde_json::Value) -> Result<RowData> {
    let serde_json::Value::Object(values) = value else {
        return Err(SoraError::ParseData {
            path: path.to_path_buf(),
            message: "JSON row data must be an object".to_owned(),
        });
    };
    Ok(RowData {
        values: values
            .into_iter()
            .map(|(key, value)| Ok((key, json_value_to_data_value(path, value)?)))
            .collect::<Result<BTreeMap<_, _>>>()?,
    })
}

fn json_value_to_data_value(path: &Path, value: serde_json::Value) -> Result<Value> {
    Ok(match value {
        serde_json::Value::Null => Value::Null,
        serde_json::Value::Bool(value) => Value::Bool(value),
        serde_json::Value::Number(value) => {
            if let Some(value) = value.as_i64() {
                Value::Integer(value)
            } else if let Some(value) = value.as_f64() {
                Value::Float(value)
            } else {
                return Err(SoraError::ParseData {
                    path: path.to_path_buf(),
                    message: "JSON number is outside supported range".to_owned(),
                });
            }
        }
        serde_json::Value::String(value) => Value::String(value),
        serde_json::Value::Array(values) => Value::List(
            values
                .into_iter()
                .map(|value| json_value_to_data_value(path, value))
                .collect::<Result<Vec<_>>>()?,
        ),
        serde_json::Value::Object(values) => Value::Object(
            values
                .into_iter()
                .map(|(key, value)| Ok((key, json_value_to_data_value(path, value)?)))
                .collect::<Result<BTreeMap<_, _>>>()?,
        ),
    })
}

fn rows_from_yaml_value(path: &Path, value: serde_yaml::Value) -> Result<Vec<RowData>> {
    let serde_yaml::Value::Sequence(rows) = value else {
        return Err(SoraError::ParseData {
            path: path.to_path_buf(),
            message: "YAML table data must be a sequence of row mappings".to_owned(),
        });
    };
    rows.into_iter()
        .map(|value| row_from_yaml_value(path, value))
        .collect()
}

fn row_from_yaml_value(path: &Path, value: serde_yaml::Value) -> Result<RowData> {
    let serde_yaml::Value::Mapping(values) = value else {
        return Err(SoraError::ParseData {
            path: path.to_path_buf(),
            message: "YAML row data must be a mapping".to_owned(),
        });
    };

    let mut row = BTreeMap::new();
    for (key, value) in values {
        let serde_yaml::Value::String(key) = key else {
            return Err(SoraError::ParseData {
                path: path.to_path_buf(),
                message: "YAML row keys must be strings".to_owned(),
            });
        };
        row.insert(key, yaml_value_to_data_value(path, value)?);
    }
    Ok(RowData { values: row })
}

fn yaml_value_to_data_value(path: &Path, value: serde_yaml::Value) -> Result<Value> {
    Ok(match value {
        serde_yaml::Value::Null => Value::Null,
        serde_yaml::Value::Bool(value) => Value::Bool(value),
        serde_yaml::Value::Number(value) => {
            if let Some(value) = value.as_i64() {
                Value::Integer(value)
            } else if let Some(value) = value.as_f64() {
                Value::Float(value)
            } else {
                return Err(SoraError::ParseData {
                    path: path.to_path_buf(),
                    message: "YAML number is outside supported range".to_owned(),
                });
            }
        }
        serde_yaml::Value::String(value) => Value::String(value),
        serde_yaml::Value::Sequence(values) => Value::List(
            values
                .into_iter()
                .map(|value| yaml_value_to_data_value(path, value))
                .collect::<Result<Vec<_>>>()?,
        ),
        serde_yaml::Value::Mapping(values) => {
            let mut object = BTreeMap::new();
            for (key, value) in values {
                let serde_yaml::Value::String(key) = key else {
                    return Err(SoraError::ParseData {
                        path: path.to_path_buf(),
                        message: "YAML object keys must be strings".to_owned(),
                    });
                };
                object.insert(key, yaml_value_to_data_value(path, value)?);
            }
            Value::Object(object)
        }
        serde_yaml::Value::Tagged(value) => yaml_value_to_data_value(path, value.value)?,
    })
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        sync::atomic::{AtomicU64, Ordering},
    };

    use super::*;

    static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn loads_json_table_data_file() {
        let base = temp_dir();
        fs::create_dir_all(&base).unwrap();
        let path = base.join("items.json");
        fs::write(
            &path,
            r#"
[
  {"id": 1001, "name": "Iron Sword", "tags": ["weapon", "starter"]},
  {"id": 1002, "name": "Health Potion", "price": 12.5}
]
"#,
        )
        .unwrap();

        let table = load_json_table_data("Item", &path).unwrap();

        assert_eq!(table.name, "Item");
        assert_eq!(table.rows.len(), 2);
        assert_eq!(table.rows[0].values["id"], Value::Integer(1001));
        assert_eq!(
            table.rows[0].values["tags"],
            Value::List(vec![
                Value::String("weapon".to_owned()),
                Value::String("starter".to_owned())
            ])
        );
        assert_eq!(table.rows[1].values["price"], Value::Float(12.5));

        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn loads_yaml_table_data_file() {
        let base = temp_dir();
        fs::create_dir_all(&base).unwrap();
        let path = base.join("items.yaml");
        fs::write(
            &path,
            r#"
- id: 1001
  name: Iron Sword
- id: 1002
  enabled: true
"#,
        )
        .unwrap();

        let table = load_yaml_table_data("Item", &path).unwrap();

        assert_eq!(table.rows.len(), 2);
        assert_eq!(
            table.rows[0].values["name"],
            Value::String("Iron Sword".to_owned())
        );
        assert_eq!(table.rows[1].values["enabled"], Value::Bool(true));

        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn loads_structured_directory_as_one_row_per_file() {
        let base = temp_dir();
        let dir = base.join("items");
        fs::create_dir_all(dir.join("nested")).unwrap();
        fs::write(dir.join("1002.json"), r#"{"id": 1002, "name": "Potion"}"#).unwrap();
        fs::write(
            dir.join("nested/1001.json"),
            r#"{"id": 1001, "name": "Sword"}"#,
        )
        .unwrap();
        fs::write(dir.join("ignored.yaml"), "id: 9999").unwrap();

        let table = load_json_table_data("Item", &dir).unwrap();

        assert_eq!(table.rows.len(), 2);
        assert_eq!(table.rows[0].values["id"], Value::Integer(1002));
        assert_eq!(table.rows[1].values["id"], Value::Integer(1001));

        let _ = fs::remove_dir_all(base);
    }

    fn temp_dir() -> PathBuf {
        let unique = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("sora-input-structured-test-{unique}"))
    }
}
