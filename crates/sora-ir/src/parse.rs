use sora_diagnostics::{Result, SoraError};

use crate::model::TypeIr;

pub fn parse_type(input: &str) -> Result<TypeIr> {
    parse_type_inner(input.trim())
}

fn parse_type_inner(input: &str) -> Result<TypeIr> {
    if input.is_empty() {
        return Err(SoraError::InvalidType(input.to_owned()));
    }

    Ok(match input {
        "bool" => TypeIr::Bool,
        "i32" => TypeIr::I32,
        "i64" => TypeIr::I64,
        "f32" => TypeIr::F32,
        "f64" => TypeIr::F64,
        "string" => TypeIr::String,
        "text" => TypeIr::Text,
        _ => {
            if let Some(inner) = generic_inner(input, "enum") {
                require_identifier(inner)?;
                TypeIr::Enum(inner.to_owned())
            } else if let Some(inner) = generic_inner(input, "struct") {
                require_identifier(inner)?;
                TypeIr::Struct(inner.to_owned())
            } else if let Some(inner) = generic_inner(input, "union") {
                require_identifier(inner)?;
                TypeIr::Union(inner.to_owned())
            } else if let Some(inner) = generic_inner(input, "list") {
                TypeIr::List(Box::new(parse_nested_type(inner)?))
            } else if let Some(inner) = generic_inner(input, "set") {
                TypeIr::Set(Box::new(parse_nested_type(inner)?))
            } else if let Some(inner) = generic_inner(input, "map") {
                parse_map_type(input, inner)?
            } else if let Some(inner) = generic_inner(input, "optional") {
                TypeIr::Optional(Box::new(parse_nested_type(inner)?))
            } else if let Some(inner) = generic_inner(input, "array") {
                parse_array_type(input, inner)?
            } else if let Some(inner) = generic_inner(input, "ref") {
                parse_ref_type(input, inner)?
            } else if is_identifier(input) {
                TypeIr::Struct(input.to_owned())
            } else {
                return Err(SoraError::UnknownType(input.to_owned()));
            }
        }
    })
}

fn parse_nested_type(input: &str) -> Result<TypeIr> {
    parse_type_inner(input.trim())
}

fn generic_inner<'a>(input: &'a str, name: &str) -> Option<&'a str> {
    let prefix = format!("{name}<");
    input
        .strip_prefix(&prefix)
        .and_then(|rest| rest.strip_suffix('>'))
}

fn parse_array_type(original: &str, inner: &str) -> Result<TypeIr> {
    let parts = split_top_level(inner, ',');
    let [element, len] = parts.as_slice() else {
        return Err(SoraError::InvalidType(original.to_owned()));
    };
    let len = len
        .trim()
        .parse::<usize>()
        .map_err(|_| SoraError::InvalidType(original.to_owned()))?;

    Ok(TypeIr::Array {
        element: Box::new(parse_nested_type(element)?),
        len,
    })
}

fn parse_map_type(original: &str, inner: &str) -> Result<TypeIr> {
    let parts = split_top_level(inner, ',');
    let [key, value] = parts.as_slice() else {
        return Err(SoraError::InvalidType(original.to_owned()));
    };

    Ok(TypeIr::Map {
        key: Box::new(parse_nested_type(key)?),
        value: Box::new(parse_nested_type(value)?),
    })
}

fn parse_ref_type(original: &str, inner: &str) -> Result<TypeIr> {
    let (table, field) = inner
        .split_once('.')
        .ok_or_else(|| SoraError::InvalidType(original.to_owned()))?;
    require_identifier(table)?;
    require_identifier(field)?;

    Ok(TypeIr::Ref {
        table: table.to_owned(),
        field: field.to_owned(),
    })
}

fn split_top_level(input: &str, separator: char) -> Vec<&str> {
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

fn require_identifier(input: &str) -> Result<()> {
    if is_identifier(input) {
        Ok(())
    } else {
        Err(SoraError::InvalidType(input.to_owned()))
    }
}

fn is_identifier(input: &str) -> bool {
    let mut chars = input.chars();
    matches!(chars.next(), Some(first) if first == '_' || first.is_ascii_alphabetic())
        && chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_required_type_forms() {
        let cases = [
            ("bool", TypeIr::Bool),
            ("i32", TypeIr::I32),
            ("i64", TypeIr::I64),
            ("f32", TypeIr::F32),
            ("f64", TypeIr::F64),
            ("string", TypeIr::String),
            ("text", TypeIr::Text),
            ("enum<ItemType>", TypeIr::Enum("ItemType".to_owned())),
            ("struct<Reward>", TypeIr::Struct("Reward".to_owned())),
            ("union<Action>", TypeIr::Union("Action".to_owned())),
            ("list<i32>", TypeIr::List(Box::new(TypeIr::I32))),
            ("set<i32>", TypeIr::Set(Box::new(TypeIr::I32))),
            (
                "map<string,i32>",
                TypeIr::Map {
                    key: Box::new(TypeIr::String),
                    value: Box::new(TypeIr::I32),
                },
            ),
            (
                "list<Reward>",
                TypeIr::List(Box::new(TypeIr::Struct("Reward".to_owned()))),
            ),
            (
                "array<i32,3>",
                TypeIr::Array {
                    element: Box::new(TypeIr::I32),
                    len: 3,
                },
            ),
            (
                "ref<Item.id>",
                TypeIr::Ref {
                    table: "Item".to_owned(),
                    field: "id".to_owned(),
                },
            ),
            (
                "optional<string>",
                TypeIr::Optional(Box::new(TypeIr::String)),
            ),
        ];

        for (source, expected) in cases {
            assert_eq!(parse_type(source).unwrap(), expected);
        }
    }

    #[test]
    fn rejects_malformed_types() {
        for source in [
            "",
            "array<i32>",
            "array<i32,nope>",
            "ref<Item>",
            "enum<1Bad>",
        ] {
            assert!(parse_type(source).is_err(), "{source} should fail");
        }
    }
}
