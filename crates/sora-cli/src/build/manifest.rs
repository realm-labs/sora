use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::args::{CodeFormatMode, SourceFormatArg};
#[derive(Debug, Deserialize)]
pub(super) struct BuildManifest {
    #[serde(default)]
    pub(super) build: BuildConfig,
}

impl BuildManifest {
    pub(super) fn load(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("failed to read project `{}`", path.display()))?;
        toml::from_str(&content)
            .with_context(|| format!("failed to parse build config from `{}`", path.display()))
    }
}

#[derive(Debug, Default, Deserialize)]
pub(super) struct BuildConfig {
    pub(super) default_source_format: Option<SourceFormatArg>,
    pub(super) data_root: Option<PathBuf>,
    pub(super) scope: Option<String>,
    pub(super) schema_lock: Option<PathBuf>,
    pub(super) excel_templates: Option<PathBuf>,

    #[serde(default)]
    pub(super) codegen: Vec<BuildCodegen>,

    #[serde(default)]
    pub(super) exports: Vec<BuildExport>,
}

impl BuildConfig {
    pub(super) fn is_empty(&self) -> bool {
        self.schema_lock.is_none()
            && self.excel_templates.is_none()
            && self.codegen.is_empty()
            && self.exports.is_empty()
    }
}

#[derive(Debug, Deserialize)]
pub(super) struct BuildCodegen {
    pub(super) target: String,
    pub(super) out: PathBuf,
    pub(super) scope: Option<String>,
    #[serde(default)]
    pub(super) format: CodeFormatMode,
}

#[derive(Debug, Deserialize)]
pub(super) struct BuildExport {
    pub(super) format: String,
    pub(super) out: PathBuf,
    pub(super) scope: Option<String>,
    #[serde(default)]
    pub(super) compression: ExportCompressionArg,
    pub(super) compression_level: Option<i32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(super) enum ExportCompressionArg {
    #[default]
    None,
    Zstd,
}

impl<'de> Deserialize<'de> for SourceFormatArg {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "csv" => Ok(Self::Csv),
            "toml" => Ok(Self::Toml),
            "xlsx" => Ok(Self::Xlsx),
            _ => Err(serde::de::Error::custom(format!(
                "unsupported source format `{value}`; expected csv, toml, or xlsx"
            ))),
        }
    }
}

impl<'de> Deserialize<'de> for ExportCompressionArg {
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
