use std::{
    fmt::{Debug, Formatter, Result as FmtResult},
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, RwLock},
};

use anyhow::{Context, Result as AnyhowResult, anyhow};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sora_config_format::parse_document;

use crate::{
    ProjectManifest, ProjectRuntime, RuntimeOptions,
    mutation::recover_transactions,
    revision::{calculate_revision, calculate_revision_with_snapshot},
    scripts::capture_project_scripts,
};

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

    pub(crate) fn generated(sequence: u64) -> Self {
        Self(format!("project-{sequence}"))
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
    pub(crate) write_lock: Mutex<()>,
}

impl ProjectSession {
    /// Opens and snapshots a project manifest and its runtime extensions.
    pub fn open(
        id: ProjectId,
        manifest_path: impl AsRef<Path>,
        options: RuntimeOptions,
    ) -> AnyhowResult<Self> {
        let manifest_path = manifest_path.as_ref().canonicalize()?;
        if let Some(project_root) = manifest_path.parent() {
            recover_transactions(project_root)?;
        }
        let (manifest, manifest_source) = capture_manifest(&manifest_path)?;
        let scripts = capture_project_scripts(&manifest_path, &manifest, None)?
            .0
            .with_extras(&options.parser_scripts, &options.type_mapping_scripts)?;
        let runtime = ProjectRuntime::load_from_sources(Some(manifest.clone()), scripts, options)?;
        Self::open_snapshot(id, manifest_path, manifest, &manifest_source, runtime)
    }

    pub(crate) fn open_snapshot(
        id: ProjectId,
        manifest_path: PathBuf,
        manifest: ProjectManifest,
        manifest_source: &str,
        runtime: ProjectRuntime,
    ) -> AnyhowResult<Self> {
        let revision = calculate_revision_with_snapshot(
            &manifest_path,
            &manifest,
            manifest_source,
            runtime.scripts(),
        )?;
        Ok(Self {
            id,
            manifest_path,
            manifest,
            runtime,
            revision: RwLock::new(revision),
            write_lock: Mutex::new(()),
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
        match self.revision.read() {
            Ok(revision) => revision.clone(),
            Err(poisoned) => poisoned.into_inner().clone(),
        }
    }

    pub(crate) fn refresh_revision(&self) -> AnyhowResult<ProjectRevision> {
        let manifest = ProjectManifest::load(&self.manifest_path)?;
        let revision = calculate_revision(&self.manifest_path, &manifest)?;
        *self
            .revision
            .write()
            .map_err(|_| anyhow!("project revision lock is poisoned"))? = revision.clone();
        Ok(revision)
    }
}

pub(crate) fn capture_manifest(path: &Path) -> AnyhowResult<(ProjectManifest, Arc<str>)> {
    let source = fs::read_to_string(path)
        .with_context(|| format!("failed to read project manifest `{}`", path.display()))?;
    let manifest = parse_document(path, &source)
        .with_context(|| format!("failed to load build config from `{}`", path.display()))?;
    Ok((manifest, source.into()))
}

impl Debug for ProjectSession {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
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
