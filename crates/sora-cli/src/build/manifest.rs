use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use serde::Deserialize;
use sora_codegen::target::CodegenTarget;

use crate::args::{BuildTarget, CodeFormatMode, SourceFormatArg};
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
    pub(super) target: BuildTarget,
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

impl<'de> Deserialize<'de> for BuildTarget {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::from_str(&value).map_err(serde::de::Error::custom)
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

impl BuildTarget {
    pub(super) fn from_str(value: &str) -> std::result::Result<Self, String> {
        match value {
            "rust" => Ok(Self::Rust),
            "kotlin" => Ok(Self::Kotlin),
            "csharp" | "cs" => Ok(Self::Csharp),
            "java" => Ok(Self::Java),
            "scala" => Ok(Self::Scala),
            "go" => Ok(Self::Go),
            "dart" => Ok(Self::Dart),
            "godot" | "gdscript" => Ok(Self::Godot),
            "c" => Ok(Self::C),
            "cpp" | "c++" => Ok(Self::Cpp),
            "typescript" | "ts" => Ok(Self::Typescript),
            "javascript" | "js" => Ok(Self::Javascript),
            "erlang" | "erl" => Ok(Self::Erlang),
            "lua" => Ok(Self::Lua),
            "proto-schema" => Ok(Self::ProtoSchema),
            "python" | "py" => Ok(Self::Python),
            _ => Err(format!(
                "unsupported codegen target `{value}`; expected rust, kotlin, csharp, java, scala, go, dart, godot, c, cpp, typescript, javascript, erlang, lua, proto-schema, or python"
            )),
        }
    }

    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Rust => "rust",
            Self::Kotlin => "kotlin",
            Self::Csharp => "csharp",
            Self::Java => "java",
            Self::Scala => "scala",
            Self::Go => "go",
            Self::Dart => "dart",
            Self::Godot => "godot",
            Self::C => "c",
            Self::Cpp => "cpp",
            Self::Typescript => "typescript",
            Self::Javascript => "javascript",
            Self::Erlang => "erlang",
            Self::Lua => "lua",
            Self::ProtoSchema => "proto-schema",
            Self::Python => "python",
        }
    }
}

impl From<BuildTarget> for CodegenTarget {
    fn from(value: BuildTarget) -> Self {
        match value {
            BuildTarget::Rust => Self::Rust,
            BuildTarget::Kotlin => Self::Kotlin,
            BuildTarget::Csharp => Self::CSharp,
            BuildTarget::Java => Self::Java,
            BuildTarget::Scala => Self::Scala,
            BuildTarget::Go => Self::Go,
            BuildTarget::Dart => Self::Dart,
            BuildTarget::Godot => Self::Godot,
            BuildTarget::C => Self::C,
            BuildTarget::Cpp => Self::Cpp,
            BuildTarget::Typescript => Self::TypeScript,
            BuildTarget::Javascript => Self::JavaScript,
            BuildTarget::Erlang => Self::Erlang,
            BuildTarget::Lua => Self::Lua,
            BuildTarget::ProtoSchema => Self::ProtoSchema,
            BuildTarget::Python => Self::Python,
        }
    }
}
