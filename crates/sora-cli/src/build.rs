use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use serde::Deserialize;
use sora_codegen::{format::FormatMode, target::CodegenTarget};
use sora_input_toml::input::TomlSchemaInput;

use crate::{
    args::{BuildArgs, BuildTarget, CodeFormatMode, DataFormat},
    commands::export_project_data,
};

pub fn run(args: BuildArgs) -> Result<()> {
    let manifest = BuildManifest::load(&args.project)?;
    let build = manifest.build;
    let project_dir = args.project.parent().unwrap_or_else(|| Path::new("."));
    let schema_input = TomlSchemaInput::new(&args.project);

    let data_format = args
        .data_format
        .or(build.data_format)
        .unwrap_or(DataFormat::Xlsx);
    let data_root = args
        .data_root
        .as_ref()
        .or(build.data_root.as_ref())
        .cloned()
        .unwrap_or_else(|| PathBuf::from("data"));
    let data_root = resolve_project_path(project_dir, &data_root);

    let requested_targets = args.target;
    let codegen = selected_codegen_targets(&build.codegen, &requested_targets)?;

    if build.is_empty() {
        bail!(
            "project `{}` does not declare any build outputs; add [build], [[build.codegen]], or [[build.exports]]",
            args.project.display()
        );
    }

    validate_export_formats(&build.exports)?;

    if args.clean {
        clean_build_outputs(project_dir, &build, &codegen)?;
    }

    sora_core::pipeline::check_schema(&schema_input)
        .with_context(|| format!("failed to check project `{}`", args.project.display()))?;

    if let Some(path) = build.schema_lock.as_ref() {
        let path = resolve_project_path(project_dir, path);
        sora_core::pipeline::generate_schema_lock(&schema_input, &path).with_context(|| {
            format!(
                "failed to generate schema lock from `{}` into `{}`",
                args.project.display(),
                path.display()
            )
        })?;
    }

    if let Some(path) = build.excel_templates.as_ref() {
        let path = resolve_project_path(project_dir, path);
        sora_core::pipeline::generate_excel_template(&schema_input, &path).with_context(|| {
            format!(
                "failed to generate Excel templates from `{}` into `{}`",
                args.project.display(),
                path.display()
            )
        })?;
    }

    for item in codegen {
        let out = resolve_project_path(project_dir, &item.out);
        sora_core::pipeline::generate_code_with_format(
            &schema_input,
            item.target.into(),
            &out,
            FormatMode::from(item.format),
        )
        .with_context(|| {
            format!(
                "failed to generate {} code from `{}` into `{}`",
                item.target.as_str(),
                args.project.display(),
                out.display()
            )
        })?;
    }

    for item in &build.exports {
        let out = resolve_project_path(project_dir, &item.out);
        export_project_data(&args.project, &data_root, data_format, &item.format, out)
            .with_context(|| {
                format!(
                    "failed to export `{}` data from `{}`",
                    item.format,
                    data_root.display()
                )
            })?;
    }

    Ok(())
}

fn validate_export_formats(exports: &[BuildExport]) -> Result<()> {
    for item in exports {
        if sora_core::pipeline::export_output_kind(&item.format).is_none() {
            bail!(
                "unknown export format `{}`; supported formats: {}",
                item.format,
                sora_core::pipeline::supported_export_formats().join(", ")
            );
        }
    }
    Ok(())
}

fn selected_codegen_targets<'a>(
    configured: &'a [BuildCodegen],
    requested: &[BuildTarget],
) -> Result<Vec<&'a BuildCodegen>> {
    if requested.is_empty() {
        return Ok(configured.iter().collect());
    }

    let selected = configured
        .iter()
        .filter(|item| requested.contains(&item.target))
        .collect::<Vec<_>>();
    for target in requested {
        if !configured.iter().any(|item| item.target == *target) {
            bail!(
                "build target `{}` was requested but is not declared in [[build.codegen]]",
                target.as_str()
            );
        }
    }
    Ok(selected)
}

fn clean_build_outputs(
    project_dir: &Path,
    build: &BuildConfig,
    codegen: &[&BuildCodegen],
) -> Result<()> {
    if let Some(path) = build.schema_lock.as_ref() {
        clean_output(project_dir, &resolve_project_path(project_dir, path))?;
    }
    if let Some(path) = build.excel_templates.as_ref() {
        clean_output(project_dir, &resolve_project_path(project_dir, path))?;
    }
    for item in codegen {
        clean_output(project_dir, &resolve_project_path(project_dir, &item.out))?;
    }
    for item in &build.exports {
        clean_output(project_dir, &resolve_project_path(project_dir, &item.out))?;
    }
    Ok(())
}

fn clean_output(project_dir: &Path, path: &Path) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }

    let project_dir = project_dir.canonicalize().with_context(|| {
        format!(
            "failed to resolve project directory `{}`",
            project_dir.display()
        )
    })?;
    let path = path
        .canonicalize()
        .with_context(|| format!("failed to resolve output path `{}`", path.display()))?;
    if path == project_dir || !path.starts_with(&project_dir) {
        bail!(
            "refusing to clean output `{}` because it is not safely inside project directory `{}`",
            path.display(),
            project_dir.display()
        );
    }

    if path.is_dir() {
        fs::remove_dir_all(&path)
            .with_context(|| format!("failed to clean directory `{}`", path.display()))?;
    } else {
        fs::remove_file(&path)
            .with_context(|| format!("failed to clean file `{}`", path.display()))?;
    }
    Ok(())
}

fn resolve_project_path(project_dir: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        project_dir.join(path)
    }
}

#[derive(Debug, Deserialize)]
struct BuildManifest {
    #[serde(default)]
    build: BuildConfig,
}

impl BuildManifest {
    fn load(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("failed to read project `{}`", path.display()))?;
        toml::from_str(&content)
            .with_context(|| format!("failed to parse build config from `{}`", path.display()))
    }
}

#[derive(Debug, Default, Deserialize)]
struct BuildConfig {
    data_format: Option<DataFormat>,
    data_root: Option<PathBuf>,
    schema_lock: Option<PathBuf>,
    excel_templates: Option<PathBuf>,

    #[serde(default)]
    codegen: Vec<BuildCodegen>,

    #[serde(default)]
    exports: Vec<BuildExport>,
}

impl BuildConfig {
    fn is_empty(&self) -> bool {
        self.schema_lock.is_none()
            && self.excel_templates.is_none()
            && self.codegen.is_empty()
            && self.exports.is_empty()
    }
}

#[derive(Debug, Deserialize)]
struct BuildCodegen {
    target: BuildTarget,
    out: PathBuf,
    #[serde(default)]
    format: CodeFormatMode,
}

#[derive(Debug, Deserialize)]
struct BuildExport {
    format: String,
    out: PathBuf,
}

impl<'de> Deserialize<'de> for DataFormat {
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
                "unsupported data format `{value}`; expected csv, toml, or xlsx"
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

impl Default for CodeFormatMode {
    fn default() -> Self {
        Self::Never
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
    fn from_str(value: &str) -> std::result::Result<Self, String> {
        match value {
            "rust" => Ok(Self::Rust),
            "kotlin" => Ok(Self::Kotlin),
            "csharp" | "cs" => Ok(Self::Csharp),
            "java" => Ok(Self::Java),
            "go" => Ok(Self::Go),
            "typescript" | "ts" => Ok(Self::Typescript),
            "javascript" | "js" => Ok(Self::Javascript),
            "erlang" | "erl" => Ok(Self::Erlang),
            "lua" => Ok(Self::Lua),
            "proto" => Ok(Self::Proto),
            _ => Err(format!(
                "unsupported codegen target `{value}`; expected rust, kotlin, csharp, java, go, typescript, javascript, erlang, lua, or proto"
            )),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Rust => "rust",
            Self::Kotlin => "kotlin",
            Self::Csharp => "csharp",
            Self::Java => "java",
            Self::Go => "go",
            Self::Typescript => "typescript",
            Self::Javascript => "javascript",
            Self::Erlang => "erlang",
            Self::Lua => "lua",
            Self::Proto => "proto",
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
            BuildTarget::Go => Self::Go,
            BuildTarget::Typescript => Self::TypeScript,
            BuildTarget::Javascript => Self::JavaScript,
            BuildTarget::Erlang => Self::Erlang,
            BuildTarget::Lua => Self::Lua,
            BuildTarget::Proto => Self::Proto,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn build_command_generates_configured_outputs() {
        let base = temp_dir();
        let project = write_project(&base);

        run(BuildArgs {
            project: project.clone(),
            data_format: None,
            data_root: None,
            target: Vec::new(),
            clean: false,
        })
        .unwrap();

        assert!(base.join("generated/schema.lock").exists());
        assert!(base.join("generated/excel/Item.xlsx").exists());
        assert!(base.join("generated/rust/item.rs").exists());
        assert!(base.join("generated/lua/item.lua").exists());
        assert!(base.join("generated/typescript/item.ts").exists());
        assert!(base.join("generated/javascript/item.js").exists());
        assert!(base.join("generated/javascript/item.d.ts").exists());
        assert!(base.join("generated/erlang/item.erl").exists());
        assert!(base.join("generated/proto/sora_config.proto").exists());
        assert!(base.join("generated/config.json").exists());
        assert!(base.join("generated/config.pb").exists());
        assert!(base.join("generated/config.typed.pb").exists());
        assert!(base.join("generated/config.cbor").exists());
        assert!(base.join("generated/debug-json/Item.json").exists());

        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn build_command_can_filter_codegen_targets() {
        let base = temp_dir();
        let project = write_project(&base);
        let rust_stale = base.join("generated/rust/stale.txt");
        let kotlin_stale = base.join("generated/kotlin/stale.txt");
        fs::create_dir_all(rust_stale.parent().unwrap()).unwrap();
        fs::create_dir_all(kotlin_stale.parent().unwrap()).unwrap();
        fs::write(&rust_stale, "stale").unwrap();
        fs::write(&kotlin_stale, "stale").unwrap();

        run(BuildArgs {
            project: project.clone(),
            data_format: None,
            data_root: None,
            target: vec![BuildTarget::Rust],
            clean: true,
        })
        .unwrap();

        assert!(base.join("generated/rust/item.rs").exists());
        assert!(!rust_stale.exists());
        assert!(kotlin_stale.exists());
        assert!(!base.join("generated/kotlin/game_config/Item.kt").exists());

        let _ = fs::remove_dir_all(base);
    }

    fn write_project(base: &Path) -> PathBuf {
        let data_dir = base.join("data");
        let schema_dir = base.join("schema");
        fs::create_dir_all(&data_dir).unwrap();
        fs::create_dir_all(&schema_dir).unwrap();

        let project = base.join("project.toml");
        fs::write(
            &project,
            r#"
package = "game_config"
includes = ["schema/items.toml"]

[build]
data_format = "toml"
data_root = "data"
schema_lock = "generated/schema.lock"
excel_templates = "generated/excel"

[[build.codegen]]
target = "rust"
out = "generated/rust"
format = "auto"

[[build.codegen]]
target = "kotlin"
out = "generated/kotlin"

[[build.codegen]]
target = "lua"
out = "generated/lua"

[[build.codegen]]
target = "typescript"
out = "generated/typescript"

[[build.codegen]]
target = "javascript"
out = "generated/javascript"

[[build.codegen]]
target = "erlang"
out = "generated/erlang"

[[build.codegen]]
target = "proto"
out = "generated/proto"

[[build.exports]]
format = "json"
out = "generated/config.json"

[[build.exports]]
format = "protobuf"
out = "generated/config.pb"

[[build.exports]]
format = "typed-protobuf"
out = "generated/config.typed.pb"

[[build.exports]]
format = "cbor"
out = "generated/config.cbor"

[[build.exports]]
format = "json-debug"
out = "generated/debug-json"
"#,
        )
        .unwrap();

        fs::write(
            schema_dir.join("items.toml"),
            r#"
[[enums]]
name = "ItemType"
values = ["Weapon", "Armor"]

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
key = true
required = true

[[tables.fields]]
name = "name"
type = "string"
required = true

[[tables.fields]]
name = "item_type"
type = "enum<ItemType>"
required = true
"#,
        )
        .unwrap();

        fs::write(
            data_dir.join("items.toml"),
            r#"
[[rows]]
id = 1001
name = "Iron Sword"
item_type = "Weapon"
"#,
        )
        .unwrap();

        project
    }

    fn temp_dir() -> PathBuf {
        let unique = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("sora-cli-build-test-{unique}"))
    }
}
