use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::{Context, Result};
use axum::{
    Json, Router,
    extract::State,
    routing::{get, put},
};
use serde::{Deserialize, Serialize};
use sora_input_schema::schema::load_project_schema_file;
use sora_ir::{
    model::{ConfigIr, FieldIr, TableModeIr, TypeIr},
    normalize::normalize_schema,
    validate::validate_config_ir,
};
use tower_http::cors::CorsLayer;

#[derive(Debug, Clone)]
pub struct StudioOptions {
    pub project: PathBuf,
    pub host: IpAddr,
    pub port: u16,
}

impl StudioOptions {
    pub fn local(project: PathBuf, port: u16) -> Self {
        Self {
            project,
            host: IpAddr::V4(Ipv4Addr::LOCALHOST),
            port,
        }
    }
}

pub fn run_blocking(options: StudioOptions) -> Result<()> {
    let runtime = tokio::runtime::Runtime::new().context("failed to start async runtime")?;
    runtime.block_on(run(options))
}

pub async fn run(options: StudioOptions) -> Result<()> {
    let addr = SocketAddr::new(options.host, options.port);
    let project = options.project.clone();
    let state = StudioState {
        project: Arc::new(project.clone()),
    };
    let app = Router::new()
        .route("/api/health", get(health))
        .route("/api/schema", get(schema))
        .route("/api/schema", put(save_schema))
        .with_state(state)
        .layer(CorsLayer::permissive());
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("failed to bind studio server at http://{addr}"))?;

    println!("Sora Studio API: http://{addr}");
    println!("Project: {}", project.display());
    println!("Frontend dev server: cd apps/studio && npm run dev");

    axum::serve(listener, app)
        .await
        .context("studio server stopped unexpectedly")
}

#[derive(Debug, Clone)]
struct StudioState {
    project: Arc<PathBuf>,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { ok: true })
}

async fn schema(State(state): State<StudioState>) -> Json<StudioSchemaResponse> {
    Json(load_studio_schema(&state.project))
}

async fn save_schema(
    State(state): State<StudioState>,
    Json(schema): Json<StudioSchema>,
) -> Json<StudioSchemaResponse> {
    Json(save_studio_schema(&state.project, &schema))
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    ok: bool,
}

#[derive(Debug, Serialize)]
pub struct StudioSchemaResponse {
    pub ok: bool,
    pub project: String,
    pub diagnostics: Vec<StudioDiagnostic>,
    pub schema: Option<StudioSchema>,
}

#[derive(Debug, Serialize)]
pub struct StudioDiagnostic {
    pub level: DiagnosticLevel,
    pub message: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticLevel {
    Error,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudioSchema {
    pub package: String,
    pub summary: StudioSummary,
    pub nodes: Vec<StudioNode>,
    pub edges: Vec<StudioEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudioSummary {
    pub enums: usize,
    pub structs: usize,
    pub unions: usize,
    pub tables: usize,
    pub edges: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudioNode {
    pub id: String,
    pub name: String,
    pub kind: StudioNodeKind,
    pub scope: String,
    pub subtitle: String,
    pub fields: Vec<StudioField>,
    pub metadata: BTreeMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum StudioNodeKind {
    Enum,
    Struct,
    Union,
    Table,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudioField {
    pub name: String,
    pub ty: String,
    pub scope: String,
    pub parser: Option<String>,
    pub comment: Option<String>,
    pub default: Option<String>,
    pub range: Option<[i64; 2]>,
    pub length: Option<[usize; 2]>,
    pub source: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct StudioEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    pub kind: StudioEdgeKind,
    pub label: String,
    #[serde(default, rename = "targetLabel")]
    pub target_label: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum StudioEdgeKind {
    Type,
    Ref,
    Derived,
}

fn load_studio_schema(project: &Path) -> StudioSchemaResponse {
    match load_project_schema_file(project)
        .and_then(normalize_schema)
        .and_then(|ir| {
            validate_config_ir(&ir)?;
            Ok(ir)
        }) {
        Ok(ir) => {
            let schema = build_schema(&ir);
            StudioSchemaResponse {
                ok: true,
                project: project.display().to_string(),
                diagnostics: vec![StudioDiagnostic {
                    level: DiagnosticLevel::Info,
                    message: "schema loaded successfully".to_owned(),
                }],
                schema: Some(schema),
            }
        }
        Err(error) => StudioSchemaResponse {
            ok: false,
            project: project.display().to_string(),
            diagnostics: vec![StudioDiagnostic {
                level: DiagnosticLevel::Error,
                message: error.to_string(),
            }],
            schema: None,
        },
    }
}

fn save_studio_schema(project: &Path, schema: &StudioSchema) -> StudioSchemaResponse {
    match write_studio_schema(project, schema) {
        Ok(path) => {
            let mut response = load_studio_schema(project);
            if response.ok {
                response.diagnostics = vec![StudioDiagnostic {
                    level: DiagnosticLevel::Info,
                    message: format!("schema saved to {}", path.display()),
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
            }],
            schema: Some(schema.clone()),
        },
    }
}

fn write_studio_schema(project: &Path, schema: &StudioSchema) -> Result<PathBuf> {
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
    if includes.len() != 1 {
        anyhow::bail!(
            "Studio save currently supports projects with exactly one TOML include; found {} includes",
            includes.len()
        );
    }

    let base_dir = project.parent().unwrap_or_else(|| Path::new("."));
    let include_path = base_dir.join(&includes[0]);
    ensure_toml_path(&include_path, "schema include")?;
    let content = render_schema_module(schema);
    fs::write(&include_path, content).with_context(|| {
        format!(
            "failed to write schema include `{}`",
            include_path.display()
        )
    })?;
    Ok(include_path)
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

fn render_schema_module(schema: &StudioSchema) -> String {
    let mut out = String::from("# Generated by Sora Studio. Review before committing.\n\n");
    for node in schema
        .nodes
        .iter()
        .filter(|node| node.kind == StudioNodeKind::Enum)
    {
        out.push_str("[[enums]]\n");
        push_string(&mut out, "name", &node.name);
        push_scope(&mut out, &node.scope);
        let values = node
            .fields
            .iter()
            .filter(|field| field.ty == "enum value")
            .map(|field| field.name.as_str())
            .collect::<Vec<_>>();
        push_string_array(&mut out, "values", &values);
        out.push('\n');
    }

    for node in schema
        .nodes
        .iter()
        .filter(|node| node.kind == StudioNodeKind::Struct)
    {
        out.push_str("[[structs]]\n");
        push_string(&mut out, "name", &node.name);
        push_scope(&mut out, &node.scope);
        out.push('\n');
        for field in &node.fields {
            out.push_str("[[structs.fields]]\n");
            push_field(&mut out, field);
            out.push('\n');
        }
    }

    for node in schema
        .nodes
        .iter()
        .filter(|node| node.kind == StudioNodeKind::Union)
    {
        out.push_str("[[unions]]\n");
        push_string(&mut out, "name", &node.name);
        push_scope(&mut out, &node.scope);
        if let Some(tag) = node.metadata.get("tag") {
            push_string(&mut out, "tag", tag);
        }
        out.push('\n');

        for (variant, fields) in union_variants(node) {
            out.push_str("[[unions.variants]]\n");
            push_string(&mut out, "name", &variant);
            out.push('\n');
            for field in fields {
                out.push_str("[[unions.variants.fields]]\n");
                push_field(&mut out, &field);
                out.push('\n');
            }
        }
    }

    for node in schema
        .nodes
        .iter()
        .filter(|node| node.kind == StudioNodeKind::Table)
    {
        out.push_str("[[tables]]\n");
        push_string(&mut out, "name", &node.name);
        push_scope(&mut out, &node.scope);
        if let Some(mode) = node.metadata.get("mode") {
            push_string(&mut out, "mode", mode);
        }
        if let Some(key) = node.metadata.get("key").filter(|value| *value != "<none>") {
            push_string(&mut out, "key", key);
        }
        if let Some(source) = node.metadata.get("source") {
            out.push_str("source = { ");
            push_inline_pair(&mut out, "file", source);
            if let Some(sheet) = node.metadata.get("sheet") {
                out.push_str(", ");
                push_inline_pair(&mut out, "sheet", sheet);
            }
            out.push_str(" }\n");
        }
        out.push('\n');
        for field in &node.fields {
            out.push_str("[[tables.fields]]\n");
            push_field(&mut out, field);
            if let Some(source) = parse_source(&field.source) {
                out.push_str("from = { ");
                push_inline_pair(&mut out, "table", &source.table);
                out.push_str(", ");
                push_inline_pair(&mut out, "parent_key", &source.parent_key);
                out.push_str(", ");
                push_inline_pair(&mut out, "child_key", &source.child_key);
                if let Some(value_field) = source.value_field {
                    out.push_str(", ");
                    push_inline_pair(&mut out, "field", &value_field);
                }
                if let Some(order_by) = source.order_by {
                    out.push_str(", ");
                    push_inline_pair(&mut out, "order_by", &order_by);
                }
                out.push_str(" }\n");
            }
            out.push('\n');
        }
    }

    out
}

fn union_variants(node: &StudioNode) -> Vec<(String, Vec<StudioField>)> {
    let mut variants = Vec::<(String, Vec<StudioField>)>::new();
    let mut current: Option<usize> = None;
    for field in &node.fields {
        if field.ty == "variant" {
            variants.push((field.name.clone(), Vec::new()));
            current = Some(variants.len() - 1);
            continue;
        }
        if let Some((variant, name)) = field.name.split_once('.') {
            let index = variants
                .iter()
                .position(|(candidate, _)| candidate == variant)
                .unwrap_or_else(|| {
                    variants.push((variant.to_owned(), Vec::new()));
                    variants.len() - 1
                });
            let mut next = field.clone();
            next.name = name.to_owned();
            variants[index].1.push(next);
            current = Some(index);
        } else if let Some(index) = current {
            variants[index].1.push(field.clone());
        }
    }
    variants
}

fn push_field(out: &mut String, field: &StudioField) {
    push_string(out, "name", &field.name);
    push_string(out, "type", &field.ty);
    push_scope(out, &field.scope);
    if let Some(comment) = &field.comment {
        push_string(out, "comment", comment);
    }
    if let Some(default) = &field.default {
        push_string(out, "default", default);
    }
    if let Some(range) = field.range {
        out.push_str(&format!("range = [{}, {}]\n", range[0], range[1]));
    }
    if let Some(length) = field.length {
        out.push_str(&format!("length = [{}, {}]\n", length[0], length[1]));
    }
    if let Some(parser) = parse_parser(&field.parser) {
        out.push_str("parser = { ");
        push_inline_pair(out, "kind", &parser.kind);
        for (key, value) in parser.options {
            out.push_str(", ");
            push_inline_pair(out, &key, &value);
        }
        out.push_str(" }\n");
    }
}

fn push_scope(out: &mut String, scope: &str) {
    if scope == "all" || scope.trim().is_empty() {
        return;
    }
    let values = scope.split(',').map(str::trim).collect::<Vec<_>>();
    if values.len() == 1 {
        push_string(out, "scope", values[0]);
    } else {
        push_string_array(out, "scope", &values);
    }
}

fn push_string(out: &mut String, key: &str, value: &str) {
    out.push_str(key);
    out.push_str(" = ");
    push_quoted(out, value);
    out.push('\n');
}

fn push_string_array(out: &mut String, key: &str, values: &[&str]) {
    out.push_str(key);
    out.push_str(" = [");
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            out.push_str(", ");
        }
        push_quoted(out, value);
    }
    out.push_str("]\n");
}

fn push_inline_pair(out: &mut String, key: &str, value: &str) {
    out.push_str(key);
    out.push_str(" = ");
    push_quoted(out, value);
}

fn push_quoted(out: &mut String, value: &str) {
    out.push('"');
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch => out.push(ch),
        }
    }
    out.push('"');
}

#[derive(Debug)]
struct ParserParts {
    kind: String,
    options: BTreeMap<String, String>,
}

fn parse_parser(value: &Option<String>) -> Option<ParserParts> {
    let value = value.as_deref()?.trim();
    if value.is_empty() {
        return None;
    }
    if let Some((kind, rest)) = value.split_once(" (") {
        let options_text = rest.strip_suffix(')').unwrap_or(rest);
        let options = options_text
            .split(',')
            .filter_map(|entry| {
                let (key, value) = entry.trim().split_once('=')?;
                Some((key.trim().to_owned(), value.trim().to_owned()))
            })
            .collect();
        return Some(ParserParts {
            kind: kind.trim().to_owned(),
            options,
        });
    }
    Some(ParserParts {
        kind: value.to_owned(),
        options: BTreeMap::new(),
    })
}

#[derive(Debug)]
struct SourceParts {
    table: String,
    parent_key: String,
    child_key: String,
    value_field: Option<String>,
    order_by: Option<String>,
}

fn parse_source(value: &Option<String>) -> Option<SourceParts> {
    let value = value.as_deref()?;
    let (table, rest) = value.split_once(':')?;
    let (keys, options) = rest.split_once(',').unwrap_or((rest, ""));
    let (child_key, parent_key) = keys.trim().split_once(" -> ")?;
    let mut value_field = None;
    let mut order_by = None;
    for option in options.split(',') {
        let Some((key, value)) = option.trim().split_once('=') else {
            continue;
        };
        match key {
            "field" => value_field = Some(value.to_owned()),
            "order_by" => order_by = Some(value.to_owned()),
            _ => {}
        }
    }
    Some(SourceParts {
        table: table.trim().to_owned(),
        parent_key: parent_key.trim().to_owned(),
        child_key: child_key.trim().to_owned(),
        value_field,
        order_by,
    })
}

fn build_schema(ir: &ConfigIr) -> StudioSchema {
    let mut nodes = Vec::new();
    let mut edges = BTreeSet::new();

    for item in &ir.enums {
        nodes.push(StudioNode {
            id: node_id(StudioNodeKind::Enum, &item.name),
            name: item.name.clone(),
            kind: StudioNodeKind::Enum,
            scope: item.scope.display(),
            subtitle: format!("{} values", item.values.len()),
            fields: item
                .values
                .iter()
                .map(|value| StudioField {
                    name: value.clone(),
                    ty: "enum value".to_owned(),
                    scope: item.scope.display(),
                    parser: None,
                    comment: None,
                    default: None,
                    range: None,
                    length: None,
                    source: None,
                })
                .collect(),
            metadata: BTreeMap::from([("values".to_owned(), item.values.len().to_string())]),
        });
    }

    for item in &ir.structs {
        let owner = node_id(StudioNodeKind::Struct, &item.name);
        collect_field_edges(&owner, &item.fields, &mut edges);
        nodes.push(StudioNode {
            id: owner,
            name: item.name.clone(),
            kind: StudioNodeKind::Struct,
            scope: item.scope.display(),
            subtitle: format!("{} fields", item.fields.len()),
            fields: item.fields.iter().map(studio_field).collect(),
            metadata: BTreeMap::from([("fields".to_owned(), item.fields.len().to_string())]),
        });
    }

    for item in &ir.unions {
        let owner = node_id(StudioNodeKind::Union, &item.name);
        for variant in &item.variants {
            for field in &variant.fields {
                collect_type_edges(
                    &owner,
                    &format!("{}.{}", variant.name, field.name),
                    &field.ty,
                    &mut edges,
                );
            }
        }
        let fields = item
            .variants
            .iter()
            .flat_map(|variant| {
                std::iter::once(StudioField {
                    name: variant.name.clone(),
                    ty: "variant".to_owned(),
                    scope: variant.scope.display(),
                    parser: None,
                    comment: None,
                    default: None,
                    range: None,
                    length: None,
                    source: None,
                })
                .chain(variant.fields.iter().map(move |field| {
                    let mut field = studio_field(field);
                    field.name = format!("{}.{}", variant.name, field.name);
                    field
                }))
            })
            .collect();
        nodes.push(StudioNode {
            id: owner,
            name: item.name.clone(),
            kind: StudioNodeKind::Union,
            scope: item.scope.display(),
            subtitle: format!("{} variants", item.variants.len()),
            fields,
            metadata: BTreeMap::from([
                ("tag".to_owned(), item.tag.clone()),
                ("variants".to_owned(), item.variants.len().to_string()),
            ]),
        });
    }

    for item in &ir.tables {
        let owner = node_id(StudioNodeKind::Table, &item.name);
        collect_field_edges(&owner, &item.fields, &mut edges);
        for field in &item.fields {
            if let Some(derived_from) = &field.derived_from {
                edges.insert(StudioEdge {
                    id: edge_id(
                        &owner,
                        &node_id(StudioNodeKind::Table, &derived_from.source_table),
                        StudioEdgeKind::Derived,
                        &field.name,
                    ),
                    source: owner.clone(),
                    target: node_id(StudioNodeKind::Table, &derived_from.source_table),
                    kind: StudioEdgeKind::Derived,
                    label: field.name.clone(),
                    target_label: Some(derived_from.child_key.clone()),
                });
            }
        }
        let mut metadata = BTreeMap::from([
            ("mode".to_owned(), table_mode(item.mode).to_owned()),
            (
                "key".to_owned(),
                item.key.as_deref().unwrap_or("<none>").to_owned(),
            ),
            ("fields".to_owned(), item.fields.len().to_string()),
        ]);
        if let Some(source) = &item.source {
            metadata.insert("source".to_owned(), source.file.clone());
            if let Some(sheet) = &source.sheet {
                metadata.insert("sheet".to_owned(), sheet.clone());
            }
        }
        nodes.push(StudioNode {
            id: owner,
            name: item.name.clone(),
            kind: StudioNodeKind::Table,
            scope: item.scope.display(),
            subtitle: format!(
                "{} table, {} fields",
                table_mode(item.mode),
                item.fields.len()
            ),
            fields: item.fields.iter().map(studio_field).collect(),
            metadata,
        });
    }

    let edges = edges.into_iter().collect::<Vec<_>>();
    StudioSchema {
        package: ir.package.clone(),
        summary: StudioSummary {
            enums: ir.enums.len(),
            structs: ir.structs.len(),
            unions: ir.unions.len(),
            tables: ir.tables.len(),
            edges: edges.len(),
        },
        nodes,
        edges,
    }
}

fn studio_field(field: &FieldIr) -> StudioField {
    StudioField {
        name: field.name.clone(),
        ty: field.ty.to_string(),
        scope: field.scope.display(),
        parser: field.parser.as_ref().map(|parser| {
            let options = parser
                .options
                .iter()
                .map(|(key, value)| format!("{key}={value}"))
                .collect::<Vec<_>>();
            if options.is_empty() {
                parser.kind.clone()
            } else {
                format!("{} ({})", parser.kind, options.join(", "))
            }
        }),
        comment: field.comment.clone(),
        default: field.default.clone(),
        range: field.range,
        length: field.length,
        source: field.derived_from.as_ref().map(|from| {
            let mut value = format!(
                "{}: {} -> {}",
                from.source_table, from.child_key, from.parent_key
            );
            if let Some(field) = &from.value_field {
                value.push_str(&format!(", field={field}"));
            }
            if let Some(order_by) = &from.order_by {
                value.push_str(&format!(", order_by={order_by}"));
            }
            value
        }),
    }
}

fn collect_field_edges(owner: &str, fields: &[FieldIr], edges: &mut BTreeSet<StudioEdge>) {
    for field in fields {
        collect_type_edges(owner, &field.name, &field.ty, edges);
    }
}

fn collect_type_edges(
    owner: &str,
    field_name: &str,
    ty: &TypeIr,
    edges: &mut BTreeSet<StudioEdge>,
) {
    match ty {
        TypeIr::Enum(name) => {
            insert_type_edge(owner, StudioNodeKind::Enum, name, field_name, edges)
        }
        TypeIr::Struct(name) => {
            insert_type_edge(owner, StudioNodeKind::Struct, name, field_name, edges)
        }
        TypeIr::Union(name) => {
            insert_type_edge(owner, StudioNodeKind::Union, name, field_name, edges)
        }
        TypeIr::Ref { table, field } => {
            let target = node_id(StudioNodeKind::Table, table);
            edges.insert(StudioEdge {
                id: edge_id(owner, &target, StudioEdgeKind::Ref, field_name),
                source: owner.to_owned(),
                target,
                kind: StudioEdgeKind::Ref,
                label: field_name.to_owned(),
                target_label: Some(field.clone()),
            });
        }
        TypeIr::List(inner) | TypeIr::Set(inner) | TypeIr::Optional(inner) => {
            collect_type_edges(owner, field_name, inner, edges)
        }
        TypeIr::Array { element, .. } => collect_type_edges(owner, field_name, element, edges),
        TypeIr::Map { key, value } => {
            collect_type_edges(owner, field_name, key, edges);
            collect_type_edges(owner, field_name, value, edges);
        }
        TypeIr::Bool | TypeIr::I32 | TypeIr::I64 | TypeIr::F32 | TypeIr::F64 | TypeIr::String => {}
    }
}

fn insert_type_edge(
    owner: &str,
    kind: StudioNodeKind,
    name: &str,
    field_name: &str,
    edges: &mut BTreeSet<StudioEdge>,
) {
    let target = node_id(kind, name);
    edges.insert(StudioEdge {
        id: edge_id(owner, &target, StudioEdgeKind::Type, field_name),
        source: owner.to_owned(),
        target,
        kind: StudioEdgeKind::Type,
        label: field_name.to_owned(),
        target_label: None,
    });
}

fn node_id(kind: StudioNodeKind, name: &str) -> String {
    let prefix = match kind {
        StudioNodeKind::Enum => "enum",
        StudioNodeKind::Struct => "struct",
        StudioNodeKind::Union => "union",
        StudioNodeKind::Table => "table",
    };
    format!("{prefix}:{name}")
}

fn edge_id(source: &str, target: &str, kind: StudioEdgeKind, label: &str) -> String {
    format!("{source}->{target}:{kind:?}:{label}")
}

fn table_mode(mode: TableModeIr) -> &'static str {
    match mode {
        TableModeIr::List => "list",
        TableModeIr::Map => "map",
        TableModeIr::Singleton => "singleton",
    }
}
