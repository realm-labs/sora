use std::collections::{BTreeMap, BTreeSet};

use sora_ir::model::{ConfigIr, FieldIr, TableModeIr, TypeIr};
use sora_schema::model::{
    FieldSchema, ParserSchema, SchemaFile, ScopeSchema, TableFieldFromSchema, TableFieldSchema,
    TableModeSchema,
};

use crate::{
    model::{
        StudioEdge, StudioEdgeKind, StudioField, StudioNode, StudioNodeKind, StudioSchema,
        StudioSummary,
    },
    render::parse_source,
};

pub(crate) fn build_schema(
    ir: &ConfigIr,
    sources: &[String],
    source_by_node: &BTreeMap<String, String>,
) -> StudioSchema {
    let mut nodes = Vec::new();
    let mut edges = BTreeSet::new();

    for item in &ir.enums {
        nodes.push(StudioNode {
            id: node_id(StudioNodeKind::Enum, &item.name),
            name: item.name.clone(),
            kind: StudioNodeKind::Enum,
            source: node_source(source_by_node, sources, StudioNodeKind::Enum, &item.name),
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
            source: node_source(source_by_node, sources, StudioNodeKind::Struct, &item.name),
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
            source: node_source(source_by_node, sources, StudioNodeKind::Union, &item.name),
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
            source: node_source(source_by_node, sources, StudioNodeKind::Table, &item.name),
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
        sources: sources.to_vec(),
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

pub(crate) fn build_schema_from_raw(
    schema: &SchemaFile,
    sources: &[String],
    source_by_node: &BTreeMap<String, String>,
) -> StudioSchema {
    let mut nodes = Vec::new();

    for item in &schema.enums {
        nodes.push(StudioNode {
            id: node_id(StudioNodeKind::Enum, &item.name),
            name: item.name.clone(),
            kind: StudioNodeKind::Enum,
            source: node_source(source_by_node, sources, StudioNodeKind::Enum, &item.name),
            scope: raw_scope(&item.scope),
            subtitle: format!("{} values", item.values.len()),
            fields: item
                .values
                .iter()
                .map(|value| StudioField {
                    name: value.clone(),
                    ty: "enum value".to_owned(),
                    scope: raw_scope(&item.scope),
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

    for item in &schema.structs {
        nodes.push(StudioNode {
            id: node_id(StudioNodeKind::Struct, &item.name),
            name: item.name.clone(),
            kind: StudioNodeKind::Struct,
            source: node_source(source_by_node, sources, StudioNodeKind::Struct, &item.name),
            scope: raw_scope(&item.scope),
            subtitle: format!("{} fields", item.fields.len()),
            fields: item.fields.iter().map(raw_field).collect(),
            metadata: BTreeMap::from([("fields".to_owned(), item.fields.len().to_string())]),
        });
    }

    for item in &schema.unions {
        let fields = item
            .variants
            .iter()
            .flat_map(|variant| {
                std::iter::once(StudioField {
                    name: variant.name.clone(),
                    ty: "variant".to_owned(),
                    scope: raw_scope(&variant.scope),
                    parser: None,
                    comment: None,
                    default: None,
                    range: None,
                    length: None,
                    source: None,
                })
                .chain(variant.fields.iter().map(move |field| {
                    let mut field = raw_field(field);
                    field.name = format!("{}.{}", variant.name, field.name);
                    field
                }))
            })
            .collect::<Vec<_>>();
        nodes.push(StudioNode {
            id: node_id(StudioNodeKind::Union, &item.name),
            name: item.name.clone(),
            kind: StudioNodeKind::Union,
            source: node_source(source_by_node, sources, StudioNodeKind::Union, &item.name),
            scope: raw_scope(&item.scope),
            subtitle: format!("{} variants", item.variants.len()),
            fields,
            metadata: BTreeMap::from([
                ("tag".to_owned(), item.tag.clone()),
                ("variants".to_owned(), item.variants.len().to_string()),
            ]),
        });
    }

    for item in &schema.tables {
        let mut metadata = BTreeMap::from([
            ("mode".to_owned(), raw_table_mode(item.mode).to_owned()),
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
            id: node_id(StudioNodeKind::Table, &item.name),
            name: item.name.clone(),
            kind: StudioNodeKind::Table,
            source: node_source(source_by_node, sources, StudioNodeKind::Table, &item.name),
            scope: raw_scope(&item.scope),
            subtitle: format!(
                "{} table, {} fields",
                raw_table_mode(item.mode),
                item.fields.len()
            ),
            fields: item.fields.iter().map(raw_table_field).collect(),
            metadata,
        });
    }

    let edges = build_studio_edges(&nodes);
    StudioSchema {
        package: schema.package.clone(),
        sources: sources.to_vec(),
        summary: StudioSummary {
            enums: schema.enums.len(),
            structs: schema.structs.len(),
            unions: schema.unions.len(),
            tables: schema.tables.len(),
            edges: edges.len(),
        },
        nodes,
        edges,
    }
}

fn raw_field(field: &FieldSchema) -> StudioField {
    StudioField {
        name: field.name.clone(),
        ty: field.ty.clone(),
        scope: raw_scope(&field.scope),
        parser: raw_parser(&field.parser),
        comment: field.comment.clone(),
        default: field.default.clone(),
        range: field.range,
        length: field.length,
        source: None,
    }
}

fn raw_table_field(field: &TableFieldSchema) -> StudioField {
    StudioField {
        name: field.name.clone(),
        ty: field.ty.clone(),
        scope: raw_scope(&field.scope),
        parser: raw_parser(&field.parser),
        comment: field.comment.clone(),
        default: field.default.clone(),
        range: field.range,
        length: field.length,
        source: raw_from(&field.from),
    }
}

fn raw_parser(parser: &Option<ParserSchema>) -> Option<String> {
    parser.as_ref().map(|parser| {
        let options = parser
            .options
            .iter()
            .map(|(key, value)| format!("{key}={}", quote_parser_option_value(value)))
            .collect::<Vec<_>>();
        if options.is_empty() {
            parser.kind.clone()
        } else {
            format!("{} ({})", parser.kind, options.join(", "))
        }
    })
}

fn raw_from(from: &Option<TableFieldFromSchema>) -> Option<String> {
    from.as_ref().map(|from| {
        let mut value = format!(
            "{}: {} -> {}",
            from.table,
            from.child_key.as_deref().unwrap_or("<child_key>"),
            from.parent_key.as_deref().unwrap_or("<parent_key>")
        );
        if let Some(field) = &from.value_field {
            value.push_str(&format!(", field={field}"));
        }
        if let Some(order_by) = &from.order_by {
            value.push_str(&format!(", order_by={order_by}"));
        }
        value
    })
}

fn raw_scope(scope: &ScopeSchema) -> String {
    scope.values.join(",")
}

fn raw_table_mode(mode: TableModeSchema) -> &'static str {
    match mode {
        TableModeSchema::List => "list",
        TableModeSchema::Map => "map",
        TableModeSchema::Singleton => "singleton",
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
                .map(|(key, value)| format!("{key}={}", quote_parser_option_value(value)))
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

fn quote_parser_option_value(value: &str) -> String {
    let mut out = String::from("\"");
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            other => out.push(other),
        }
    }
    out.push('"');
    out
}

fn build_studio_edges(nodes: &[StudioNode]) -> Vec<StudioEdge> {
    let mut edges = BTreeSet::new();
    for node in nodes {
        for field in &node.fields {
            for (table, ref_field) in ref_targets(&field.ty) {
                if let Some(target) = nodes
                    .iter()
                    .find(|item| item.kind == StudioNodeKind::Table && item.name == table)
                {
                    edges.insert(StudioEdge {
                        id: edge_id(&node.id, &target.id, StudioEdgeKind::Ref, &field.name),
                        source: node.id.clone(),
                        target: target.id.clone(),
                        kind: StudioEdgeKind::Ref,
                        label: field.name.clone(),
                        target_label: Some(ref_field),
                    });
                }
            }

            for target in nodes
                .iter()
                .filter(|item| item.kind != StudioNodeKind::Table)
            {
                if type_mentions_symbol(&field.ty, &target.name) {
                    edges.insert(StudioEdge {
                        id: edge_id(&node.id, &target.id, StudioEdgeKind::Type, &field.name),
                        source: node.id.clone(),
                        target: target.id.clone(),
                        kind: StudioEdgeKind::Type,
                        label: field.name.clone(),
                        target_label: None,
                    });
                }
            }

            if let Some(source) = parse_source(&field.source) {
                if let Some(target) = nodes
                    .iter()
                    .find(|item| item.kind == StudioNodeKind::Table && item.name == source.table)
                {
                    edges.insert(StudioEdge {
                        id: edge_id(&node.id, &target.id, StudioEdgeKind::Derived, &field.name),
                        source: node.id.clone(),
                        target: target.id.clone(),
                        kind: StudioEdgeKind::Derived,
                        label: field.name.clone(),
                        target_label: Some(source.child_key),
                    });
                }
            }
        }
    }
    edges.into_iter().collect()
}

fn ref_targets(ty: &str) -> Vec<(String, String)> {
    let mut targets = Vec::new();
    let mut rest = ty;
    while let Some(start) = rest.find("ref<") {
        let after_start = &rest[start + 4..];
        let Some(end) = after_start.find('>') else {
            break;
        };
        let body = &after_start[..end];
        if let Some((table, field)) = body.split_once('.') {
            targets.push((table.trim().to_owned(), field.trim().to_owned()));
        }
        rest = &after_start[end + 1..];
    }
    targets
}

fn type_mentions_symbol(ty: &str, symbol: &str) -> bool {
    if symbol.is_empty() {
        return false;
    }
    let bytes = ty.as_bytes();
    let symbol_bytes = symbol.as_bytes();
    let mut offset = 0;
    while let Some(position) = ty[offset..].find(symbol) {
        let start = offset + position;
        let end = start + symbol_bytes.len();
        let before = start
            .checked_sub(1)
            .and_then(|index| bytes.get(index))
            .copied();
        let after = bytes.get(end).copied();
        if !is_ident_byte(before) && !is_ident_byte(after) {
            return true;
        }
        offset = end;
    }
    false
}

fn is_ident_byte(value: Option<u8>) -> bool {
    value.is_some_and(|value| value.is_ascii_alphanumeric() || value == b'_')
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
        TypeIr::Bool
        | TypeIr::I8
        | TypeIr::U8
        | TypeIr::I16
        | TypeIr::U16
        | TypeIr::I32
        | TypeIr::U32
        | TypeIr::I64
        | TypeIr::Duration
        | TypeIr::DateTime
        | TypeIr::F32
        | TypeIr::F64
        | TypeIr::String
        | TypeIr::Text => {}
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

pub(crate) fn node_id(kind: StudioNodeKind, name: &str) -> String {
    let prefix = match kind {
        StudioNodeKind::Enum => "enum",
        StudioNodeKind::Struct => "struct",
        StudioNodeKind::Union => "union",
        StudioNodeKind::Table => "table",
    };
    format!("{prefix}:{name}")
}

fn node_source(
    source_by_node: &BTreeMap<String, String>,
    sources: &[String],
    kind: StudioNodeKind,
    name: &str,
) -> String {
    source_by_node
        .get(&node_id(kind, name))
        .cloned()
        .or_else(|| sources.first().cloned())
        .unwrap_or_default()
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
