use heck::{ToLowerCamelCase, ToPascalCase, ToSnakeCase};
use serde::Serialize;
use sora_diagnostics::Result;
use sora_ir::model::{ConfigIr, FieldIr, TableIr, TableModeIr, TypeIr};

#[derive(Debug, Clone, Serialize)]
pub struct CodegenModel {
    pub package: String,
    pub enums: Vec<CodegenEnum>,
    pub unions: Vec<CodegenUnion>,
    pub records: Vec<CodegenRecord>,
    pub tables: Vec<CodegenTable>,
    pub modules: Vec<String>,
    pub has_map_tables: bool,
    pub has_singleton_tables: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct CodegenEnum {
    pub name: String,
    pub values: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CodegenUnion {
    pub name: String,
    pub pascal_name: String,
    pub snake_name: String,
    pub tag: String,
    pub variants: Vec<CodegenUnionVariant>,
    pub imports: Vec<CodegenImport>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CodegenUnionVariant {
    pub name: String,
    pub fields: Vec<CodegenField>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CodegenRecord {
    pub name: String,
    pub pascal_name: String,
    pub snake_name: String,
    pub camel_name: String,
    pub imports: Vec<CodegenImport>,
    pub fields: Vec<CodegenField>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CodegenImport {
    pub module: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CodegenTable {
    pub name: String,
    pub pascal_name: String,
    pub camel_name: String,
    pub snake_name: String,
    pub mode: String,
    pub container_type: String,
    pub row_type: String,
    pub key_name: Option<String>,
    pub key_field_name: Option<String>,
    pub key_type: Option<String>,
    pub key_is_copy: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct CodegenField {
    pub raw_name: String,
    pub name: String,
    pub type_name: String,
    pub decode: String,
    pub comment: Option<String>,
}

pub trait LanguageBackend {
    fn field_name(&self, raw_name: &str) -> String;
    fn type_name(&self, ir: &ConfigIr, ty: &TypeIr) -> String;
    fn decode_expr(&self, ir: &ConfigIr, ty: &TypeIr) -> String;
    fn row_type(&self, table: &TableNameParts<'_>) -> String;
    fn container_type(
        &self,
        table: &TableNameParts<'_>,
        mode: TableModeIr,
        row_type: &str,
        key_type: Option<&str>,
    ) -> String;

    fn key_is_copy(&self, _ir: &ConfigIr, _ty: &TypeIr) -> bool {
        false
    }
}

pub struct TableNameParts<'a> {
    pub name: &'a str,
    pub pascal_name: &'a str,
    pub camel_name: &'a str,
    pub snake_name: &'a str,
}

pub fn build_model(ir: &ConfigIr, backend: &impl LanguageBackend) -> Result<CodegenModel> {
    let enums = ir
        .enums
        .iter()
        .map(|item| CodegenEnum {
            name: item.name.clone(),
            values: item.values.clone(),
        })
        .collect::<Vec<_>>();

    let records = ir
        .structs
        .iter()
        .map(|item| {
            build_record(
                ir,
                backend,
                &TableLike {
                    name: &item.name,
                    fields: &item.fields,
                },
            )
        })
        .chain(
            ir.tables
                .iter()
                .map(|item| build_record(ir, backend, &item.into())),
        )
        .collect::<Result<Vec<_>>>()?;

    let unions = ir
        .unions
        .iter()
        .map(|item| build_union(ir, backend, item))
        .collect::<Result<Vec<_>>>()?;

    let tables = ir
        .tables
        .iter()
        .map(|item| build_table(ir, backend, item))
        .collect::<Result<Vec<_>>>()?;

    let modules = enums
        .iter()
        .map(|item| item.name.to_snake_case())
        .chain(records.iter().map(|item| item.snake_name.clone()))
        .chain(unions.iter().map(|item| item.snake_name.clone()))
        .collect();

    Ok(CodegenModel {
        package: ir.package.clone(),
        enums,
        unions,
        records,
        has_map_tables: tables
            .iter()
            .any(|table| table.mode == "map" && table.key_field_name.is_some()),
        has_singleton_tables: tables.iter().any(|table| table.mode == "singleton"),
        tables,
        modules,
    })
}

fn build_union(
    ir: &ConfigIr,
    backend: &impl LanguageBackend,
    item: &sora_ir::model::UnionIr,
) -> Result<CodegenUnion> {
    let mut imports = Vec::new();
    let variants = item
        .variants
        .iter()
        .map(|variant| {
            for field in &variant.fields {
                collect_type_imports(ir, &item.name, &field.ty, &mut imports);
            }
            Ok(CodegenUnionVariant {
                name: variant.name.to_pascal_case(),
                fields: variant
                    .fields
                    .iter()
                    .map(|field| build_field(ir, backend, field))
                    .collect::<Vec<_>>(),
            })
        })
        .collect::<Result<Vec<_>>>()?;

    imports.sort_by(|a, b| a.module.cmp(&b.module).then(a.name.cmp(&b.name)));
    imports.dedup_by(|a, b| a.module == b.module && a.name == b.name);

    Ok(CodegenUnion {
        name: item.name.clone(),
        pascal_name: item.name.to_pascal_case(),
        snake_name: item.name.to_snake_case(),
        tag: item.tag.clone(),
        variants,
        imports,
    })
}

fn build_table(
    ir: &ConfigIr,
    backend: &impl LanguageBackend,
    table: &TableIr,
) -> Result<CodegenTable> {
    let pascal_name = table.name.to_pascal_case();
    let camel_name = table.name.to_lower_camel_case();
    let snake_name = table.name.to_snake_case();
    let parts = TableNameParts {
        name: &table.name,
        pascal_name: &pascal_name,
        camel_name: &camel_name,
        snake_name: &snake_name,
    };
    let row_type = backend.row_type(&parts);
    let key_field = table.key.as_ref().and_then(|key| {
        table
            .fields
            .iter()
            .find(|field| field.name == *key)
            .map(|field| {
                (
                    backend.field_name(&field.name),
                    backend.type_name(ir, &field.ty),
                    backend.key_is_copy(ir, &field.ty),
                )
            })
    });
    let container_type = backend.container_type(
        &parts,
        table.mode,
        &row_type,
        key_field.as_ref().map(|(_, ty, _)| ty.as_str()),
    );

    Ok(CodegenTable {
        name: table.name.clone(),
        pascal_name,
        camel_name,
        snake_name,
        mode: match table.mode {
            TableModeIr::List => "list",
            TableModeIr::Map => "map",
            TableModeIr::Singleton => "singleton",
        }
        .to_owned(),
        container_type,
        row_type,
        key_name: table.key.clone(),
        key_field_name: key_field.as_ref().map(|(name, _, _)| name.clone()),
        key_type: key_field.as_ref().map(|(_, ty, _)| ty.clone()),
        key_is_copy: key_field.as_ref().is_some_and(|(_, _, is_copy)| *is_copy),
    })
}

fn build_record(
    ir: &ConfigIr,
    backend: &impl LanguageBackend,
    item: &TableLike<'_>,
) -> Result<CodegenRecord> {
    let fields = item
        .fields
        .iter()
        .map(|field| build_field(ir, backend, field))
        .collect::<Vec<_>>();

    Ok(CodegenRecord {
        name: item.name.to_owned(),
        pascal_name: item.name.to_pascal_case(),
        snake_name: item.name.to_snake_case(),
        camel_name: item.name.to_lower_camel_case(),
        imports: build_imports(ir, item),
        fields,
    })
}

fn build_field(ir: &ConfigIr, backend: &impl LanguageBackend, field: &FieldIr) -> CodegenField {
    CodegenField {
        raw_name: field.name.clone(),
        name: backend.field_name(&field.name),
        type_name: backend.type_name(ir, &field.ty),
        decode: backend.decode_expr(ir, &field.ty),
        comment: field.comment.clone(),
    }
}

fn build_imports(ir: &ConfigIr, item: &TableLike<'_>) -> Vec<CodegenImport> {
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
    imports: &mut Vec<CodegenImport>,
) {
    match ty {
        TypeIr::Enum(name) | TypeIr::Struct(name) | TypeIr::Union(name) => {
            push_named_import(owner_name, name, imports)
        }
        TypeIr::List(element) | TypeIr::Optional(element) => {
            collect_type_imports(ir, owner_name, element, imports);
        }
        TypeIr::Array { element, .. } => collect_type_imports(ir, owner_name, element, imports),
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

fn push_named_import(owner_name: &str, name: &str, imports: &mut Vec<CodegenImport>) {
    if name == owner_name {
        return;
    }
    imports.push(CodegenImport {
        module: name.to_snake_case(),
        name: name.to_pascal_case(),
    });
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
