use sora_diagnostics::{Result, SoraError};
use sora_schema::model::{
    EnumAliasSchema, FieldSchema, GroupSetSchema, IndexSchema, LocalizationSchema, ParserSchema,
    ProjectSchema, TableFieldSchema, TableModeSchema, TableSchema, TableSourceSchema, UnionSchema,
    UnionVariantSchema,
};

use crate::{
    input_projection::{COLUMNS_PARSER, TAGGED_COLUMNS_PARSER},
    model::{
        ConfigIr, DerivedFieldIr, EnumAliasIr, EnumIr, EnumValueIr, FieldIr, GroupSetIr, IndexIr,
        ParserIr, StructIr, TableIr, TableModeIr, TableSourceIr, TypeIr, UnionIr, UnionVariantIr,
        ViewIr, ViewTableSelectionIr,
    },
    parse::parse_type,
    parser::ParserRegistry,
};

pub fn normalize_schema(schema: ProjectSchema) -> Result<ConfigIr> {
    normalize_schema_with_parsers(schema, &ParserRegistry::builtin())
}

pub fn normalize_schema_with_parsers(
    schema: ProjectSchema,
    parser_registry: &ParserRegistry,
) -> Result<ConfigIr> {
    validate_project_id(&schema.project.id)?;
    let group_defaults = normalize_group_definitions(&schema.groups)?;
    let views = normalize_views(schema.views, &group_defaults)?;
    let project_id = schema.project.id;
    let mut ir = ConfigIr {
        contract_id: project_id.clone(),
        project_id,
        view: None,
        group_defaults,
        views,
        enums: schema
            .enums
            .into_iter()
            .map(|item| {
                Ok(EnumIr {
                    name: item.name,
                    comment: item.comment,
                    groups: GroupSetIr::try_from(item.groups)?,
                    values: item
                        .values
                        .into_iter()
                        .map(|value| EnumValueIr {
                            id: value.id,
                            name: value.name,
                            comment: value.comment,
                        })
                        .collect(),
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
                    groups: GroupSetIr::try_from(item.groups)?,
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
    };
    apply_and_validate_groups(&mut ir)?;
    Ok(ir)
}

fn validate_project_id(project_id: &str) -> Result<()> {
    if project_id.is_empty()
        || !project_id
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-' | '/'))
    {
        return Err(SoraError::InvalidSchema(format!(
            "project id `{project_id}` must contain only ASCII letters, digits, `.`, `_`, `-`, or `/`"
        )));
    }
    Ok(())
}

fn normalize_group_definitions(
    groups: &std::collections::BTreeMap<String, sora_schema::model::GroupSchema>,
) -> Result<std::collections::BTreeMap<String, bool>> {
    if groups.is_empty() {
        return Err(SoraError::InvalidSchema(
            "project must declare at least one group".to_owned(),
        ));
    }
    let mut normalized = std::collections::BTreeMap::new();
    for (name, group) in groups {
        validate_group_name(name)?;
        normalized.insert(name.clone(), group.default);
    }
    if !normalized.values().any(|is_default| *is_default) {
        return Err(SoraError::InvalidSchema(
            "project must declare at least one default group".to_owned(),
        ));
    }
    Ok(normalized)
}

fn normalize_views(
    views: std::collections::BTreeMap<String, sora_schema::model::ViewSchema>,
    groups: &std::collections::BTreeMap<String, bool>,
) -> Result<std::collections::BTreeMap<String, ViewIr>> {
    if views.is_empty() {
        return Err(SoraError::InvalidSchema(
            "project must declare at least one view".to_owned(),
        ));
    }
    views
        .into_iter()
        .map(|(name, view)| {
            validate_group_name(&name)?;
            validate_project_id(&view.contract)?;
            if view.groups.is_empty() {
                return Err(SoraError::InvalidSchema(format!(
                    "view `{name}` must select at least one group"
                )));
            }
            let selected_groups = normalize_group_names(view.groups, groups, "view", &name)?;
            Ok((
                name,
                ViewIr {
                    contract: view.contract,
                    groups: selected_groups,
                    tables: ViewTableSelectionIr {
                        include: view.tables.include,
                        exclude: view.tables.exclude,
                    },
                    table_names: view.names.tables,
                    bindings: view.bindings,
                },
            ))
        })
        .collect()
}

fn apply_and_validate_groups(ir: &mut ConfigIr) -> Result<()> {
    let defaults = ir
        .group_defaults
        .iter()
        .filter_map(|(name, is_default)| is_default.then_some(name.clone()))
        .collect::<Vec<_>>();
    let definitions = &ir.group_defaults;
    for item in &mut ir.enums {
        resolve_groups(&mut item.groups, &defaults, definitions, "enum", &item.name)?;
    }
    for item in &mut ir.structs {
        resolve_groups(
            &mut item.groups,
            &defaults,
            definitions,
            "struct",
            &item.name,
        )?;
        for field in &mut item.fields {
            resolve_groups(
                &mut field.groups,
                &defaults,
                definitions,
                "field",
                &format!("{}.{}", item.name, field.name),
            )?;
        }
    }
    for item in &mut ir.unions {
        resolve_groups(
            &mut item.groups,
            &defaults,
            definitions,
            "union",
            &item.name,
        )?;
        for variant in &mut item.variants {
            resolve_groups(
                &mut variant.groups,
                &defaults,
                definitions,
                "union variant",
                &format!("{}.{}", item.name, variant.name),
            )?;
            for field in &mut variant.fields {
                resolve_groups(
                    &mut field.groups,
                    &defaults,
                    definitions,
                    "field",
                    &format!("{}.{}.{}", item.name, variant.name, field.name),
                )?;
            }
        }
    }
    for table in &mut ir.tables {
        resolve_groups(
            &mut table.groups,
            &defaults,
            definitions,
            "table",
            &table.name,
        )?;
        for field in &mut table.fields {
            resolve_groups(
                &mut field.groups,
                &defaults,
                definitions,
                "field",
                &format!("{}.{}", table.name, field.name),
            )?;
        }
    }
    Ok(())
}

fn resolve_groups(
    groups: &mut GroupSetIr,
    defaults: &[String],
    definitions: &std::collections::BTreeMap<String, bool>,
    kind: &str,
    name: &str,
) -> Result<()> {
    if groups.values.is_empty() {
        groups.values = defaults.to_vec();
        return Ok(());
    }
    groups.values =
        normalize_group_names(std::mem::take(&mut groups.values), definitions, kind, name)?;
    Ok(())
}

fn normalize_group_names(
    values: Vec<String>,
    definitions: &std::collections::BTreeMap<String, bool>,
    kind: &str,
    name: &str,
) -> Result<Vec<String>> {
    let mut normalized = Vec::with_capacity(values.len());
    for value in values {
        let value = value.trim();
        validate_group_name(value)?;
        if !definitions.contains_key(value) {
            return Err(SoraError::InvalidSchema(format!(
                "{kind} `{name}` references undeclared group `{value}`"
            )));
        }
        if !normalized.iter().any(|item| item == value) {
            normalized.push(value.to_owned());
        }
    }
    Ok(normalized)
}

fn validate_group_name(value: &str) -> Result<()> {
    if value.is_empty()
        || !value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
    {
        return Err(SoraError::InvalidSchema(format!(
            "group or view name `{value}` must contain only ASCII letters, digits, `_`, or `-`"
        )));
    }
    Ok(())
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
        sources,
    }))
}

impl TryFrom<ProjectSchema> for ConfigIr {
    type Error = SoraError;

    fn try_from(schema: ProjectSchema) -> Result<Self> {
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
            groups: GroupSetIr::try_from(union.groups)?,
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
            groups: GroupSetIr::try_from(variant.groups)?,
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

impl TryFrom<GroupSetSchema> for GroupSetIr {
    type Error = SoraError;

    fn try_from(groups: GroupSetSchema) -> Result<Self> {
        let mut values = Vec::with_capacity(groups.values.len());
        for value in groups.values {
            let value = value.trim();
            if value.is_empty() {
                return Err(SoraError::InvalidSchema(
                    "group values must not be empty".to_owned(),
                ));
            }
            if !value
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
            {
                return Err(SoraError::InvalidSchema(format!(
                    "group `{value}` must contain only ASCII letters, digits, `_`, or `-`"
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
        groups: GroupSetIr::try_from(field.groups)?,
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
                map_key: from.map_key,
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
        groups: GroupSetIr::try_from(field.groups)?,
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
        groups: GroupSetIr::try_from(union.groups)?,
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
        groups: GroupSetIr::try_from(variant.groups)?,
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
        id: table.id,
        canonical_name: table.name.clone(),
        name: table.name,
        groups: GroupSetIr::try_from(table.groups)?,
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
