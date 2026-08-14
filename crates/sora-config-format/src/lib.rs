use std::{
    fs, io,
    path::{Path, PathBuf},
};

use mlua::{Lua, LuaOptions, LuaSerdeExt, StdLib};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Map, Value};

#[derive(Debug, thiserror::Error)]
pub enum DocumentError {
    #[error("failed to read file `{path}`: {source}")]
    Read { path: PathBuf, source: io::Error },

    #[error("failed to parse `{path}`: {message}")]
    Parse { path: PathBuf, message: String },

    #[error("failed to render `{path}`: {message}")]
    Render { path: PathBuf, message: String },

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
        DocumentFormat::Scon => parse_scon(path, content),
    }
}

pub fn render_document<T>(path: &Path, value: &T) -> Result<String>
where
    T: Serialize,
{
    match document_format(path)? {
        DocumentFormat::Toml => {
            toml::to_string_pretty(value).map_err(|source| DocumentError::Render {
                path: path.to_path_buf(),
                message: source.to_string(),
            })
        }
        DocumentFormat::Yaml => {
            serde_yaml::to_string(value).map_err(|source| DocumentError::Render {
                path: path.to_path_buf(),
                message: source.to_string(),
            })
        }
        DocumentFormat::Json => serde_json::to_string_pretty(value)
            .map(|mut text| {
                text.push('\n');
                text
            })
            .map_err(|source| DocumentError::Render {
                path: path.to_path_buf(),
                message: source.to_string(),
            }),
        DocumentFormat::Lua => render_lua(path, value),
        DocumentFormat::Scon => scon::to_string(value).map_err(|source| DocumentError::Render {
            path: path.to_path_buf(),
            message: source.to_string(),
        }),
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

fn parse_scon<T>(path: &Path, content: &str) -> Result<T>
where
    T: DeserializeOwned,
{
    let analysis = scon::analyze_source(
        content,
        scon::ParseOptions {
            file: Some(path.to_path_buf()),
        },
    );
    if let Some(include) = analysis.includes.first() {
        return Err(DocumentError::Parse {
            path: path.to_path_buf(),
            message: format!(
                "native SCON include is not supported in Sora documents at {}:{}; use the `includes` property",
                include.range.start.line + 1,
                include.range.start.character + 1
            ),
        });
    }
    if let Some(reference) = analysis.references.first() {
        return Err(DocumentError::Parse {
            path: path.to_path_buf(),
            message: format!(
                "SCON substitution, interpolation, and spread are not supported in Sora documents at {}:{}",
                reference.range.start.line + 1,
                reference.range.start.character + 1
            ),
        });
    }
    if let Some(diagnostic) = analysis.diagnostics.first() {
        let location = diagnostic.range.as_ref().map_or_else(String::new, |range| {
            format!(" at {}:{}", range.start.line + 1, range.start.character + 1)
        });
        return Err(DocumentError::Parse {
            path: path.to_path_buf(),
            message: format!("{:?}{location}: {}", diagnostic.code, diagnostic.message),
        });
    }
    let value = analysis.value.ok_or_else(|| DocumentError::Parse {
        path: path.to_path_buf(),
        message: "SCON analysis produced no value".to_owned(),
    })?;
    scon::from_value(value).map_err(|source| DocumentError::Parse {
        path: path.to_path_buf(),
        message: source.to_string(),
    })
}

fn render_lua<T>(path: &Path, value: &T) -> Result<String>
where
    T: Serialize,
{
    let value = serde_json::to_value(value).map_err(|source| DocumentError::Render {
        path: path.to_path_buf(),
        message: source.to_string(),
    })?;
    let mut output = String::from("return ");
    push_lua_value(&mut output, &value, 0);
    output.push('\n');
    Ok(output)
}

fn push_lua_value(output: &mut String, value: &Value, indent: usize) {
    match value {
        Value::Null => output.push_str("nil"),
        Value::Bool(value) => output.push_str(if *value { "true" } else { "false" }),
        Value::Number(value) => output.push_str(&value.to_string()),
        Value::String(value) => push_lua_string(output, value),
        Value::Array(values) => {
            output.push_str("{\n");
            for value in values {
                push_indent(output, indent + 1);
                push_lua_value(output, value, indent + 1);
                output.push_str(",\n");
            }
            push_indent(output, indent);
            output.push('}');
        }
        Value::Object(values) => push_lua_object(output, values, indent),
    }
}

fn push_lua_object(output: &mut String, values: &Map<String, Value>, indent: usize) {
    output.push_str("{\n");
    for (key, value) in values {
        push_indent(output, indent + 1);
        if is_lua_identifier(key) {
            output.push_str(key);
        } else {
            output.push('[');
            push_lua_string(output, key);
            output.push(']');
        }
        output.push_str(" = ");
        push_lua_value(output, value, indent + 1);
        output.push_str(",\n");
    }
    push_indent(output, indent);
    output.push('}');
}

fn push_lua_string(output: &mut String, value: &str) {
    output.push('"');
    for ch in value.chars() {
        match ch {
            '\\' => output.push_str("\\\\"),
            '"' => output.push_str("\\\""),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            ch => output.push(ch),
        }
    }
    output.push('"');
}

fn push_indent(output: &mut String, indent: usize) {
    for _ in 0..indent {
        output.push_str("  ");
    }
}

fn is_lua_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first == '_' || first.is_ascii_alphabetic())
        && chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
}

fn document_format(path: &Path) -> Result<DocumentFormat> {
    match path.extension().and_then(|extension| extension.to_str()) {
        Some("toml") => Ok(DocumentFormat::Toml),
        Some("yaml" | "yml") => Ok(DocumentFormat::Yaml),
        Some("json") => Ok(DocumentFormat::Json),
        Some("lua") => Ok(DocumentFormat::Lua),
        Some("scon") => Ok(DocumentFormat::Scon),
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
    Scon,
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
