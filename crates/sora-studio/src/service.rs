use std::{
    collections::BTreeMap,
    fs,
    path::{Component, Path, PathBuf},
};

use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::Value;
use sora_config_format::load_document;
use sora_diagnostics::SoraError;
use sora_input_schema::schema::load_project_schema_file;
use sora_ir::{normalize::normalize_schema, validate::validate_config_ir};
use sora_schema::model::{CodegenSchema, EnumSchema, StructSchema, TableSchema, UnionSchema};

use crate::{
    diff::simple_diff,
    graph::{build_schema, build_schema_from_raw, node_id},
    model::{
        DiagnosticLevel, StudioDiagnostic, StudioNodeKind, StudioPreviewResponse, StudioSchema,
        StudioSchemaResponse,
    },
    render::{
        StudioDocumentFormat, document_format, render_lua_document, render_schema_module,
        render_schema_module_for_path,
    },
};

pub(crate) fn load_studio_schema(project: &Path) -> StudioSchemaResponse {
    let source_index = match schema_source_index(project) {
        Ok(index) => index,
        Err(error) => {
            return StudioSchemaResponse {
                ok: false,
                project: project.display().to_string(),
                diagnostics: vec![StudioDiagnostic {
                    level: DiagnosticLevel::Error,
                    message: error.to_string(),
                    target_id: None,
                }],
                schema: None,
            };
        }
    };
    let raw_schema = match load_project_schema_file(project) {
        Ok(schema) => schema,
        Err(error) => {
            return StudioSchemaResponse {
                ok: false,
                project: project.display().to_string(),
                diagnostics: studio_diagnostics(&error),
                schema: None,
            };
        }
    };

    let raw_fallback = || {
        build_schema_from_raw(
            &raw_schema,
            &source_index.sources,
            &source_index.source_by_node,
        )
    };
    match normalize_schema(raw_schema.clone()) {
        Ok(ir) => match validate_config_ir(&ir) {
            Ok(()) => StudioSchemaResponse {
                ok: true,
                project: project.display().to_string(),
                diagnostics: vec![StudioDiagnostic {
                    level: DiagnosticLevel::Info,
                    message: "schema loaded successfully".to_owned(),
                    target_id: None,
                }],
                schema: Some(build_schema(
                    &ir,
                    &source_index.sources,
                    &source_index.source_by_node,
                )),
            },
            Err(error) => StudioSchemaResponse {
                ok: false,
                project: project.display().to_string(),
                diagnostics: studio_diagnostics(&error),
                schema: Some(build_schema(
                    &ir,
                    &source_index.sources,
                    &source_index.source_by_node,
                )),
            },
        },
        Err(error) => StudioSchemaResponse {
            ok: false,
            project: project.display().to_string(),
            diagnostics: studio_diagnostics(&error),
            schema: Some(raw_fallback()),
        },
    }
}

pub(crate) fn save_studio_schema(project: &Path, schema: &StudioSchema) -> StudioSchemaResponse {
    match write_studio_schema(project, schema) {
        Ok(path) => {
            let mut response = load_studio_schema(project);
            if response.ok {
                let targets = path
                    .iter()
                    .map(|path| path.display().to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                response.diagnostics = vec![StudioDiagnostic {
                    level: DiagnosticLevel::Info,
                    message: format!("schema saved to {targets}"),
                    target_id: None,
                }];
            }
            response
        }
        Err(error) => StudioSchemaResponse {
            ok: false,
            project: project.display().to_string(),
            diagnostics: vec![StudioDiagnostic {
                level: DiagnosticLevel::Error,
                message: error.to_string(),
                target_id: None,
            }],
            schema: Some(schema.clone()),
        },
    }
}

pub(crate) fn preview_studio_schema(
    project: &Path,
    schema: &StudioSchema,
) -> StudioPreviewResponse {
    match schema_includes_from_sources(project, &schema.sources) {
        Ok(includes) => {
            if let Err(error) = validate_node_sources(schema, &includes) {
                return StudioPreviewResponse {
                    ok: false,
                    project: project.display().to_string(),
                    target: None,
                    content: Some(render_schema_module(schema)),
                    diff: None,
                    diagnostics: vec![StudioDiagnostic {
                        level: DiagnosticLevel::Error,
                        message: error.to_string(),
                        target_id: None,
                    }],
                };
            }
            let current_project = fs::read_to_string(project).unwrap_or_default();
            let next_project =
                project_text_with_schema_files(project, &schema.package, &schema.sources)
                    .unwrap_or(current_project.clone());
            let base_schema = load_studio_schema(project).schema;
            let mut diff = format!(
                "project: {}\n{}",
                project.display(),
                simple_diff(&current_project, &next_project)
            );
            let mut content = String::new();
            for include in &includes {
                let current_include = fs::read_to_string(&include.path).unwrap_or_default();
                let next_schema = schema_for_source(schema, &include.source);
                let next_content = match render_schema_module_for_path(
                    &next_schema,
                    &include.path,
                    &current_include,
                ) {
                    Ok(content) => content,
                    Err(error) => {
                        return StudioPreviewResponse {
                            ok: false,
                            project: project.display().to_string(),
                            target: None,
                            content: Some(render_schema_module(schema)),
                            diff: None,
                            diagnostics: vec![StudioDiagnostic {
                                level: DiagnosticLevel::Error,
                                message: error.to_string(),
                                target_id: None,
                            }],
                        };
                    }
                };
                let base_content = base_schema
                    .as_ref()
                    .and_then(|schema| {
                        render_schema_module_for_path(
                            &schema_for_source(schema, &include.source),
                            &include.path,
                            &current_include,
                        )
                        .ok()
                    })
                    .unwrap_or_else(|| current_include.clone());
                diff.push_str(&format!(
                    "\ninclude: {}\n{}",
                    include.path.display(),
                    simple_diff(&base_content, &next_content)
                ));
                content.push_str(&format!(
                    "### {}\n{}\n",
                    include.path.display(),
                    next_content
                ));
            }
            StudioPreviewResponse {
                ok: true,
                project: project.display().to_string(),
                target: Some(format!(
                    "{} + {} schema files",
                    project.display(),
                    includes.len()
                )),
                diff: Some(diff),
                content: Some(content),
                diagnostics: Vec::new(),
            }
        }
        Err(error) => StudioPreviewResponse {
            ok: false,
            project: project.display().to_string(),
            target: None,
            content: Some(render_schema_module(schema)),
            diff: None,
            diagnostics: vec![StudioDiagnostic {
                level: DiagnosticLevel::Error,
                message: error.to_string(),
                target_id: None,
            }],
        },
    }
}

fn studio_diagnostics(error: &SoraError) -> Vec<StudioDiagnostic> {
    if let Some(errors) = error.errors() {
        return errors
            .iter()
            .flat_map(studio_diagnostics)
            .collect::<Vec<_>>();
    }

    vec![StudioDiagnostic {
        level: DiagnosticLevel::Error,
        message: error.to_string(),
        target_id: diagnostic_target_id(error),
    }]
}

fn diagnostic_target_id(error: &SoraError) -> Option<String> {
    match error {
        SoraError::DuplicateSchemaName { kind, name } => {
            studio_node_kind(kind).map(|kind| node_id(kind, name))
        }
        SoraError::DuplicateFieldName {
            owner_kind, owner, ..
        }
        | SoraError::UnknownTypeReference {
            owner_kind, owner, ..
        }
        | SoraError::UnknownRefTable {
            owner_kind, owner, ..
        }
        | SoraError::UnknownRefField {
            owner_kind, owner, ..
        } => studio_node_kind(owner_kind).map(|kind| node_id(kind, owner)),
        SoraError::MissingTableKey { table, .. }
        | SoraError::UnknownIndexField { table, .. }
        | SoraError::MissingTableSource { table }
        | SoraError::UnknownField { table, .. }
        | SoraError::TypeMismatch { table, .. }
        | SoraError::InvalidEnumValue { table, .. }
        | SoraError::DuplicateKey { table, .. }
        | SoraError::DuplicateIndexKey { table, .. }
        | SoraError::RangeOutOfBounds { table, .. }
        | SoraError::LengthOutOfBounds { table, .. }
        | SoraError::MissingReference { table, .. }
        | SoraError::InvalidTableRowCount { table, .. } => {
            Some(node_id(StudioNodeKind::Table, table))
        }
        _ => None,
    }
}

fn studio_node_kind(kind: &str) -> Option<StudioNodeKind> {
    match kind {
        "enum" => Some(StudioNodeKind::Enum),
        "struct" => Some(StudioNodeKind::Struct),
        "union" => Some(StudioNodeKind::Union),
        "table" => Some(StudioNodeKind::Table),
        _ => None,
    }
}

#[derive(Debug, Clone)]
struct SchemaInclude {
    source: String,
    path: PathBuf,
}

#[derive(Debug)]
struct SchemaSourceIndex {
    sources: Vec<String>,
    source_by_node: BTreeMap<String, String>,
}

fn schema_source_index(project: &Path) -> Result<SchemaSourceIndex> {
    let includes = schema_include_paths(project)?;
    let mut source_by_node = BTreeMap::new();
    for include in &includes {
        let module = load_schema_document(&include.path)?;
        for id in schema_document_node_order(&module) {
            source_by_node.insert(id, include.source.clone());
        }
    }
    Ok(SchemaSourceIndex {
        sources: includes.into_iter().map(|include| include.source).collect(),
        source_by_node,
    })
}

fn load_schema_document(path: &Path) -> Result<SchemaDocument> {
    load_document(path).with_context(|| format!("failed to load schema `{}`", path.display()))
}

fn schema_document_node_order(module: &SchemaDocument) -> Vec<String> {
    module
        .enums
        .iter()
        .map(|item| node_id(StudioNodeKind::Enum, &item.name))
        .chain(
            module
                .structs
                .iter()
                .map(|item| node_id(StudioNodeKind::Struct, &item.name)),
        )
        .chain(
            module
                .unions
                .iter()
                .map(|item| node_id(StudioNodeKind::Union, &item.name)),
        )
        .chain(
            module
                .tables
                .iter()
                .map(|item| node_id(StudioNodeKind::Table, &item.name)),
        )
        .collect()
}

#[derive(Debug, Deserialize)]
struct SchemaDocument {
    #[serde(default)]
    pub enums: Vec<EnumSchema>,
    #[serde(default)]
    pub structs: Vec<StructSchema>,
    #[serde(default)]
    pub unions: Vec<UnionSchema>,
    #[serde(default)]
    pub tables: Vec<TableSchema>,
}

pub(crate) fn write_studio_schema(project: &Path, schema: &StudioSchema) -> Result<Vec<PathBuf>> {
    let includes = schema_includes_from_sources(project, &schema.sources)?;
    validate_node_sources(schema, &includes)?;
    write_project_settings(project, &schema.package, &schema.sources)?;
    let mut written = Vec::new();
    for include in includes {
        let current_include = fs::read_to_string(&include.path).unwrap_or_default();
        let content = render_schema_module_for_path(
            &schema_for_source(schema, &include.source),
            &include.path,
            &current_include,
        )?;
        if let Some(parent) = include.path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!(
                    "failed to create schema include directory `{}`",
                    parent.display()
                )
            })?;
        }
        fs::write(&include.path, content).with_context(|| {
            format!(
                "failed to write schema include `{}`",
                include.path.display()
            )
        })?;
        written.push(include.path);
    }
    Ok(written)
}

fn schema_includes_from_sources(project: &Path, sources: &[String]) -> Result<Vec<SchemaInclude>> {
    validate_schema_sources(sources)?;
    let base_dir = project.parent().unwrap_or_else(|| Path::new("."));
    sources
        .iter()
        .map(|source| {
            let path = base_dir.join(source);
            document_format(&path)?;
            Ok(SchemaInclude {
                source: source.clone(),
                path,
            })
        })
        .collect()
}

fn validate_schema_sources(sources: &[String]) -> Result<()> {
    if sources.is_empty() {
        anyhow::bail!("Studio project must declare at least one schema include");
    }
    let mut seen = BTreeMap::<&str, usize>::new();
    for source in sources {
        let clean = source.trim();
        if clean.is_empty() {
            anyhow::bail!("schema include path cannot be empty");
        }
        if clean != source {
            anyhow::bail!("schema include path `{source}` must not contain surrounding whitespace");
        }
        let path = Path::new(source);
        if path.is_absolute() {
            anyhow::bail!("schema include path `{source}` must be relative");
        }
        document_format(path)?;
        for component in path.components() {
            if matches!(
                component,
                Component::CurDir
                    | Component::ParentDir
                    | Component::RootDir
                    | Component::Prefix(_)
            ) {
                anyhow::bail!("schema include path `{source}` must be a plain relative path");
            }
        }
        if let Some(first_index) = seen.insert(clean, seen.len()) {
            anyhow::bail!(
                "schema include path `{source}` duplicates include at index {}",
                first_index + 1
            );
        }
    }
    Ok(())
}

fn validate_node_sources(schema: &StudioSchema, includes: &[SchemaInclude]) -> Result<()> {
    for node in &schema.nodes {
        if !includes.iter().any(|include| include.source == node.source) {
            anyhow::bail!(
                "{} `{}` belongs to unknown schema file `{}`",
                node_kind_name(node.kind),
                node.name,
                node.source
            );
        }
    }
    Ok(())
}

fn node_kind_name(kind: StudioNodeKind) -> &'static str {
    match kind {
        StudioNodeKind::Enum => "enum",
        StudioNodeKind::Struct => "struct",
        StudioNodeKind::Union => "union",
        StudioNodeKind::Table => "table",
    }
}

fn schema_for_source(schema: &StudioSchema, source: &str) -> StudioSchema {
    let nodes = schema
        .nodes
        .iter()
        .filter(|node| node.source == source)
        .cloned()
        .collect::<Vec<_>>();
    let edges = schema
        .edges
        .iter()
        .filter(|edge| {
            nodes.iter().any(|node| node.id == edge.source)
                || nodes.iter().any(|node| node.id == edge.target)
        })
        .cloned()
        .collect::<Vec<_>>();
    StudioSchema {
        package: schema.package.clone(),
        sources: vec![source.to_owned()],
        summary: crate::model::StudioSummary {
            enums: nodes
                .iter()
                .filter(|node| node.kind == StudioNodeKind::Enum)
                .count(),
            structs: nodes
                .iter()
                .filter(|node| node.kind == StudioNodeKind::Struct)
                .count(),
            unions: nodes
                .iter()
                .filter(|node| node.kind == StudioNodeKind::Union)
                .count(),
            tables: nodes
                .iter()
                .filter(|node| node.kind == StudioNodeKind::Table)
                .count(),
            edges: edges.len(),
        },
        nodes,
        edges,
    }
}

fn write_project_settings(project: &Path, package: &str, sources: &[String]) -> Result<()> {
    let content = project_text_with_schema_files(project, package, sources)?;
    fs::write(project, content)
        .with_context(|| format!("failed to write project file `{}`", project.display()))
}

pub(crate) fn project_text_with_schema_files(
    project: &Path,
    package: &str,
    sources: &[String],
) -> Result<String> {
    validate_schema_sources(sources)?;
    match document_format(project)? {
        StudioDocumentFormat::Toml => {
            project_toml_text_with_schema_files(project, package, sources)
        }
        StudioDocumentFormat::Yaml => {
            project_yaml_text_with_schema_files(project, package, sources)
        }
        StudioDocumentFormat::Json => {
            project_json_text_with_schema_files(project, package, sources)
        }
        StudioDocumentFormat::Lua => project_lua_text_with_schema_files(project, package, sources),
    }
}

fn project_toml_text_with_schema_files(
    project: &Path,
    package: &str,
    sources: &[String],
) -> Result<String> {
    let project_text = fs::read_to_string(project)
        .with_context(|| format!("failed to read project file `{}`", project.display()))?;
    let mut project_doc = project_text
        .parse::<toml::Value>()
        .with_context(|| format!("failed to parse project TOML `{}`", project.display()))?;
    let table = project_doc
        .as_table_mut()
        .context("project TOML root must be a table")?;
    if table.get("package").and_then(toml::Value::as_str) == Some(package)
        && table
            .get("includes")
            .and_then(toml_array_strings)
            .is_some_and(|current| current == sources)
    {
        return Ok(project_text);
    }
    table.insert(
        "package".to_owned(),
        toml::Value::String(package.to_owned()),
    );
    table.insert(
        "includes".to_owned(),
        toml::Value::Array(
            sources
                .iter()
                .map(|source| toml::Value::String(source.clone()))
                .collect(),
        ),
    );
    toml::to_string_pretty(&project_doc).context("failed to render project TOML")
}

fn project_yaml_text_with_schema_files(
    project: &Path,
    package: &str,
    sources: &[String],
) -> Result<String> {
    let mut value = load_document::<serde_yaml::Value>(project)
        .with_context(|| format!("failed to load project `{}`", project.display()))?;
    set_yaml_project_fields(&mut value, package, sources)?;
    serde_yaml::to_string(&value).context("failed to render project YAML")
}

fn project_json_text_with_schema_files(
    project: &Path,
    package: &str,
    sources: &[String],
) -> Result<String> {
    let mut value = load_document::<Value>(project)
        .with_context(|| format!("failed to load project `{}`", project.display()))?;
    set_json_project_fields(&mut value, package, sources)?;
    let mut out = serde_json::to_string_pretty(&value).context("failed to render project JSON")?;
    out.push('\n');
    Ok(out)
}

fn project_lua_text_with_schema_files(
    project: &Path,
    package: &str,
    sources: &[String],
) -> Result<String> {
    let mut value = load_document::<Value>(project)
        .with_context(|| format!("failed to load project `{}`", project.display()))?;
    set_json_project_fields(&mut value, package, sources)?;
    Ok(render_lua_document(&value))
}

fn set_json_project_fields(value: &mut Value, package: &str, sources: &[String]) -> Result<()> {
    let object = value
        .as_object_mut()
        .context("project document root must be an object")?;
    object.insert("package".to_owned(), Value::String(package.to_owned()));
    object.insert(
        "includes".to_owned(),
        Value::Array(
            sources
                .iter()
                .map(|source| Value::String(source.clone()))
                .collect(),
        ),
    );
    Ok(())
}

fn set_yaml_project_fields(
    value: &mut serde_yaml::Value,
    package: &str,
    sources: &[String],
) -> Result<()> {
    let object = value
        .as_mapping_mut()
        .context("project document root must be a mapping")?;
    object.insert(
        serde_yaml::Value::String("package".to_owned()),
        serde_yaml::Value::String(package.to_owned()),
    );
    object.insert(
        serde_yaml::Value::String("includes".to_owned()),
        serde_yaml::Value::Sequence(
            sources
                .iter()
                .map(|source| serde_yaml::Value::String(source.clone()))
                .collect(),
        ),
    );
    Ok(())
}

fn schema_include_paths(project: &Path) -> Result<Vec<SchemaInclude>> {
    document_format(project)?;
    let project_doc = load_document::<ProjectDocument>(project)
        .with_context(|| format!("failed to load project `{}`", project.display()))?;
    let includes = project_doc.includes;
    if includes.is_empty() {
        anyhow::bail!("Studio project must declare at least one schema include");
    }

    schema_includes_from_sources(project, &includes)
}

fn toml_array_strings(value: &toml::Value) -> Option<Vec<String>> {
    Some(
        value
            .as_array()?
            .iter()
            .filter_map(|item| item.as_str().map(ToOwned::to_owned))
            .collect(),
    )
}

#[derive(Debug, Deserialize)]
struct ProjectDocument {
    #[serde(default)]
    includes: Vec<String>,
    #[allow(dead_code)]
    package: Option<String>,
    #[allow(dead_code)]
    codegen: Option<CodegenSchema>,
}
