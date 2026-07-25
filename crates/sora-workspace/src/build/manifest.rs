use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sora_config_format::load_document;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectManifest {
    pub project: ManifestProject,
    #[serde(default)]
    pub includes: Vec<String>,
    #[serde(default)]
    pub parsers: ScriptConfig,
    #[serde(default)]
    pub type_mappings: ScriptConfig,
    #[serde(default)]
    pub build: BuildConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestProject {
    pub id: String,
    #[serde(default)]
    pub views: Vec<String>,
}

impl ProjectManifest {
    pub fn load(path: &Path) -> Result<Self> {
        load_document(path)
            .with_context(|| format!("failed to load build config from `{}`", path.display()))
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScriptConfig {
    #[serde(default)]
    pub scripts: Vec<PathBuf>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BuildConfig {
    pub default_source_format: Option<SourceFormat>,
    pub data_root: Option<PathBuf>,
    pub view: Option<String>,
    pub schema_lock: Option<PathBuf>,
    pub excel_templates: Option<PathBuf>,

    #[serde(default)]
    pub codegen: Vec<BuildCodegen>,

    #[serde(default)]
    pub exports: Vec<BuildExport>,
}

impl BuildConfig {
    pub fn is_empty(&self) -> bool {
        self.schema_lock.is_none()
            && self.excel_templates.is_none()
            && self.codegen.is_empty()
            && self.exports.is_empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildCodegen {
    pub target: String,
    pub out: PathBuf,
    pub view: Option<String>,
    #[serde(default)]
    pub format: CodeFormatMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildExport {
    pub format: String,
    pub out: PathBuf,
    pub view: Option<String>,
    pub locale: Option<String>,
    #[serde(default)]
    pub compression: ExportCompression,
    pub compression_level: Option<i32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportCompression {
    #[default]
    None,
    Zstd,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceFormat {
    Csv,
    Json,
    Toml,
    Xlsx,
    Yaml,
}

impl SourceFormat {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Csv => "csv",
            Self::Json => "json",
            Self::Toml => "toml",
            Self::Xlsx => "xlsx",
            Self::Yaml => "yaml",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CodeFormatMode {
    #[default]
    Never,
    Auto,
    Required,
}

impl<'de> Deserialize<'de> for SourceFormat {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "csv" => Ok(Self::Csv),
            "json" => Ok(Self::Json),
            "toml" => Ok(Self::Toml),
            "xlsx" => Ok(Self::Xlsx),
            "yaml" | "yml" => Ok(Self::Yaml),
            _ => Err(serde::de::Error::custom(format!(
                "unsupported source format `{value}`; expected csv, json, toml, xlsx, or yaml"
            ))),
        }
    }
}

impl<'de> Deserialize<'de> for ExportCompression {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "none" => Ok(Self::None),
            "zstd" => Ok(Self::Zstd),
            _ => Err(serde::de::Error::custom(format!(
                "unsupported export compression `{value}`; expected none or zstd"
            ))),
        }
    }
}

impl<'de> Deserialize<'de> for CodeFormatMode {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "never" => Ok(Self::Never),
            "auto" => Ok(Self::Auto),
            "required" => Ok(Self::Required),
            _ => Err(serde::de::Error::custom(format!(
                "unsupported code format mode `{value}`; expected never, auto, or required"
            ))),
        }
    }
}
