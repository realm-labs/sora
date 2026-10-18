use std::{
    fmt, fs,
    path::{Component, Path, PathBuf},
    result::Result as StdResult,
    sync::Arc,
};

use anyhow::{Context, Result};
use sha2::{Digest, Sha256};

use crate::{ProjectManifest, ProjectScript, ProjectScriptKind, WorkspaceError};

/// A diagnostic/dependency label bound to immutable executable source bytes.
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct ScriptSource {
    path: PathBuf,
    source: Arc<str>,
}

impl ScriptSource {
    pub(crate) fn new(path: PathBuf, source: String) -> Self {
        Self {
            path,
            source: source.into(),
        }
    }

    pub(crate) fn read(path: &Path) -> Result<Self> {
        let source = fs::read_to_string(path)
            .with_context(|| format!("failed to read Lua script `{}`", path.display()))?;
        Ok(Self::new(path.to_path_buf(), source))
    }

    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    pub(crate) fn source(&self) -> &str {
        &self.source
    }
}

impl fmt::Debug for ScriptSource {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ScriptSource")
            .field("path", &self.path)
            .finish_non_exhaustive()
    }
}

/// Order and multiplicity are significant within each extension category.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ScriptSources {
    pub(crate) parsers: Vec<ScriptSource>,
    pub(crate) type_mappings: Vec<ScriptSource>,
    pub(crate) source_loaders: Vec<ScriptSource>,
}

impl ScriptSources {
    pub(crate) fn with_extras(
        mut self,
        parsers: &[PathBuf],
        type_mappings: &[PathBuf],
    ) -> Result<Self> {
        for path in parsers {
            self.parsers.push(read_extra(path)?);
        }
        for path in type_mappings {
            self.type_mappings.push(read_extra(path)?);
        }
        Ok(self)
    }
}

fn read_extra(path: &Path) -> Result<ScriptSource> {
    let canonical = path
        .canonicalize()
        .with_context(|| format!("failed to resolve Lua script `{}`", path.display()))?;
    ScriptSource::read(&canonical)
}

/// Captures without evaluating Lua. Inspection digests describe only these bytes.
pub(crate) fn capture_project_scripts(
    manifest_path: &Path,
    manifest: &ProjectManifest,
    allowed_root: Option<&Path>,
) -> StdResult<(ScriptSources, Vec<ProjectScript>), WorkspaceError> {
    let project_dir = manifest_path.parent().unwrap_or_else(|| Path::new("."));
    let mut sources = ScriptSources::default();
    let mut inspection = Vec::new();
    let script_groups = [
        (
            ProjectScriptKind::Parser,
            &manifest.parsers.scripts,
            &mut sources.parsers,
        ),
        (
            ProjectScriptKind::SourceLoader,
            &manifest.source_loaders.scripts,
            &mut sources.source_loaders,
        ),
        (
            ProjectScriptKind::TypeMapping,
            &manifest.type_mappings.scripts,
            &mut sources.type_mappings,
        ),
    ];
    for (kind, declarations, destination) in script_groups {
        for declared in declarations {
            if kind == ProjectScriptKind::SourceLoader
                && (declared.is_absolute()
                    || declared.components().any(|component| {
                        !matches!(component, Component::Normal(_) | Component::CurDir)
                    }))
            {
                return Err(WorkspaceError::InvalidSourceLoaderScriptPath(
                    declared.clone(),
                ));
            }
            let path = project_dir.join(declared);
            let canonical =
                path.canonicalize()
                    .map_err(|source| WorkspaceError::ReadProjectScript {
                        path: path.clone(),
                        source,
                    })?;
            if allowed_root.is_some_and(|root| !canonical.starts_with(root)) {
                return Err(WorkspaceError::ProjectScriptOutsideRoot { path });
            }
            let source = fs::read_to_string(&canonical).map_err(|source| {
                WorkspaceError::ReadProjectScript {
                    path: canonical.clone(),
                    source,
                }
            })?;
            let relative = allowed_root
                .and_then(|root| canonical.strip_prefix(root).ok())
                .unwrap_or(&canonical);
            inspection.push(ProjectScript {
                kind,
                path: normalized_path(relative),
                digest: format_digest(Sha256::digest(source.as_bytes())),
            });
            let label = if kind == ProjectScriptKind::SourceLoader {
                declared.clone()
            } else {
                canonical
            };
            destination.push(ScriptSource::new(label, source));
        }
    }
    let script_kind_rank = |kind| match kind {
        ProjectScriptKind::Parser => 0,
        ProjectScriptKind::SourceLoader => 1,
        ProjectScriptKind::TypeMapping => 2,
    };
    // Preserve the established metadata ordering without changing execution order.
    inspection.sort_by(|left, right| {
        script_kind_rank(left.kind)
            .cmp(&script_kind_rank(right.kind))
            .then_with(|| left.path.cmp(&right.path))
    });
    inspection.dedup();
    Ok((sources, inspection))
}

pub(crate) fn format_digest(digest: impl AsRef<[u8]>) -> String {
    let hex_digest = digest
        .as_ref()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    format!("sha256:{hex_digest}")
}

fn normalized_path(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}
