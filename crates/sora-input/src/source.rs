use std::{collections::BTreeMap, path::Path};

use sora_data::model::TableData;
use sora_diagnostics::{Result, SoraError};
use sora_execution::ExecutionContext;
use sora_ir::model::ConfigIr;
use sora_ir::model::{TableIr, TableSourceIr};

use crate::parser::ParserRegistry;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceFormat {
    Csv,
    Json,
    Toml,
    Xlsx,
    Yaml,
}

impl SourceFormat {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Csv => "csv",
            Self::Json => "json",
            Self::Toml => "toml",
            Self::Xlsx => "xlsx",
            Self::Yaml => "yaml",
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "csv" => Ok(Self::Csv),
            "json" => Ok(Self::Json),
            "toml" => Ok(Self::Toml),
            "xlsx" => Ok(Self::Xlsx),
            "yaml" | "yml" => Ok(Self::Yaml),
            _ => Err(SoraError::InvalidSchema(format!(
                "unsupported source format `{value}`; expected csv, json, toml, xlsx, or yaml"
            ))),
        }
    }

    fn infer_from_file(file: &str) -> Option<Self> {
        match Path::new(file).extension().and_then(|ext| ext.to_str()) {
            Some("csv") => Some(Self::Csv),
            Some("json") => Some(Self::Json),
            Some("toml") => Some(Self::Toml),
            Some("xlsx") => Some(Self::Xlsx),
            Some("yaml" | "yml") => Some(Self::Yaml),
            _ => None,
        }
    }
}

pub struct DataSourceRequest<'a> {
    pub ir: &'a ConfigIr,
    pub table: &'a TableIr,
    pub source: &'a TableSourceIr,
    pub path: &'a Path,
    pub execution: &'a ExecutionContext,
    pub parser_registry: &'a ParserRegistry,
}

pub trait DataSourceLoader: Send + Sync {
    fn format_name(&self) -> &'static str;

    fn file_extensions(&self) -> &'static [&'static str] {
        &[]
    }

    fn load_table(&self, request: DataSourceRequest<'_>) -> Result<TableData>;
}

#[derive(Default)]
pub struct DataSourceRegistry {
    loaders: BTreeMap<String, Box<dyn DataSourceLoader>>,
}

impl DataSourceRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register<L: DataSourceLoader + 'static>(&mut self, loader: L) {
        self.loaders
            .insert(loader.format_name().to_owned(), Box::new(loader));
    }

    pub fn get(&self, format_name: &str) -> Option<&dyn DataSourceLoader> {
        self.loaders.get(format_name).map(Box::as_ref)
    }

    pub fn supported_formats(&self) -> Vec<&str> {
        self.loaders.keys().map(String::as_str).collect()
    }

    fn infer_from_file(&self, file: &str) -> Option<&str> {
        let extension = Path::new(file).extension()?.to_str()?;
        self.loaders
            .values()
            .find(|loader| loader.file_extensions().contains(&extension))
            .map(|loader| loader.format_name())
    }
}

pub fn resolve_table_source_format_with_registry<'a>(
    table: &TableIr,
    default_source_format: Option<&str>,
    registry: &'a DataSourceRegistry,
) -> Result<&'a str> {
    let source = table
        .source
        .as_ref()
        .ok_or_else(|| SoraError::MissingTableSource {
            table: table.name.clone(),
        })?;
    resolve_source_format_with_registry(&table.name, source, default_source_format, registry)
}

pub fn resolve_source_format_with_registry<'a>(
    table_name: &str,
    source: &TableSourceIr,
    default_source_format: Option<&str>,
    registry: &'a DataSourceRegistry,
) -> Result<&'a str> {
    if let Some(format) = source.format.as_deref() {
        return validate_registered_format(format, registry);
    }

    if let Some(format) = registry.infer_from_file(&source.file) {
        return Ok(format);
    }

    if let Some(format) = default_source_format {
        return validate_registered_format(format, registry);
    }

    Err(SoraError::InvalidSchema(format!(
        "table `{table_name}` source file `{}` does not have a supported extension; set `source.format` or `[build].default_source_format`; supported formats: {}",
        source.file,
        registry.supported_formats().join(", ")
    )))
}

fn validate_registered_format<'a>(
    format: &str,
    registry: &'a DataSourceRegistry,
) -> Result<&'a str> {
    registry
        .get(format)
        .map(|loader| loader.format_name())
        .ok_or_else(|| {
            SoraError::InvalidSchema(format!(
                "unsupported source format `{}`; supported formats: {}",
                format,
                registry.supported_formats().join(", ")
            ))
        })
}

pub fn resolve_table_source_format(
    table: &TableIr,
    default_source_format: Option<&str>,
) -> Result<SourceFormat> {
    let source = table
        .source
        .as_ref()
        .ok_or_else(|| SoraError::MissingTableSource {
            table: table.name.clone(),
        })?;
    resolve_source_format(&table.name, source, default_source_format)
}

pub fn resolve_source_format(
    table_name: &str,
    source: &TableSourceIr,
    default_source_format: Option<&str>,
) -> Result<SourceFormat> {
    if let Some(format) = source.format.as_deref() {
        return SourceFormat::parse(format);
    }

    if let Some(format) = SourceFormat::infer_from_file(&source.file) {
        return Ok(format);
    }

    if let Some(format) = default_source_format {
        return SourceFormat::parse(format);
    }

    Err(SoraError::InvalidSchema(format!(
        "table `{table_name}` source file `{}` does not have a supported extension; set `source.format` or `[build].default_source_format`",
        source.file
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_explicit_format_before_file_extension() {
        let source = TableSourceIr {
            format: Some("toml".to_owned()),
            file: "items.data".to_owned(),
            sheet: None,
        };

        assert_eq!(
            resolve_source_format("Item", &source, None).unwrap(),
            SourceFormat::Toml
        );
    }

    #[test]
    fn resolves_format_from_file_extension() {
        let source = TableSourceIr {
            format: None,
            file: "items.csv".to_owned(),
            sheet: None,
        };

        assert_eq!(
            resolve_source_format("Item", &source, Some("xlsx")).unwrap(),
            SourceFormat::Csv
        );
    }

    #[test]
    fn resolves_json_and_yaml_extensions() {
        let json = TableSourceIr {
            format: None,
            file: "items.json".to_owned(),
            sheet: None,
        };
        let yaml = TableSourceIr {
            format: None,
            file: "items.yml".to_owned(),
            sheet: None,
        };

        assert_eq!(
            resolve_source_format("Item", &json, None).unwrap(),
            SourceFormat::Json
        );
        assert_eq!(
            resolve_source_format("Item", &yaml, None).unwrap(),
            SourceFormat::Yaml
        );
    }

    #[test]
    fn falls_back_to_default_source_format() {
        let source = TableSourceIr {
            format: None,
            file: "items.data".to_owned(),
            sheet: None,
        };

        assert_eq!(
            resolve_source_format("Item", &source, Some("toml")).unwrap(),
            SourceFormat::Toml
        );
    }

    #[test]
    fn registry_resolves_registered_formats() {
        let mut registry = DataSourceRegistry::new();
        registry.register(TestLoader);
        let source = TableSourceIr {
            format: None,
            file: "items.data".to_owned(),
            sheet: None,
        };

        assert_eq!(
            resolve_source_format_with_registry("Item", &source, None, &registry).unwrap(),
            "test"
        );
    }

    #[test]
    fn registry_rejects_unregistered_formats() {
        let registry = DataSourceRegistry::new();
        let source = TableSourceIr {
            format: Some("json".to_owned()),
            file: "items.json".to_owned(),
            sheet: None,
        };

        let error =
            resolve_source_format_with_registry("Item", &source, None, &registry).unwrap_err();

        assert!(
            error
                .to_string()
                .contains("unsupported source format `json`")
        );
    }

    struct TestLoader;

    impl DataSourceLoader for TestLoader {
        fn format_name(&self) -> &'static str {
            "test"
        }

        fn file_extensions(&self) -> &'static [&'static str] {
            &["data"]
        }

        fn load_table(&self, _request: DataSourceRequest<'_>) -> Result<TableData> {
            unreachable!("registry format resolution test does not load data")
        }
    }
}
