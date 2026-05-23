use std::{
    fs, io,
    path::{Path, PathBuf},
};

use mlua::{Lua, LuaOptions, LuaSerdeExt, StdLib};
use serde::de::DeserializeOwned;

#[derive(Debug, thiserror::Error)]
pub enum DocumentError {
    #[error("failed to read file `{path}`: {source}")]
    Read { path: PathBuf, source: io::Error },

    #[error("failed to parse `{path}`: {message}")]
    Parse { path: PathBuf, message: String },

    #[error("file `{path}` has unsupported extension `{extension}`")]
    UnsupportedExtension { path: PathBuf, extension: String },

    #[error("file `{path}` must have an extension")]
    MissingExtension { path: PathBuf },
}

pub type Result<T> = std::result::Result<T, DocumentError>;

pub fn load_document<T>(path: &Path) -> Result<T>
where
    T: DeserializeOwned,
{
    let content = fs::read_to_string(path).map_err(|source| DocumentError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    parse_document(path, &content)
}

pub fn parse_document<T>(path: &Path, content: &str) -> Result<T>
where
    T: DeserializeOwned,
{
    match document_format(path)? {
        DocumentFormat::Toml => parse_toml(path, content),
        DocumentFormat::Yaml => parse_yaml(path, content),
        DocumentFormat::Json => parse_json(path, content),
        DocumentFormat::Lua => parse_lua(path, content),
    }
}

fn parse_toml<T>(path: &Path, content: &str) -> Result<T>
where
    T: DeserializeOwned,
{
    toml::from_str(content).map_err(|source| DocumentError::Parse {
        path: path.to_path_buf(),
        message: source.to_string(),
    })
}

fn parse_yaml<T>(path: &Path, content: &str) -> Result<T>
where
    T: DeserializeOwned,
{
    serde_yaml::from_str(content).map_err(|source| DocumentError::Parse {
        path: path.to_path_buf(),
        message: source.to_string(),
    })
}

fn parse_json<T>(path: &Path, content: &str) -> Result<T>
where
    T: DeserializeOwned,
{
    serde_json::from_str(content).map_err(|source| DocumentError::Parse {
        path: path.to_path_buf(),
        message: source.to_string(),
    })
}

fn parse_lua<T>(path: &Path, content: &str) -> Result<T>
where
    T: DeserializeOwned,
{
    let lua = Lua::new_with(
        StdLib::TABLE | StdLib::STRING | StdLib::MATH | StdLib::UTF8,
        LuaOptions::default(),
    )
    .map_err(|source| DocumentError::Parse {
        path: path.to_path_buf(),
        message: source.to_string(),
    })?;
    let value = lua
        .load(content)
        .eval()
        .map_err(|source| DocumentError::Parse {
            path: path.to_path_buf(),
            message: source.to_string(),
        })?;

    lua.from_value(value)
        .map_err(|source| DocumentError::Parse {
            path: path.to_path_buf(),
            message: source.to_string(),
        })
}

fn document_format(path: &Path) -> Result<DocumentFormat> {
    match path.extension().and_then(|extension| extension.to_str()) {
        Some("toml") => Ok(DocumentFormat::Toml),
        Some("yaml" | "yml") => Ok(DocumentFormat::Yaml),
        Some("json") => Ok(DocumentFormat::Json),
        Some("lua") => Ok(DocumentFormat::Lua),
        Some(extension) => Err(DocumentError::UnsupportedExtension {
            path: path.to_path_buf(),
            extension: extension.to_owned(),
        }),
        None => Err(DocumentError::MissingExtension {
            path: path.to_path_buf(),
        }),
    }
}

#[derive(Debug, Clone, Copy)]
enum DocumentFormat {
    Toml,
    Yaml,
    Json,
    Lua,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    struct Document {
        package: String,
        includes: Vec<String>,
    }

    #[test]
    fn parses_supported_document_formats() {
        let toml: Document = parse_document(
            Path::new("project.toml"),
            r#"
package = "game_config"
includes = ["schema/items.toml"]
"#,
        )
        .unwrap();
        let yaml: Document = parse_document(
            Path::new("project.yaml"),
            r#"
package: game_config
includes:
  - schema/items.yaml
"#,
        )
        .unwrap();
        let json: Document = parse_document(
            Path::new("project.json"),
            r#"
{
  "package": "game_config",
  "includes": ["schema/items.json"]
}
"#,
        )
        .unwrap();
        let lua: Document = parse_document(
            Path::new("project.lua"),
            r#"
return {
  package = "game_config",
  includes = { "schema/items.lua" },
}
"#,
        )
        .unwrap();

        assert_eq!(toml.package, "game_config");
        assert_eq!(yaml.includes, ["schema/items.yaml"]);
        assert_eq!(json.includes, ["schema/items.json"]);
        assert_eq!(lua.includes, ["schema/items.lua"]);
    }
}
