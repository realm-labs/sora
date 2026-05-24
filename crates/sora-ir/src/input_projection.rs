use std::collections::BTreeSet;

use crate::model::{ConfigIr, FieldIr, ParserIr, StructIr, TypeIr, UnionIr};

pub const COLUMNS_PARSER: &str = "columns";
pub const TAGGED_COLUMNS_PARSER: &str = "tagged_columns";

#[derive(Debug, Clone, Copy)]
pub enum TaggedColumnKind<'a> {
    Tag,
    VariantField(&'a FieldIr),
}

#[derive(Debug, Clone)]
pub struct TaggedColumn<'a> {
    pub name: String,
    pub kind: TaggedColumnKind<'a>,
}

#[derive(Debug, Clone)]
pub struct StructColumn<'a> {
    pub name: String,
    pub field: &'a FieldIr,
}

pub fn is_columns_parser(parser: Option<&ParserIr>) -> bool {
    parser.is_some_and(|parser| parser.kind.as_str() == COLUMNS_PARSER)
}

pub fn is_tagged_columns_parser(parser: Option<&ParserIr>) -> bool {
    parser.is_some_and(|parser| parser.kind.as_str() == TAGGED_COLUMNS_PARSER)
}

pub fn columns_prefix(field: &FieldIr) -> Option<String> {
    let parser = field.parser.as_ref()?;
    if parser.kind != COLUMNS_PARSER {
        return None;
    }
    Some(
        parser
            .options
            .get("prefix")
            .cloned()
            .unwrap_or_else(|| format!("{}.", field.name)),
    )
}

pub fn tagged_columns_prefix(field: &FieldIr) -> Option<String> {
    let parser = field.parser.as_ref()?;
    if parser.kind != TAGGED_COLUMNS_PARSER {
        return None;
    }
    Some(
        parser
            .options
            .get("prefix")
            .cloned()
            .unwrap_or_else(|| format!("{}.", field.name)),
    )
}

pub fn tagged_columns_union<'a>(ir: &'a ConfigIr, field: &FieldIr) -> Option<&'a UnionIr> {
    if !is_tagged_columns_parser(field.parser.as_ref()) {
        return None;
    }
    let TypeIr::Union(union_name) = &field.ty else {
        return None;
    };
    ir.unions.iter().find(|item| item.name == *union_name)
}

pub fn columns_struct<'a>(ir: &'a ConfigIr, field: &FieldIr) -> Option<&'a StructIr> {
    if !is_columns_parser(field.parser.as_ref()) {
        return None;
    }
    let struct_name = struct_type_name(&field.ty)?;
    ir.structs.iter().find(|item| item.name == struct_name)
}

pub fn struct_columns<'a>(ir: &'a ConfigIr, field: &FieldIr) -> Option<Vec<StructColumn<'a>>> {
    let struct_ir = columns_struct(ir, field)?;
    let prefix = columns_prefix(field)?;
    Some(
        struct_ir
            .fields
            .iter()
            .map(|struct_field| StructColumn {
                name: format!("{prefix}{}", struct_field.name),
                field: struct_field,
            })
            .collect(),
    )
}

pub fn tagged_columns<'a>(ir: &'a ConfigIr, field: &FieldIr) -> Option<Vec<TaggedColumn<'a>>> {
    let union = tagged_columns_union(ir, field)?;
    let prefix = tagged_columns_prefix(field)?;
    let mut columns = vec![TaggedColumn {
        name: format!("{prefix}{}", union.tag),
        kind: TaggedColumnKind::Tag,
    }];

    let mut seen = BTreeSet::new();
    for variant in &union.variants {
        for variant_field in &variant.fields {
            let name = format!("{prefix}{}", variant_field.name);
            if seen.insert(name.clone()) {
                columns.push(TaggedColumn {
                    name,
                    kind: TaggedColumnKind::VariantField(variant_field),
                });
            }
        }
    }

    Some(columns)
}

fn struct_type_name(ty: &TypeIr) -> Option<&str> {
    match ty {
        TypeIr::Struct(name) => Some(name),
        TypeIr::Optional(inner) => struct_type_name(inner),
        _ => None,
    }
}
