use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum SoraError {
    #[error("failed to read file `{path}`: {source}")]
    ReadFile {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("failed to parse schema `{path}`: {source}")]
    ParseSchema {
        path: PathBuf,
        source: toml::de::Error,
    },

    #[error("unknown type `{0}`")]
    UnknownType(String),

    #[error("invalid type `{0}`")]
    InvalidType(String),

    #[error("invalid schema: {0}")]
    InvalidSchema(String),

    #[error("unknown export format `{format}`; supported formats: {supported}")]
    UnknownExportFormat { format: String, supported: String },

    #[error("missing required field `{field}` in table `{table}`")]
    MissingRequiredField { table: String, field: String },
}

pub type Result<T> = std::result::Result<T, SoraError>;
