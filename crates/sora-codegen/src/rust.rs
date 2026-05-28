use std::path::Path;

use heck::ToSnakeCase;
use minijinja::context;
use serde::Serialize;
use sora_diagnostics::Result;
use sora_ir::model::{ConfigIr, TableModeIr, TypeIr};

use crate::{
    generator::{CodeGenerator, CodegenContext, runtime_format_name},
    model::{
        BaseField, BaseImport, BaseIndex, BaseModel, BaseRecord, BaseTable, BaseUnion,
        BaseUnionVariant, build_base_model,
    },
    options::{RustCodegenOptions, RustDateTimeType, RustMapType, RustStringStorage},
    render::{ensure_dir, render_template, write_file},
    type_mapping::{TypeMapping, TypeMappingContext, TypeMappingRegistry},
};

pub struct RustCodeGenerator;
crate::impl_test_codegen_generate!(RustCodeGenerator, "rust");

impl CodeGenerator for RustCodeGenerator {
    fn generate(&self, context: CodegenContext<'_>, out_dir: &Path) -> Result<()> {
        let ir = context.ir;
        let options = context.options::<RustCodegenOptions>()?;
        ensure_dir(out_dir)?;
        let mapper = RustTypeMapper::new(context.target, ir, context.type_mappings, &options);
        let model = RustModel::from_base_model(ir, build_base_model(ir)?, &mapper);
        let runtime_format = runtime_format_name(options.runtime_format);

        for item in &model.enums {
            let rendered = render_template("rust", "enum.rs.j2", context! { enum => item })?;
            write_file(
                &out_dir.join(format!("{}.rs", item.name.to_snake_case())),
                rendered,
            )?;
        }

        for record in &model.records {
            let rendered = render_template("rust", "struct.rs.j2", context! { record => record })?;
            write_file(&out_dir.join(format!("{}.rs", record.snake_name)), rendered)?;
        }

        for union in &model.unions {
            let rendered = render_template("rust", "union.rs.j2", context! { union => union })?;
            write_file(&out_dir.join(format!("{}.rs", union.snake_name)), rendered)?;
        }

        let rust_map_type = match options.map_type {
            RustMapType::Std => "std",
            RustMapType::FxHashMap => "fx_hash_map",
        };
        let rendered = render_template(
            "rust",
            "mod.rs.j2",
            context! { model => &model, rust_map_type => rust_map_type, runtime_format => runtime_format },
        )?;
        write_file(&out_dir.join("mod.rs"), rendered)?;

        let rendered = render_template(
            "rust",
            "runtime.rs.j2",
            context! {
                runtime_format => runtime_format,
                string_storage => rust_string_storage_name(options.string_storage),
                datetime_type => rust_datetime_type_name(options.datetime_type),
            },
        )?;
        write_file(&out_dir.join("runtime.rs"), rendered)
    }
}

#[derive(Debug, Clone, Serialize)]
struct RustModel {
    package: String,
    schema_fingerprint: String,
    enums: Vec<RustEnum>,
    unions: Vec<RustUnion>,
    records: Vec<RustRecord>,
    tables: Vec<RustTable>,
    modules: Vec<String>,
    has_map_tables: bool,
    has_singleton_tables: bool,
    has_unique_indexes: bool,
    has_non_unique_list_indexes: bool,
    has_non_unique_map_indexes: bool,
    has_localization: bool,
    locales: Vec<String>,
    default_locale: String,
}

#[derive(Debug, Clone, Serialize)]
struct RustEnum {
    name: String,
    snake_name: String,
    values: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct RustUnion {
    pascal_name: String,
    snake_name: String,
    tag: String,
    variants: Vec<RustUnionVariant>,
    imports: Vec<RustImport>,
    custom_imports: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct RustUnionVariant {
    name: String,
    fields: Vec<RustField>,
    text_key_patterns: String,
}

#[derive(Debug, Clone, Serialize)]
struct RustRecord {
    pascal_name: String,
    snake_name: String,
    imports: Vec<RustImport>,
    custom_imports: Vec<String>,
    fields: Vec<RustField>,
    table: Option<RustTable>,
}

#[derive(Debug, Clone, Serialize)]
struct RustImport {
    module: String,
    name: String,
}

#[derive(Debug, Clone, Serialize)]
struct RustTable {
    name: String,
    pascal_name: String,
    snake_name: String,
    mode: String,
    container_type: String,
    row_type: String,
    row_path: String,
    table_path: String,
    key_name: Option<String>,
    key_field_name: Option<String>,
    key_type: Option<String>,
    key_param_type: Option<String>,
    key_is_copy: bool,
    unique_indexes: Vec<RustIndex>,
    non_unique_indexes: Vec<RustIndex>,
}

#[derive(Debug, Clone, Serialize)]
struct RustIndex {
    name: String,
    method_name: String,
    field_name: String,
    param_name: String,
    param_type: String,
    key_type: String,
    key_is_copy: bool,
}

#[derive(Debug, Clone, Serialize)]
struct RustField {
    raw_name: String,
    name: String,
    type_name: String,
    serde_with: Option<String>,
    decode: String,
    collect_text_keys: String,
    text_key_binding: String,
    imports: Vec<String>,
    comment: Option<String>,
}

impl RustModel {
    fn from_base_model(ir: &ConfigIr, model: BaseModel, mapper: &RustTypeMapper<'_>) -> Self {
        let tables = model
            .tables
            .into_iter()
            .map(|item| rust_table(ir, item, mapper))
            .collect::<Vec<_>>();
        Self {
            package: model.package,
            schema_fingerprint: model.schema_fingerprint,
            enums: model
                .enums
                .into_iter()
                .map(|item| RustEnum {
                    name: item.pascal_name,
                    snake_name: item.snake_name,
                    values: item.values,
                })
                .collect(),
            unions: model
                .unions
                .into_iter()
                .map(|item| rust_union(ir, item, mapper))
                .collect(),
            records: model
                .records
                .into_iter()
                .map(|item| {
                    let table = tables
                        .iter()
                        .find(|table| table.snake_name == item.snake_name)
                        .cloned();
                    rust_record(ir, item, table, mapper)
                })
                .collect(),
            has_map_tables: tables
                .iter()
                .any(|table| table.mode == "map" && table.key_field_name.is_some()),
            has_singleton_tables: tables.iter().any(|table| table.mode == "singleton"),
            has_unique_indexes: tables.iter().any(|table| !table.unique_indexes.is_empty()),
            has_non_unique_list_indexes: tables
                .iter()
                .any(|table| table.mode == "list" && !table.non_unique_indexes.is_empty()),
            has_non_unique_map_indexes: tables
                .iter()
                .any(|table| table.mode == "map" && !table.non_unique_indexes.is_empty()),
            has_localization: ir.localization.is_some(),
            locales: ir
                .localization
                .as_ref()
                .map(|item| item.locales.clone())
                .unwrap_or_default(),
            default_locale: ir
                .localization
                .as_ref()
                .map(|item| item.default_locale.clone())
                .unwrap_or_default(),
            tables,
            modules: model.modules,
        }
    }
}

fn rust_union(ir: &ConfigIr, union: BaseUnion, mapper: &RustTypeMapper<'_>) -> RustUnion {
    let variants = union
        .variants
        .into_iter()
        .map(|variant| rust_variant(ir, variant, mapper))
        .collect::<Vec<_>>();
    let custom_imports = collect_rust_imports(variants.iter().flat_map(|variant| &variant.fields));
    RustUnion {
        pascal_name: union.pascal_name,
        snake_name: union.snake_name,
        tag: union.tag,
        variants,
        imports: union.imports.into_iter().map(rust_import).collect(),
        custom_imports,
    }
}

fn rust_variant(
    ir: &ConfigIr,
    variant: BaseUnionVariant,
    mapper: &RustTypeMapper<'_>,
) -> RustUnionVariant {
    let mut fields = variant
        .fields
        .into_iter()
        .map(|field| rust_field(ir, field, mapper))
        .collect::<Vec<_>>();
    for (index, field) in fields.iter_mut().enumerate() {
        field.text_key_binding = format!("__sora_text_field_{index}");
    }
    let text_key_patterns = fields
        .iter()
        .filter(|field| !field.collect_text_keys.is_empty())
        .map(|field| format!("{}: {}", field.name, field.text_key_binding))
        .collect::<Vec<_>>()
        .join(", ");
    RustUnionVariant {
        name: variant.pascal_name,
        fields,
        text_key_patterns,
    }
}

fn rust_record(
    ir: &ConfigIr,
    record: BaseRecord,
    table: Option<RustTable>,
    mapper: &RustTypeMapper<'_>,
) -> RustRecord {
    let fields = record
        .fields
        .into_iter()
        .map(|field| rust_field(ir, field, mapper))
        .collect::<Vec<_>>();
    let custom_imports = collect_rust_imports(fields.iter());
    RustRecord {
        pascal_name: record.pascal_name,
        snake_name: record.snake_name,
        imports: record.imports.into_iter().map(rust_import).collect(),
        custom_imports,
        fields,
        table,
    }
}

fn rust_table(ir: &ConfigIr, table: BaseTable, mapper: &RustTypeMapper<'_>) -> RustTable {
    let row_type = table.pascal_name.clone();
    let row_path = format!("{}::{}", table.snake_name, table.pascal_name);
    let table_path = format!("{}::{}Table", table.snake_name, table.pascal_name);
    let key_type = table
        .key_field
        .as_ref()
        .map(|field| mapper.local_table_key_type(&field.ty));
    let key_param_type = table
        .key_field
        .as_ref()
        .map(|field| mapper.key_param_type(&field.ty));
    let container_type = rust_container_type(table.mode, &row_type, key_type.as_deref());
    let key_field_name = table
        .key_field
        .as_ref()
        .map(|field| field.snake_name.clone());
    let key_is_copy = table
        .key_field
        .as_ref()
        .is_some_and(|field| mapper.key_type_is_copy(&field.ty));

    RustTable {
        name: table.name,
        pascal_name: table.pascal_name,
        snake_name: table.snake_name,
        mode: table.mode_name,
        container_type,
        row_type,
        row_path,
        table_path,
        key_name: table.key_name,
        key_field_name,
        key_type,
        key_param_type,
        key_is_copy,
        unique_indexes: table
            .unique_indexes
            .into_iter()
            .map(|index| rust_index(ir, index, mapper))
            .collect(),
        non_unique_indexes: table
            .non_unique_indexes
            .into_iter()
            .map(|index| rust_index(ir, index, mapper))
            .collect(),
    }
}

fn rust_index(_ir: &ConfigIr, index: BaseIndex, mapper: &RustTypeMapper<'_>) -> RustIndex {
    RustIndex {
        name: index.snake_name,
        method_name: index.method_name,
        field_name: index.field.snake_name.clone(),
        param_name: index.field.snake_name.clone(),
        param_type: mapper.key_param_type(&index.field.ty),
        key_type: mapper.local_table_key_type(&index.field.ty),
        key_is_copy: mapper.key_type_is_copy(&index.field.ty),
    }
}

fn rust_field(ir: &ConfigIr, field: BaseField, mapper: &RustTypeMapper<'_>) -> RustField {
    let collect_text_keys =
        rust_collect_text_keys(ir, &field.ty, &format!("self.{}", field.snake_name), mapper);
    RustField {
        raw_name: field.raw_name,
        name: field.snake_name,
        type_name: mapper.type_name(&field.ty),
        serde_with: rust_serde_with(&field.ty, mapper.options),
        decode: rust_decode_expr(ir, &field.ty, mapper),
        collect_text_keys,
        text_key_binding: String::new(),
        imports: mapper.imports(&field.ty),
        comment: field.comment,
    }
}

fn collect_rust_imports<'a>(fields: impl Iterator<Item = &'a RustField>) -> Vec<String> {
    let mut imports = fields
        .flat_map(|field| field.imports.iter().cloned())
        .collect::<Vec<_>>();
    imports.sort();
    imports.dedup();
    imports
}

fn rust_collect_text_keys(
    ir: &ConfigIr,
    ty: &TypeIr,
    value: &str,
    mapper: &RustTypeMapper<'_>,
) -> String {
    if mapper.mapping(ty).is_some() {
        return String::new();
    }
    match ty {
        TypeIr::Text => format!("out.push(&{value});"),
        TypeIr::Optional(element) => {
            let inner = rust_collect_text_keys(ir, element, "value", mapper);
            if inner.is_empty() {
                String::new()
            } else {
                format!("if let Some(value) = &{value} {{ {inner} }}")
            }
        }
        TypeIr::List(element) | TypeIr::Set(element) | TypeIr::Array { element, .. } => {
            let inner = rust_collect_text_keys(ir, element, "value", mapper);
            if inner.is_empty() {
                String::new()
            } else {
                format!("for value in {value}.iter() {{ {inner} }}")
            }
        }
        TypeIr::Map {
            key,
            value: element,
        } => {
            let key_inner = rust_collect_text_keys(ir, key, "key", mapper);
            let value_inner = rust_collect_text_keys(ir, element, "value", mapper);
            if key_inner.is_empty() && value_inner.is_empty() {
                String::new()
            } else {
                format!("for (key, value) in {value}.iter() {{ {key_inner} {value_inner} }}")
            }
        }
        TypeIr::Struct(_) | TypeIr::Union(_) => format!("{value}.collect_text_keys(out);"),
        TypeIr::Ref { table, field } => ir
            .tables
            .iter()
            .find(|candidate| candidate.name == *table)
            .and_then(|table| {
                table
                    .fields
                    .iter()
                    .find(|candidate| candidate.name == *field)
            })
            .map(|field| rust_collect_text_keys(ir, &field.ty, value, mapper))
            .unwrap_or_default(),
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
        | TypeIr::Enum(_) => String::new(),
    }
}

fn rust_import(import: BaseImport) -> RustImport {
    RustImport {
        module: import.module,
        name: import.name,
    }
}

struct RustTypeMapper<'a> {
    target: &'a str,
    ir: &'a ConfigIr,
    mappings: &'a TypeMappingRegistry,
    options: &'a RustCodegenOptions,
}

impl<'a> RustTypeMapper<'a> {
    fn new(
        target: &'a str,
        ir: &'a ConfigIr,
        mappings: &'a TypeMappingRegistry,
        options: &'a RustCodegenOptions,
    ) -> Self {
        Self {
            target,
            ir,
            mappings,
            options,
        }
    }

    fn type_name(&self, ty: &TypeIr) -> String {
        if let Some(mapping) = self.mapping(ty) {
            return mapping.type_name;
        }

        match ty {
            TypeIr::Bool => "bool".to_owned(),
            TypeIr::I8 => "i8".to_owned(),
            TypeIr::U8 => "u8".to_owned(),
            TypeIr::I16 => "i16".to_owned(),
            TypeIr::U16 => "u16".to_owned(),
            TypeIr::I32 => "i32".to_owned(),
            TypeIr::U32 => "u32".to_owned(),
            TypeIr::I64 => "i64".to_owned(),
            TypeIr::Duration => "std::time::Duration".to_owned(),
            TypeIr::DateTime => match self.options.datetime_type {
                RustDateTimeType::SystemTime => "std::time::SystemTime".to_owned(),
                RustDateTimeType::Chrono => "chrono::DateTime<chrono::Utc>".to_owned(),
            },
            TypeIr::F32 => "f32".to_owned(),
            TypeIr::F64 => "f64".to_owned(),
            TypeIr::String => match self.options.string_storage {
                RustStringStorage::Owned => "String".to_owned(),
                RustStringStorage::Arc => "std::sync::Arc<str>".to_owned(),
            },
            TypeIr::Text => "super::runtime::TextKey".to_owned(),
            TypeIr::Enum(name) | TypeIr::Struct(name) | TypeIr::Union(name) => name.clone(),
            TypeIr::List(element) => format!("Vec<{}>", self.type_name(element)),
            TypeIr::Set(element) => {
                format!("std::collections::HashSet<{}>", self.type_name(element))
            }
            TypeIr::Map { key, value } => {
                format!(
                    "std::collections::HashMap<{}, {}>",
                    self.type_name(key),
                    self.type_name(value)
                )
            }
            TypeIr::Array { element, len } => format!("[{}; {len}]", self.type_name(element)),
            TypeIr::Ref { table, field } => ref_target_type(self.ir, table, field)
                .map(|ty| self.type_name(ty))
                .unwrap_or_else(|| "i32".to_owned()),
            TypeIr::Optional(element) => format!("Option<{}>", self.type_name(element)),
        }
    }

    fn key_param_type(&self, ty: &TypeIr) -> String {
        let type_name = self.local_table_key_type(ty);
        if type_name == "String" || type_name == "std::sync::Arc<str>" {
            "str".to_owned()
        } else {
            type_name
        }
    }

    fn local_table_key_type(&self, ty: &TypeIr) -> String {
        match ty {
            TypeIr::Ref { table, field } => ref_target_type(self.ir, table, field)
                .map(|ty| self.local_table_key_type(ty))
                .unwrap_or_else(|| self.type_name(ty)),
            _ => self.type_name(ty),
        }
    }

    fn key_type_is_copy(&self, ty: &TypeIr) -> bool {
        match ty {
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
            | TypeIr::Enum(_) => self.mapping(ty).is_none(),
            TypeIr::Ref { table, field } => {
                ref_target_type(self.ir, table, field).is_some_and(|ty| self.key_type_is_copy(ty))
            }
            TypeIr::Optional(element) => self.key_type_is_copy(element),
            TypeIr::String
            | TypeIr::Text
            | TypeIr::Struct(_)
            | TypeIr::Union(_)
            | TypeIr::List(_)
            | TypeIr::Set(_)
            | TypeIr::Map { .. }
            | TypeIr::Array { .. } => false,
        }
    }

    fn imports(&self, ty: &TypeIr) -> Vec<String> {
        self.mappings.imports_for(self.target, self.ir, ty)
    }

    fn mapping(&self, ty: &TypeIr) -> Option<TypeMapping> {
        self.mappings.map_type(TypeMappingContext {
            target: self.target,
            ir: self.ir,
            ty,
        })
    }

    fn wrap_decode(&self, ty: &TypeIr, base_expr: String) -> String {
        self.mapping(ty)
            .map(|mapping| mapping.wrap_decode(&base_expr))
            .unwrap_or(base_expr)
    }
}

fn rust_decode_expr(ir: &ConfigIr, ty: &TypeIr, mapper: &RustTypeMapper<'_>) -> String {
    match ty {
        TypeIr::Enum(name) | TypeIr::Struct(name) | TypeIr::Union(name) => mapper.wrap_decode(
            ty,
            format!("<{name} as super::runtime::SoraDecode>::decode(reader)?"),
        ),
        TypeIr::List(element) => format!(
            "{{ let len = reader.read_var_u32()? as usize; let mut values = Vec::with_capacity(len); for _ in 0..len {{ values.push({}); }} values }}",
            rust_decode_expr(ir, element, mapper)
        ),
        TypeIr::Set(element) => format!(
            "{{ let len = reader.read_var_u32()? as usize; let mut values = std::collections::HashSet::with_capacity(len); for _ in 0..len {{ values.insert({}); }} values }}",
            rust_decode_expr(ir, element, mapper)
        ),
        TypeIr::Map { key, value } => format!(
            "{{ let len = reader.read_var_u32()? as usize; let mut values = std::collections::HashMap::with_capacity(len); for _ in 0..len {{ values.insert({}, {}); }} values }}",
            rust_decode_expr(ir, key, mapper),
            rust_decode_expr(ir, value, mapper)
        ),
        TypeIr::Array { element, len } => {
            let element_type = mapper.type_name(element);
            format!(
                "{{ let mut values = Vec::with_capacity({len}); let actual_len = reader.read_var_u32()? as usize; if actual_len != {len} {{ return Err(super::runtime::SoraReadError::new(format!(\"expected array length {len}, got {{}}\", actual_len))); }} for _ in 0..{len} {{ values.push({}); }} values.try_into().map_err(|values: Vec<{element_type}>| super::runtime::SoraReadError::new(format!(\"expected array length {len}, got {{}}\", values.len())))? }}",
                rust_decode_expr(ir, element, mapper)
            )
        }
        TypeIr::Ref { table, field } => ref_target_type(ir, table, field)
            .map(|ty| rust_decode_expr(ir, ty, mapper))
            .unwrap_or_else(|| "<i32 as super::runtime::SoraDecode>::decode(reader)?".to_owned()),
        TypeIr::Optional(element) => {
            format!(
                "match reader.read_u8()? {{ 0 => None, 1 => Some({}), value => return Err(super::runtime::SoraReadError::new(format!(\"invalid option presence {{}}\", value))), }}",
                rust_decode_expr(ir, element, mapper)
            )
        }
        _ => format!(
            "<{} as super::runtime::SoraDecode>::decode(reader)?",
            mapper.type_name(ty)
        ),
    }
}

fn rust_container_type(mode: TableModeIr, row_type: &str, key_type: Option<&str>) -> String {
    match mode {
        TableModeIr::List => format!("Vec<{row_type}>"),
        TableModeIr::Map => match key_type {
            Some(key_type) => format!("SoraMap<{key_type}, {row_type}>"),
            None => format!("Vec<{row_type}>"),
        },
        TableModeIr::Singleton => row_type.to_owned(),
    }
}

fn rust_string_storage_name(storage: RustStringStorage) -> &'static str {
    match storage {
        RustStringStorage::Owned => "owned",
        RustStringStorage::Arc => "arc",
    }
}

fn rust_datetime_type_name(datetime_type: RustDateTimeType) -> &'static str {
    match datetime_type {
        RustDateTimeType::SystemTime => "system_time",
        RustDateTimeType::Chrono => "chrono",
    }
}

fn rust_serde_with(ty: &TypeIr, options: &RustCodegenOptions) -> Option<String> {
    match ty {
        TypeIr::DateTime => Some(
            match options.datetime_type {
                RustDateTimeType::SystemTime => "super::runtime::serde_system_time_millis",
                RustDateTimeType::Chrono => "super::runtime::serde_chrono_datetime_millis",
            }
            .to_owned(),
        ),
        _ => None,
    }
}

fn ref_target_type<'a>(ir: &'a ConfigIr, table: &str, field: &str) -> Option<&'a TypeIr> {
    ir.tables
        .iter()
        .find(|candidate| candidate.name == table)
        .and_then(|table| {
            table
                .fields
                .iter()
                .find(|candidate| candidate.name == field)
        })
        .map(|field| &field.ty)
}
