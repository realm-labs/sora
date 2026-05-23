use std::collections::BTreeSet;

use crate::model::{ConfigIr, FieldIr, ParserIr, TypeIr, UnionIr};

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

pub fn is_tagged_columns_parser(parser: Option<&ParserIr>) -> bool {
    parser.is_some_and(|parser| parser.kind.as_str() == TAGGED_COLUMNS_PARSER)
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
