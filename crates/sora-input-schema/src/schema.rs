use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use sora_config_format::{DocumentError, load_document, render_document};
use sora_diagnostics::{Result, SoraError};
use sora_schema::model::{
    CodegenSchema, EnumAliasSchema, EnumSchema, EnumValueSchema, FieldSchema, GroupSchema,
    GroupSetSchema, IndexSchema, LocalizationSchema, LocalizationSourceSchema, ParserSchema,
    ProjectMetadataSchema, ProjectSchema, SchemaModule, StructSchema, TableFieldFromSchema,
    TableFieldSchema, TableModeSchema, TableSchema, TableSourceSchema, UnionSchema,
    UnionVariantSchema, ViewSchema,
};

#[derive(Debug, Clone)]
pub struct LoadedSchemaModule {
    pub path: PathBuf,
    pub module: SchemaModule,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SchemaDeclarationKind {
    Enum,
    Struct,
    Union,
    Table,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SchemaDeclarationKey {
    pub kind: SchemaDeclarationKind,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct LoadedProjectSchema {
    pub schema: ProjectSchema,
    pub modules: Vec<LoadedSchemaModule>,
    pub declaration_sources: BTreeMap<SchemaDeclarationKey, PathBuf>,
}

pub fn load_project_schema(path: &Path) -> Result<ProjectSchema> {
    Ok(load_project_schema_with_modules(path)?.schema)
}

pub fn load_project_schema_with_modules(path: &Path) -> Result<LoadedProjectSchema> {
    let root = load_schema_module(path)?;
    let project = root.project.clone().ok_or_else(|| {
        SoraError::InvalidSchema(format!(
            "project schema `{}` must declare `project`",
            path.display()
        ))
    })?;
    let views = load_views(path, &project, root.views.clone())?;
    let qualified_root = qualify_module(&root)?;
    let mut merged = ProjectSchema {
        project,
        groups: root.groups.clone(),
        views,
        codegen: root.codegen.clone().unwrap_or_default(),
        localization: root.localization.clone(),
        includes: root.includes.clone(),
        enums: qualified_root.enums.clone(),
        structs: qualified_root.structs.clone(),
        unions: qualified_root.unions.clone(),
        tables: qualified_root.tables.clone(),
    };

    let root_path = canonical_or_owned(path);
    let mut visited = BTreeSet::from([root_path.clone()]);
    let mut modules = vec![LoadedSchemaModule {
        path: root_path.clone(),
        module: root.clone(),
    }];
    let mut declaration_sources = BTreeMap::new();
    register_declaration_sources(&qualified_root, &root_path, &mut declaration_sources)?;
    merge_includes(
        path,
        &root.includes,
        &mut merged,
        &mut visited,
        &mut modules,
        &mut declaration_sources,
    )?;
    sort_project_declarations(&mut merged);
    Ok(LoadedProjectSchema {
        schema: merged,
        modules,
        declaration_sources,
    })
}

fn merge_includes(
    parent_path: &Path,
    includes: &[String],
    merged: &mut ProjectSchema,
    visited: &mut BTreeSet<PathBuf>,
    modules: &mut Vec<LoadedSchemaModule>,
    declaration_sources: &mut BTreeMap<SchemaDeclarationKey, PathBuf>,
) -> Result<()> {
    let base_dir = parent_path.parent().unwrap_or_else(|| Path::new("."));

    for include in includes {
        let include_path = base_dir.join(include);
        let canonical_key = canonical_or_owned(&include_path);
        if !visited.insert(canonical_key.clone()) {
            return Err(SoraError::InvalidSchema(format!(
                "schema include cycle or duplicate include `{}`",
                include_path.display()
            )));
        }

        let module = load_schema_module(&include_path)?;
        if module.project.is_some() {
            return Err(SoraError::InvalidSchema(format!(
                "included schema module `{}` must not declare `project`",
                include_path.display()
            )));
        }
        if !module.groups.is_empty() || !module.views.is_empty() {
            return Err(SoraError::InvalidSchema(format!(
                "included schema module `{}` must not declare groups or views",
                include_path.display()
            )));
        }
        if module.codegen.is_some() {
            return Err(SoraError::InvalidSchema(format!(
                "included schema module `{}` must not declare `codegen`",
                include_path.display()
            )));
        }
        if module.localization.is_some() {
            return Err(SoraError::InvalidSchema(format!(
                "included schema module `{}` must not declare `localization`",
                include_path.display()
            )));
        }

        let qualified_module = qualify_module(&module)?;
        register_declaration_sources(&qualified_module, &canonical_key, declaration_sources)?;
        merged.enums.extend(qualified_module.enums);
        merged.structs.extend(qualified_module.structs);
        merged.unions.extend(qualified_module.unions);
        merged.tables.extend(qualified_module.tables);
        modules.push(LoadedSchemaModule {
            path: canonical_key,
            module: module.clone(),
        });
        merge_includes(
            &include_path,
            &module.includes,
            merged,
            visited,
            modules,
            declaration_sources,
        )?;
    }

    Ok(())
}

fn qualify_module(module: &SchemaModule) -> Result<SchemaModule> {
    validate_namespace(&module.namespace)?;
    for (alias, namespace) in &module.imports {
        validate_identifier(alias, "import alias")?;
        validate_namespace(namespace)?;
    }

    let mut qualified = module.clone();
    for item in &mut qualified.enums {
        validate_identifier(&item.name, "enum")?;
        item.name = qualify_local_name(&module.namespace, &item.name);
    }
    for item in &mut qualified.structs {
        validate_identifier(&item.name, "struct")?;
        item.name = qualify_local_name(&module.namespace, &item.name);
        qualify_fields(&mut item.fields, module)?;
    }
    for item in &mut qualified.unions {
        validate_identifier(&item.name, "union")?;
        item.name = qualify_local_name(&module.namespace, &item.name);
        for variant in &mut item.variants {
            qualify_fields(&mut variant.fields, module)?;
        }
    }
    for item in &mut qualified.tables {
        let local_name = item.name.clone();
        validate_identifier(&local_name, "table")?;
        item.name = qualify_local_name(&module.namespace, &local_name);
        if !module.namespace.is_empty() && item.id == default_table_id(&local_name) {
            item.id = format!("{}.{}", module.namespace, item.id);
        }
        for field in &mut item.fields {
            field.ty = qualify_type_expression(&field.ty, module)?;
            if let Some(from) = &mut field.from {
                from.table = resolve_schema_name(&from.table, module)?;
            }
        }
    }
    Ok(qualified)
}

fn qualify_fields(fields: &mut [FieldSchema], module: &SchemaModule) -> Result<()> {
    for field in fields {
        field.ty = qualify_type_expression(&field.ty, module)?;
    }
    Ok(())
}

fn qualify_type_expression(input: &str, module: &SchemaModule) -> Result<String> {
    let input = input.trim();
    if matches!(
        input,
        "bool"
            | "i8"
            | "u8"
            | "i16"
            | "u16"
            | "i32"
            | "u32"
            | "i64"
            | "f32"
            | "f64"
            | "string"
            | "duration"
            | "datetime"
            | "text"
    ) {
        return Ok(input.to_owned());
    }

    for kind in ["enum", "struct", "union"] {
        if let Some(inner) = schema_generic_inner(input, kind) {
            return Ok(format!("{kind}<{}>", resolve_schema_name(inner, module)?));
        }
    }
    for kind in ["list", "set", "optional"] {
        if let Some(inner) = schema_generic_inner(input, kind) {
            return Ok(format!(
                "{kind}<{}>",
                qualify_type_expression(inner, module)?
            ));
        }
    }
    if let Some(inner) = schema_generic_inner(input, "map") {
        let parts = split_schema_top_level(inner, ',');
        let [key, value] = parts.as_slice() else {
            return Err(SoraError::InvalidType(input.to_owned()));
        };
        return Ok(format!(
            "map<{},{}>",
            qualify_type_expression(key, module)?,
            qualify_type_expression(value, module)?
        ));
    }
    if let Some(inner) = schema_generic_inner(input, "array") {
        let parts = split_schema_top_level(inner, ',');
        let [element, len] = parts.as_slice() else {
            return Err(SoraError::InvalidType(input.to_owned()));
        };
        let len = len
            .trim()
            .parse::<usize>()
            .map_err(|_| SoraError::InvalidType(input.to_owned()))?;
        return Ok(format!(
            "array<{},{}>",
            qualify_type_expression(element, module)?,
            len
        ));
    }
    if let Some(inner) = schema_generic_inner(input, "ref") {
        let (table, field) = inner
            .rsplit_once('.')
            .ok_or_else(|| SoraError::InvalidType(input.to_owned()))?;
        validate_identifier(field.trim(), "reference field")?;
        return Ok(format!(
            "ref<{}.{}>",
            resolve_schema_name(table, module)?,
            field.trim()
        ));
    }

    resolve_schema_name(input, module)
}

fn schema_generic_inner<'a>(input: &'a str, name: &str) -> Option<&'a str> {
    input
        .strip_prefix(&format!("{name}<"))
        .and_then(|rest| rest.strip_suffix('>'))
}

fn split_schema_top_level(input: &str, separator: char) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth = 0usize;
    let mut start = 0usize;
    for (index, ch) in input.char_indices() {
        match ch {
            '<' => depth += 1,
            '>' => depth = depth.saturating_sub(1),
            _ if ch == separator && depth == 0 => {
                parts.push(input[start..index].trim());
                start = index + ch.len_utf8();
            }
            _ => {}
        }
    }
    parts.push(input[start..].trim());
    parts
}

fn resolve_schema_name(input: &str, module: &SchemaModule) -> Result<String> {
    let input = input.trim();
    validate_qualified_name(input, "schema reference")?;
    let mut segments = input.split('.');
    let first = segments.next().expect("qualified name is non-empty");
    if let Some(imported) = module.imports.get(first) {
        let suffix = segments.collect::<Vec<_>>().join(".");
        return if suffix.is_empty() {
            Err(SoraError::InvalidSchema(format!(
                "schema reference `{input}` names import alias `{first}` without a declaration"
            )))
        } else {
            Ok(format!("{imported}.{suffix}"))
        };
    }
    if input.contains('.') || module.namespace.is_empty() {
        Ok(input.to_owned())
    } else {
        Ok(format!("{}.{}", module.namespace, input))
    }
}

fn qualify_local_name(namespace: &str, name: &str) -> String {
    if namespace.is_empty() {
        name.to_owned()
    } else {
        format!("{namespace}.{name}")
    }
}

fn validate_namespace(namespace: &str) -> Result<()> {
    if namespace.is_empty() {
        return Ok(());
    }
    validate_qualified_name(namespace, "namespace")
}

fn validate_qualified_name(value: &str, kind: &str) -> Result<()> {
    if value.is_empty() {
        return Err(SoraError::InvalidSchema(format!(
            "{kind} must not be empty"
        )));
    }
    for segment in value.split('.') {
        validate_identifier(segment, kind)?;
    }
    Ok(())
}

fn validate_identifier(value: &str, kind: &str) -> Result<()> {
    let mut chars = value.chars();
    if !matches!(chars.next(), Some(first) if first == '_' || first.is_ascii_alphabetic())
        || !chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
    {
        return Err(SoraError::InvalidSchema(format!(
            "{kind} `{value}` must be an ASCII identifier"
        )));
    }
    Ok(())
}

fn load_views(
    project_path: &Path,
    project: &ProjectMetadataSchema,
    mut views: BTreeMap<String, ViewSchema>,
) -> Result<BTreeMap<String, ViewSchema>> {
    let base_dir = project_path.parent().unwrap_or_else(|| Path::new("."));
    for relative in &project.views {
        let path = base_dir.join(relative);
        let document = load_document::<ViewDocumentRepr>(&path).map_err(schema_document_error)?;
        if views.insert(document.name.clone(), document.view).is_some() {
            return Err(SoraError::InvalidSchema(format!(
                "view `{}` is declared more than once",
                document.name
            )));
        }
    }
    Ok(views)
}

pub fn load_schema_module(path: &Path) -> Result<SchemaModule> {
    let document = load_document::<SchemaDocumentRepr>(path).map_err(schema_document_error)?;
    document.lower()
}

pub fn render_schema_module(path: &Path, module: &SchemaModule) -> Result<String> {
    let lua_ordered = path.extension().and_then(|value| value.to_str()) == Some("lua");
    let explicit_fields = path.extension().and_then(|value| value.to_str()) == Some("toml");
    let document = SchemaDocumentRepr::from_module(module, lua_ordered, explicit_fields);
    render_document(path, &document).map_err(schema_document_error)
}

fn schema_document_error(error: DocumentError) -> SoraError {
    match error {
        DocumentError::Read { path, source } => SoraError::ReadFile { path, source },
        DocumentError::Parse { path, message } => SoraError::ParseSchema { path, message },
        DocumentError::Render { path, message } => SoraError::ParseSchema { path, message },
        DocumentError::UnsupportedExtension { path, extension } => {
            SoraError::InvalidSchema(format!(
                "schema file `{}` has unsupported extension `{extension}`",
                path.display()
            ))
        }
        DocumentError::MissingExtension { path } => SoraError::InvalidSchema(format!(
            "schema file `{}` must have an extension",
            path.display()
        )),
    }
}

fn canonical_or_owned(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

fn register_declaration_sources(
    module: &SchemaModule,
    path: &Path,
    sources: &mut BTreeMap<SchemaDeclarationKey, PathBuf>,
) -> Result<()> {
    let declarations = module
        .enums
        .iter()
        .map(|item| (SchemaDeclarationKind::Enum, item.name.as_str()))
        .chain(
            module
                .structs
                .iter()
                .map(|item| (SchemaDeclarationKind::Struct, item.name.as_str())),
        )
        .chain(
            module
                .unions
                .iter()
                .map(|item| (SchemaDeclarationKind::Union, item.name.as_str())),
        )
        .chain(
            module
                .tables
                .iter()
                .map(|item| (SchemaDeclarationKind::Table, item.name.as_str())),
        );
    for (kind, name) in declarations {
        let key = SchemaDeclarationKey {
            kind,
            name: name.to_owned(),
        };
        if let Some(previous) = sources.insert(key, path.to_path_buf()) {
            return Err(SoraError::InvalidSchema(format!(
                "duplicate {kind:?} declaration `{name}` in `{}` and `{}`",
                previous.display(),
                path.display()
            )));
        }
    }
    Ok(())
}

fn sort_project_declarations(schema: &mut ProjectSchema) {
    schema
        .enums
        .sort_by(|left, right| left.name.cmp(&right.name));
    schema
        .structs
        .sort_by(|left, right| left.name.cmp(&right.name));
    schema
        .unions
        .sort_by(|left, right| left.name.cmp(&right.name));
    schema
        .tables
        .sort_by(|left, right| left.name.cmp(&right.name));
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct SchemaDocumentRepr {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    namespace: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    imports: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    project: Option<ProjectMetadataSchema>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    groups: BTreeMap<String, GroupSchema>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    views: BTreeMap<String, ViewSchema>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    codegen: Option<BTreeMap<String, serde_json::Value>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    localization: Option<LocalizationRepr>,
    #[serde(rename = "build", default, skip_serializing_if = "Option::is_none")]
    _build: Option<serde_json::Value>,
    #[serde(rename = "parsers", default, skip_serializing_if = "Option::is_none")]
    _parsers: Option<serde_json::Value>,
    #[serde(
        rename = "source_loaders",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    _source_loaders: Option<serde_json::Value>,
    #[serde(
        rename = "type_mappings",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    _type_mappings: Option<serde_json::Value>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    includes: Vec<String>,

    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    enums: IndexMap<String, EnumRepr>,

    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    structs: IndexMap<String, StructRepr>,

    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    unions: IndexMap<String, UnionRepr>,

    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    tables: IndexMap<String, TableRepr>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
enum EnumRepr {
    Values(Vec<EnumValueRepr>),
    Detailed(EnumBodyRepr),
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
enum EnumValueRepr {
    Name(String),
    Detailed {
        id: u32,
        name: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        comment: Option<String>,
    },
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct EnumBodyRepr {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    comment: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    groups: Option<GroupSetRepr>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    values: Vec<EnumValueRepr>,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    aliases: IndexMap<String, String>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct StructRepr {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    groups: Option<GroupSetRepr>,
    #[serde(default, skip_serializing_if = "OrderedNamed::is_empty")]
    fields: OrderedNamed<FieldRepr>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct UnionRepr {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    groups: Option<GroupSetRepr>,
    #[serde(
        default = "default_union_tag_repr",
        skip_serializing_if = "is_default_union_tag"
    )]
    tag: String,
    #[serde(default, skip_serializing_if = "OrderedNamed::is_empty")]
    variants: OrderedNamed<UnionVariantRepr>,
}

fn default_union_tag_repr() -> String {
    "type".to_owned()
}

fn is_default_union_tag(value: &String) -> bool {
    value == "type"
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct UnionVariantRepr {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    groups: Option<GroupSetRepr>,
    #[serde(default, skip_serializing_if = "OrderedNamed::is_empty")]
    fields: OrderedNamed<FieldRepr>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct TableRepr {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    groups: Option<GroupSetRepr>,
    mode: TableModeSchema,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    source: Option<TableSourceRepr>,
    #[serde(default, skip_serializing_if = "OrderedNamed::is_empty")]
    fields: OrderedNamed<TableFieldRepr>,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    indexes: IndexMap<String, IndexRepr>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
enum FieldRepr {
    Type(String),
    Detailed(FieldBodyRepr),
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
enum TableFieldRepr {
    Type(String),
    Detailed(Box<TableFieldBodyRepr>),
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct FieldBodyRepr {
    #[serde(rename = "type")]
    ty: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    groups: Option<GroupSetRepr>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    comment: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    default: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    range: Option<[i64; 2]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    length: Option<[usize; 2]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    parser: Option<ParserRepr>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct TableFieldBodyRepr {
    #[serde(flatten)]
    field: FieldBodyRepr,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    from: Option<TableFieldFromRepr>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
enum ParserRepr {
    Kind(String),
    Detailed(ParserBodyRepr),
}

#[derive(Debug, Deserialize, Serialize)]
struct ParserBodyRepr {
    kind: String,
    #[serde(flatten)]
    options: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
enum IndexRepr {
    Fields(Vec<String>),
    Detailed(IndexBodyRepr),
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct IndexBodyRepr {
    #[serde(default)]
    fields: Vec<String>,
    #[serde(default)]
    unique: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct TableSourceRepr {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    format: Option<String>,
    file: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    sheet: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct TableFieldFromRepr {
    table: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    parent_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    child_key: Option<String>,
    #[serde(default, rename = "field", skip_serializing_if = "Option::is_none")]
    value_field: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    order_by: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct LocalizationRepr {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    locales: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    default_locale: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    fallback_locale: Option<String>,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    sources: IndexMap<String, LocalizationSourceRepr>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct LocalizationSourceRepr {
    file: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    sheet: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    format: Option<String>,
    #[serde(
        default = "default_localization_key_repr",
        skip_serializing_if = "is_default_localization_key"
    )]
    key: String,
}

fn default_localization_key_repr() -> String {
    "key".to_owned()
}

fn is_default_localization_key(value: &String) -> bool {
    value == "key"
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
enum GroupSetRepr {
    One(String),
    Many(Vec<String>),
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ViewDocumentRepr {
    name: String,
    #[serde(flatten)]
    view: ViewSchema,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
enum OrderedNamed<T> {
    Map(IndexMap<String, T>),
    List(Vec<IndexMap<String, T>>),
}

impl<T> Default for OrderedNamed<T> {
    fn default() -> Self {
        Self::Map(IndexMap::new())
    }
}

impl<T> OrderedNamed<T> {
    fn is_empty(&self) -> bool {
        match self {
            Self::Map(values) => values.is_empty(),
            Self::List(values) => values.is_empty(),
        }
    }

    fn into_entries(self, kind: &str) -> Result<Vec<(String, T)>> {
        match self {
            Self::Map(values) => Ok(values.into_iter().collect()),
            Self::List(values) => values
                .into_iter()
                .map(|mut entry| {
                    if entry.len() != 1 {
                        return Err(SoraError::InvalidSchema(format!(
                            "ordered {kind} entries must contain exactly one named item"
                        )));
                    }
                    Ok(entry.shift_remove_index(0).expect("entry length checked"))
                })
                .collect(),
        }
    }
}

impl SchemaDocumentRepr {
    fn lower(self) -> Result<SchemaModule> {
        let mut enums = self
            .enums
            .into_iter()
            .map(|(name, value)| lower_enum(name, value))
            .collect::<Vec<_>>();
        let mut structs = self
            .structs
            .into_iter()
            .map(|(name, value)| lower_struct(name, value))
            .collect::<Result<Vec<_>>>()?;
        let mut unions = self
            .unions
            .into_iter()
            .map(|(name, value)| lower_union(name, value))
            .collect::<Result<Vec<_>>>()?;
        let mut tables = self
            .tables
            .into_iter()
            .map(|(name, value)| lower_table(name, value))
            .collect::<Result<Vec<_>>>()?;
        enums.sort_by(|left, right| left.name.cmp(&right.name));
        structs.sort_by(|left, right| left.name.cmp(&right.name));
        unions.sort_by(|left, right| left.name.cmp(&right.name));
        tables.sort_by(|left, right| left.name.cmp(&right.name));
        Ok(SchemaModule {
            namespace: self.namespace,
            imports: self.imports,
            project: self.project,
            groups: self.groups,
            views: self.views,
            codegen: self.codegen.map(|targets| CodegenSchema { targets }),
            localization: self.localization.map(lower_localization),
            includes: self.includes,
            enums,
            structs,
            unions,
            tables,
        })
    }

    fn from_module(module: &SchemaModule, lua_ordered: bool, explicit_fields: bool) -> Self {
        Self {
            namespace: module.namespace.clone(),
            imports: module.imports.clone(),
            project: module.project.clone(),
            groups: module.groups.clone(),
            views: module.views.clone(),
            codegen: module.codegen.as_ref().map(|value| value.targets.clone()),
            localization: module.localization.as_ref().map(localization_repr),
            _build: None,
            _parsers: None,
            _source_loaders: None,
            _type_mappings: None,
            includes: module.includes.clone(),
            enums: module
                .enums
                .iter()
                .map(|value| (value.name.clone(), enum_repr(value)))
                .collect(),
            structs: module
                .structs
                .iter()
                .map(|value| {
                    (
                        value.name.clone(),
                        struct_repr(value, lua_ordered, explicit_fields),
                    )
                })
                .collect(),
            unions: module
                .unions
                .iter()
                .map(|value| {
                    (
                        value.name.clone(),
                        union_repr(value, lua_ordered, explicit_fields),
                    )
                })
                .collect(),
            tables: module
                .tables
                .iter()
                .map(|value| {
                    (
                        value.name.clone(),
                        table_repr(value, lua_ordered, explicit_fields),
                    )
                })
                .collect(),
        }
    }
}

fn lower_groups(value: Option<GroupSetRepr>) -> GroupSetSchema {
    GroupSetSchema {
        values: match value {
            None => Vec::new(),
            Some(GroupSetRepr::One(value)) => vec![value],
            Some(GroupSetRepr::Many(values)) => values,
        },
    }
}

fn groups_repr(value: &GroupSetSchema) -> Option<GroupSetRepr> {
    match value.values.as_slice() {
        [] => None,
        [value] => Some(GroupSetRepr::One(value.clone())),
        values => Some(GroupSetRepr::Many(values.to_vec())),
    }
}

fn lower_enum(name: String, value: EnumRepr) -> EnumSchema {
    let (comment, groups, values, mut aliases) = match value {
        EnumRepr::Values(values) => (None, GroupSetSchema::default(), values, Vec::new()),
        EnumRepr::Detailed(value) => (
            value.comment,
            lower_groups(value.groups),
            value.values,
            value
                .aliases
                .into_iter()
                .map(|(alias, name)| EnumAliasSchema { name, alias })
                .collect(),
        ),
    };
    aliases.sort_by(|left, right| left.alias.cmp(&right.alias));
    let values = values
        .into_iter()
        .enumerate()
        .map(|(index, value)| match value {
            EnumValueRepr::Name(name) => EnumValueSchema {
                id: index as u32,
                name,
                comment: None,
            },
            EnumValueRepr::Detailed { id, name, comment } => EnumValueSchema { id, name, comment },
        })
        .collect();
    EnumSchema {
        name,
        comment,
        groups,
        values,
        aliases,
    }
}

fn lower_struct(name: String, value: StructRepr) -> Result<StructSchema> {
    Ok(StructSchema {
        name,
        groups: lower_groups(value.groups),
        fields: value
            .fields
            .into_entries("field")?
            .into_iter()
            .map(|(name, value)| lower_field(name, value))
            .collect(),
    })
}

fn lower_union(name: String, value: UnionRepr) -> Result<UnionSchema> {
    Ok(UnionSchema {
        name,
        groups: lower_groups(value.groups),
        tag: value.tag,
        variants: value
            .variants
            .into_entries("union variant")?
            .into_iter()
            .map(|(name, value)| {
                Ok(UnionVariantSchema {
                    name,
                    groups: lower_groups(value.groups),
                    fields: value
                        .fields
                        .into_entries("field")?
                        .into_iter()
                        .map(|(name, value)| lower_field(name, value))
                        .collect(),
                })
            })
            .collect::<Result<Vec<_>>>()?,
    })
}

fn lower_table(name: String, value: TableRepr) -> Result<TableSchema> {
    let mut indexes = value
        .indexes
        .into_iter()
        .map(|(name, value)| {
            let (fields, unique) = match value {
                IndexRepr::Fields(fields) => (fields, false),
                IndexRepr::Detailed(value) => (value.fields, value.unique),
            };
            IndexSchema {
                name,
                fields,
                unique,
            }
        })
        .collect::<Vec<_>>();
    indexes.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(TableSchema {
        id: value.id.unwrap_or_else(|| default_table_id(&name)),
        name,
        groups: lower_groups(value.groups),
        mode: value.mode,
        key: value.key,
        source: value.source.map(|value| TableSourceSchema {
            format: value.format,
            file: value.file,
            sheet: value.sheet,
        }),
        fields: value
            .fields
            .into_entries("table field")?
            .into_iter()
            .map(|(name, value)| lower_table_field(name, value))
            .collect(),
        indexes,
    })
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

fn lower_field(name: String, value: FieldRepr) -> FieldSchema {
    let value = match value {
        FieldRepr::Type(ty) => FieldBodyRepr {
            ty,
            groups: None,
            comment: None,
            default: None,
            range: None,
            length: None,
            parser: None,
        },
        FieldRepr::Detailed(value) => value,
    };
    FieldSchema {
        name,
        ty: value.ty,
        groups: lower_groups(value.groups),
        comment: value.comment,
        default: value.default,
        range: value.range,
        length: value.length,
        parser: value.parser.map(lower_parser),
    }
}

fn lower_table_field(name: String, value: TableFieldRepr) -> TableFieldSchema {
    let (field, from) = match value {
        TableFieldRepr::Type(ty) => (
            FieldBodyRepr {
                ty,
                groups: None,
                comment: None,
                default: None,
                range: None,
                length: None,
                parser: None,
            },
            None,
        ),
        TableFieldRepr::Detailed(value) => {
            let value = *value;
            (value.field, value.from)
        }
    };
    TableFieldSchema {
        name,
        ty: field.ty,
        groups: lower_groups(field.groups),
        comment: field.comment,
        default: field.default,
        range: field.range,
        length: field.length,
        parser: field.parser.map(lower_parser),
        from: from.map(|value| TableFieldFromSchema {
            table: value.table,
            parent_key: value.parent_key,
            child_key: value.child_key,
            value_field: value.value_field,
            order_by: value.order_by,
        }),
    }
}

fn lower_parser(value: ParserRepr) -> ParserSchema {
    match value {
        ParserRepr::Kind(kind) => ParserSchema {
            kind,
            options: BTreeMap::new(),
        },
        ParserRepr::Detailed(value) => ParserSchema {
            kind: value.kind,
            options: value.options,
        },
    }
}

fn lower_localization(value: LocalizationRepr) -> LocalizationSchema {
    let mut sources = value
        .sources
        .into_iter()
        .map(|(name, value)| LocalizationSourceSchema {
            name,
            file: value.file,
            sheet: value.sheet,
            format: value.format,
            key: value.key,
        })
        .collect::<Vec<_>>();
    sources.sort_by(|left, right| left.name.cmp(&right.name));
    LocalizationSchema {
        locales: value.locales,
        default_locale: value.default_locale,
        fallback_locale: value.fallback_locale,
        sources,
    }
}

fn enum_repr(value: &EnumSchema) -> EnumRepr {
    let sequential = value
        .values
        .iter()
        .enumerate()
        .all(|(index, value)| value.id == index as u32);
    if value.comment.is_none()
        && groups_repr(&value.groups).is_none()
        && value.aliases.is_empty()
        && sequential
        && value.values.iter().all(|value| value.comment.is_none())
    {
        return EnumRepr::Values(
            value
                .values
                .iter()
                .map(|value| EnumValueRepr::Name(value.name.clone()))
                .collect(),
        );
    }
    EnumRepr::Detailed(EnumBodyRepr {
        comment: value.comment.clone(),
        groups: groups_repr(&value.groups),
        values: value
            .values
            .iter()
            .map(|value| EnumValueRepr::Detailed {
                id: value.id,
                name: value.name.clone(),
                comment: value.comment.clone(),
            })
            .collect(),
        aliases: value
            .aliases
            .iter()
            .map(|value| (value.alias.clone(), value.name.clone()))
            .collect(),
    })
}

fn struct_repr(value: &StructSchema, lua_ordered: bool, explicit_fields: bool) -> StructRepr {
    StructRepr {
        groups: groups_repr(&value.groups),
        fields: ordered_named(
            value
                .fields
                .iter()
                .map(|value| (value.name.clone(), field_repr(value, explicit_fields)))
                .collect(),
            lua_ordered,
        ),
    }
}

fn union_repr(value: &UnionSchema, lua_ordered: bool, explicit_fields: bool) -> UnionRepr {
    UnionRepr {
        groups: groups_repr(&value.groups),
        tag: value.tag.clone(),
        variants: ordered_named(
            value
                .variants
                .iter()
                .map(|value| {
                    (
                        value.name.clone(),
                        UnionVariantRepr {
                            groups: groups_repr(&value.groups),
                            fields: ordered_named(
                                value
                                    .fields
                                    .iter()
                                    .map(|value| {
                                        (value.name.clone(), field_repr(value, explicit_fields))
                                    })
                                    .collect(),
                                lua_ordered,
                            ),
                        },
                    )
                })
                .collect(),
            lua_ordered,
        ),
    }
}

fn table_repr(value: &TableSchema, lua_ordered: bool, explicit_fields: bool) -> TableRepr {
    TableRepr {
        id: Some(value.id.clone()),
        groups: groups_repr(&value.groups),
        mode: value.mode,
        key: value.key.clone(),
        source: value.source.as_ref().map(|value| TableSourceRepr {
            format: value.format.clone(),
            file: value.file.clone(),
            sheet: value.sheet.clone(),
        }),
        fields: ordered_named(
            value
                .fields
                .iter()
                .map(|value| (value.name.clone(), table_field_repr(value, explicit_fields)))
                .collect(),
            lua_ordered,
        ),
        indexes: value
            .indexes
            .iter()
            .map(|value| {
                let repr = if value.unique {
                    IndexRepr::Detailed(IndexBodyRepr {
                        fields: value.fields.clone(),
                        unique: true,
                    })
                } else {
                    IndexRepr::Fields(value.fields.clone())
                };
                (value.name.clone(), repr)
            })
            .collect(),
    }
}

fn field_repr(value: &FieldSchema, explicit: bool) -> FieldRepr {
    let detailed = FieldBodyRepr {
        ty: value.ty.clone(),
        groups: groups_repr(&value.groups),
        comment: value.comment.clone(),
        default: value.default.clone(),
        range: value.range,
        length: value.length,
        parser: value.parser.as_ref().map(parser_repr),
    };
    if !explicit && field_body_is_shorthand(&detailed) {
        FieldRepr::Type(detailed.ty)
    } else {
        FieldRepr::Detailed(detailed)
    }
}

fn table_field_repr(value: &TableFieldSchema, explicit: bool) -> TableFieldRepr {
    let field = FieldBodyRepr {
        ty: value.ty.clone(),
        groups: groups_repr(&value.groups),
        comment: value.comment.clone(),
        default: value.default.clone(),
        range: value.range,
        length: value.length,
        parser: value.parser.as_ref().map(parser_repr),
    };
    if !explicit && field_body_is_shorthand(&field) && value.from.is_none() {
        TableFieldRepr::Type(field.ty)
    } else {
        TableFieldRepr::Detailed(Box::new(TableFieldBodyRepr {
            field,
            from: value.from.as_ref().map(|value| TableFieldFromRepr {
                table: value.table.clone(),
                parent_key: value.parent_key.clone(),
                child_key: value.child_key.clone(),
                value_field: value.value_field.clone(),
                order_by: value.order_by.clone(),
            }),
        }))
    }
}

fn field_body_is_shorthand(value: &FieldBodyRepr) -> bool {
    value.groups.is_none()
        && value.comment.is_none()
        && value.default.is_none()
        && value.range.is_none()
        && value.length.is_none()
        && value.parser.is_none()
}

fn parser_repr(value: &ParserSchema) -> ParserRepr {
    if value.options.is_empty() {
        ParserRepr::Kind(value.kind.clone())
    } else {
        ParserRepr::Detailed(ParserBodyRepr {
            kind: value.kind.clone(),
            options: value.options.clone(),
        })
    }
}

fn localization_repr(value: &LocalizationSchema) -> LocalizationRepr {
    LocalizationRepr {
        locales: value.locales.clone(),
        default_locale: value.default_locale.clone(),
        fallback_locale: value.fallback_locale.clone(),
        sources: value
            .sources
            .iter()
            .map(|value| {
                (
                    value.name.clone(),
                    LocalizationSourceRepr {
                        file: value.file.clone(),
                        sheet: value.sheet.clone(),
                        format: value.format.clone(),
                        key: value.key.clone(),
                    },
                )
            })
            .collect(),
    }
}

fn ordered_named<T>(values: Vec<(String, T)>, lua_ordered: bool) -> OrderedNamed<T> {
    if lua_ordered {
        OrderedNamed::List(
            values
                .into_iter()
                .map(|(name, value)| IndexMap::from([(name, value)]))
                .collect(),
        )
    } else {
        OrderedNamed::Map(values.into_iter().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        sync::atomic::{AtomicU64, Ordering},
    };

    static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn loads_project_schema_with_toml_includes() {
        let base = temp_dir();
        let schema_dir = base.join("schema");
        fs::create_dir_all(&schema_dir).unwrap();
        let project_path = base.join("project.toml");
        fs::write(
            &project_path,
            r#"
project = { id = "game_config" }
includes = ["schema/items.toml"]

[codegen.rust]
map_type = "fx_hash_map"
"#,
        )
        .unwrap();
        fs::write(
            schema_dir.join("items.toml"),
            r#"
[enums]
ItemType = ["Weapon", "Armor"]

[tables.Item]
mode = "map"
key = "id"
"#,
        )
        .unwrap();

        let schema = load_project_schema(&project_path).unwrap();

        assert_eq!(schema.project.id, "game_config");
        assert_eq!(
            schema.codegen.targets["rust"]["map_type"].as_str(),
            Some("fx_hash_map")
        );
        assert_eq!(schema.includes, ["schema/items.toml"]);
        assert_eq!(schema.enums[0].name, "ItemType");
        assert_eq!(schema.tables[0].name, "Item");

        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn loads_project_schema_with_yaml_includes() {
        let base = temp_dir();
        let schema_dir = base.join("schema");
        fs::create_dir_all(&schema_dir).unwrap();
        let project_path = base.join("project.yaml");
        fs::write(
            &project_path,
            r#"
project: { id: game_config }
includes:
  - schema/items.yml
codegen:
  rust:
    map_type: fx_hash_map
"#,
        )
        .unwrap();
        fs::write(
            schema_dir.join("items.yml"),
            r#"
enums:
  ItemType: [Weapon, Armor]
tables:
  Item:
    mode: map
    key: id
"#,
        )
        .unwrap();

        let schema = load_project_schema(&project_path).unwrap();

        assert_eq!(schema.project.id, "game_config");
        assert_eq!(
            schema.codegen.targets["rust"]["map_type"].as_str(),
            Some("fx_hash_map")
        );
        assert_eq!(schema.includes, ["schema/items.yml"]);
        assert_eq!(schema.enums[0].name, "ItemType");
        assert_eq!(schema.tables[0].name, "Item");

        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn loads_project_schema_with_json_includes() {
        let base = temp_dir();
        let schema_dir = base.join("schema");
        fs::create_dir_all(&schema_dir).unwrap();
        let project_path = base.join("project.json");
        fs::write(
            &project_path,
            r#"
{
  "project": { "id": "game_config" },
  "includes": ["schema/items.json"],
  "codegen": {
    "rust": {
      "map_type": "fx_hash_map"
    }
  }
}
"#,
        )
        .unwrap();
        fs::write(
            schema_dir.join("items.json"),
            r#"
{
  "enums": { "ItemType": ["Weapon", "Armor"] },
  "tables": { "Item": { "mode": "map", "key": "id" } }
}
"#,
        )
        .unwrap();

        let schema = load_project_schema(&project_path).unwrap();

        assert_eq!(schema.project.id, "game_config");
        assert_eq!(
            schema.codegen.targets["rust"]["map_type"].as_str(),
            Some("fx_hash_map")
        );
        assert_eq!(schema.includes, ["schema/items.json"]);
        assert_eq!(schema.enums[0].name, "ItemType");
        assert_eq!(schema.tables[0].name, "Item");

        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn loads_project_schema_with_lua_includes() {
        let base = temp_dir();
        let schema_dir = base.join("schema");
        fs::create_dir_all(&schema_dir).unwrap();
        let project_path = base.join("project.lua");
        fs::write(
            &project_path,
            r#"
return {
  project = { id = "game_config" },
  includes = { "schema/items.lua" },
  codegen = {
    rust = {
      map_type = "fx_hash_map",
    },
  },
}
"#,
        )
        .unwrap();
        fs::write(
            schema_dir.join("items.lua"),
            r#"
return {
  enums = {
    ItemType = { "Weapon", "Armor" },
  },
  tables = {
    Item = { mode = "map", key = "id" },
  },
}
"#,
        )
        .unwrap();

        let schema = load_project_schema(&project_path).unwrap();

        assert_eq!(schema.project.id, "game_config");
        assert_eq!(
            schema.codegen.targets["rust"]["map_type"].as_str(),
            Some("fx_hash_map")
        );
        assert_eq!(schema.includes, ["schema/items.lua"]);
        assert_eq!(schema.enums[0].name, "ItemType");
        assert_eq!(schema.tables[0].name, "Item");

        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn allows_mixed_schema_include_formats() {
        let base = temp_dir();
        let schema_dir = base.join("schema");
        fs::create_dir_all(&schema_dir).unwrap();
        let project_path = base.join("project.yaml");
        fs::write(
            &project_path,
            r#"
project: { id: game_config }
includes:
  - schema/items.toml
"#,
        )
        .unwrap();
        fs::write(
            schema_dir.join("items.toml"),
            r#"
[tables.Item]
mode = "map"
key = "id"
"#,
        )
        .unwrap();

        let schema = load_project_schema(&project_path).unwrap();

        assert_eq!(schema.tables[0].name, "Item");

        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn loads_and_renders_scon_schema() {
        let base = temp_dir();
        let schema_dir = base.join("schema");
        fs::create_dir_all(&schema_dir).unwrap();
        let project_path = base.join("project.scon");
        fs::write(
            &project_path,
            r#"
project { id = "game_config" }
includes = ["schema/items.scon"]
codegen {
  rust {
    crate { name = "game-config" }
  }
}
"#,
        )
        .unwrap();
        let module_path = schema_dir.join("items.scon");
        fs::write(
            &module_path,
            r#"
enums {
  ItemType = ["Weapon", "Armor"]
}
tables {
  Item {
    mode = "map"
    key = "id"
    fields {
      id = "i32"
      name {
        type = "string"
        length = [2, 32]
      }
    }
  }
}
"#,
        )
        .unwrap();

        let loaded = load_project_schema_with_modules(&project_path).unwrap();
        assert_eq!(loaded.schema.tables[0].fields[0].name, "id");
        assert_eq!(loaded.schema.tables[0].fields[1].name, "name");
        assert_eq!(loaded.modules.len(), 2);
        assert_eq!(
            loaded.schema.codegen.targets["rust"]["crate"]["name"].as_str(),
            Some("game-config")
        );

        let module = load_schema_module(&module_path).unwrap();
        let rendered = render_schema_module(&module_path, &module).unwrap();
        assert!(rendered.contains("ItemType"));
        assert!(rendered.contains("id = \"i32\""));

        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn rejects_scon_composition() {
        let base = temp_dir();
        fs::create_dir_all(&base).unwrap();
        let path = base.join("project.scon");
        fs::write(&path, "project { id = \"game\" }\ncopy = ${project}\n").unwrap();
        let error = load_project_schema(&path).unwrap_err();
        assert!(error.to_string().contains("not supported"));
        assert!(error.to_string().contains("2:"));
        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn complete_schema_is_equivalent_in_all_frontends() {
        let base = temp_dir();
        fs::create_dir_all(&base).unwrap();
        let source = base.join("project.json");
        fs::write(
            &source,
            r#"{
  "project": { "id": "game_config" },
  "groups": {
    "client": { "default": true },
    "server": { "default": false }
  },
  "views": {
    "default": { "contract": "game_config/default", "groups": ["client"] }
  },
  "codegen": { "rust": { "map_type": "fx_hash_map" } },
  "localization": {
    "locales": ["zh_cn", "en_us"],
    "default_locale": "zh_cn",
    "fallback_locale": "en_us",
    "sources": {
      "ui": { "file": "Core.xlsx", "sheet": "UILocalization" }
    }
  },
  "enums": {
    "Rarity": {
      "comment": "Item rarity",
      "groups": ["client", "server"],
      "values": [
        { "id": 0, "name": "Common", "comment": "Common item" },
        { "id": 1, "name": "Epic", "comment": "Epic item" }
      ],
      "aliases": { "Purple": "Epic" }
    }
  },
  "structs": {
    "Cost": {
      "fields": {
        "kind": "string",
        "count": { "type": "i32", "range": [1, 9999] }
      }
    }
  },
  "unions": {
    "Reward": {
      "tag": "kind",
      "variants": {
        "Item": { "fields": { "item_id": "i32", "count": "i32" } },
        "Currency": { "fields": { "amount": "i64" } }
      }
    }
  },
  "tables": {
    "Item": {
      "mode": "map",
      "key": "id",
      "source": { "format": "xlsx", "file": "Item.xlsx", "sheet": "Item" },
      "fields": {
        "id": "i32",
        "name": { "type": "string", "length": [2, 32] },
        "rarity": "enum<Rarity>",
        "cost": {
          "type": "struct<Cost>",
          "parser": { "kind": "tuple", "separator": "," }
        },
        "rewards": { "type": "list<union<Reward>>", "parser": "json" }
      },
      "indexes": {
        "by_rarity": ["rarity"],
        "by_name": { "fields": ["name"], "unique": true }
      }
    }
  }
}"#,
        )
        .unwrap();

        let expected_module = load_schema_module(&source).unwrap();
        let expected_schema = load_project_schema(&source).unwrap();
        let expected_ir = sora_ir::normalize::normalize_schema(expected_schema.clone()).unwrap();
        assert_eq!(
            expected_schema.enums[0].comment.as_deref(),
            Some("Item rarity")
        );
        assert_eq!(
            expected_schema.enums[0].values[0].comment.as_deref(),
            Some("Common item")
        );
        assert_eq!(expected_ir.enums[0].comment.as_deref(), Some("Item rarity"));
        assert_eq!(
            expected_ir.enums[0].values[0].comment.as_deref(),
            Some("Common item")
        );

        for extension in ["scon", "toml", "yaml", "json", "lua"] {
            let path = base.join(format!("project.{extension}"));
            let rendered = render_schema_module(&path, &expected_module).unwrap();
            fs::write(&path, &rendered).unwrap();
            assert_eq!(
                load_schema_module(&path).unwrap(),
                expected_module,
                "{extension}"
            );
            let schema = load_project_schema(&path).unwrap();
            assert_eq!(schema, expected_schema, "{extension}");
            assert_eq!(
                sora_ir::normalize::normalize_schema(schema).unwrap(),
                expected_ir,
                "{extension}"
            );
            if extension == "lua" {
                assert!(rendered.contains("          id = \"i32\","));
            }
        }

        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn rejects_non_keyed_and_unknown_schema_properties() {
        let base = temp_dir();
        fs::create_dir_all(&base).unwrap();
        for (name, content) in [
            (
                "array.json",
                r#"{ "project": { "id": "game" }, "tables": [{ "name": "Item", "mode": "map" }] }"#,
            ),
            (
                "body-name.json",
                r#"{ "project": { "id": "game" }, "enums": { "Kind": { "name": "Other", "values": [] } } }"#,
            ),
            (
                "unknown.json",
                r#"{ "project": { "id": "game" }, "tables": { "Item": { "mode": "map", "mystery": true } } }"#,
            ),
        ] {
            let path = base.join(name);
            fs::write(&path, content).unwrap();
            assert!(load_project_schema(&path).is_err(), "{name}");
        }
        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn rejects_invalid_include_graphs_and_module_ownership() {
        let base = temp_dir();
        let schema_dir = base.join("schema");
        fs::create_dir_all(&schema_dir).unwrap();
        let project = base.join("project.scon");

        fs::write(
            &project,
            "project { id = \"game\" }\nincludes = [\"schema/a.scon\", \"schema/b.scon\"]\n",
        )
        .unwrap();
        fs::write(
            schema_dir.join("a.scon"),
            "tables { Item { mode = \"list\" } }\n",
        )
        .unwrap();
        fs::write(
            schema_dir.join("b.scon"),
            "tables { Item { mode = \"list\" } }\n",
        )
        .unwrap();
        assert!(
            load_project_schema(&project)
                .unwrap_err()
                .to_string()
                .contains("duplicate Table declaration `Item`")
        );

        fs::write(
            &project,
            "project { id = \"game\" }\nincludes = [\"schema/a.scon\"]\n",
        )
        .unwrap();
        fs::write(
            schema_dir.join("a.scon"),
            "includes = [\"../project.scon\"]\n",
        )
        .unwrap();
        assert!(
            load_project_schema(&project)
                .unwrap_err()
                .to_string()
                .contains("cycle or duplicate include")
        );

        fs::write(schema_dir.join("a.scon"), "project { id = \"nested\" }\n").unwrap();
        assert!(
            load_project_schema(&project)
                .unwrap_err()
                .to_string()
                .contains("must not declare `project`")
        );

        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn rejects_every_scon_composition_form_with_location() {
        let base = temp_dir();
        fs::create_dir_all(&base).unwrap();
        for (name, content) in [
            (
                "include.scon",
                "project { id = \"game\" }\ninclude \"./other.scon\"\n",
            ),
            (
                "substitution.scon",
                "project { id = \"game\" }\ncopy = ${project}\n",
            ),
            (
                "interpolation.scon",
                "project { id = \"game\" }\ncopy = \"${project}\"\n",
            ),
            (
                "object-spread.scon",
                "project { id = \"game\" }\nbase { value = 1 }\ncopy { ...${base} }\n",
            ),
            (
                "array-spread.scon",
                "project { id = \"game\" }\nbase = [1]\ncopy = [...${base}]\n",
            ),
        ] {
            let path = base.join(name);
            fs::write(&path, content).unwrap();
            let message = load_project_schema(&path).unwrap_err().to_string();
            assert!(message.contains("not supported"), "{name}: {message}");
            assert!(
                message.contains("2:") || message.contains("3:"),
                "{name}: {message}"
            );
        }
        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn qualifies_module_declarations_and_references() {
        let base = temp_dir();
        let schema_dir = base.join("schema");
        fs::create_dir_all(&schema_dir).unwrap();
        let project = base.join("project.scon");
        fs::write(
            &project,
            "project { id = \"game\" }\nincludes = [\"schema/common.scon\", \"schema/items.scon\"]\n",
        )
        .unwrap();
        fs::write(
            schema_dir.join("common.scon"),
            "namespace = \"common\"\nenums { Rarity = [\"Common\", \"Rare\"] }\n",
        )
        .unwrap();
        fs::write(
            schema_dir.join("items.scon"),
            r#"
namespace = "items"
imports { shared = "common" }
structs {
  Reward { fields { rarity = "enum<shared.Rarity>" } }
}
tables {
  Item {
    mode = "map"
    key = "id"
    fields {
      id = "string"
      reward = "struct<Reward>"
      parent = "optional<ref<items.Item.id>>"
    }
  }
}
"#,
        )
        .unwrap();

        let schema = load_project_schema(&project).unwrap();
        assert_eq!(schema.enums[0].name, "common.Rarity");
        assert_eq!(schema.structs[0].name, "items.Reward");
        assert_eq!(schema.structs[0].fields[0].ty, "enum<common.Rarity>");
        assert_eq!(schema.tables[0].name, "items.Item");
        assert_eq!(schema.tables[0].id, "items.item");
        assert_eq!(schema.tables[0].fields[1].ty, "struct<items.Reward>");
        assert_eq!(
            schema.tables[0].fields[2].ty,
            "optional<ref<items.Item.id>>"
        );
        let _ = fs::remove_dir_all(base);
    }

    fn temp_dir() -> PathBuf {
        let unique = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("sora-input-schema-test-{unique}"))
    }
}
