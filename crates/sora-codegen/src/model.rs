use heck::{ToLowerCamelCase, ToPascalCase, ToSnakeCase};
use serde::Serialize;
use sora_diagnostics::Result;
use sora_ir::model::{ConfigIr, FieldIr, IndexIr, TableIr, TableModeIr, TypeIr};

#[derive(Debug, Clone, Serialize)]
pub struct BaseModel {
    pub package: String,
    pub schema_fingerprint: String,
    pub enums: Vec<BaseEnum>,
    pub unions: Vec<BaseUnion>,
    pub records: Vec<BaseRecord>,
    pub tables: Vec<BaseTable>,
    pub modules: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BaseEnum {
    pub qualified_name: String,
    pub namespace: String,
    pub namespace_path: String,
    pub module_path: String,
    pub qualified_pascal_name: String,
    pub qualified_snake_name: String,
    pub name: String,
    pub pascal_name: String,
    pub snake_name: String,
    pub comment: Option<String>,
    pub values: Vec<BaseEnumValue>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BaseEnumValue {
    pub id: u32,
    pub name: String,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BaseUnion {
    pub qualified_name: String,
    pub namespace: String,
    pub namespace_path: String,
    pub module_path: String,
    pub qualified_pascal_name: String,
    pub qualified_snake_name: String,
    pub name: String,
    pub pascal_name: String,
    pub snake_name: String,
    pub tag: String,
    pub variants: Vec<BaseUnionVariant>,
    pub imports: Vec<BaseImport>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BaseUnionVariant {
    pub name: String,
    pub pascal_name: String,
    pub snake_name: String,
    pub fields: Vec<BaseField>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BaseRecord {
    pub qualified_name: String,
    pub namespace: String,
    pub namespace_path: String,
    pub module_path: String,
    pub qualified_pascal_name: String,
    pub qualified_snake_name: String,
    pub qualified_camel_name: String,
    pub name: String,
    pub pascal_name: String,
    pub snake_name: String,
    pub camel_name: String,
    pub imports: Vec<BaseImport>,
    pub fields: Vec<BaseField>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BaseImport {
    pub module: String,
    pub absolute_module: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct BaseTable {
    pub name: String,
    pub local_name: String,
    pub namespace: String,
    pub namespace_path: String,
    pub module_path: String,
    pub qualified_pascal_name: String,
    pub qualified_snake_name: String,
    pub qualified_camel_name: String,
    pub pascal_name: String,
    pub camel_name: String,
    pub snake_name: String,
    pub mode: TableModeIr,
    pub mode_name: String,
    pub key_name: Option<String>,
    pub key_field: Option<BaseField>,
    pub unique_indexes: Vec<BaseIndex>,
    pub non_unique_indexes: Vec<BaseIndex>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BaseIndex {
    pub name: String,
    pub pascal_name: String,
    pub camel_name: String,
    pub snake_name: String,
    pub method_name: String,
    pub field: BaseField,
}

#[derive(Debug, Clone, Serialize)]
pub struct BaseField {
    pub raw_name: String,
    pub pascal_name: String,
    pub snake_name: String,
    pub camel_name: String,
    pub ty: TypeIr,
    pub comment: Option<String>,
}

pub fn build_base_model(ir: &ConfigIr) -> Result<BaseModel> {
    let enums = ir
        .enums
        .iter()
        .map(|item| {
            let parts = SchemaNameParts::new(&item.name);
            BaseEnum {
                qualified_name: item.name.clone(),
                namespace: parts.namespace,
                namespace_path: parts.namespace_path,
                module_path: parts.module_path,
                qualified_pascal_name: schema_flat_pascal_name(&item.name),
                qualified_snake_name: schema_flat_snake_name(&item.name),
                name: parts.local.clone(),
                pascal_name: parts.local.to_pascal_case(),
                snake_name: parts.local.to_snake_case(),
                comment: item.comment.clone(),
                values: item
                    .values
                    .iter()
                    .map(|value| BaseEnumValue {
                        id: value.id,
                        name: value.name.clone(),
                        comment: value.comment.clone(),
                    })
                    .collect(),
            }
        })
        .collect::<Vec<_>>();

    let records = ir
        .structs
        .iter()
        .map(|item| {
            build_base_record(
                ir,
                &TableLike {
                    name: &item.name,
                    fields: &item.fields,
                },
            )
        })
        .chain(
            ir.tables
                .iter()
                .map(|item| build_base_record(ir, &item.into())),
        )
        .collect::<Result<Vec<_>>>()?;

    let unions = ir
        .unions
        .iter()
        .map(|item| build_base_union(ir, item))
        .collect::<Result<Vec<_>>>()?;

    let tables = ir.tables.iter().map(build_base_table).collect::<Vec<_>>();

    let modules = enums
        .iter()
        .map(|item| item.module_path.clone())
        .chain(records.iter().map(|item| item.module_path.clone()))
        .chain(unions.iter().map(|item| item.module_path.clone()))
        .collect();

    Ok(BaseModel {
        // Language-specific generators must fill this from their target
        // binding. Contract identity is deliberately not a namespace fallback.
        package: String::new(),
        schema_fingerprint: sora_ir::fingerprint::schema_fingerprint(ir)?,
        enums,
        unions,
        records,
        tables,
        modules,
    })
}

fn build_base_union(ir: &ConfigIr, item: &sora_ir::model::UnionIr) -> Result<BaseUnion> {
    let mut imports = Vec::new();
    let variants = item
        .variants
        .iter()
        .map(|variant| {
            for field in &variant.fields {
                collect_type_imports(ir, &item.name, &field.ty, &mut imports);
            }
            Ok(BaseUnionVariant {
                name: variant.name.clone(),
                pascal_name: variant.name.to_pascal_case(),
                snake_name: variant.name.to_snake_case(),
                fields: variant
                    .fields
                    .iter()
                    .map(build_base_field)
                    .collect::<Vec<_>>(),
            })
        })
        .collect::<Result<Vec<_>>>()?;

    imports.sort_by(|a, b| a.module.cmp(&b.module).then(a.name.cmp(&b.name)));
    imports.dedup_by(|a, b| a.module == b.module && a.name == b.name);

    Ok(BaseUnion {
        qualified_name: item.name.clone(),
        namespace: schema_namespace(&item.name).to_owned(),
        namespace_path: schema_namespace_path(&item.name),
        module_path: schema_module_path(&item.name),
        qualified_pascal_name: schema_flat_pascal_name(&item.name),
        qualified_snake_name: schema_flat_snake_name(&item.name),
        name: schema_local_name(&item.name).to_owned(),
        pascal_name: schema_local_name(&item.name).to_pascal_case(),
        snake_name: schema_local_name(&item.name).to_snake_case(),
        tag: item.tag.clone(),
        variants,
        imports,
    })
}

fn build_base_record(ir: &ConfigIr, item: &TableLike<'_>) -> Result<BaseRecord> {
    let parts = SchemaNameParts::new(item.name);
    let fields = item.fields.iter().map(build_base_field).collect::<Vec<_>>();
    let imports = build_imports(ir, item);

    Ok(BaseRecord {
        qualified_name: item.name.to_owned(),
        namespace: parts.namespace,
        namespace_path: parts.namespace_path,
        module_path: parts.module_path,
        qualified_pascal_name: schema_flat_pascal_name(item.name),
        qualified_snake_name: schema_flat_snake_name(item.name),
        qualified_camel_name: schema_flat_pascal_name(item.name).to_lower_camel_case(),
        name: parts.local.clone(),
        pascal_name: parts.local.to_pascal_case(),
        snake_name: parts.local.to_snake_case(),
        camel_name: parts.local.to_lower_camel_case(),
        imports,
        fields,
    })
}

fn build_base_table(table: &TableIr) -> BaseTable {
    let parts = SchemaNameParts::new(&table.name);
    let key_field = table.key.as_ref().and_then(|key| {
        table
            .fields
            .iter()
            .find(|field| field.name == *key)
            .map(build_base_field)
    });
    let unique_indexes = table
        .indexes
        .iter()
        .filter(|index| index.unique)
        .filter_map(|index| build_base_index(table, index))
        .collect::<Vec<_>>();
    let non_unique_indexes = table
        .indexes
        .iter()
        .filter(|index| !index.unique)
        .filter_map(|index| build_base_index(table, index))
        .collect::<Vec<_>>();

    BaseTable {
        name: table.name.clone(),
        local_name: parts.local.clone(),
        namespace: parts.namespace,
        namespace_path: parts.namespace_path,
        module_path: parts.module_path,
        qualified_pascal_name: schema_flat_pascal_name(&table.name),
        qualified_snake_name: schema_flat_snake_name(&table.name),
        qualified_camel_name: schema_flat_pascal_name(&table.name).to_lower_camel_case(),
        pascal_name: parts.local.to_pascal_case(),
        camel_name: parts.local.to_lower_camel_case(),
        snake_name: parts.local.to_snake_case(),
        mode: table.mode,
        mode_name: table_mode_name(table.mode),
        key_name: table.key.clone(),
        key_field,
        unique_indexes,
        non_unique_indexes,
    }
}

fn build_base_index(table: &TableIr, index: &IndexIr) -> Option<BaseIndex> {
    if index.fields.len() != 1 || table.mode == TableModeIr::Singleton {
        return None;
    }

    let field = table
        .fields
        .iter()
        .find(|field| field.name == index.fields[0])?;
    Some(BaseIndex {
        name: index.name.clone(),
        pascal_name: index.name.to_pascal_case(),
        camel_name: index.name.to_lower_camel_case(),
        snake_name: index.name.to_snake_case(),
        method_name: format!("get_{}", index.name.to_snake_case()),
        field: build_base_field(field),
    })
}

fn build_base_field(field: &FieldIr) -> BaseField {
    BaseField {
        raw_name: field.name.clone(),
        pascal_name: field.name.to_pascal_case(),
        snake_name: field.name.to_snake_case(),
        camel_name: field.name.to_lower_camel_case(),
        ty: field.ty.clone(),
        comment: field.comment.clone(),
    }
}

fn table_mode_name(mode: TableModeIr) -> String {
    match mode {
        TableModeIr::List => "list",
        TableModeIr::Map => "map",
        TableModeIr::Singleton => "singleton",
    }
    .to_owned()
}

fn build_imports(ir: &ConfigIr, item: &TableLike<'_>) -> Vec<BaseImport> {
    let mut imports = Vec::new();
    for field in item.fields {
        collect_type_imports(ir, item.name, &field.ty, &mut imports);
    }
    imports.sort_by(|a, b| a.module.cmp(&b.module).then(a.name.cmp(&b.name)));
    imports.dedup_by(|a, b| a.module == b.module && a.name == b.name);
    imports
}

fn collect_type_imports(
    ir: &ConfigIr,
    owner_name: &str,
    ty: &TypeIr,
    imports: &mut Vec<BaseImport>,
) {
    match ty {
        TypeIr::Enum(name) | TypeIr::Struct(name) | TypeIr::Union(name) => {
            push_named_import(owner_name, name, imports)
        }
        TypeIr::List(element) | TypeIr::Set(element) | TypeIr::Optional(element) => {
            collect_type_imports(ir, owner_name, element, imports);
        }
        TypeIr::Map { key, value } => {
            collect_type_imports(ir, owner_name, key, imports);
            collect_type_imports(ir, owner_name, value, imports);
        }
        TypeIr::Array { element, .. } => {
            collect_type_imports(ir, owner_name, element, imports);
        }
        TypeIr::Ref { table, field } => {
            if let Some(target_field) = ir
                .tables
                .iter()
                .find(|candidate| candidate.name == *table)
                .and_then(|table| {
                    table
                        .fields
                        .iter()
                        .find(|candidate| candidate.name == *field)
                })
            {
                collect_type_imports(ir, owner_name, &target_field.ty, imports);
            }
        }
        _ => {}
    }
}

fn push_named_import(owner_name: &str, name: &str, imports: &mut Vec<BaseImport>) {
    if name == owner_name {
        return;
    }
    imports.push(BaseImport {
        module: relative_schema_module_path(owner_name, name),
        absolute_module: schema_module_path(name),
        name: schema_local_name(name).to_pascal_case(),
    });
}

pub fn schema_local_name(name: &str) -> &str {
    name.rsplit('.').next().unwrap_or(name)
}

pub fn schema_namespace(name: &str) -> &str {
    name.rsplit_once('.').map_or("", |(namespace, _)| namespace)
}

pub fn schema_namespace_path(name: &str) -> String {
    schema_namespace(name)
        .split('.')
        .filter(|segment| !segment.is_empty())
        .map(ToSnakeCase::to_snake_case)
        .collect::<Vec<_>>()
        .join("/")
}

pub fn schema_module_path(name: &str) -> String {
    let namespace = schema_namespace_path(name);
    let module = schema_local_name(name).to_snake_case();
    if namespace.is_empty() {
        module
    } else {
        format!("{namespace}/{module}")
    }
}

pub fn schema_flat_pascal_name(name: &str) -> String {
    name.split('.')
        .map(ToPascalCase::to_pascal_case)
        .collect::<String>()
}

pub fn schema_flat_snake_name(name: &str) -> String {
    name.split('.')
        .map(ToSnakeCase::to_snake_case)
        .collect::<Vec<_>>()
        .join("_")
}

fn relative_schema_module_path(owner: &str, target: &str) -> String {
    let owner_namespace = schema_namespace_path(owner);
    let target_module = schema_module_path(target);
    let owner_segments = owner_namespace
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();
    let target_segments = target_module.split('/').collect::<Vec<_>>();
    let shared = owner_segments
        .iter()
        .zip(&target_segments)
        .take_while(|(left, right)| left == right)
        .count();
    let mut parts = vec![".."; owner_segments.len().saturating_sub(shared)];
    parts.extend_from_slice(&target_segments[shared..]);
    parts.join("/")
}

struct SchemaNameParts {
    local: String,
    namespace: String,
    namespace_path: String,
    module_path: String,
}

impl SchemaNameParts {
    fn new(name: &str) -> Self {
        Self {
            local: schema_local_name(name).to_owned(),
            namespace: schema_namespace(name).to_owned(),
            namespace_path: schema_namespace_path(name),
            module_path: schema_module_path(name),
        }
    }
}

struct TableLike<'a> {
    name: &'a str,
    fields: &'a [FieldIr],
}

impl<'a> From<&'a TableIr> for TableLike<'a> {
    fn from(table: &'a TableIr) -> Self {
        Self {
            name: &table.name,
            fields: &table.fields,
        }
    }
}
