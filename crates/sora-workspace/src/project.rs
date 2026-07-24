use std::{
    path::{Path, PathBuf},
    sync::RwLock,
};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{ProjectManifest, ProjectRuntime, RuntimeOptions, revision::calculate_revision};

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
pub struct ProjectSession {
    id: ProjectId,
    manifest_path: PathBuf,
    manifest: ProjectManifest,
    runtime: ProjectRuntime,
    revision: RwLock<ProjectRevision>,
}

impl ProjectSession {
    /// Opens and snapshots a project manifest and its runtime extensions.
    pub fn open(
        id: ProjectId,
        manifest_path: impl AsRef<Path>,
        options: RuntimeOptions,
    ) -> anyhow::Result<Self> {
        let manifest_path = manifest_path.as_ref().canonicalize()?;
        let manifest = ProjectManifest::load(&manifest_path)?;
        let runtime = ProjectRuntime::load_with_manifest(
            Some(&manifest_path),
            Some(manifest.clone()),
            options,
        )?;
        let revision = calculate_revision(&manifest_path, &manifest)?;
        Ok(Self {
            id,
            manifest_path,
            manifest,
            runtime,
            revision: RwLock::new(revision),
        })
    }

    pub fn id(&self) -> &ProjectId {
        &self.id
    }

    pub fn manifest_path(&self) -> &Path {
        &self.manifest_path
    }

    pub fn manifest(&self) -> &ProjectManifest {
        &self.manifest
    }

    pub fn runtime(&self) -> &ProjectRuntime {
        &self.runtime
    }

    pub fn revision(&self) -> ProjectRevision {
        self.revision
            .read()
            .expect("project revision lock should not be poisoned")
            .clone()
    }

    pub(crate) fn refresh_revision(&self) -> anyhow::Result<ProjectRevision> {
        let manifest = ProjectManifest::load(&self.manifest_path)?;
        let revision = calculate_revision(&self.manifest_path, &manifest)?;
        *self
            .revision
            .write()
            .map_err(|_| anyhow::anyhow!("project revision lock is poisoned"))? = revision.clone();
        Ok(revision)
    }
}

impl std::fmt::Debug for ProjectSession {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ProjectSession")
            .field("id", &self.id)
            .field("manifest_path", &self.manifest_path)
            .field("revision", &self.revision())
            .finish_non_exhaustive()
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
