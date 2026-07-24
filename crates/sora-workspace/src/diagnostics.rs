use std::path::PathBuf;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sora_diagnostics::SoraError;

/// Adapter-neutral diagnostic emitted by workspace operations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Diagnostic {
    pub level: DiagnosticLevel,
    pub code: Option<String>,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity: Option<DiagnosticEntity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span: Option<DiagnosticSpan>,
    #[serde(rename = "targetId", skip_serializing_if = "Option::is_none")]
    pub target_id: Option<String>,
}

impl Diagnostic {
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            level: DiagnosticLevel::Error,
            code: None,
            message: message.into(),
            path: None,
            entity: None,
            span: None,
            target_id: None,
        }
    }

    pub fn info(message: impl Into<String>) -> Self {
        Self {
            level: DiagnosticLevel::Info,
            code: None,
            message: message.into(),
            path: None,
            entity: None,
            span: None,
            target_id: None,
        }
    }

    pub fn from_sora_error(error: &SoraError) -> Self {
        Self {
            level: DiagnosticLevel::Error,
            code: Some(error.code().to_owned()),
            message: error.to_string(),
            path: error.path().map(PathBuf::from),
            entity: diagnostic_entity(error),
            span: None,
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
pub enum DiagnosticLevel {
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
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DiagnosticSpan {
    pub start_line: u32,
    pub start_column: u32,
    pub end_line: u32,
    pub end_column: u32,
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
    let (kind, name, field) = match error {
        SoraError::DuplicateSchemaName { kind, name } => ((*kind).to_owned(), name.clone(), None),
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
        } => ((*owner_kind).to_owned(), owner.clone(), Some(field.clone())),
        SoraError::MissingTableKey { table, field }
        | SoraError::UnknownField { table, field }
        | SoraError::MissingRequiredField { table, field }
        | SoraError::TypeMismatch { table, field, .. }
        | SoraError::InvalidEnumValue { table, field, .. }
        | SoraError::RangeOutOfBounds { table, field, .. }
        | SoraError::LengthOutOfBounds { table, field, .. }
        | SoraError::MissingReference { table, field, .. } => {
            ("table".to_owned(), table.clone(), Some(field.clone()))
        }
        SoraError::UnknownIndexField { table, .. }
        | SoraError::DuplicateKey { table, .. }
        | SoraError::DuplicateIndexKey { table, .. }
        | SoraError::InvalidTableRowCount { table, .. }
        | SoraError::MissingTableSource { table } => ("table".to_owned(), table.clone(), None),
        _ => return None,
    };
    Some(DiagnosticEntity { kind, name, field })
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
        assert_eq!(diagnostic.path, Some(PathBuf::from("data/items.csv")));
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
            })
        );
    }
}
