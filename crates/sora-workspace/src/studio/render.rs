use std::{collections::BTreeMap, path::Path};

use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::{Map, Value};
use sora_input_schema::schema::render_schema_module as render_canonical_schema_module;
use sora_schema::model::{EnumSchema, SchemaModule, StructSchema, TableSchema, UnionSchema};

use super::model::{StudioField, StudioNode, StudioNodeKind, StudioSchema};

pub(crate) fn render_schema_module(schema: &StudioSchema) -> String {
    render_schema_module_for_path(schema, Path::new("schema.scon"), "")
        .unwrap_or_else(|error| format!("# Failed to render schema: {error:#}\n"))
}

pub(crate) fn render_schema_module_for_path(
    schema: &StudioSchema,
    path: &Path,
    current: &str,
) -> Result<String> {
    let _ = current;
    let module = schema_module_from_studio(schema)?;
    render_canonical_schema_module(path, &module).map_err(anyhow::Error::from)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StudioDocumentFormat {
    Scon,
    Toml,
    Yaml,
    Json,
    Lua,
}

pub(crate) fn document_format(path: &Path) -> Result<StudioDocumentFormat> {
    match path.extension().and_then(|extension| extension.to_str()) {
        Some("scon") => Ok(StudioDocumentFormat::Scon),
        Some("toml") => Ok(StudioDocumentFormat::Toml),
        Some("yaml" | "yml") => Ok(StudioDocumentFormat::Yaml),
        Some("json") => Ok(StudioDocumentFormat::Json),
        Some("lua") => Ok(StudioDocumentFormat::Lua),
        Some(extension) => anyhow::bail!(
            "Studio supports schema files with scon, toml, yaml, yml, json, or lua extensions: `{}` has `{extension}`",
            path.display()
        ),
        None => anyhow::bail!(
            "Studio schema file `{}` must have an extension",
            path.display()
        ),
    }
}

fn schema_module_from_studio(schema: &StudioSchema) -> Result<SchemaModule> {
    let document = serde_json::from_value::<StudioModuleDocument>(schema_module_value(schema))
        .context("failed to materialize Studio schema module")?;
    Ok(SchemaModule {
        project: None,
        groups: BTreeMap::new(),
        views: BTreeMap::new(),
        codegen: None,
        localization: None,
        includes: Vec::new(),
        enums: document.enums,
        structs: document.structs,
        unions: document.unions,
        tables: document.tables,
    })
}

#[derive(Debug, Deserialize)]
struct StudioModuleDocument {
    #[serde(default)]
    enums: Vec<EnumSchema>,
    #[serde(default)]
    structs: Vec<StructSchema>,
    #[serde(default)]
    unions: Vec<UnionSchema>,
    #[serde(default)]
    tables: Vec<TableSchema>,
}

pub(super) fn schema_module_value(schema: &StudioSchema) -> Value {
    let mut root = Map::new();
    let enums = schema
        .nodes
        .iter()
        .filter(|node| node.kind == StudioNodeKind::Enum)
        .map(enum_node_value)
        .collect::<Vec<_>>();
    let structs = schema
        .nodes
        .iter()
        .filter(|node| node.kind == StudioNodeKind::Struct)
        .map(struct_node_value)
        .collect::<Vec<_>>();
    let unions = schema
        .nodes
        .iter()
        .filter(|node| node.kind == StudioNodeKind::Union)
        .map(union_node_value)
        .collect::<Vec<_>>();
    let tables = schema
        .nodes
        .iter()
        .filter(|node| node.kind == StudioNodeKind::Table)
        .map(table_node_value)
        .collect::<Vec<_>>();
    if !enums.is_empty() {
        root.insert("enums".to_owned(), Value::Array(enums));
    }
    if !structs.is_empty() {
        root.insert("structs".to_owned(), Value::Array(structs));
    }
    if !unions.is_empty() {
        root.insert("unions".to_owned(), Value::Array(unions));
    }
    if !tables.is_empty() {
        root.insert("tables".to_owned(), Value::Array(tables));
    }
    Value::Object(root)
}

fn enum_node_value(node: &StudioNode) -> Value {
    let mut object = node_object(node);
    if let Some(comment) = node.metadata.get("comment") {
        object.insert("comment".to_owned(), Value::String(comment.clone()));
    }
    let values = node
        .fields
        .iter()
        .filter(|field| field.ty == "enum value")
        .map(|field| {
            let mut value = Map::from_iter([
                (
                    "id".to_owned(),
                    Value::Number(field.enum_value_id.unwrap_or_default().into()),
                ),
                ("name".to_owned(), Value::String(field.name.clone())),
            ]);
            if let Some(comment) = &field.comment {
                value.insert("comment".to_owned(), Value::String(comment.clone()));
            }
            Value::Object(value)
        })
        .collect::<Vec<_>>();
    object.insert("values".to_owned(), Value::Array(values));
    if !node.aliases.is_empty() {
        object.insert(
            "aliases".to_owned(),
            Value::Array(
                node.aliases
                    .iter()
                    .map(|alias| {
                        serde_json::json!({
                            "name": alias.name,
                            "alias": alias.alias,
                        })
                    })
                    .collect(),
            ),
        );
    }
    Value::Object(object)
}

fn struct_node_value(node: &StudioNode) -> Value {
    let mut object = node_object(node);
    let fields = node.fields.iter().map(field_value).collect::<Vec<_>>();
    if !fields.is_empty() {
        object.insert("fields".to_owned(), Value::Array(fields));
    }
    Value::Object(object)
}

fn union_node_value(node: &StudioNode) -> Value {
    let mut object = node_object(node);
    if let Some(tag) = node.metadata.get("tag").filter(|tag| *tag != "type") {
        object.insert("tag".to_owned(), Value::String(tag.clone()));
    }
    let variants = union_variants(node)
        .into_iter()
        .map(|(name, fields)| {
            let mut variant = Map::new();
            variant.insert("name".to_owned(), Value::String(name.clone()));
            if let Some(groups) = node
                .fields
                .iter()
                .find(|field| field.ty == "variant" && field.name == name)
                .map(|field| field.groups.as_slice())
            {
                push_groups_value(&mut variant, groups);
            }
            let fields = fields.iter().map(field_value).collect::<Vec<_>>();
            if !fields.is_empty() {
                variant.insert("fields".to_owned(), Value::Array(fields));
            }
            Value::Object(variant)
        })
        .collect::<Vec<_>>();
    if !variants.is_empty() {
        object.insert("variants".to_owned(), Value::Array(variants));
    }
    Value::Object(object)
}

fn table_node_value(node: &StudioNode) -> Value {
    let mut object = node_object(node);
    let id = node
        .metadata
        .get("id")
        .cloned()
        .unwrap_or_else(|| default_table_id(&node.name));
    object.insert("id".to_owned(), Value::String(id));
    if let Some(mode) = node.metadata.get("mode") {
        object.insert("mode".to_owned(), Value::String(mode.clone()));
    }
    if let Some(key) = node
        .metadata
        .get("key")
        .filter(|_| node.metadata.get("mode").is_none_or(|mode| mode == "map"))
        .filter(|value| *value != "<none>")
    {
        object.insert("key".to_owned(), Value::String(key.clone()));
    }
    if let Some(source) = node.metadata.get("source") {
        let mut source_object = Map::new();
        source_object.insert("file".to_owned(), Value::String(source.clone()));
        if let Some(format) = node.metadata.get("format") {
            source_object.insert("format".to_owned(), Value::String(format.clone()));
        }
        if let Some(sheet) = node.metadata.get("sheet") {
            source_object.insert("sheet".to_owned(), Value::String(sheet.clone()));
        }
        object.insert("source".to_owned(), Value::Object(source_object));
    }
    let fields = node
        .fields
        .iter()
        .map(table_field_value)
        .collect::<Vec<_>>();
    if !fields.is_empty() {
        object.insert("fields".to_owned(), Value::Array(fields));
    }
    if !node.indexes.is_empty() {
        object.insert(
            "indexes".to_owned(),
            Value::Array(
                node.indexes
                    .iter()
                    .map(|index| {
                        serde_json::json!({
                            "name": index.name,
                            "fields": index.fields,
                            "unique": index.unique,
                        })
                    })
                    .collect(),
            ),
        );
    }
    Value::Object(object)
}

fn default_table_id(name: &str) -> String {
    let chars = name.chars().collect::<Vec<_>>();
    let mut id = String::new();
    for (index, ch) in chars.iter().copied().enumerate() {
        if ch.is_ascii_uppercase()
            && index > 0
            && (chars[index - 1].is_ascii_lowercase()
                || chars[index - 1].is_ascii_digit()
                || chars
                    .get(index + 1)
                    .is_some_and(|next| next.is_ascii_lowercase()))
        {
            id.push('_');
        }
        id.push(ch.to_ascii_lowercase());
    }
    id
}

fn node_object(node: &StudioNode) -> Map<String, Value> {
    let mut object = Map::new();
    object.insert("name".to_owned(), Value::String(node.name.clone()));
    push_groups_value(&mut object, &node.groups);
    object
}

fn field_value(field: &StudioField) -> Value {
    Value::Object(field_object(field))
}

fn table_field_value(field: &StudioField) -> Value {
    let mut object = field_object(field);
    if let Some(source) = parse_source(&field.source) {
        let mut from = Map::new();
        from.insert("table".to_owned(), Value::String(source.table));
        from.insert("parent_key".to_owned(), Value::String(source.parent_key));
        from.insert("child_key".to_owned(), Value::String(source.child_key));
        if let Some(value_field) = source.value_field {
            from.insert("field".to_owned(), Value::String(value_field));
        }
        if let Some(order_by) = source.order_by {
            from.insert("order_by".to_owned(), Value::String(order_by));
        }
        object.insert("from".to_owned(), Value::Object(from));
    }
    Value::Object(object)
}

fn field_object(field: &StudioField) -> Map<String, Value> {
    let mut object = Map::new();
    object.insert("name".to_owned(), Value::String(field.name.clone()));
    object.insert("type".to_owned(), Value::String(field.ty.clone()));
    push_groups_value(&mut object, &field.groups);
    if let Some(comment) = &field.comment {
        object.insert("comment".to_owned(), Value::String(comment.clone()));
    }
    if let Some(default) = &field.default {
        object.insert("default".to_owned(), Value::String(default.clone()));
    }
    if let Some(range) = field.range {
        object.insert(
            "range".to_owned(),
            Value::Array(vec![range[0].into(), range[1].into()]),
        );
    }
    if let Some(length) = field.length {
        object.insert(
            "length".to_owned(),
            Value::Array(vec![length[0].into(), length[1].into()]),
        );
    }
    if let Some(parser) = parse_parser(&field.parser) {
        let mut parser_object = Map::new();
        parser_object.insert("kind".to_owned(), Value::String(parser.kind));
        for (key, value) in parser.options {
            parser_object.insert(key, Value::String(value));
        }
        object.insert("parser".to_owned(), Value::Object(parser_object));
    }
    object
}

fn push_groups_value(object: &mut Map<String, Value>, groups: &[String]) {
    if groups.is_empty() {
        return;
    }
    if groups.len() == 1 {
        object.insert("groups".to_owned(), Value::String(groups[0].clone()));
    } else {
        object.insert(
            "groups".to_owned(),
            Value::Array(groups.iter().cloned().map(Value::String).collect()),
        );
    }
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

pub(crate) fn push_quoted(out: &mut String, value: &str) {
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
pub(crate) struct ParserParts {
    pub(crate) kind: String,
    pub(crate) options: BTreeMap<String, String>,
}

pub(crate) fn parse_parser(value: &Option<String>) -> Option<ParserParts> {
    let value = value.as_deref()?.trim();
    if value.is_empty() {
        return None;
    }
    if let Some((kind, rest)) = value.split_once(" (") {
        let options_text = rest.strip_suffix(')').unwrap_or(rest);
        return Some(ParserParts {
            kind: kind.trim().to_owned(),
            options: parse_parser_options(options_text),
        });
    }
    Some(ParserParts {
        kind: value.to_owned(),
        options: BTreeMap::new(),
    })
}

pub(crate) fn parse_parser_options(options_text: &str) -> BTreeMap<String, String> {
    let mut options = BTreeMap::new();
    for entry in split_parser_options(options_text) {
        let entry = entry.trim();
        let Some((key, value)) = entry.split_once('=') else {
            continue;
        };
        options.insert(
            key.trim().to_owned(),
            parse_parser_option_value(value.trim()),
        );
    }
    options
}

fn split_parser_options(options_text: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut start = 0;
    let mut quoted = false;
    let mut escaped = false;
    for (index, byte) in options_text.bytes().enumerate() {
        if escaped {
            escaped = false;
            continue;
        }
        if byte == b'\\' {
            escaped = true;
            continue;
        }
        if byte == b'"' {
            quoted = !quoted;
        }
        if byte == b',' && !quoted {
            parts.push(&options_text[start..index]);
            start = index + 1;
        }
    }
    parts.push(&options_text[start..]);
    parts
}

pub(crate) fn parse_parser_option_value(value: &str) -> String {
    value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .map(unescape_parser_option_value)
        .unwrap_or_else(|| value.to_owned())
}

fn unescape_parser_option_value(value: &str) -> String {
    let mut out = String::new();
    let mut escaped = false;
    for ch in value.chars() {
        if escaped {
            out.push(match ch {
                '"' => '"',
                '\\' => '\\',
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                other => other,
            });
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
        } else {
            out.push(ch);
        }
    }
    if escaped {
        out.push('\\');
    }
    out
}

#[derive(Debug)]
pub(crate) struct SourceParts {
    pub(crate) table: String,
    pub(crate) parent_key: String,
    pub(crate) child_key: String,
    value_field: Option<String>,
    order_by: Option<String>,
}

pub(crate) fn parse_source(value: &Option<String>) -> Option<SourceParts> {
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

pub(crate) fn render_lua_document(value: &Value) -> String {
    let mut out = String::from("return ");
    push_lua_value(&mut out, value, 0);
    out.push('\n');
    out
}

pub(crate) fn push_lua_value(out: &mut String, value: &Value, indent: usize) {
    match value {
        Value::Null => out.push_str("nil"),
        Value::Bool(value) => out.push_str(if *value { "true" } else { "false" }),
        Value::Number(value) => out.push_str(&value.to_string()),
        Value::String(value) => push_quoted(out, value),
        Value::Array(values) => push_lua_array(out, values, indent),
        Value::Object(values) => push_lua_object(out, values, indent),
    }
}

fn push_lua_array(out: &mut String, values: &[Value], indent: usize) {
    if values.is_empty() {
        out.push_str("{}");
        return;
    }
    out.push_str("{\n");
    for value in values {
        push_indent(out, indent + 1);
        push_lua_value(out, value, indent + 1);
        out.push_str(",\n");
    }
    push_indent(out, indent);
    out.push('}');
}

fn push_lua_object(out: &mut String, values: &Map<String, Value>, indent: usize) {
    if values.is_empty() {
        out.push_str("{}");
        return;
    }
    out.push_str("{\n");
    for (key, value) in values {
        push_indent(out, indent + 1);
        push_lua_key(out, key);
        out.push_str(" = ");
        push_lua_value(out, value, indent + 1);
        out.push_str(",\n");
    }
    push_indent(out, indent);
    out.push('}');
}

fn push_lua_key(out: &mut String, key: &str) {
    if is_lua_identifier(key) {
        out.push_str(key);
    } else {
        out.push('[');
        push_quoted(out, key);
        out.push(']');
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

fn push_indent(out: &mut String, indent: usize) {
    for _ in 0..indent {
        out.push_str("  ");
    }
}
