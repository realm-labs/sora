use std::collections::BTreeMap;

use sora_diagnostics::{Result, SoraError};
use sora_schema::model::ParserSchema;

use crate::model::TypeIr;

pub trait ParserValidator: Send + Sync {
    fn kind(&self) -> &'static str;

    fn validate(&self, field_name: &str, ty: &TypeIr, parser: &ParserSchema) -> Result<()>;
}

pub struct ParserRegistry {
    validators: BTreeMap<String, Box<dyn ParserValidator>>,
}

impl ParserRegistry {
    pub fn new() -> Self {
        Self {
            validators: BTreeMap::new(),
        }
    }

    pub fn builtin() -> Self {
        let mut registry = Self::new();
        registry.register(SplitParserValidator);
        registry.register(TupleParserValidator);
        registry.register(TupleListParserValidator);
        registry.register(JsonParserValidator);
        registry
    }

    pub fn register(&mut self, validator: impl ParserValidator + 'static) {
        self.validators
            .insert(validator.kind().to_owned(), Box::new(validator));
    }

    pub fn validate_field_parser(
        &self,
        field_name: &str,
        ty: &TypeIr,
        parser: Option<&ParserSchema>,
    ) -> Result<()> {
        let Some(parser) = parser else {
            return Ok(());
        };
        validate_required_non_empty(field_name, "parser.kind", Some(&parser.kind))?;
        let Some(validator) = self.validators.get(&parser.kind) else {
            return Err(SoraError::InvalidSchema(format!(
                "field `{field_name}` declares unsupported parser `{}`",
                parser.kind
            )));
        };
        validator.validate(field_name, ty, parser)
    }
}

impl Default for ParserRegistry {
    fn default() -> Self {
        Self::builtin()
    }
}

struct SplitParserValidator;

impl ParserValidator for SplitParserValidator {
    fn kind(&self) -> &'static str {
        "split"
    }

    fn validate(&self, field_name: &str, ty: &TypeIr, parser: &ParserSchema) -> Result<()> {
        validate_collection_target(field_name, ty)?;
        validate_parser_options(field_name, &parser.kind, &parser.options, &["separator"])
    }
}

struct TupleParserValidator;

impl ParserValidator for TupleParserValidator {
    fn kind(&self) -> &'static str {
        "tuple"
    }

    fn validate(&self, field_name: &str, ty: &TypeIr, parser: &ParserSchema) -> Result<()> {
        validate_tuple_target(field_name, ty)?;
        validate_parser_options(field_name, &parser.kind, &parser.options, &["separator"])
    }
}

struct TupleListParserValidator;

impl ParserValidator for TupleListParserValidator {
    fn kind(&self) -> &'static str {
        "tuple_list"
    }

    fn validate(&self, field_name: &str, ty: &TypeIr, parser: &ParserSchema) -> Result<()> {
        validate_tuple_list_target(field_name, ty)?;
        validate_parser_options(
            field_name,
            &parser.kind,
            &parser.options,
            &["separator", "item_separator"],
        )
    }
}

struct JsonParserValidator;

impl ParserValidator for JsonParserValidator {
    fn kind(&self) -> &'static str {
        "json"
    }

    fn validate(&self, field_name: &str, _ty: &TypeIr, parser: &ParserSchema) -> Result<()> {
        validate_parser_options(field_name, &parser.kind, &parser.options, &[])
    }
}

fn validate_required_non_empty(
    field_name: &str,
    property: &str,
    value: Option<&str>,
) -> Result<()> {
    match value {
        Some(value) if !value.trim().is_empty() => Ok(()),
        _ => Err(SoraError::InvalidSchema(format!(
            "field `{field_name}` declares empty `{property}`"
        ))),
    }
}

fn validate_tuple_target(field_name: &str, ty: &TypeIr) -> Result<()> {
    match ty {
        TypeIr::Struct(_) => Ok(()),
        TypeIr::Optional(inner) => validate_tuple_target(field_name, inner),
        _ => Err(SoraError::InvalidSchema(format!(
            "field `{field_name}` declares parser `tuple` but type `{ty}` is not struct"
        ))),
    }
}

fn validate_tuple_list_target(field_name: &str, ty: &TypeIr) -> Result<()> {
    match ty {
        TypeIr::List(element) | TypeIr::Array { element, .. } => validate_tuple_target(
            field_name,
            element,
        )
        .map_err(|_| {
            SoraError::InvalidSchema(format!(
                "field `{field_name}` declares parser `tuple_list` but type `{ty}` is not list or array of struct"
            ))
        }),
        TypeIr::Optional(inner) => validate_tuple_list_target(field_name, inner),
        _ => Err(SoraError::InvalidSchema(format!(
            "field `{field_name}` declares parser `tuple_list` but type `{ty}` is not list or array of struct"
        ))),
    }
}

fn validate_collection_target(field_name: &str, ty: &TypeIr) -> Result<()> {
    match ty {
        TypeIr::List(_) | TypeIr::Array { .. } => Ok(()),
        TypeIr::Optional(inner) => validate_collection_target(field_name, inner),
        _ => Err(SoraError::InvalidSchema(format!(
            "field `{field_name}` declares parser `split` but type `{ty}` is not list or array"
        ))),
    }
}

fn validate_parser_options(
    field_name: &str,
    parser: &str,
    options: &BTreeMap<String, String>,
    allowed: &[&str],
) -> Result<()> {
    for (key, value) in options {
        if !allowed.contains(&key.as_str()) {
            return Err(SoraError::InvalidSchema(format!(
                "field `{field_name}` declares unsupported option `{key}` for parser `{parser}`"
            )));
        }
        validate_required_non_empty(field_name, &format!("parser.{key}"), Some(value))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DslParserValidator;

    impl ParserValidator for DslParserValidator {
        fn kind(&self) -> &'static str {
            "dsl"
        }

        fn validate(&self, _field_name: &str, _ty: &TypeIr, _parser: &ParserSchema) -> Result<()> {
            Ok(())
        }
    }

    #[test]
    fn registry_accepts_custom_parser_validators() {
        let mut registry = ParserRegistry::builtin();
        registry.register(DslParserValidator);
        let parser = ParserSchema {
            kind: "dsl".to_owned(),
            options: BTreeMap::new(),
        };

        registry
            .validate_field_parser("condition", &TypeIr::String, Some(&parser))
            .unwrap();
    }
}
