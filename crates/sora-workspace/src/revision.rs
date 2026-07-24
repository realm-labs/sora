use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::{ProjectManifest, ProjectRevision};

pub(crate) fn calculate_revision(
    manifest_path: &Path,
    manifest: &ProjectManifest,
) -> Result<ProjectRevision> {
    let manifest_digest = digest_files(manifest_path.parent(), &[manifest_path.to_path_buf()])?;
    let schema_files = schema_files(manifest_path)?;
    let schema_digest = digest_files(manifest_path.parent(), &schema_files)?;
    let project_dir = manifest_path.parent().unwrap_or_else(|| Path::new("."));
    let data_root = manifest
        .build
        .data_root
        .as_deref()
        .unwrap_or_else(|| Path::new("data"));
    let data_root = resolve_path(project_dir, data_root);
    let data_files = collect_files(&data_root)?;
    let data_digest = digest_files(Some(&data_root), &data_files)?;
    let project_digest = digest_parts([
        manifest_digest.as_bytes(),
        schema_digest.as_bytes(),
        data_digest.as_bytes(),
    ]);
    Ok(ProjectRevision {
        project: project_digest,
        manifest: manifest_digest,
        schema: schema_digest,
        data: data_digest,
    })
}

fn schema_files(manifest_path: &Path) -> Result<Vec<PathBuf>> {
    let mut visited = BTreeSet::new();
    let mut files = Vec::new();
    collect_schema_files(manifest_path, &mut visited, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_schema_files(
    path: &Path,
    visited: &mut BTreeSet<PathBuf>,
    files: &mut Vec<PathBuf>,
) -> Result<()> {
    let key = path.canonicalize().with_context(|| {
        format!(
            "failed to resolve schema document while calculating revision `{}`",
            path.display()
        )
    })?;
    if !visited.insert(key.clone()) {
        bail!(
            "schema include cycle or duplicate include `{}`",
            path.display()
        );
    }
    let document: IncludeDocument = sora_config_format::load_document(&key)
        .with_context(|| format!("failed to inspect schema includes in `{}`", path.display()))?;
    files.push(key.clone());
    let parent = key.parent().unwrap_or_else(|| Path::new("."));
    for include in document.includes {
        collect_schema_files(&parent.join(include), visited, files)?;
    }
    Ok(())
}

fn collect_files(root: &Path) -> Result<Vec<PathBuf>> {
    if !root.exists() {
        return Ok(Vec::new());
    }
    if !root.is_dir() {
        bail!("data root `{}` is not a directory", root.display());
    }
    let mut pending = vec![root.to_path_buf()];
    let mut files = Vec::new();
    while let Some(directory) = pending.pop() {
        let entries = fs::read_dir(&directory)
            .with_context(|| format!("failed to read data directory `{}`", directory.display()))?;
        let mut paths = entries
            .map(|entry| entry.map(|entry| entry.path()))
            .collect::<std::io::Result<Vec<_>>>()
            .with_context(|| {
                format!(
                    "failed to enumerate data directory `{}`",
                    directory.display()
                )
            })?;
        paths.sort();
        for path in paths.into_iter().rev() {
            if path.is_dir() {
                if path.file_name().and_then(|name| name.to_str()) == Some(".sora") {
                    continue;
                }
                pending.push(path);
            } else if path.is_file() {
                files.push(path);
            }
        }
    }
    files.sort();
    Ok(files)
}

fn digest_files(base: Option<&Path>, files: &[PathBuf]) -> Result<String> {
    let mut hasher = Sha256::new();
    for path in files {
        let identity = base
            .and_then(|base| path.strip_prefix(base).ok())
            .unwrap_or(path);
        hasher.update(normalized_path(identity).as_bytes());
        hasher.update([0]);
        let content = fs::read(path)
            .with_context(|| format!("failed to read revision input `{}`", path.display()))?;
        hasher.update((content.len() as u64).to_le_bytes());
        hasher.update(content);
    }
    Ok(format_digest(hasher.finalize()))
}

fn digest_parts<'a>(parts: impl IntoIterator<Item = &'a [u8]>) -> String {
    let mut hasher = Sha256::new();
    for part in parts {
        hasher.update((part.len() as u64).to_le_bytes());
        hasher.update(part);
    }
    format_digest(hasher.finalize())
}

fn format_digest(digest: impl AsRef<[u8]>) -> String {
    format!(
        "sha256:{}",
        digest
            .as_ref()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    )
}

fn normalized_path(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn resolve_path(base: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        base.join(path)
    }
}

#[derive(Debug, Deserialize)]
struct IncludeDocument {
    #[serde(default)]
    includes: Vec<String>,
}

#[cfg(test)]
mod tests {
    use std::{
        sync::atomic::{AtomicU64, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::*;

    static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn revisions_change_only_for_the_affected_content_group() {
        let base = temp_dir();
        fs::create_dir_all(base.join("schema")).unwrap();
        fs::create_dir_all(base.join("data")).unwrap();
        let project = base.join("project.toml");
        fs::write(
            &project,
            "package = \"demo\"\nincludes = [\"schema/items.toml\"]\n",
        )
        .unwrap();
        fs::write(base.join("schema/items.toml"), "enums = []\n").unwrap();
        fs::write(base.join("data/items.json"), "[]\n").unwrap();
        let manifest = ProjectManifest::load(&project).unwrap();

        let first = calculate_revision(&project, &manifest).unwrap();
        fs::write(base.join("data/items.json"), "[{\"id\":1}]\n").unwrap();
        let second = calculate_revision(&project, &manifest).unwrap();

        assert_eq!(first.manifest, second.manifest);
        assert_eq!(first.schema, second.schema);
        assert_ne!(first.data, second.data);
        assert_ne!(first.project, second.project);
        let _ = fs::remove_dir_all(base);
    }

    fn temp_dir() -> PathBuf {
        let nonce = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "sora-workspace-revision-{}-{time}-{nonce}",
            std::process::id()
        ))
    }
}
