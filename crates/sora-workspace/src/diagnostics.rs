use std::path::PathBuf;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sora_diagnostics::{DataLocation, SoraError};

/// Adapter-neutral diagnostic emitted by workspace operations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Diagnostic {
    pub severity: DiagnosticSeverity,
    pub code: Option<String>,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity: Option<DiagnosticEntity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span: Option<DiagnosticSpan>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cell: Option<DiagnosticCell>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub related: Vec<Diagnostic>,
    #[serde(rename = "targetId", skip_serializing_if = "Option::is_none")]
    pub target_id: Option<String>,
}

impl Diagnostic {
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            severity: DiagnosticSeverity::Error,
            code: None,
            message: message.into(),
            file: None,
            entity: None,
            span: None,
            cell: None,
            hint: None,
            related: Vec::new(),
            target_id: None,
        }
    }

    pub fn info(message: impl Into<String>) -> Self {
        Self {
            severity: DiagnosticSeverity::Info,
            code: None,
            message: message.into(),
            file: None,
            entity: None,
            span: None,
            cell: None,
            hint: None,
            related: Vec::new(),
            target_id: None,
        }
    }

    pub fn from_sora_error(error: &SoraError) -> Self {
        let (span, cell) = diagnostic_location(error);
        Self {
            severity: DiagnosticSeverity::Error,
            code: Some(error.code().to_owned()),
            message: error.to_string(),
            file: error.path().map(PathBuf::from),
            entity: diagnostic_entity(error),
            span,
            cell,
            hint: diagnostic_hint(error),
            related: Vec::new(),
            target_id: None,
        }
    }

    pub fn with_target_id(mut self, target_id: Option<String>) -> Self {
        self.target_id = target_id;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DiagnosticEntity {
    pub kind: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub row_key: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DiagnosticSpan {
    pub start_line: u32,
    pub start_column: u32,
    pub end_line: u32,
    pub end_column: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DiagnosticCell {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sheet: Option<String>,
    pub row: usize,
    pub column: usize,
}

pub fn diagnostics_from_sora_error(error: &SoraError) -> Vec<Diagnostic> {
    match error.errors() {
        Some(errors) => errors
            .iter()
            .flat_map(diagnostics_from_sora_error)
            .collect(),
        None => vec![Diagnostic::from_sora_error(error)],
    }
}

pub fn diagnostics_from_anyhow(error: &anyhow::Error) -> Vec<Diagnostic> {
    error
        .chain()
        .find_map(|cause| cause.downcast_ref::<SoraError>())
        .map(diagnostics_from_sora_error)
        .unwrap_or_else(|| vec![Diagnostic::error(error.to_string())])
}

fn diagnostic_entity(error: &SoraError) -> Option<DiagnosticEntity> {
    let (kind, name, field, row_key) = match error {
        SoraError::ParseDataAt { field, .. } => (
            "data".to_owned(),
            "source".to_owned(),
            Some(field.clone()),
            None,
        ),
        SoraError::SourceLoaderDiagnostic { path, field, .. } => (
            "source_loader".to_owned(),
            path.to_string_lossy().into_owned(),
            field.clone(),
            None,
        ),
        SoraError::DuplicateSchemaName { kind, name } => {
            ((*kind).to_owned(), name.clone(), None, None)
        }
        SoraError::DuplicateFieldName {
            owner_kind,
            owner,
            field,
        }
        | SoraError::UnknownTypeReference {
            owner_kind,
            owner,
            field,
            ..
        }
        | SoraError::UnknownRefTable {
            owner_kind,
            owner,
            field,
            ..
        }
        | SoraError::UnknownRefField {
            owner_kind,
            owner,
            field,
            ..
        } => (
            (*owner_kind).to_owned(),
            owner.clone(),
            Some(field.clone()),
            None,
        ),
        SoraError::MissingTableKey { table, field }
        | SoraError::UnknownField { table, field }
        | SoraError::MissingRequiredField { table, field }
        | SoraError::TypeMismatch { table, field, .. }
        | SoraError::InvalidEnumValue { table, field, .. }
        | SoraError::RangeOutOfBounds { table, field, .. }
        | SoraError::LengthOutOfBounds { table, field, .. }
        | SoraError::MissingReference { table, field, .. } => {
            ("table".to_owned(), table.clone(), Some(field.clone()), None)
        }
        SoraError::DuplicateKey { table, key } => {
            ("table".to_owned(), table.clone(), None, Some(key.clone()))
        }
        SoraError::UnknownIndexField { table, .. }
        | SoraError::DuplicateIndexKey { table, .. }
        | SoraError::InvalidTableRowCount { table, .. }
        | SoraError::MissingTableSource { table } => {
            ("table".to_owned(), table.clone(), None, None)
        }
        _ => return None,
    };
    Some(DiagnosticEntity {
        kind,
        name,
        field,
        row_key,
    })
}

fn diagnostic_location(error: &SoraError) -> (Option<DiagnosticSpan>, Option<DiagnosticCell>) {
    if let SoraError::SourceLoaderDiagnostic { line, column, .. } = error {
        return match (line, column) {
            (None, None) => (None, None),
            (line, column) => {
                let line = line.unwrap_or(1);
                let column = column.unwrap_or(1);
                (Some(single_position_span(line, column)), None)
            }
        };
    }
    let SoraError::ParseDataAt { location, .. } = error else {
        return (None, None);
    };
    match location {
        DataLocation::SchemaDefault => (None, None),
        DataLocation::Csv { row, column } => (
            Some(single_position_span(*row, *column)),
            Some(DiagnosticCell {
                sheet: None,
                row: *row,
                column: *column,
            }),
        ),
        DataLocation::Worksheet { sheet, row, column } => (
            Some(single_position_span(*row, *column)),
            Some(DiagnosticCell {
                sheet: Some(sheet.clone()),
                row: *row,
                column: *column,
            }),
        ),
    }
}

fn single_position_span(row: usize, column: usize) -> DiagnosticSpan {
    DiagnosticSpan {
        start_line: row.try_into().unwrap_or(u32::MAX),
        start_column: column.try_into().unwrap_or(u32::MAX),
        end_line: row.try_into().unwrap_or(u32::MAX),
        end_column: column.try_into().unwrap_or(u32::MAX),
    }
}

fn diagnostic_hint(error: &SoraError) -> Option<String> {
    match error {
        SoraError::UnknownField { .. } => {
            Some("add the field to the schema or remove it from the source row".to_owned())
        }
        SoraError::MissingRequiredField { .. } => {
            Some("provide the required field or make it optional in the schema".to_owned())
        }
        SoraError::InvalidEnumValue { .. } => {
            Some("use a value declared by the referenced enum".to_owned())
        }
        SoraError::MissingReference { .. } => {
            Some("create the referenced row or correct the reference value".to_owned())
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_stable_code_path_and_entity() {
        let diagnostic = Diagnostic::from_sora_error(&SoraError::ParseData {
            path: "data/items.csv".into(),
            message: "bad row".to_owned(),
        });
        assert_eq!(diagnostic.code.as_deref(), Some("SORA0005"));
        assert_eq!(diagnostic.file, Some(PathBuf::from("data/items.csv")));
        assert!(diagnostic.entity.is_none());

        let diagnostic = Diagnostic::from_sora_error(&SoraError::UnknownField {
            table: "Item".to_owned(),
            field: "rarity".to_owned(),
        });
        assert_eq!(
            diagnostic.entity,
            Some(DiagnosticEntity {
                kind: "table".to_owned(),
                name: "Item".to_owned(),
                field: Some("rarity".to_owned()),
                row_key: None,
            })
        );
    }

    #[test]
    fn maps_excel_cell_without_parsing_error_text() {
        let diagnostic = Diagnostic::from_sora_error(&SoraError::ParseDataAt {
            path: "data/items.xlsx".into(),
            field: "rarity".to_owned(),
            location: DataLocation::Worksheet {
                sheet: "Items".to_owned(),
                row: 7,
                column: 3,
            },
            message: "invalid enum value".to_owned(),
        });

        assert_eq!(
            diagnostic.cell,
            Some(DiagnosticCell {
                sheet: Some("Items".to_owned()),
                row: 7,
                column: 3,
            })
        );
        assert_eq!(
            diagnostic.span,
            Some(DiagnosticSpan {
                start_line: 7,
                start_column: 3,
                end_line: 7,
                end_column: 3,
            })
        );
    }
}
