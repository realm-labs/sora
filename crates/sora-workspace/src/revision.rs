use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Result as IoResult,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use sora_config_format::load_document;

use crate::{
    ProjectManifest, ProjectRevision,
    scripts::{ScriptSources, format_digest},
};

pub(crate) fn calculate_revision(
    manifest_path: &Path,
    manifest: &ProjectManifest,
) -> Result<ProjectRevision> {
    calculate_revision_with_inputs(manifest_path, manifest, &RevisionInputs::default())
}

/// Root declarations and executable contents come from the loaded snapshot;
/// all other schema and data inputs retain their normal filesystem semantics.
pub(crate) fn calculate_revision_with_snapshot(
    manifest_path: &Path,
    manifest: &ProjectManifest,
    manifest_source: &str,
    scripts: &ScriptSources,
) -> Result<ProjectRevision> {
    let project_dir = manifest_path.parent().unwrap_or_else(|| Path::new("."));
    let mut inputs = RevisionInputs::default();
    inputs
        .contents
        .insert(manifest_path.to_path_buf(), manifest_source.as_bytes());
    // Scripts may also be encountered as ordinary files under the data root.
    for source in scripts.parsers.iter().chain(&scripts.type_mappings) {
        inputs
            .contents
            .insert(source.path().to_path_buf(), source.source().as_bytes());
    }
    for source in &scripts.source_loaders {
        inputs
            .contents
            .insert(project_dir.join(source.path()), source.source().as_bytes());
    }
    calculate_revision_with_inputs(manifest_path, manifest, &inputs)
}

#[derive(Default)]
struct RevisionInputs<'a> {
    contents: BTreeMap<PathBuf, &'a [u8]>,
}

impl RevisionInputs<'_> {
    fn read(&self, path: &Path) -> Result<Cow<'_, [u8]>> {
        match self.contents.get(path) {
            Some(bytes) => Ok(Cow::Borrowed(bytes)),
            None => fs::read(path)
                .map(Cow::Owned)
                .with_context(|| format!("failed to read revision input `{}`", path.display())),
        }
    }
}

fn calculate_revision_with_inputs(
    manifest_path: &Path,
    manifest: &ProjectManifest,
    inputs: &RevisionInputs<'_>,
) -> Result<ProjectRevision> {
    let manifest_digest = digest_files(
        manifest_path.parent(),
        &[manifest_path.to_path_buf()],
        inputs,
    )?;
    let schema_files = schema_files(manifest_path, manifest, inputs)?;
    let schema_digest = digest_files(manifest_path.parent(), &schema_files, inputs)?;
    let project_dir = manifest_path.parent().unwrap_or_else(|| Path::new("."));
    let data_root = manifest
        .build
        .data_root
        .as_deref()
        .unwrap_or_else(|| Path::new("data"));
    let data_root = resolve_path(project_dir, data_root);
    let data_files = collect_files(&data_root)?;
    let data_content_digest = digest_files(Some(&data_root), &data_files, inputs)?;
    let source_loader_files = manifest
        .source_loaders
        .scripts
        .iter()
        .map(|path| resolve_path(project_dir, path))
        .collect::<Vec<_>>();
    let data_digest = if source_loader_files.is_empty() {
        data_content_digest
    } else {
        let source_loader_digest = digest_files(Some(project_dir), &source_loader_files, inputs)?;
        digest_parts([
            data_content_digest.as_bytes(),
            source_loader_digest.as_bytes(),
        ])
    };
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

fn schema_files(
    manifest_path: &Path,
    manifest: &ProjectManifest,
    inputs: &RevisionInputs<'_>,
) -> Result<Vec<PathBuf>> {
    let mut visited = BTreeSet::new();
    let mut files = Vec::new();
    collect_schema_files(
        manifest_path,
        Some(manifest),
        inputs,
        &mut visited,
        &mut files,
    )?;
    files.sort();
    Ok(files)
}

fn collect_schema_files(
    path: &Path,
    root_manifest: Option<&ProjectManifest>,
    inputs: &RevisionInputs<'_>,
    visited: &mut BTreeSet<PathBuf>,
    files: &mut Vec<PathBuf>,
) -> Result<()> {
    // A captured root is already canonical and may have been deleted since prepare.
    let key = if root_manifest.is_some() && inputs.contents.contains_key(path) {
        path.to_path_buf()
    } else {
        path.canonicalize().with_context(|| {
            format!(
                "failed to resolve schema document while calculating revision `{}`",
                path.display()
            )
        })?
    };
    if !visited.insert(key.clone()) {
        bail!(
            "schema include cycle or duplicate include `{}`",
            path.display()
        );
    }
    let document;
    let (views, includes) = match root_manifest {
        Some(manifest) => (
            manifest.project.views.as_slice(),
            manifest.includes.as_slice(),
        ),
        None => {
            document = load_document::<IncludeDocument>(&key).with_context(|| {
                format!("failed to inspect schema includes in `{}`", path.display())
            })?;
            (
                document
                    .project
                    .as_ref()
                    .map_or(&[][..], |project| project.views.as_slice()),
                document.includes.as_slice(),
            )
        }
    };
    files.push(key.clone());
    let parent = key.parent().unwrap_or_else(|| Path::new("."));
    for view in views {
        collect_schema_dependency(&parent.join(view), visited, files)?;
    }
    for include in includes {
        collect_schema_files(&parent.join(include), None, inputs, visited, files)?;
    }
    Ok(())
}

fn collect_schema_dependency(
    path: &Path,
    visited: &mut BTreeSet<PathBuf>,
    files: &mut Vec<PathBuf>,
) -> Result<()> {
    let key = path.canonicalize().with_context(|| {
        format!(
            "failed to resolve schema dependency while calculating revision `{}`",
            path.display()
        )
    })?;
    if !visited.insert(key.clone()) {
        bail!("duplicate schema dependency `{}`", path.display());
    }
    files.push(key);
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
            .collect::<IoResult<Vec<_>>>()
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

fn digest_files(
    base: Option<&Path>,
    files: &[PathBuf],
    inputs: &RevisionInputs<'_>,
) -> Result<String> {
    let mut hasher = Sha256::new();
    for path in files {
        let identity = base
            .and_then(|base| path.strip_prefix(base).ok())
            .unwrap_or(path);
        hasher.update(normalized_path(identity).as_bytes());
        hasher.update([0]);
        let content = inputs.read(path)?;
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
    project: Option<RevisionProject>,
    #[serde(default)]
    includes: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct RevisionProject {
    #[serde(default)]
    views: Vec<String>,
}

#[cfg(test)]
mod tests {
    use std::{env::temp_dir as system_temp_dir, process::id as process_id};
    use std::{
        sync::atomic::{AtomicU64, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    use uuid::Uuid;

    use super::*;
    use crate::{project::capture_manifest, scripts::capture_project_scripts};

    static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn snapshot_revisions_preserve_the_existing_format_without_reopening_scripts() {
        let root = system_temp_dir().join(format!("sora-revision-snapshot-{}", Uuid::new_v4()));
        fs::create_dir_all(root.join("data")).unwrap();
        let path = root.join("project.toml");
        fs::write(&path, "project = { id = 'revision', views = ['view.toml'] }\nincludes = ['schema.toml']\n[parsers]\nscripts = ['data/parser.lua']\n[type_mappings]\nscripts = ['data/mapping.lua']\n[source_loaders]\nscripts = ['data/loader.lua']\n").unwrap();
        fs::write(root.join("schema.toml"), "includes = []\n").unwrap();
        fs::write(root.join("view.toml"), "view = {}\n").unwrap();
        fs::write(root.join("data/input.txt"), "data").unwrap();
        for name in ["parser.lua", "mapping.lua", "loader.lua"] {
            fs::write(root.join("data").join(name), "return {}\n").unwrap();
        }
        let path = path.canonicalize().unwrap();
        let (manifest, source) = capture_manifest(&path).unwrap();
        let scripts = capture_project_scripts(&path, &manifest, None).unwrap().0;
        let expected = calculate_revision(&path, &manifest).unwrap();
        assert_eq!(
            calculate_revision_with_snapshot(&path, &manifest, &source, &scripts).unwrap(),
            expected
        );
        for name in ["parser.lua", "mapping.lua", "loader.lua"] {
            fs::write(root.join("data").join(name), "error('replacement')\n").unwrap();
        }
        fs::write(&path, "invalid manifest replacement").unwrap();
        assert_eq!(
            calculate_revision_with_snapshot(&path, &manifest, &source, &scripts).unwrap(),
            expected
        );
        fs::remove_file(&path).unwrap();
        assert_eq!(
            calculate_revision_with_snapshot(&path, &manifest, &source, &scripts).unwrap(),
            expected
        );
        fs::write(root.join("data/input.txt"), "changed data").unwrap();
        let changed =
            calculate_revision_with_snapshot(&path, &manifest, &source, &scripts).unwrap();
        assert_eq!(changed.manifest, expected.manifest);
        assert_eq!(changed.schema, expected.schema);
        assert_ne!(changed.data, expected.data);
        fs::write(
            root.join("schema.toml"),
            "includes = []\n# changed schema\n",
        )
        .unwrap();
        let changed_schema =
            calculate_revision_with_snapshot(&path, &manifest, &source, &scripts).unwrap();
        assert_eq!(changed_schema.manifest, expected.manifest);
        assert_ne!(changed_schema.schema, expected.schema);
        assert_eq!(changed_schema.data, changed.data);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn revisions_change_only_for_the_affected_content_group() {
        let base = temp_dir();
        fs::create_dir_all(base.join("schema")).unwrap();
        fs::create_dir_all(base.join("views")).unwrap();
        fs::create_dir_all(base.join("data")).unwrap();
        let project = base.join("project.toml");
        fs::write(
            &project,
            "project = { id = \"demo\", views = [\"views/default.toml\"] }\ngroups = { common = { default = true } }\nviews = { default = { contract = \"demo/default\", groups = [\"common\"] } }\nincludes = [\"schema/items.toml\"]\n",
        )
        .unwrap();
        fs::write(
            base.join("views/default.toml"),
            "name = \"external\"\ncontract = \"demo/external\"\ngroups = [\"common\"]\n",
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
        system_temp_dir().join(format!(
            "sora-workspace-revision-{}-{time}-{nonce}",
            process_id()
        ))
    }
}
