use std::path::Path;

use heck::{ToLowerCamelCase, ToSnakeCase};
use minijinja::context;
use serde::Serialize;
use sora_diagnostics::{Result, SoraError};
use sora_ir::model::{ConfigIr, TableModeIr, TypeIr};

use crate::{
    generator::{CodeGenerator, CodegenContext, runtime_format_name},
    model::{
        BaseField, BaseIndex, BaseModel, BaseRecord, BaseTable, BaseUnion, BaseUnionVariant,
        build_base_model,
    },
    options::LanguageCodegenOptions,
    render::{ensure_dir, render_template, write_file},
    type_mapping::{TypeMapping, TypeMappingContext, TypeMappingRegistry},
};

pub struct GoCodeGenerator;
crate::impl_test_codegen_generate!(GoCodeGenerator, "go");

impl CodeGenerator for GoCodeGenerator {
    fn generate(&self, context: CodegenContext<'_>, out_dir: &Path) -> Result<()> {
        let ir = context.ir;
        let options = context.options::<LanguageCodegenOptions>()?;
        ensure_dir(out_dir)?;
        let mapper = GoTypeMapper::new(context.target, ir, context.type_mappings);
        let model = GoModel::from_base_model(ir, build_base_model(ir)?, &mapper);
        let package = go_package_name(&model.package)?;
        let runtime_format = runtime_format_name(options.runtime_format);

        for item in &model.enums {
            let rendered = render_template(
                "go",
                "enum.go.j2",
                context! { package => &package, enum => item, runtime_format => runtime_format },
            )?;
            write_file(&out_dir.join(format!("{}.go", item.snake_name)), rendered)?;
        }

        for record in &model.records {
            let rendered = render_template(
                "go",
                "record.go.j2",
                context! { package => &package, record => record, runtime_format => runtime_format },
            )?;
            write_file(&out_dir.join(format!("{}.go", record.snake_name)), rendered)?;
        }

        for union in &model.unions {
            let rendered = render_template(
                "go",
                "union.go.j2",
                context! { package => &package, union => union, runtime_format => runtime_format },
            )?;
            write_file(&out_dir.join(format!("{}.go", union.snake_name)), rendered)?;
        }

        let rendered = render_template(
            "go",
            "runtime.go.j2",
            context! { package => &package, runtime_format => runtime_format },
        )?;
        write_file(&out_dir.join("runtime.go"), rendered)?;

        let rendered = render_template(
            "go",
            "config.go.j2",
            context! { package => &package, model => &model, runtime_format => runtime_format },
        )?;
        write_file(&out_dir.join("config.go"), rendered)
    }
}

#[derive(Debug, Clone, Serialize)]
struct GoModel {
    package: String,
    schema_fingerprint: String,
    enums: Vec<GoEnum>,
    unions: Vec<GoUnion>,
    records: Vec<GoRecord>,
    tables: Vec<GoTable>,
    has_unique_indexes: bool,
    has_non_unique_indexes: bool,
    has_localization: bool,
    locales: Vec<String>,
    default_locale: String,
}

#[derive(Debug, Clone, Serialize)]
struct GoEnum {
    name: String,
    snake_name: String,
    values: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct GoUnion {
    pascal_name: String,
    snake_name: String,
    tag: String,
    variants: Vec<GoUnionVariant>,
    has_time: bool,
    imports: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct GoUnionVariant {
    name: String,
    fields: Vec<GoField>,
    has_text_keys: bool,
}

#[derive(Debug, Clone, Serialize)]
struct GoRecord {
    pascal_name: String,
    snake_name: String,
    fields: Vec<GoField>,
    has_time: bool,
    table: Option<GoTable>,
    imports: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct GoTable {
    name: String,
    pascal_name: String,
    camel_name: String,
    mode: String,
    container_type: String,
    row_type: String,
    key_name: Option<String>,
    key_field_name: Option<String>,
    key_type: Option<String>,
    unique_indexes: Vec<GoIndex>,
    non_unique_indexes: Vec<GoIndex>,
}

#[derive(Debug, Clone, Serialize)]
struct GoIndex {
    pascal_name: String,
    camel_name: String,
    field_name: String,
    param_camel_name: String,
    key_type: String,
}

#[derive(Debug, Clone, Serialize)]
struct GoField {
    raw_name: String,
    name: String,
    type_name: String,
    decode: String,
    value_decode: String,
    collect_text_keys: String,
    imports: Vec<String>,
    comment: Option<String>,
}

impl GoModel {
    fn from_base_model(ir: &ConfigIr, model: BaseModel, mapper: &GoTypeMapper<'_>) -> Self {
        let tables = model
            .tables
            .into_iter()
            .map(|table| go_table(ir, table, mapper))
            .collect::<Vec<_>>();
        Self {
            package: model.package,
            schema_fingerprint: model.schema_fingerprint,
            enums: model
                .enums
                .into_iter()
                .map(|item| GoEnum {
                    name: item.pascal_name,
                    snake_name: item.snake_name,
                    values: item.values,
                })
                .collect(),
            unions: model
                .unions
                .into_iter()
                .map(|item| go_union(ir, item, mapper))
                .collect(),
            records: model
                .records
                .into_iter()
                .map(|item| {
                    let table = tables
                        .iter()
                        .find(|table| table.row_type == item.pascal_name)
                        .cloned();
                    go_record(ir, item, table, mapper)
                })
                .collect(),
            has_unique_indexes: tables.iter().any(|table| !table.unique_indexes.is_empty()),
            has_non_unique_indexes: tables
                .iter()
                .any(|table| !table.non_unique_indexes.is_empty()),
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
        }
    }
}

fn go_union(ir: &ConfigIr, union: BaseUnion, mapper: &GoTypeMapper<'_>) -> GoUnion {
    let variants = union
        .variants
        .into_iter()
        .map(|variant| go_variant(ir, variant, mapper))
        .collect::<Vec<_>>();
    let has_time = variants
        .iter()
        .flat_map(|variant| variant.fields.iter())
        .any(|field| field.type_name.contains("time."));
    let imports = collect_go_imports(variants.iter().flat_map(|variant| &variant.fields));
    GoUnion {
        pascal_name: union.pascal_name,
        snake_name: union.snake_name,
        tag: union.tag,
        variants,
        has_time,
        imports,
    }
}

fn go_variant(
    ir: &ConfigIr,
    variant: BaseUnionVariant,
    mapper: &GoTypeMapper<'_>,
) -> GoUnionVariant {
    let fields = variant
        .fields
        .into_iter()
        .map(|field| go_field(ir, field, mapper))
        .collect::<Vec<_>>();
    let has_text_keys = fields
        .iter()
        .any(|field| !field.collect_text_keys.is_empty());
    GoUnionVariant {
        name: variant.pascal_name,
        fields,
        has_text_keys,
    }
}

fn go_record(
    ir: &ConfigIr,
    record: BaseRecord,
    table: Option<GoTable>,
    mapper: &GoTypeMapper<'_>,
) -> GoRecord {
    let fields = record
        .fields
        .into_iter()
        .map(|field| go_field(ir, field, mapper))
        .collect::<Vec<_>>();
    let has_time = fields.iter().any(|field| field.type_name.contains("time."));
    let imports = collect_go_imports(fields.iter());
    GoRecord {
        pascal_name: record.pascal_name,
        snake_name: record.snake_name,
        fields,
        has_time,
        table,
        imports,
    }
}

fn go_table(ir: &ConfigIr, table: BaseTable, mapper: &GoTypeMapper<'_>) -> GoTable {
    let row_type = table.pascal_name.clone();
    let key_type = table
        .key_field
        .as_ref()
        .map(|field| mapper.type_name(&field.ty));
    let container_type = go_container_type(table.mode, &row_type, key_type.as_deref());
    let key_field_name = table
        .key_field
        .as_ref()
        .map(|field| field.pascal_name.clone());

    GoTable {
        name: table.name,
        pascal_name: table.pascal_name,
        camel_name: table.camel_name,
        mode: table.mode_name,
        container_type,
        row_type,
        key_name: table.key_name,
        key_field_name,
        key_type,
        unique_indexes: table
            .unique_indexes
            .into_iter()
            .map(|index| go_index(ir, index, mapper))
            .collect(),
        non_unique_indexes: table
            .non_unique_indexes
            .into_iter()
            .map(|index| go_index(ir, index, mapper))
            .collect(),
    }
}

fn go_index(_ir: &ConfigIr, index: BaseIndex, mapper: &GoTypeMapper<'_>) -> GoIndex {
    GoIndex {
        pascal_name: index.pascal_name,
        camel_name: index.name.to_lower_camel_case(),
        field_name: index.field.pascal_name.clone(),
        param_camel_name: index.field.camel_name,
        key_type: mapper.type_name(&index.field.ty),
    }
}

fn go_field(ir: &ConfigIr, field: BaseField, mapper: &GoTypeMapper<'_>) -> GoField {
    let value_decode = go_value_decode_expr(ir, &field.ty, "__VALUE__", mapper);
    let collect_text_keys = go_collect_text_keys(
        ir,
        &field.ty,
        &format!("value.{}", field.pascal_name),
        mapper,
    );
    GoField {
        raw_name: field.raw_name,
        name: field.pascal_name,
        type_name: mapper.type_name(&field.ty),
        decode: go_decode_expr(ir, &field.ty, mapper),
        value_decode,
        collect_text_keys,
        imports: mapper.imports(&field.ty),
        comment: field.comment,
    }
}

fn collect_go_imports<'a>(fields: impl Iterator<Item = &'a GoField>) -> Vec<String> {
    let mut imports = fields
        .flat_map(|field| field.imports.iter().cloned())
        .collect::<Vec<_>>();
    imports.sort();
    imports.dedup();
    imports
}

fn go_container_type(mode: TableModeIr, row_type: &str, key_type: Option<&str>) -> String {
    match mode {
        TableModeIr::List => format!("[]{row_type}"),
        TableModeIr::Map => match key_type {
            Some(key_type) => format!("map[{key_type}]{row_type}"),
            None => format!("[]{row_type}"),
        },
        TableModeIr::Singleton => row_type.to_owned(),
    }
}

struct GoTypeMapper<'a> {
    target: &'a str,
    ir: &'a ConfigIr,
    mappings: &'a TypeMappingRegistry,
}

impl<'a> GoTypeMapper<'a> {
    fn new(target: &'a str, ir: &'a ConfigIr, mappings: &'a TypeMappingRegistry) -> Self {
        Self {
            target,
            ir,
            mappings,
        }
    }

    fn type_name(&self, ty: &TypeIr) -> String {
        if let Some(mapping) = self.mapping(ty) {
            return mapping.type_name;
        }

        match ty {
            TypeIr::Bool => "bool".to_owned(),
            TypeIr::I8 => "int8".to_owned(),
            TypeIr::U8 => "uint8".to_owned(),
            TypeIr::I16 => "int16".to_owned(),
            TypeIr::U16 => "uint16".to_owned(),
            TypeIr::I32 => "int32".to_owned(),
            TypeIr::U32 => "uint32".to_owned(),
            TypeIr::I64 => "int64".to_owned(),
            TypeIr::Duration => "time.Duration".to_owned(),
            TypeIr::DateTime => "time.Time".to_owned(),
            TypeIr::F32 => "float32".to_owned(),
            TypeIr::F64 => "float64".to_owned(),
            TypeIr::String => "string".to_owned(),
            TypeIr::Text => "TextKey".to_owned(),
            TypeIr::Enum(name) | TypeIr::Struct(name) | TypeIr::Union(name) => name.clone(),
            TypeIr::List(element) | TypeIr::Set(element) | TypeIr::Array { element, .. } => {
                format!("[]{}", self.type_name(element))
            }
            TypeIr::Map { key, value } => {
                format!("map[{}]{}", self.type_name(key), self.type_name(value))
            }
            TypeIr::Ref { table, field } => ref_target_type(self.ir, table, field)
                .map(|ty| self.type_name(ty))
                .unwrap_or_else(|| "int32".to_owned()),
            TypeIr::Optional(element) => format!("*{}", self.type_name(element)),
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

    fn wrap_value_decode(&self, ty: &TypeIr, base_expr: String) -> String {
        self.mapping(ty)
            .map(|mapping| mapping.wrap_value_decode(&base_expr))
            .unwrap_or(base_expr)
    }
}

fn go_decode_expr(ir: &ConfigIr, ty: &TypeIr, mapper: &GoTypeMapper<'_>) -> String {
    match ty {
        TypeIr::Bool => "reader.ReadBool()".to_owned(),
        TypeIr::I8 => "reader.ReadInt8()".to_owned(),
        TypeIr::U8 => "reader.ReadUInt8Value()".to_owned(),
        TypeIr::I16 => "reader.ReadInt16()".to_owned(),
        TypeIr::U16 => "reader.ReadUInt16()".to_owned(),
        TypeIr::I32 => "reader.ReadInt32()".to_owned(),
        TypeIr::U32 => "reader.ReadUInt32()".to_owned(),
        TypeIr::I64 => "reader.ReadInt64()".to_owned(),
        TypeIr::Duration => "ReadDuration(reader)".to_owned(),
        TypeIr::DateTime => "ReadDateTime(reader)".to_owned(),
        TypeIr::F32 => "reader.ReadFloat32()".to_owned(),
        TypeIr::F64 => "reader.ReadFloat64()".to_owned(),
        TypeIr::String => "reader.ReadString()".to_owned(),
        TypeIr::Text => "ReadTextKey(reader)".to_owned(),
        TypeIr::Enum(name) | TypeIr::Struct(name) | TypeIr::Union(name) => {
            mapper.wrap_decode(ty, format!("decode{name}(reader)"))
        }
        TypeIr::List(element) | TypeIr::Set(element) | TypeIr::Array { element, .. } => {
            format!(
                "ReadList(reader, func(reader *SoraReader) ({}, error) {{ return {} }})",
                mapper.type_name(element),
                go_decode_expr(ir, element, mapper)
            )
        }
        TypeIr::Map { key, value } => format!(
            "ReadMap(reader, func(reader *SoraReader) ({}, error) {{ return {} }}, func(reader *SoraReader) ({}, error) {{ return {} }})",
            mapper.type_name(key),
            go_decode_expr(ir, key, mapper),
            mapper.type_name(value),
            go_decode_expr(ir, value, mapper)
        ),
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
            .map(|field| go_decode_expr(ir, &field.ty, mapper))
            .unwrap_or_else(|| "reader.ReadInt32()".to_owned()),
        TypeIr::Optional(element) => {
            format!(
                "ReadOptional(reader, func(reader *SoraReader) ({}, error) {{ return {} }})",
                mapper.type_name(element),
                go_decode_expr(ir, element, mapper)
            )
        }
    }
}

fn go_value_decode_expr(
    ir: &ConfigIr,
    ty: &TypeIr,
    value: &str,
    mapper: &GoTypeMapper<'_>,
) -> String {
    match ty {
        TypeIr::Bool => format!("{value}.AsBool()"),
        TypeIr::I8 => format!("{value}.AsInt8()"),
        TypeIr::U8 => format!("{value}.AsUInt8()"),
        TypeIr::I16 => format!("{value}.AsInt16()"),
        TypeIr::U16 => format!("{value}.AsUInt16()"),
        TypeIr::I32 => format!("{value}.AsInt32()"),
        TypeIr::U32 => format!("{value}.AsUInt32()"),
        TypeIr::I64 => format!("{value}.AsInt64()"),
        TypeIr::Duration => format!("DecodeDurationValue({value})"),
        TypeIr::DateTime => format!("DecodeDateTimeValue({value})"),
        TypeIr::F32 => format!("{value}.AsFloat32()"),
        TypeIr::F64 => format!("{value}.AsFloat64()"),
        TypeIr::String => format!("{value}.AsString()"),
        TypeIr::Text => format!("DecodeTextKeyValue({value})"),
        TypeIr::Enum(name) | TypeIr::Struct(name) | TypeIr::Union(name) => {
            mapper.wrap_value_decode(ty, format!("decode{name}Value({value})"))
        }
        TypeIr::List(element) | TypeIr::Set(element) | TypeIr::Array { element, .. } => {
            format!(
                "DecodeSoraValueList({value}, func(item SoraValue) ({}, error) {{ return {} }})",
                mapper.type_name(element),
                go_value_decode_expr(ir, element, "item", mapper)
            )
        }
        TypeIr::Map {
            key,
            value: element,
        } => format!(
            "DecodeSoraValueMap({value}, func(item SoraValue) ({}, error) {{ return {} }}, func(item SoraValue) ({}, error) {{ return {} }})",
            mapper.type_name(key),
            go_value_decode_expr(ir, key, "item", mapper),
            mapper.type_name(element),
            go_value_decode_expr(ir, element, "item", mapper)
        ),
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
            .map(|field| go_value_decode_expr(ir, &field.ty, value, mapper))
            .unwrap_or_else(|| format!("{value}.AsInt32()")),
        TypeIr::Optional(element) => {
            format!(
                "DecodeOptionalSoraValue({value}, func(item SoraValue) ({}, error) {{ return {} }})",
                mapper.type_name(element),
                go_value_decode_expr(ir, element, "item", mapper)
            )
        }
    }
}

fn go_collect_text_keys(
    ir: &ConfigIr,
    ty: &TypeIr,
    value: &str,
    mapper: &GoTypeMapper<'_>,
) -> String {
    if mapper.mapping(ty).is_some() {
        return String::new();
    }
    match ty {
        TypeIr::Text => format!("*out = append(*out, {value})"),
        TypeIr::Optional(element) => {
            if matches!(element.as_ref(), TypeIr::Text) {
                return format!("if {value} != nil {{ *out = append(*out, *{value}) }}");
            }
            let inner = go_collect_text_keys(ir, element, "item", mapper);
            if inner.is_empty() {
                String::new()
            } else {
                format!("if {value} != nil {{ item := {value}; {inner} }}")
            }
        }
        TypeIr::List(element) | TypeIr::Set(element) | TypeIr::Array { element, .. } => {
            let inner = go_collect_text_keys(ir, element, "item", mapper);
            if inner.is_empty() {
                String::new()
            } else {
                format!("for _, item := range {value} {{ {inner} }}")
            }
        }
        TypeIr::Map {
            key,
            value: element,
        } => {
            let key_inner = go_collect_text_keys(ir, key, "key", mapper);
            let value_inner = go_collect_text_keys(ir, element, "item", mapper);
            if key_inner.is_empty() && value_inner.is_empty() {
                String::new()
            } else {
                format!("for key, item := range {value} {{ {key_inner}; {value_inner} }}")
            }
        }
        TypeIr::Struct(_) => format!("{value}.collectTextKeys(out)"),
        TypeIr::Union(name) => format!("collect{name}TextKeys({value}, out)"),
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
            .map(|field| go_collect_text_keys(ir, &field.ty, value, mapper))
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

fn go_package_name(package: &str) -> Result<String> {
    let Some(segment) = package.rsplit('.').next() else {
        return Err(SoraError::InvalidSchema(
            "go package must not be empty".to_owned(),
        ));
    };
    let package = segment.to_snake_case();
    if !is_go_package_name(&package) {
        return Err(SoraError::InvalidSchema(format!(
            "go package `{package}` must be a valid identifier"
        )));
    }
    Ok(package)
}

fn is_go_package_name(package: &str) -> bool {
    let mut chars = package.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || first == '_')
        && chars.all(|value| value.is_ascii_alphanumeric() || value == '_')
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
