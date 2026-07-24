use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};

use base64::{Engine as _, engine::general_purpose::STANDARD};
use rmcp::model::{ReadResourceResult, ResourceContents};
use sora_workspace::{BuildReport, ProjectId};
use uuid::Uuid;

const ARTIFACT_TTL: Duration = Duration::from_secs(30 * 60);
const MAX_ARTIFACTS: usize = 128;
const MAX_ARTIFACT_BYTES: usize = 16 * 1024 * 1024;
const MAX_TOTAL_BYTES: usize = 64 * 1024 * 1024;

#[derive(Debug, Clone)]
pub(crate) struct ArtifactDescriptor {
    pub artifact_id: String,
    pub name: String,
    pub mime_type: String,
    pub size: usize,
}

#[derive(Debug, Clone)]
struct StoredArtifact {
    sequence: u64,
    owner: String,
    project_id: ProjectId,
    mime_type: String,
    bytes: Arc<[u8]>,
    expires_at: Instant,
}

#[derive(Debug, Default)]
struct ArtifactState {
    next_sequence: u64,
    artifacts: BTreeMap<String, StoredArtifact>,
}

#[derive(Debug, Default)]
pub(crate) struct ArtifactStore {
    state: RwLock<ArtifactState>,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum ArtifactError {
    #[error("artifact store lock is poisoned")]
    StatePoisoned,
    #[error("artifact path resolves outside the project")]
    OutsideProject,
    #[error("artifact path contains a symbolic link")]
    SymbolicLink,
    #[error("artifact exceeds the {MAX_ARTIFACT_BYTES} byte size limit")]
    TooLarge,
    #[error("artifact store capacity is exhausted")]
    Capacity,
    #[error("artifact is unknown, expired, or belongs to another authorization context")]
    NotFound,
    #[error("failed to read artifact `{path}`")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

impl ArtifactStore {
    pub fn register_build(
        &self,
        owner: &str,
        project_id: &ProjectId,
        project_root: &Path,
        report: &BuildReport,
    ) -> Result<Vec<ArtifactDescriptor>, ArtifactError> {
        let root = project_root
            .canonicalize()
            .map_err(|source| ArtifactError::Read {
                path: project_root.to_path_buf(),
                source,
            })?;
        let mut files = Vec::new();
        for artifact in &report.artifacts {
            collect_files(&root, &artifact.path, &mut files)?;
        }
        files.sort();
        files.dedup();

        let mut pending: Vec<(String, String, Arc<[u8]>)> = Vec::with_capacity(files.len());
        for path in files {
            let bytes = fs::read(&path).map_err(|source| ArtifactError::Read {
                path: path.clone(),
                source,
            })?;
            if bytes.len() > MAX_ARTIFACT_BYTES {
                return Err(ArtifactError::TooLarge);
            }
            let name = path
                .strip_prefix(&root)
                .map_err(|_| ArtifactError::OutsideProject)?
                .to_string_lossy()
                .replace('\\', "/");
            pending.push((name, mime_type(&path).to_owned(), Arc::<[u8]>::from(bytes)));
        }

        let mut state = self
            .state
            .write()
            .map_err(|_| ArtifactError::StatePoisoned)?;
        prune(&mut state);
        let pending_bytes = pending
            .iter()
            .map(|(_, _, bytes)| bytes.len())
            .sum::<usize>();
        while state.artifacts.len() + pending.len() > MAX_ARTIFACTS
            || total_bytes(&state) + pending_bytes > MAX_TOTAL_BYTES
        {
            let Some(oldest) = state
                .artifacts
                .iter()
                .min_by_key(|(_, artifact)| artifact.sequence)
                .map(|(id, _)| id.clone())
            else {
                return Err(ArtifactError::Capacity);
            };
            state.artifacts.remove(&oldest);
        }

        let mut descriptors = Vec::with_capacity(pending.len());
        for (name, mime_type, bytes) in pending {
            state.next_sequence = state.next_sequence.saturating_add(1);
            let sequence = state.next_sequence;
            let artifact_id = format!("artifact:{}", Uuid::new_v4());
            let size = bytes.len();
            state.artifacts.insert(
                artifact_id.clone(),
                StoredArtifact {
                    sequence,
                    owner: owner.to_owned(),
                    project_id: project_id.clone(),
                    mime_type: mime_type.clone(),
                    bytes,
                    expires_at: Instant::now() + ARTIFACT_TTL,
                },
            );
            descriptors.push(ArtifactDescriptor {
                artifact_id,
                name,
                mime_type,
                size,
            });
        }
        Ok(descriptors)
    }

    pub fn read(
        &self,
        owner: &str,
        project_id: &ProjectId,
        artifact_id: &str,
        uri: &str,
    ) -> Result<ReadResourceResult, ArtifactError> {
        let mut state = self
            .state
            .write()
            .map_err(|_| ArtifactError::StatePoisoned)?;
        prune(&mut state);
        let artifact = state
            .artifacts
            .get(artifact_id)
            .filter(|artifact| artifact.owner == owner && &artifact.project_id == project_id)
            .ok_or(ArtifactError::NotFound)?;
        let contents = if artifact.mime_type.starts_with("text/")
            || matches!(
                artifact.mime_type.as_str(),
                "application/json" | "application/toml" | "application/yaml"
            ) {
            match std::str::from_utf8(&artifact.bytes) {
                Ok(text) => ResourceContents::text(text, uri).with_mime_type(&artifact.mime_type),
                Err(_) => ResourceContents::blob(STANDARD.encode(&artifact.bytes), uri)
                    .with_mime_type(&artifact.mime_type),
            }
        } else {
            ResourceContents::blob(STANDARD.encode(&artifact.bytes), uri)
                .with_mime_type(&artifact.mime_type)
        };
        Ok(ReadResourceResult::new(vec![contents]))
    }

    pub fn list_ids(
        &self,
        owner: &str,
        project_id: &ProjectId,
    ) -> Result<Vec<String>, ArtifactError> {
        let mut state = self
            .state
            .write()
            .map_err(|_| ArtifactError::StatePoisoned)?;
        prune(&mut state);
        Ok(state
            .artifacts
            .iter()
            .filter(|(_, artifact)| artifact.owner == owner && &artifact.project_id == project_id)
            .map(|(id, _)| id.clone())
            .collect())
    }
}

fn collect_files(root: &Path, path: &Path, files: &mut Vec<PathBuf>) -> Result<(), ArtifactError> {
    let canonical = path.canonicalize().map_err(|source| ArtifactError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    if !canonical.starts_with(root) {
        return Err(ArtifactError::OutsideProject);
    }
    let metadata = fs::symlink_metadata(path).map_err(|source| ArtifactError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    if metadata.file_type().is_symlink() {
        return Err(ArtifactError::SymbolicLink);
    }
    if metadata.is_file() {
        files.push(canonical);
        return Ok(());
    }
    for entry in fs::read_dir(&canonical).map_err(|source| ArtifactError::Read {
        path: canonical.clone(),
        source,
    })? {
        let entry = entry.map_err(|source| ArtifactError::Read {
            path: canonical.clone(),
            source,
        })?;
        collect_files(root, &entry.path(), files)?;
    }
    Ok(())
}

fn prune(state: &mut ArtifactState) {
    let now = Instant::now();
    state
        .artifacts
        .retain(|_, artifact| artifact.expires_at > now);
}

fn total_bytes(state: &ArtifactState) -> usize {
    state
        .artifacts
        .values()
        .map(|artifact| artifact.bytes.len())
        .sum()
}

fn mime_type(path: &Path) -> &'static str {
    match path.extension().and_then(|extension| extension.to_str()) {
        Some("json") => "application/json",
        Some("toml") => "application/toml",
        Some("yaml" | "yml") => "application/yaml",
        Some("md") => "text/markdown",
        Some(
            "rs" | "kt" | "kts" | "cs" | "java" | "scala" | "go" | "dart" | "gd" | "c" | "h" | "cc"
            | "cpp" | "hpp" | "ts" | "js" | "mjs" | "d.ts" | "erl" | "hrl" | "lua" | "py" | "proto",
        ) => "text/plain",
        Some("xlsx") => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        Some("cbor") => "application/cbor",
        Some("pb") => "application/x-protobuf",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sora_workspace::{BuildArtifact, BuildArtifactKind};

    #[test]
    fn artifacts_are_bound_to_project_and_authorization() {
        let root = std::env::temp_dir().join(format!("sora-artifacts-{}", Uuid::new_v4()));
        fs::create_dir_all(root.join("generated")).unwrap();
        let path = root.join("generated/config.json");
        fs::write(&path, br#"{"ok":true}"#).unwrap();
        let project = ProjectId::new("project-1").unwrap();
        let store = ArtifactStore::default();
        let descriptors = store
            .register_build(
                "owner-a",
                &project,
                &root,
                &BuildReport {
                    artifacts: vec![BuildArtifact {
                        kind: BuildArtifactKind::Export {
                            format: "json".to_owned(),
                        },
                        path,
                    }],
                },
            )
            .unwrap();

        assert_eq!(descriptors.len(), 1);
        assert!(
            store
                .read(
                    "owner-a",
                    &project,
                    &descriptors[0].artifact_id,
                    "sora://artifact"
                )
                .is_ok()
        );
        assert!(
            store
                .read(
                    "owner-b",
                    &project,
                    &descriptors[0].artifact_id,
                    "sora://artifact"
                )
                .is_err()
        );
        let _ = fs::remove_dir_all(root);
    }
}
