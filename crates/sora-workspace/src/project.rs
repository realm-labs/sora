use std::path::{Path, PathBuf};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Opaque identifier for an opened project within one workspace service.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(transparent)]
pub struct ProjectId(String);

impl ProjectId {
    /// Creates a project identifier after validating its protocol-safe form.
    pub fn new(value: impl Into<String>) -> Result<Self, InvalidProjectId> {
        let value = value.into();
        let valid_length = !value.is_empty() && value.len() <= 128;
        let valid_characters = value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'));
        if valid_length && valid_characters {
            Ok(Self(value))
        } else {
            Err(InvalidProjectId)
        }
    }

    /// Returns the opaque identifier as a string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Error returned when a project identifier is not protocol-safe.
#[derive(Debug, Clone, Copy, thiserror::Error)]
#[error("project id must contain 1-128 ASCII letters, digits, '-' or '_'")]
pub struct InvalidProjectId;

/// Content revisions associated with one loaded project snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ProjectRevision {
    pub project: String,
    pub manifest: String,
    pub schema: String,
    pub data: String,
}

/// One opened Sora project.
#[derive(Debug)]
pub struct ProjectSession {
    id: ProjectId,
    manifest_path: PathBuf,
    revision: ProjectRevision,
}

impl ProjectSession {
    /// Creates a project session from an already resolved manifest path.
    pub fn new(id: ProjectId, manifest_path: PathBuf, revision: ProjectRevision) -> Self {
        Self {
            id,
            manifest_path,
            revision,
        }
    }

    pub fn id(&self) -> &ProjectId {
        &self.id
    }

    pub fn manifest_path(&self) -> &Path {
        &self.manifest_path
    }

    pub fn revision(&self) -> &ProjectRevision {
        &self.revision
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_ids_reject_values_that_are_not_protocol_safe() {
        assert!(ProjectId::new("").is_err());
        assert!(ProjectId::new("has spaces").is_err());
        assert!(ProjectId::new("../outside").is_err());
        assert!(ProjectId::new("project_01-client").is_ok());
    }
}
