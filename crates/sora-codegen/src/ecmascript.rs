use heck::ToLowerCamelCase;
use serde::Serialize;
use sora_ir::model::{ConfigIr, EnumReprIr, TypeIr};

use crate::model::{
    BaseField, BaseImport, BaseIndex, BaseModel, BaseRecord, BaseTable, BaseUnion, BaseUnionVariant,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EcmaScriptTarget {
    TypeScript,
    JavaScript,
}

#[derive(Debug, Clone, Serialize)]
pub struct EcmaScriptOptionsView {
    pub import_ext: &'static str,
    pub enum_is_integer: bool,
    pub emit_dts: bool,
}

impl EcmaScriptOptionsView {
    pub fn new(target: EcmaScriptTarget, enum_repr: EnumReprIr, emit_dts: bool) -> Self {
        Self {
            import_ext: ".js",
            enum_is_integer: enum_repr == EnumReprIr::Integer,
            emit_dts: target == EcmaScriptTarget::JavaScript && emit_dts,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct EcmaScriptModel {
    pub package: String,
    pub enums: Vec<EcmaScriptEnum>,
    pub unions: Vec<EcmaScriptUnion>,
    pub records: Vec<EcmaScriptRecord>,
    pub tables: Vec<EcmaScriptTable>,
    pub modules: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EcmaScriptEnum {
    pub name: String,
    pub snake_name: String,
    pub values: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EcmaScriptUnion {
    pub pascal_name: String,
    pub snake_name: String,
    pub tag: String,
    pub variants: Vec<EcmaScriptUnionVariant>,
    pub imports: Vec<EcmaScriptImport>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EcmaScriptUnionVariant {
    pub raw_name: String,
    pub name: String,
    pub fields: Vec<EcmaScriptField>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EcmaScriptRecord {
    pub pascal_name: String,
    pub snake_name: String,
    pub imports: Vec<EcmaScriptImport>,
    pub fields: Vec<EcmaScriptField>,
    pub table: Option<EcmaScriptTable>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EcmaScriptImport {
    pub module: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct EcmaScriptTable {
    pub name: String,
    pub pascal_name: String,
    pub camel_name: String,
    pub snake_name: String,
    pub mode: String,
    pub row_type: String,
    pub key_name: Option<String>,
    pub key_field_name: Option<String>,
    pub key_type: Option<String>,
    pub unique_indexes: Vec<EcmaScriptIndex>,
    pub non_unique_indexes: Vec<EcmaScriptIndex>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EcmaScriptIndex {
    pub name: String,
    pub pascal_name: String,
    pub camel_name: String,
    pub field_name: String,
    pub param_camel_name: String,
    pub param_type: String,
    pub key_type: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct EcmaScriptField {
    pub raw_name: String,
    pub name: String,
    pub type_name: String,
    pub decode: String,
    pub value_decode: String,
    pub comment: Option<String>,
}

impl EcmaScriptModel {
    pub fn from_base_model(ir: &ConfigIr, model: BaseModel) -> Self {
        let tables = model
            .tables
            .into_iter()
            .map(|item| ecmascript_table(ir, item))
            .collect::<Vec<_>>();
        Self {
            package: model.package,
            enums: model
                .enums
                .into_iter()
                .map(|item| EcmaScriptEnum {
                    name: item.pascal_name,
                    snake_name: item.snake_name,
                    values: item.values,
                })
                .collect(),
            unions: model
                .unions
                .into_iter()
                .map(|item| ecmascript_union(ir, item))
                .collect(),
            records: model
                .records
                .into_iter()
                .map(|item| {
                    let table = tables
                        .iter()
                        .find(|table| table.row_type == item.pascal_name)
                        .cloned();
                    ecmascript_record(ir, item, table)
                })
                .collect(),
            tables,
            modules: model.modules,
        }
    }
}

fn ecmascript_union(ir: &ConfigIr, union: BaseUnion) -> EcmaScriptUnion {
    EcmaScriptUnion {
        pascal_name: union.pascal_name,
        snake_name: union.snake_name,
        tag: union.tag.to_lower_camel_case(),
        variants: union
            .variants
            .into_iter()
            .map(|variant| ecmascript_variant(ir, variant))
            .collect(),
        imports: union.imports.into_iter().map(ecmascript_import).collect(),
    }
}

fn ecmascript_variant(ir: &ConfigIr, variant: BaseUnionVariant) -> EcmaScriptUnionVariant {
    EcmaScriptUnionVariant {
        raw_name: variant.name,
        name: variant.pascal_name,
        fields: variant
            .fields
            .into_iter()
            .map(|field| ecmascript_field(ir, field))
            .collect(),
    }
}

fn ecmascript_record(
    ir: &ConfigIr,
    record: BaseRecord,
    table: Option<EcmaScriptTable>,
) -> EcmaScriptRecord {
    EcmaScriptRecord {
        pascal_name: record.pascal_name,
        snake_name: record.snake_name,
        imports: record.imports.into_iter().map(ecmascript_import).collect(),
        fields: record
            .fields
            .into_iter()
            .map(|field| ecmascript_field(ir, field))
            .collect(),
        table,
    }
}

fn ecmascript_table(ir: &ConfigIr, table: BaseTable) -> EcmaScriptTable {
    let row_type = table.pascal_name.clone();
    let key_type = table
        .key_field
        .as_ref()
        .map(|field| ecmascript_type_name(ir, &field.ty));
    let key_field_name = table
        .key_field
        .as_ref()
        .map(|field| field.camel_name.clone());

    EcmaScriptTable {
        name: table.name,
        pascal_name: table.pascal_name,
        camel_name: table.camel_name,
        snake_name: table.snake_name,
        mode: table.mode_name,
        row_type,
        key_name: table.key_name,
        key_field_name,
        key_type,
        unique_indexes: table
            .unique_indexes
            .into_iter()
            .map(|index| ecmascript_index(ir, index))
            .collect(),
        non_unique_indexes: table
            .non_unique_indexes
            .into_iter()
            .map(|index| ecmascript_index(ir, index))
            .collect(),
    }
}

fn ecmascript_index(ir: &ConfigIr, index: BaseIndex) -> EcmaScriptIndex {
    let key_type = ecmascript_type_name(ir, &index.field.ty);
    EcmaScriptIndex {
        name: index.snake_name,
        pascal_name: index.pascal_name,
        camel_name: index.camel_name,
        field_name: index.field.camel_name.clone(),
        param_camel_name: index.field.camel_name,
        param_type: key_type.clone(),
        key_type,
    }
}

fn ecmascript_field(ir: &ConfigIr, field: BaseField) -> EcmaScriptField {
    EcmaScriptField {
        raw_name: field.raw_name,
        name: field.camel_name,
        type_name: ecmascript_type_name(ir, &field.ty),
        decode: ecmascript_decode_expr(ir, &field.ty),
        value_decode: ecmascript_value_decode_expr(ir, &field.ty, "__VALUE__"),
        comment: field.comment,
    }
}

fn ecmascript_import(import: BaseImport) -> EcmaScriptImport {
    EcmaScriptImport {
        module: import.module,
        name: import.name,
    }
}

pub fn ecmascript_type_name(ir: &ConfigIr, ty: &TypeIr) -> String {
    match ty {
        TypeIr::Bool => "boolean".to_owned(),
        TypeIr::I32 => "number".to_owned(),
        TypeIr::I64 => "bigint".to_owned(),
        TypeIr::F32 | TypeIr::F64 => "number".to_owned(),
        TypeIr::String => "string".to_owned(),
        TypeIr::Enum(name) | TypeIr::Struct(name) | TypeIr::Union(name) => name.clone(),
        TypeIr::List(element) | TypeIr::Set(element) | TypeIr::Array { element, .. } => {
            format!("{}[]", array_element_type(ir, element))
        }
        TypeIr::Map { key, value } => format!(
            "Map<{}, {}>",
            ecmascript_type_name(ir, key),
            ecmascript_type_name(ir, value)
        ),
        TypeIr::Ref { table, field } => ref_type(ir, table, field)
            .map(|ty| ecmascript_type_name(ir, ty))
            .unwrap_or_else(|| "number".to_owned()),
        TypeIr::Optional(element) => format!("{} | undefined", ecmascript_type_name(ir, element)),
    }
}

pub fn ecmascript_decode_expr(ir: &ConfigIr, ty: &TypeIr) -> String {
    match ty {
        TypeIr::Bool => "reader.readBool()".to_owned(),
        TypeIr::I32 => "reader.readI32()".to_owned(),
        TypeIr::I64 => "reader.readI64()".to_owned(),
        TypeIr::F32 => "reader.readF32()".to_owned(),
        TypeIr::F64 => "reader.readF64()".to_owned(),
        TypeIr::String => "reader.readString()".to_owned(),
        TypeIr::Enum(name) | TypeIr::Struct(name) | TypeIr::Union(name) => {
            format!("decode{name}(reader)")
        }
        TypeIr::List(element) | TypeIr::Set(element) | TypeIr::Array { element, .. } => {
            format!(
                "reader.readList(() => {})",
                ecmascript_decode_expr(ir, element)
            )
        }
        TypeIr::Map { key, value } => format!(
            "reader.readMap(() => {}, () => {})",
            ecmascript_decode_expr(ir, key),
            ecmascript_decode_expr(ir, value)
        ),
        TypeIr::Ref { table, field } => ref_type(ir, table, field)
            .map(|ty| ecmascript_decode_expr(ir, ty))
            .unwrap_or_else(|| "reader.readI32()".to_owned()),
        TypeIr::Optional(element) => {
            format!(
                "reader.readOptional(() => {})",
                ecmascript_decode_expr(ir, element)
            )
        }
    }
}

pub fn ecmascript_value_decode_expr(ir: &ConfigIr, ty: &TypeIr, value: &str) -> String {
    match ty {
        TypeIr::Bool => format!("{value}.asBool()"),
        TypeIr::I32 => format!("{value}.asInt()"),
        TypeIr::I64 => format!("{value}.asBigInt()"),
        TypeIr::F32 | TypeIr::F64 => format!("{value}.asNumber()"),
        TypeIr::String => format!("{value}.asString()"),
        TypeIr::Enum(name) | TypeIr::Struct(name) | TypeIr::Union(name) => {
            format!("decode{name}Value({value})")
        }
        TypeIr::List(element) | TypeIr::Set(element) | TypeIr::Array { element, .. } => {
            let item_decode = ecmascript_value_decode_expr(ir, element, "item");
            format!("{value}.asList((item) => {item_decode})")
        }
        TypeIr::Map {
            key,
            value: element,
        } => {
            let key_decode = ecmascript_value_decode_expr(ir, key, "item");
            let value_decode = ecmascript_value_decode_expr(ir, element, "item");
            format!("{value}.asMap((item) => {key_decode}, (item) => {value_decode})")
        }
        TypeIr::Ref { table, field } => ref_type(ir, table, field)
            .map(|ty| ecmascript_value_decode_expr(ir, ty, value))
            .unwrap_or_else(|| format!("{value}.asInt()")),
        TypeIr::Optional(element) => {
            let item_decode = ecmascript_value_decode_expr(ir, element, value);
            format!("{value}.isNull() ? undefined : {item_decode}")
        }
    }
}

fn array_element_type(ir: &ConfigIr, ty: &TypeIr) -> String {
    let name = ecmascript_type_name(ir, ty);
    if name.contains(" | ") {
        format!("({name})")
    } else {
        name
    }
}

fn ref_type<'a>(ir: &'a ConfigIr, table_name: &str, field_name: &str) -> Option<&'a TypeIr> {
    ir.tables
        .iter()
        .find(|table| table.name == table_name)
        .and_then(|table| table.fields.iter().find(|field| field.name == field_name))
        .map(|field| &field.ty)
}
