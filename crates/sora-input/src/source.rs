use std::path::Path;

use sora_diagnostics::{Result, SoraError};
use sora_ir::model::{TableIr, TableSourceIr};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceFormat {
    Csv,
    Toml,
    Xlsx,
}

impl SourceFormat {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Csv => "csv",
            Self::Toml => "toml",
            Self::Xlsx => "xlsx",
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "csv" => Ok(Self::Csv),
            "toml" => Ok(Self::Toml),
            "xlsx" => Ok(Self::Xlsx),
            _ => Err(SoraError::InvalidSchema(format!(
                "unsupported source format `{value}`; expected csv, toml, or xlsx"
            ))),
        }
    }

    fn infer_from_file(file: &str) -> Option<Self> {
        match Path::new(file).extension().and_then(|ext| ext.to_str()) {
            Some("csv") => Some(Self::Csv),
            Some("toml") => Some(Self::Toml),
            Some("xlsx") => Some(Self::Xlsx),
            _ => None,
        }
    }
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
}
