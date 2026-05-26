use sora_diagnostics::{Result, SoraError};
use sora_schema::model::{
    EnumAliasSchema, FieldSchema, IndexSchema, LocalizationSchema, ParserSchema, SchemaFile,
    ScopeSchema, TableFieldSchema, TableModeSchema, TableSchema, TableSourceSchema, UnionSchema,
    UnionVariantSchema,
};

use crate::{
    input_projection::{COLUMNS_PARSER, TAGGED_COLUMNS_PARSER},
    model::{
        ConfigIr, DerivedFieldIr, EnumAliasIr, EnumIr, FieldIr, IndexIr, ParserIr, ScopeIr,
        StructIr, TableIr, TableModeIr, TableSourceIr, TypeIr, UnionIr, UnionVariantIr,
    },
    parse::parse_type,
    parser::ParserRegistry,
};

pub fn normalize_schema(schema: SchemaFile) -> Result<ConfigIr> {
    normalize_schema_with_parsers(schema, &ParserRegistry::builtin())
}

pub fn normalize_schema_with_parsers(
    schema: SchemaFile,
    parser_registry: &ParserRegistry,
) -> Result<ConfigIr> {
    Ok(ConfigIr {
        package: schema.package,
        enums: schema
            .enums
            .into_iter()
            .map(|item| {
                Ok(EnumIr {
                    name: item.name,
                    scope: ScopeIr::try_from(item.scope)?,
                    values: item.values,
                    aliases: convert_enum_aliases(item.aliases),
                })
            })
            .collect::<Result<Vec<_>>>()?,
        structs: schema
            .structs
            .into_iter()
            .map(|item| {
                Ok(StructIr {
                    name: item.name,
                    scope: ScopeIr::try_from(item.scope)?,
                    fields: convert_fields_with_parsers(item.fields, parser_registry)?,
                })
            })
            .collect::<Result<Vec<_>>>()?,
        unions: schema
            .unions
            .into_iter()
            .map(|union| convert_union_with_parsers(union, parser_registry))
            .collect::<Result<Vec<_>>>()?,
        localization: convert_localization(schema.localization.as_ref())?,
        tables: schema
            .tables
            .into_iter()
            .map(|table| convert_table_with_parsers(table, parser_registry))
            .collect::<Result<Vec<_>>>()?,
    })
}

fn convert_localization(
    localization: Option<&LocalizationSchema>,
) -> Result<Option<crate::model::LocalizationIr>> {
    let Some(localization) = localization else {
        return Ok(None);
    };
    if localization.locales.is_empty() {
        return Err(SoraError::InvalidSchema(
            "localization.locales must contain at least one locale".to_owned(),
        ));
    }
    for locale in &localization.locales {
        validate_locale(locale)?;
    }
    let default_locale = localization
        .default_locale
        .clone()
        .unwrap_or_else(|| localization.locales[0].clone());
    if !localization.locales.contains(&default_locale) {
        return Err(SoraError::InvalidSchema(format!(
            "localization.default_locale `{default_locale}` is not listed in localization.locales"
        )));
    }
    if let Some(fallback) = &localization.fallback_locale
        && !localization.locales.contains(fallback)
    {
        return Err(SoraError::InvalidSchema(format!(
            "localization.fallback_locale `{fallback}` is not listed in localization.locales"
        )));
    }
    if localization.sources.is_empty() {
        return Err(SoraError::InvalidSchema(
            "localization.sources must contain at least one source".to_owned(),
        ));
    }
    let mut sources = Vec::with_capacity(localization.sources.len());
    for source in &localization.sources {
        validate_identifier("localization source", &source.name)?;
        validate_identifier("localization key field", &source.key)?;
        sources.push(crate::model::LocalizationSourceIr {
            name: source.name.clone(),
            format: source.format.clone(),
            file: source.file.clone(),
            sheet: source.sheet.clone(),
            key: source.key.clone(),
        });
    }
    Ok(Some(crate::model::LocalizationIr {
        locales: localization.locales.clone(),
        default_locale,
        fallback_locale: localization.fallback_locale.clone(),
        strict: localization.strict,
        sources,
    }))
}

impl TryFrom<SchemaFile> for ConfigIr {
    type Error = SoraError;

    fn try_from(schema: SchemaFile) -> Result<Self> {
        normalize_schema(schema)
    }
}

impl TryFrom<UnionSchema> for UnionIr {
    type Error = SoraError;

    fn try_from(union: UnionSchema) -> Result<Self> {
        if union.tag.is_empty() {
            return Err(SoraError::InvalidSchema(format!(
                "union `{}` declares empty `tag`",
                union.name
            )));
        }

        Ok(Self {
            name: union.name,
            scope: ScopeIr::try_from(union.scope)?,
            tag: union.tag,
            variants: union
                .variants
                .into_iter()
                .map(UnionVariantIr::try_from)
                .collect::<Result<Vec<_>>>()?,
        })
    }
}

impl TryFrom<UnionVariantSchema> for UnionVariantIr {
    type Error = SoraError;

    fn try_from(variant: UnionVariantSchema) -> Result<Self> {
        Ok(Self {
            name: variant.name,
            scope: ScopeIr::try_from(variant.scope)?,
            fields: convert_fields(variant.fields)?,
        })
    }
}

impl TryFrom<TableSchema> for TableIr {
    type Error = SoraError;

    fn try_from(table: TableSchema) -> Result<Self> {
        convert_table_with_parsers(table, &ParserRegistry::builtin())
    }
}

impl TryFrom<FieldSchema> for FieldIr {
    type Error = SoraError;

    fn try_from(field: FieldSchema) -> Result<Self> {
        convert_field_with_parsers(field, &ParserRegistry::builtin())
    }
}

impl From<ParserSchema> for ParserIr {
    fn from(parser: ParserSchema) -> Self {
        Self {
            kind: parser.kind,
            options: parser.options,
        }
    }
}

impl TryFrom<ScopeSchema> for ScopeIr {
    type Error = SoraError;

    fn try_from(scope: ScopeSchema) -> Result<Self> {
        if scope.values.is_empty() {
            return Err(SoraError::InvalidSchema(
                "scope must contain at least one value".to_owned(),
            ));
        }

        let mut values = Vec::with_capacity(scope.values.len());
        for value in scope.values {
            let value = value.trim();
            if value.is_empty() {
                return Err(SoraError::InvalidSchema(
                    "scope values must not be empty".to_owned(),
                ));
            }
            if !value
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
            {
                return Err(SoraError::InvalidSchema(format!(
                    "scope `{value}` must contain only ASCII letters, digits, `_`, or `-`"
                )));
            }
            if !values.iter().any(|item| item == value) {
                values.push(value.to_owned());
            }
        }

        Ok(Self { values })
    }
}

impl From<TableModeSchema> for TableModeIr {
    fn from(mode: TableModeSchema) -> Self {
        match mode {
            TableModeSchema::List => Self::List,
            TableModeSchema::Map => Self::Map,
            TableModeSchema::Singleton => Self::Singleton,
        }
    }
}

impl From<IndexSchema> for IndexIr {
    fn from(index: IndexSchema) -> Self {
        Self {
            name: index.name,
            fields: index.fields,
            unique: index.unique,
        }
    }
}

impl From<TableSourceSchema> for TableSourceIr {
    fn from(source: TableSourceSchema) -> Self {
        Self {
            format: source.format,
            file: source.file,
            sheet: source.sheet,
        }
    }
}

fn convert_fields(fields: Vec<FieldSchema>) -> Result<Vec<FieldIr>> {
    convert_fields_with_parsers(fields, &ParserRegistry::builtin())
}

fn convert_enum_aliases(aliases: Vec<EnumAliasSchema>) -> Vec<EnumAliasIr> {
    aliases
        .into_iter()
        .map(|item| EnumAliasIr {
            name: item.name,
            alias: item.alias,
        })
        .collect()
}

fn convert_fields_with_parsers(
    fields: Vec<FieldSchema>,
    parser_registry: &ParserRegistry,
) -> Result<Vec<FieldIr>> {
    fields
        .into_iter()
        .map(|field| convert_field_with_parsers(field, parser_registry))
        .collect()
}

fn convert_field_with_parsers(
    field: FieldSchema,
    parser_registry: &ParserRegistry,
) -> Result<FieldIr> {
    let ty = parse_type(&field.ty)?;
    validate_length_constraint(&field.name, &ty, field.length)?;
    if field.default.is_some()
        && field
            .parser
            .as_ref()
            .is_some_and(|parser| is_projection_parser(&parser.kind))
    {
        return Err(SoraError::InvalidSchema(format!(
            "field `{}` declares both `default` and parser `{}`",
            field.name,
            field
                .parser
                .as_ref()
                .map(|parser| parser.kind.as_str())
                .unwrap_or("")
        )));
    }
    parser_registry.validate_field_parser(&field.name, &ty, field.parser.as_ref())?;

    Ok(FieldIr {
        name: field.name,
        ty,
        scope: ScopeIr::try_from(field.scope)?,
        key: false,
        comment: field.comment,
        default: field.default,
        range: field.range,
        length: field.length,
        parser: field.parser.map(ParserIr::from),
        derived_from: None,
    })
}

fn convert_table_fields_with_parsers(
    fields: Vec<TableFieldSchema>,
    table_key: Option<&str>,
    parser_registry: &ParserRegistry,
) -> Result<Vec<FieldIr>> {
    fields
        .into_iter()
        .map(|field| {
            let is_key = table_key == Some(field.name.as_str());
            convert_table_field_with_parsers(field, is_key, parser_registry)
        })
        .collect()
}

fn convert_table_field_with_parsers(
    field: TableFieldSchema,
    is_key: bool,
    parser_registry: &ParserRegistry,
) -> Result<FieldIr> {
    let derived_from = field
        .from
        .map(|from| {
            let Some(parent_key) = from.parent_key else {
                return Err(SoraError::InvalidSchema(format!(
                    "field `{}` has incomplete `from` metadata: missing `parent_key`",
                    field.name
                )));
            };
            let Some(child_key) = from.child_key else {
                return Err(SoraError::InvalidSchema(format!(
                    "field `{}` has incomplete `from` metadata: missing `child_key`",
                    field.name
                )));
            };
            Ok(DerivedFieldIr {
                source_table: from.table,
                parent_key,
                child_key,
                value_field: from.value_field,
                order_by: from.order_by,
            })
        })
        .transpose()?;

    let ty = parse_type(&field.ty)?;
    validate_length_constraint(&field.name, &ty, field.length)?;
    if field.default.is_some() && derived_from.is_some() {
        return Err(SoraError::InvalidSchema(format!(
            "field `{}` declares both `default` and `from` metadata",
            field.name
        )));
    }
    if field.default.is_some()
        && field
            .parser
            .as_ref()
            .is_some_and(|parser| is_projection_parser(&parser.kind))
    {
        return Err(SoraError::InvalidSchema(format!(
            "field `{}` declares both `default` and parser `{}`",
            field.name,
            field
                .parser
                .as_ref()
                .map(|parser| parser.kind.as_str())
                .unwrap_or("")
        )));
    }
    parser_registry.validate_field_parser(&field.name, &ty, field.parser.as_ref())?;

    Ok(FieldIr {
        name: field.name,
        ty,
        scope: ScopeIr::try_from(field.scope)?,
        key: is_key,
        comment: field.comment,
        default: field.default,
        range: field.range,
        length: field.length,
        parser: field.parser.map(ParserIr::from),
        derived_from,
    })
}

fn convert_union_with_parsers(
    union: UnionSchema,
    parser_registry: &ParserRegistry,
) -> Result<UnionIr> {
    if union.tag.is_empty() {
        return Err(SoraError::InvalidSchema(format!(
            "union `{}` declares empty `tag`",
            union.name
        )));
    }

    Ok(UnionIr {
        name: union.name,
        scope: ScopeIr::try_from(union.scope)?,
        tag: union.tag,
        variants: union
            .variants
            .into_iter()
            .map(|variant| convert_union_variant_with_parsers(variant, parser_registry))
            .collect::<Result<Vec<_>>>()?,
    })
}

fn convert_union_variant_with_parsers(
    variant: UnionVariantSchema,
    parser_registry: &ParserRegistry,
) -> Result<UnionVariantIr> {
    Ok(UnionVariantIr {
        name: variant.name,
        scope: ScopeIr::try_from(variant.scope)?,
        fields: convert_fields_with_parsers(variant.fields, parser_registry)?,
    })
}

fn convert_table_with_parsers(
    table: TableSchema,
    parser_registry: &ParserRegistry,
) -> Result<TableIr> {
    let key = table.key;
    let fields = convert_table_fields_with_parsers(table.fields, key.as_deref(), parser_registry)?;
    Ok(TableIr {
        name: table.name,
        scope: ScopeIr::try_from(table.scope)?,
        mode: table.mode.into(),
        key,
        source: table.source.map(Into::into),
        fields,
        indexes: table.indexes.into_iter().map(IndexIr::from).collect(),
    })
}

fn validate_length_constraint(
    field_name: &str,
    ty: &TypeIr,
    length: Option<[usize; 2]>,
) -> Result<()> {
    let Some([min, max]) = length else {
        return Ok(());
    };
    if min > max {
        return Err(SoraError::InvalidSchema(format!(
            "field `{field_name}` declares invalid `length` [{min}, {max}]"
        )));
    }

    match ty {
        TypeIr::String
        | TypeIr::Text
        | TypeIr::List(_)
        | TypeIr::Set(_)
        | TypeIr::Map { .. }
        | TypeIr::Array { .. } => Ok(()),
        TypeIr::Optional(inner) => validate_length_constraint(field_name, inner, length),
        _ => Err(SoraError::InvalidSchema(format!(
            "field `{field_name}` declares `length` but type `{ty}` is not string, list, or array"
        ))),
    }
}

fn validate_identifier(kind: &str, value: &str) -> Result<()> {
    let mut chars = value.chars();
    if !matches!(chars.next(), Some(first) if first == '_' || first.is_ascii_alphabetic()) {
        return Err(SoraError::InvalidSchema(format!(
            "{kind} `{value}` must start with an ASCII letter or `_`"
        )));
    }
    if !chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric()) {
        return Err(SoraError::InvalidSchema(format!(
            "{kind} `{value}` must contain only ASCII letters, digits, or `_`"
        )));
    }
    Ok(())
}

fn validate_locale(locale: &str) -> Result<()> {
    if locale.is_empty() {
        return Err(SoraError::InvalidSchema(
            "localization locale must not be empty".to_owned(),
        ));
    }
    if !locale
        .chars()
        .all(|ch| ch == '_' || ch == '-' || ch.is_ascii_alphanumeric())
    {
        return Err(SoraError::InvalidSchema(format!(
            "localization locale `{locale}` must contain only ASCII letters, digits, `_`, or `-`"
        )));
    }
    Ok(())
}

fn is_projection_parser(parser: &str) -> bool {
    matches!(parser, TAGGED_COLUMNS_PARSER | COLUMNS_PARSER)
}

#[cfg(test)]
mod tests;
