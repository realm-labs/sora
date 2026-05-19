use std::path::PathBuf;

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

    #[error("failed to parse schema `{path}`: {source}")]
    ParseSchema {
        path: PathBuf,
        source: toml::de::Error,
    },

    #[error("failed to parse data `{path}`: {source}")]
    ParseData {
        path: PathBuf,
        source: toml::de::Error,
    },

    #[error("failed to serialize data: {0}")]
    SerializeData(serde_json::Error),

    #[error("unknown type `{0}`")]
    UnknownType(String),

    #[error("invalid type `{0}`")]
    InvalidType(String),

    #[error("invalid schema: {0}")]
    InvalidSchema(String),

    #[error("failed to render template `{template}`: {message}")]
    RenderTemplate { template: String, message: String },

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

    #[error("table `{table}` does not declare a data source")]
    MissingTableSource { table: String },
}

pub type Result<T> = std::result::Result<T, SoraError>;
