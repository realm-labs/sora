use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum SoraError {
    #[error("failed to read file `{path}`: {source}")]
    ReadFile {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("failed to write file `{path}`: {source}")]
    WriteFile {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("failed to create directory `{path}`: {source}")]
    CreateDir {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("failed to parse schema `{path}`: {message}")]
    ParseSchema { path: PathBuf, message: String },

    #[error("failed to parse data `{path}`: {message}")]
    ParseData { path: PathBuf, message: String },

    #[error("failed to serialize data: {0}")]
    SerializeData(serde_json::Error),

    #[error("failed to serialize data as {format}: {message}")]
    SerializeDataFormat {
        format: &'static str,
        message: String,
    },

    #[error("unknown type `{0}`")]
    UnknownType(String),

    #[error("invalid type `{0}`")]
    InvalidType(String),

    #[error("invalid schema: {0}")]
    InvalidSchema(String),

    #[error("duplicate {kind} `{name}`")]
    DuplicateSchemaName { kind: &'static str, name: String },

    #[error("duplicate field `{field}` in {owner_kind} `{owner}`")]
    DuplicateFieldName {
        owner_kind: &'static str,
        owner: String,
        field: String,
    },

    #[error("unknown {kind} `{name}` referenced by field `{field}` in {owner_kind} `{owner}`")]
    UnknownTypeReference {
        kind: &'static str,
        name: String,
        owner_kind: &'static str,
        owner: String,
        field: String,
    },

    #[error("table `{table}` key field `{field}` does not exist")]
    MissingTableKey { table: String, field: String },

    #[error("index `{index}` in table `{table}` references unknown field `{field}`")]
    UnknownIndexField {
        table: String,
        index: String,
        field: String,
    },

    #[error("field `{field}` in {owner_kind} `{owner}` references unknown table `{table}`")]
    UnknownRefTable {
        owner_kind: &'static str,
        owner: String,
        field: String,
        table: String,
    },

    #[error(
        "field `{field}` in {owner_kind} `{owner}` references unknown ref field `{ref_field}` in table `{table}`"
    )]
    UnknownRefField {
        owner_kind: &'static str,
        owner: String,
        field: String,
        table: String,
        ref_field: String,
    },

    #[error("failed to render template `{template}`: {message}")]
    RenderTemplate { template: String, message: String },

    #[error("failed to format {language} code with `{command}`: {message}")]
    FormatCode {
        language: &'static str,
        command: String,
        message: String,
    },

    #[error("failed to write Excel template `{path}`: {message}")]
    ExcelTemplate { path: PathBuf, message: String },

    #[error("unknown export format `{format}`; supported formats: {supported}")]
    UnknownExportFormat { format: String, supported: String },

    #[error("export format `{format}` expects {expected} output")]
    InvalidExportOutput {
        format: String,
        expected: &'static str,
    },

    #[error("missing required field `{field}` in table `{table}`")]
    MissingRequiredField { table: String, field: String },

    #[error("unknown field `{field}` in table `{table}`")]
    UnknownField { table: String, field: String },

    #[error(
        "type mismatch for field `{field}` in table `{table}`: expected {expected}, got {actual}"
    )]
    TypeMismatch {
        table: String,
        field: String,
        expected: String,
        actual: String,
    },

    #[error("invalid enum value `{value}` for field `{field}` in table `{table}`")]
    InvalidEnumValue {
        table: String,
        field: String,
        value: String,
    },

    #[error("duplicate key `{key}` in table `{table}`")]
    DuplicateKey { table: String, key: String },

    #[error("duplicate unique index `{index}` key `{key}` in table `{table}`")]
    DuplicateIndexKey {
        table: String,
        index: String,
        key: String,
    },

    #[error(
        "value `{value}` for field `{field}` in table `{table}` is outside range [{min}, {max}]"
    )]
    RangeOutOfBounds {
        table: String,
        field: String,
        value: String,
        min: i64,
        max: i64,
    },

    #[error(
        "length `{actual}` for field `{field}` in table `{table}` is outside range [{min}, {max}]"
    )]
    LengthOutOfBounds {
        table: String,
        field: String,
        actual: usize,
        min: usize,
        max: usize,
    },

    #[error(
        "missing reference `{value}` for field `{field}` in table `{table}`; target is `{ref_table}.{ref_field}`"
    )]
    MissingReference {
        table: String,
        field: String,
        ref_table: String,
        ref_field: String,
        value: String,
    },

    #[error("invalid row count for `{mode}` table `{table}`: expected {expected}, got {actual}")]
    InvalidTableRowCount {
        table: String,
        mode: &'static str,
        expected: &'static str,
        actual: usize,
    },

    #[error("table `{table}` does not declare a data source")]
    MissingTableSource { table: String },

    #[error("input source does not provide data")]
    MissingInputData,

    #[error("{count} validation errors")]
    ValidationErrors {
        count: usize,
        errors: Vec<SoraError>,
    },
}

pub type Result<T> = std::result::Result<T, SoraError>;

impl SoraError {
    pub fn validation_errors(errors: Vec<SoraError>) -> Self {
        Self::ValidationErrors {
            count: errors.len(),
            errors,
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            Self::ReadFile { .. } => "SORA0001",
            Self::WriteFile { .. } => "SORA0002",
            Self::CreateDir { .. } => "SORA0003",
            Self::ParseSchema { .. } => "SORA0004",
            Self::ParseData { .. } => "SORA0005",
            Self::SerializeData(_) => "SORA0006",
            Self::SerializeDataFormat { .. } => "SORA0007",
            Self::UnknownType(_) => "SORA0008",
            Self::InvalidType(_) => "SORA0009",
            Self::InvalidSchema(_) => "SORA0010",
            Self::DuplicateSchemaName { .. } => "SORA0011",
            Self::DuplicateFieldName { .. } => "SORA0012",
            Self::UnknownTypeReference { .. } => "SORA0013",
            Self::MissingTableKey { .. } => "SORA0014",
            Self::UnknownIndexField { .. } => "SORA0015",
            Self::UnknownRefTable { .. } => "SORA0016",
            Self::UnknownRefField { .. } => "SORA0017",
            Self::RenderTemplate { .. } => "SORA0018",
            Self::FormatCode { .. } => "SORA0019",
            Self::ExcelTemplate { .. } => "SORA0020",
            Self::UnknownExportFormat { .. } => "SORA0021",
            Self::InvalidExportOutput { .. } => "SORA0022",
            Self::MissingRequiredField { .. } => "SORA0023",
            Self::UnknownField { .. } => "SORA0024",
            Self::TypeMismatch { .. } => "SORA0025",
            Self::InvalidEnumValue { .. } => "SORA0026",
            Self::DuplicateKey { .. } => "SORA0027",
            Self::DuplicateIndexKey { .. } => "SORA0028",
            Self::RangeOutOfBounds { .. } => "SORA0029",
            Self::LengthOutOfBounds { .. } => "SORA0030",
            Self::MissingReference { .. } => "SORA0031",
            Self::InvalidTableRowCount { .. } => "SORA0032",
            Self::MissingTableSource { .. } => "SORA0033",
            Self::MissingInputData => "SORA0034",
            Self::ValidationErrors { .. } => "SORA0035",
        }
    }

    pub fn path(&self) -> Option<&Path> {
        match self {
            Self::ReadFile { path, .. }
            | Self::WriteFile { path, .. }
            | Self::CreateDir { path, .. }
            | Self::ParseSchema { path, .. }
            | Self::ParseData { path, .. }
            | Self::ExcelTemplate { path, .. } => Some(path),
            _ => None,
        }
    }

    pub fn errors(&self) -> Option<&[SoraError]> {
        match self {
            Self::ValidationErrors { errors, .. } => Some(errors),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_stable_error_code() {
        let error = SoraError::MissingInputData;

        assert_eq!(error.code(), "SORA0034");
    }

    #[test]
    fn exposes_file_path_for_file_errors() {
        let error = SoraError::ParseData {
            path: PathBuf::from("data/items.csv"),
            message: "bad row".to_owned(),
        };

        assert_eq!(error.path(), Some(Path::new("data/items.csv")));
    }
}
