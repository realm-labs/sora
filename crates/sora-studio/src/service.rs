use std::{
    collections::BTreeMap,
    fs,
    path::{Component, Path, PathBuf},
};

use anyhow::{Context, Result};
use sora_diagnostics::SoraError;
use sora_input_schema::schema::load_project_schema_file;
use sora_ir::{normalize::normalize_schema, validate::validate_config_ir};

use crate::{
    diff::simple_diff,
    graph::{build_schema, build_schema_from_raw, node_id},
    model::{
        DiagnosticLevel, StudioDiagnostic, StudioNodeKind, StudioPreviewResponse, StudioSchema,
        StudioSchemaResponse,
    },
    render::{push_quoted, render_schema_module, render_schema_module_like, schema_node_order},
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
                let next_content = render_schema_module_like(&next_schema, &current_include);
                let base_content = base_schema
                    .as_ref()
                    .map(|schema| {
                        render_schema_module_like(
                            &schema_for_source(schema, &include.source),
                            &current_include,
                        )
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
        let content = fs::read_to_string(&include.path).with_context(|| {
            format!("failed to read schema include `{}`", include.path.display())
        })?;
        for id in schema_node_order(&content) {
            source_by_node.insert(id, include.source.clone());
        }
    }
    Ok(SchemaSourceIndex {
        sources: includes.into_iter().map(|include| include.source).collect(),
        source_by_node,
    })
}

pub(crate) fn write_studio_schema(project: &Path, schema: &StudioSchema) -> Result<Vec<PathBuf>> {
    let includes = schema_includes_from_sources(project, &schema.sources)?;
    validate_node_sources(schema, &includes)?;
    write_project_settings(project, &schema.package, &schema.sources)?;
    let mut written = Vec::new();
    for include in includes {
        let current_include = fs::read_to_string(&include.path).unwrap_or_default();
        let content = render_schema_module_like(
            &schema_for_source(schema, &include.source),
            &current_include,
        );
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
            ensure_toml_path(&path, "schema include")?;
            Ok(SchemaInclude {
                source: source.clone(),
                path,
            })
        })
        .collect()
}

fn validate_schema_sources(sources: &[String]) -> Result<()> {
    if sources.is_empty() {
        anyhow::bail!("Studio project must declare at least one TOML include");
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
        if path.extension().and_then(|value| value.to_str()) != Some("toml") {
            anyhow::bail!("schema include path `{source}` must end with .toml");
        }
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
    let content = replace_root_package_line(&project_text, package)
        .unwrap_or_else(|| prepend_root_package_line(&project_text, package));
    Ok(replace_root_includes_line(&content, sources)
        .unwrap_or_else(|| insert_root_includes_line(&content, sources)))
}

fn replace_root_package_line(project_text: &str, package: &str) -> Option<String> {
    let mut changed = false;
    let mut out = String::with_capacity(project_text.len() + package.len());
    let mut quoted_package = String::new();
    push_quoted(&mut quoted_package, package);
    for line in project_text.split_inclusive('\n') {
        let newline_len = if line.ends_with("\r\n") {
            2
        } else if line.ends_with('\n') {
            1
        } else {
            0
        };
        let (body, newline) = line.split_at(line.len() - newline_len);
        let indent_len = body.len() - body.trim_start().len();
        let trimmed = &body[indent_len..];
        if !changed && root_package_assignment(trimmed) {
            out.push_str(&body[..indent_len]);
            out.push_str("package = ");
            out.push_str(&quoted_package);
            out.push_str(newline);
            changed = true;
        } else {
            out.push_str(line);
        }
    }
    changed.then_some(out)
}

fn root_package_assignment(line: &str) -> bool {
    let Some(rest) = line.strip_prefix("package") else {
        return false;
    };
    rest.trim_start().starts_with('=')
}

fn replace_root_includes_line(project_text: &str, sources: &[String]) -> Option<String> {
    replace_root_line(
        project_text,
        root_includes_assignment,
        &includes_line(sources),
    )
}

fn replace_root_line(
    project_text: &str,
    matcher: impl Fn(&str) -> bool,
    replacement: &str,
) -> Option<String> {
    let mut changed = false;
    let mut out = String::with_capacity(project_text.len() + replacement.len());
    for line in project_text.split_inclusive('\n') {
        let newline_len = if line.ends_with("\r\n") {
            2
        } else if line.ends_with('\n') {
            1
        } else {
            0
        };
        let (body, newline) = line.split_at(line.len() - newline_len);
        let indent_len = body.len() - body.trim_start().len();
        let trimmed = &body[indent_len..];
        if !changed && matcher(trimmed) {
            out.push_str(&body[..indent_len]);
            out.push_str(replacement);
            out.push_str(newline);
            changed = true;
        } else {
            out.push_str(line);
        }
    }
    changed.then_some(out)
}

fn prepend_root_package_line(project_text: &str, package: &str) -> String {
    let mut package_line = String::new();
    package_line.push_str("package = ");
    push_quoted(&mut package_line, package);
    package_line.push('\n');
    package_line.push_str(project_text);
    package_line
}

fn insert_root_includes_line(project_text: &str, sources: &[String]) -> String {
    let mut out =
        String::with_capacity(project_text.len() + sources.iter().map(String::len).sum::<usize>());
    let mut inserted = false;
    for line in project_text.split_inclusive('\n') {
        out.push_str(line);
        if !inserted && root_package_assignment(line.trim()) {
            out.push_str(&includes_line(sources));
            out.push('\n');
            inserted = true;
        }
    }
    if !inserted {
        out.push_str(&includes_line(sources));
        out.push('\n');
    }
    out
}

fn root_includes_assignment(line: &str) -> bool {
    let Some(rest) = line.strip_prefix("includes") else {
        return false;
    };
    rest.trim_start().starts_with('=')
}

fn includes_line(sources: &[String]) -> String {
    let mut out = String::from("includes = [");
    for (index, source) in sources.iter().enumerate() {
        if index > 0 {
            out.push_str(", ");
        }
        push_quoted(&mut out, source);
    }
    out.push(']');
    out
}

fn schema_include_paths(project: &Path) -> Result<Vec<SchemaInclude>> {
    ensure_toml_path(project, "project")?;
    let project_text = fs::read_to_string(project)
        .with_context(|| format!("failed to read project file `{}`", project.display()))?;
    let project_doc = project_text
        .parse::<toml::Value>()
        .with_context(|| format!("failed to parse project TOML `{}`", project.display()))?;
    let includes = project_doc
        .get("includes")
        .and_then(toml_array_strings)
        .unwrap_or_default();
    if includes.is_empty() {
        anyhow::bail!("Studio project must declare at least one TOML include");
    }

    schema_includes_from_sources(project, &includes)
}

fn ensure_toml_path(path: &Path, label: &str) -> Result<()> {
    let extension = path.extension().and_then(|value| value.to_str());
    if extension != Some("toml") {
        anyhow::bail!(
            "Studio save currently supports TOML {label} files only: {}",
            path.display()
        );
    }
    Ok(())
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
