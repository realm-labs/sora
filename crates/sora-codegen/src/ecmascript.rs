use heck::ToLowerCamelCase;
use serde::Serialize;
use sora_ir::model::{ConfigIr, EnumReprIr, TableModeIr, TypeIr};

use crate::model::{LanguageBackend, TableNameParts};

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

#[derive(Debug, Clone)]
pub struct EcmaScriptBackend;

impl LanguageBackend for EcmaScriptBackend {
    fn field_name(&self, raw_name: &str) -> String {
        raw_name.to_lower_camel_case()
    }

    fn type_name(&self, ir: &ConfigIr, ty: &TypeIr) -> String {
        ecmascript_type_name(ir, ty)
    }

    fn decode_expr(&self, ir: &ConfigIr, ty: &TypeIr) -> String {
        ecmascript_decode_expr(ir, ty)
    }

    fn row_type(&self, table: &TableNameParts<'_>) -> String {
        table.pascal_name.to_owned()
    }

    fn container_type(
        &self,
        _table: &TableNameParts<'_>,
        mode: TableModeIr,
        row_type: &str,
        key_type: Option<&str>,
    ) -> String {
        match mode {
            TableModeIr::List => format!("readonly {row_type}[]"),
            TableModeIr::Map => match key_type {
                Some(key_type) => format!("ReadonlyMap<{key_type}, {row_type}>"),
                None => format!("readonly {row_type}[]"),
            },
            TableModeIr::Singleton => row_type.to_owned(),
        }
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
        TypeIr::List(element) | TypeIr::Array { element, .. } => {
            format!("{}[]", array_element_type(ir, element))
        }
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
        TypeIr::List(element) | TypeIr::Array { element, .. } => {
            format!(
                "reader.readList(() => {})",
                ecmascript_decode_expr(ir, element)
            )
        }
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
