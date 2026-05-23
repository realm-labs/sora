use std::{
    collections::{BTreeMap, BTreeSet},
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::{Context, Result};
use axum::{Json, Router, extract::State, routing::get};
use serde::Serialize;
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

#[derive(Debug, Serialize)]
pub struct StudioSchema {
    pub package: String,
    pub summary: StudioSummary,
    pub nodes: Vec<StudioNode>,
    pub edges: Vec<StudioEdge>,
}

#[derive(Debug, Serialize)]
pub struct StudioSummary {
    pub enums: usize,
    pub structs: usize,
    pub unions: usize,
    pub tables: usize,
    pub edges: usize,
}

#[derive(Debug, Serialize)]
pub struct StudioNode {
    pub id: String,
    pub name: String,
    pub kind: StudioNodeKind,
    pub scope: String,
    pub subtitle: String,
    pub fields: Vec<StudioField>,
    pub metadata: BTreeMap<String, String>,
}

#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum StudioNodeKind {
    Enum,
    Struct,
    Union,
    Table,
}

#[derive(Debug, Serialize)]
pub struct StudioField {
    pub name: String,
    pub ty: String,
    pub scope: String,
    pub parser: Option<String>,
    pub comment: Option<String>,
    pub source: Option<String>,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct StudioEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    pub kind: StudioEdgeKind,
    pub label: String,
}

#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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
            collect_field_edges(&owner, &variant.fields, &mut edges);
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
        TypeIr::Ref { table, .. } => {
            let target = node_id(StudioNodeKind::Table, table);
            edges.insert(StudioEdge {
                id: edge_id(owner, &target, StudioEdgeKind::Ref, field_name),
                source: owner.to_owned(),
                target,
                kind: StudioEdgeKind::Ref,
                label: field_name.to_owned(),
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
