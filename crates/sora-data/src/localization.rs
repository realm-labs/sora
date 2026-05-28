use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;
use sora_diagnostics::{Result, SoraError};
use sora_ir::model::{ConfigIr, FieldIr, TypeIr, UnionIr};

use crate::model::{ConfigData, LocalizationData, LocalizationSourceData, Value};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LocaleCatalog {
    pub locales: Vec<String>,
    pub default_locale: String,
    pub fallback_locale: Option<String>,
    pub entries: BTreeMap<String, BTreeMap<String, String>>,
}

impl LocaleCatalog {
    pub fn for_locale(&self, locale: &str) -> Result<BTreeMap<String, String>> {
        if !self.locales.iter().any(|candidate| candidate == locale) {
            return Err(SoraError::InvalidSchema(format!(
                "unknown localization locale `{locale}`"
            )));
        }
        Ok(self
            .entries
            .iter()
            .filter_map(|(key, values)| {
                values.get(locale).map(|value| (key.clone(), value.clone()))
            })
            .collect())
    }
}

pub fn build_locale_catalog(
    ir: &ConfigIr,
    config_data: &ConfigData,
    localization_data: &LocalizationData,
) -> Result<Option<LocaleCatalog>> {
    let Some(localization) = &ir.localization else {
        return Ok(None);
    };

    let mut entries = BTreeMap::<String, BTreeMap<String, String>>::new();
    for source in &localization.sources {
        let source_data = localization_data
            .sources
            .iter()
            .find(|source_data| source_data.name == source.name)
            .ok_or_else(|| {
                SoraError::InvalidSchema(format!(
                    "missing localization source data `{}`",
                    source.name
                ))
            })?;

        validate_source_columns(source_data, &source.key, &localization.locales)?;

        for row in &source_data.rows {
            let key = row.values.get(&source.key).ok_or_else(|| {
                SoraError::InvalidSchema(format!(
                    "localization source `{}` has a row without key field `{}`",
                    source.name, source.key
                ))
            })?;
            if entries.contains_key(key) {
                return Err(SoraError::InvalidSchema(format!(
                    "duplicate localization key `{key}` in source `{}`",
                    source.name
                )));
            }
            let mut values = BTreeMap::new();
            for locale in &localization.locales {
                let value = row.values.get(locale).ok_or_else(|| {
                    SoraError::InvalidSchema(format!(
                        "localization key `{key}` in source `{}` is missing locale `{locale}`",
                        source.name
                    ))
                })?;
                if value.is_empty() {
                    return Err(SoraError::InvalidSchema(format!(
                        "localization key `{key}` in source `{}` has empty `{locale}` text",
                        source.name
                    )));
                }
                values.insert(locale.clone(), value.clone());
            }
            entries.insert(key.clone(), values);
        }
    }

    validate_text_references(ir, config_data, &entries)?;

    Ok(Some(LocaleCatalog {
        locales: localization.locales.clone(),
        default_locale: localization.default_locale.clone(),
        fallback_locale: localization.fallback_locale.clone(),
        entries,
    }))
}

fn validate_source_columns(
    source: &LocalizationSourceData,
    key_field: &str,
    locales: &[String],
) -> Result<()> {
    if !source.columns.iter().any(|column| column == key_field) {
        return Err(SoraError::InvalidSchema(format!(
            "localization source `{}` is missing key column `{key_field}`",
            source.name
        )));
    }
    for locale in locales {
        if !source.columns.iter().any(|column| column == locale) {
            return Err(SoraError::InvalidSchema(format!(
                "localization source `{}` is missing locale column `{locale}`",
                source.name
            )));
        }
    }
    Ok(())
}

fn validate_text_references(
    ir: &ConfigIr,
    data: &ConfigData,
    entries: &BTreeMap<String, BTreeMap<String, String>>,
) -> Result<()> {
    let mut missing = BTreeSet::new();
    for table in &ir.tables {
        let Some(table_data) = data
            .tables
            .iter()
            .find(|candidate| candidate.name == table.name)
        else {
            continue;
        };
        for row in &table_data.rows {
            for field in &table.fields {
                if let Some(value) = row.values.get(&field.name) {
                    collect_missing_text_keys(ir, &field.ty, value, &mut missing, entries);
                }
            }
        }
    }
    if let Some(key) = missing.into_iter().next() {
        return Err(SoraError::InvalidSchema(format!(
            "text key `{key}` is not present in localization catalog"
        )));
    }
    Ok(())
}

fn collect_missing_text_keys(
    ir: &ConfigIr,
    ty: &TypeIr,
    value: &Value,
    missing: &mut BTreeSet<String>,
    entries: &BTreeMap<String, BTreeMap<String, String>>,
) {
    match ty {
        TypeIr::Text => {
            if let Value::String(key) = value
                && !entries.contains_key(key)
            {
                missing.insert(key.clone());
            }
        }
        TypeIr::Optional(inner) => {
            if !matches!(value, Value::Null) {
                collect_missing_text_keys(ir, inner, value, missing, entries);
            }
        }
        TypeIr::Struct(name) => {
            let Some(struct_ir) = ir.structs.iter().find(|item| item.name == *name) else {
                return;
            };
            let Value::Object(values) = value else {
                return;
            };
            collect_object_text_keys(ir, &struct_ir.fields, values, missing, entries);
        }
        TypeIr::Union(name) => collect_union_text_keys(ir, name, value, missing, entries),
        TypeIr::List(element) | TypeIr::Set(element) | TypeIr::Array { element, .. } => {
            if let Value::List(values) = value {
                for value in values {
                    collect_missing_text_keys(ir, element, value, missing, entries);
                }
            }
        }
        TypeIr::Map {
            key,
            value: element,
        } => {
            if let Value::List(values) = value {
                for entry in values {
                    let Value::List(pair) = entry else {
                        continue;
                    };
                    if pair.len() == 2 {
                        collect_missing_text_keys(ir, key, &pair[0], missing, entries);
                        collect_missing_text_keys(ir, element, &pair[1], missing, entries);
                    }
                }
            }
        }
        TypeIr::Ref { table, field } => {
            if let Some(target) = ir
                .tables
                .iter()
                .find(|candidate| candidate.name == *table)
                .and_then(|table| {
                    table
                        .fields
                        .iter()
                        .find(|candidate| candidate.name == *field)
                })
            {
                collect_missing_text_keys(ir, &target.ty, value, missing, entries);
            }
        }
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
        | TypeIr::Enum(_) => {}
    }
}

fn collect_object_text_keys(
    ir: &ConfigIr,
    fields: &[FieldIr],
    values: &BTreeMap<String, Value>,
    missing: &mut BTreeSet<String>,
    entries: &BTreeMap<String, BTreeMap<String, String>>,
) {
    for field in fields {
        if let Some(value) = values.get(&field.name) {
            collect_missing_text_keys(ir, &field.ty, value, missing, entries);
        }
    }
}

fn collect_union_text_keys(
    ir: &ConfigIr,
    name: &str,
    value: &Value,
    missing: &mut BTreeSet<String>,
    entries: &BTreeMap<String, BTreeMap<String, String>>,
) {
    let Some(union_ir): Option<&UnionIr> = ir.unions.iter().find(|item| item.name == name) else {
        return;
    };
    let Value::Object(values) = value else {
        return;
    };
    let Some(Value::String(variant_name)) = values.get(&union_ir.tag) else {
        return;
    };
    let Some(variant) = union_ir
        .variants
        .iter()
        .find(|item| item.name == *variant_name)
    else {
        return;
    };
    collect_object_text_keys(ir, &variant.fields, values, missing, entries);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ConfigData, LocalizationRowData, RowData, TableData};
    use sora_ir::{normalize::normalize_schema, validate::validate_config_ir};
    use sora_schema::model::SchemaFile;

    #[test]
    fn builds_catalog_from_multiple_sources() {
        let ir = example_ir();
        let data = ConfigData {
            tables: vec![TableData {
                name: "Quest".to_owned(),
                rows: vec![RowData {
                    values: BTreeMap::from([
                        ("id".to_owned(), Value::Integer(1)),
                        ("title".to_owned(), Value::String("quest.title".to_owned())),
                    ]),
                }],
            }],
        };
        let localization_data = LocalizationData {
            sources: vec![
                locale_source("UiText", "ui.ok", "确认", "OK"),
                locale_source("QuestText", "quest.title", "任务", "Quest"),
            ],
        };

        let catalog = build_locale_catalog(&ir, &data, &localization_data)
            .unwrap()
            .unwrap();
        assert_eq!(catalog.entries.len(), 2);
        assert_eq!(catalog.for_locale("zh_cn").unwrap()["quest.title"], "任务");
    }

    #[test]
    fn rejects_missing_text_key() {
        let ir = example_ir();
        let data = ConfigData {
            tables: vec![TableData {
                name: "Quest".to_owned(),
                rows: vec![RowData {
                    values: BTreeMap::from([
                        ("id".to_owned(), Value::Integer(1)),
                        ("title".to_owned(), Value::String("missing".to_owned())),
                    ]),
                }],
            }],
        };
        let localization_data = LocalizationData {
            sources: vec![
                locale_source("UiText", "ui.ok", "确认", "OK"),
                locale_source("QuestText", "quest.title", "任务", "Quest"),
            ],
        };

        assert!(build_locale_catalog(&ir, &data, &localization_data).is_err());
    }

    #[test]
    fn rejects_empty_translation() {
        let ir = example_ir();
        let data = ConfigData {
            tables: vec![TableData {
                name: "Quest".to_owned(),
                rows: vec![RowData {
                    values: BTreeMap::from([
                        ("id".to_owned(), Value::Integer(1)),
                        ("title".to_owned(), Value::String("quest.title".to_owned())),
                    ]),
                }],
            }],
        };
        let localization_data = LocalizationData {
            sources: vec![locale_source("QuestText", "quest.title", "", "Quest")],
        };

        assert!(build_locale_catalog(&ir, &data, &localization_data).is_err());
    }

    fn locale_source(name: &str, key: &str, zh_cn: &str, en_us: &str) -> LocalizationSourceData {
        LocalizationSourceData {
            name: name.to_owned(),
            columns: vec!["key".to_owned(), "zh_cn".to_owned(), "en_us".to_owned()],
            rows: vec![LocalizationRowData {
                values: BTreeMap::from([
                    ("key".to_owned(), key.to_owned()),
                    ("zh_cn".to_owned(), zh_cn.to_owned()),
                    ("en_us".to_owned(), en_us.to_owned()),
                ]),
            }],
        }
    }

    fn example_ir() -> ConfigIr {
        let schema: SchemaFile = toml::from_str(
            r#"
package = "game_config"

[localization]
locales = ["zh_cn", "en_us"]
default_locale = "zh_cn"
fallback_locale = "en_us"

[[localization.sources]]
name = "UiText"
file = "Core.xlsx"

[[localization.sources]]
name = "QuestText"
file = "Quest.xlsx"

[[tables]]
name = "Quest"
mode = "map"
key = "id"

[[tables.fields]]
name = "id"
type = "i32"

[[tables.fields]]
name = "title"
type = "text"
"#,
        )
        .unwrap();
        let ir = normalize_schema(schema).unwrap();
        validate_config_ir(&ir).unwrap();
        ir
    }
}
