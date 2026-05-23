use sora_diagnostics::{Result, SoraError};
use sora_schema::model::{
    EnumAliasSchema, FieldSchema, IndexSchema, ParserSchema, SchemaFile, ScopeSchema,
    TableFieldSchema, TableModeSchema, TableSchema, TableSourceSchema, UnionSchema,
    UnionVariantSchema,
};

use crate::{
    input_projection::TAGGED_COLUMNS_PARSER,
    model::{
        AggregationIr, ConfigIr, EnumAliasIr, EnumIr, FieldIr, IndexIr, ParserIr, ScopeIr,
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
        tables: schema
            .tables
            .into_iter()
            .map(|table| convert_table_with_parsers(table, parser_registry))
            .collect::<Result<Vec<_>>>()?,
    })
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
        Ok(Self {
            name: table.name,
            scope: ScopeIr::try_from(table.scope)?,
            mode: table.mode.into(),
            key: table.key,
            source: table.source.map(Into::into),
            fields: convert_table_fields(table.fields)?,
            indexes: table.indexes.into_iter().map(IndexIr::from).collect(),
        })
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
            .is_some_and(|parser| parser.kind == TAGGED_COLUMNS_PARSER)
    {
        return Err(SoraError::InvalidSchema(format!(
            "field `{}` declares both `default` and parser `tagged_columns`",
            field.name
        )));
    }
    parser_registry.validate_field_parser(&field.name, &ty, field.parser.as_ref())?;

    Ok(FieldIr {
        name: field.name,
        ty,
        scope: ScopeIr::try_from(field.scope)?,
        key: false,
        comment: field.comment,
        required: field.required.unwrap_or(false),
        default: field.default,
        range: field.range,
        length: field.length,
        parser: field.parser.map(ParserIr::from),
        aggregation: None,
    })
}

fn convert_table_fields(fields: Vec<TableFieldSchema>) -> Result<Vec<FieldIr>> {
    convert_table_fields_with_parsers(fields, &ParserRegistry::builtin())
}

fn convert_table_fields_with_parsers(
    fields: Vec<TableFieldSchema>,
    parser_registry: &ParserRegistry,
) -> Result<Vec<FieldIr>> {
    fields
        .into_iter()
        .map(|field| convert_table_field_with_parsers(field, parser_registry))
        .collect()
}

fn convert_table_field_with_parsers(
    field: TableFieldSchema,
    parser_registry: &ParserRegistry,
) -> Result<FieldIr> {
    let value_field = field.value_field;
    let order_by = field.order_by;
    let aggregation = match (field.source_table, field.parent_key, field.child_key) {
        (None, None, None) if value_field.is_none() && order_by.is_none() => None,
        (Some(source_table), Some(parent_key), Some(child_key)) => Some(AggregationIr {
            source_table,
            parent_key,
            child_key,
            value_field,
            order_by,
        }),
        _ => {
            return Err(SoraError::InvalidSchema(format!(
                "field `{}` has incomplete aggregation metadata",
                field.name
            )));
        }
    };

    let ty = parse_type(&field.ty)?;
    validate_length_constraint(&field.name, &ty, field.length)?;
    if field.default.is_some() && aggregation.is_some() {
        return Err(SoraError::InvalidSchema(format!(
            "field `{}` declares both `default` and aggregation metadata",
            field.name
        )));
    }
    if field.default.is_some()
        && field
            .parser
            .as_ref()
            .is_some_and(|parser| parser.kind == TAGGED_COLUMNS_PARSER)
    {
        return Err(SoraError::InvalidSchema(format!(
            "field `{}` declares both `default` and parser `tagged_columns`",
            field.name
        )));
    }
    parser_registry.validate_field_parser(&field.name, &ty, field.parser.as_ref())?;

    Ok(FieldIr {
        name: field.name,
        ty,
        scope: ScopeIr::try_from(field.scope)?,
        key: field.key,
        comment: field.comment,
        required: field.required.unwrap_or(false),
        default: field.default,
        range: field.range,
        length: field.length,
        parser: field.parser.map(ParserIr::from),
        aggregation,
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
    Ok(TableIr {
        name: table.name,
        scope: ScopeIr::try_from(table.scope)?,
        mode: table.mode.into(),
        key: table.key,
        source: table.source.map(Into::into),
        fields: convert_table_fields_with_parsers(table.fields, parser_registry)?,
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

#[cfg(test)]
mod tests;
