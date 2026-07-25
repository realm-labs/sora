use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    sync::{
        Arc, RwLock,
        atomic::{AtomicU64, Ordering},
    },
};

use schemars::JsonSchema;
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::{
    ProjectId, ProjectSession, RuntimeOptions,
    mutation::{
        DataMutationCoordinator, ExcelSyncCoordinator, MutationCoordinator, ProjectInitCoordinator,
    },
};

/// Registry and coordination point for opened Sora projects.
#[derive(Debug, Default)]
pub struct WorkspaceService {
    sessions: RwLock<BTreeMap<ProjectId, Arc<ProjectSession>>>,
    roots: RwLock<BTreeMap<String, WorkspaceRoot>>,
    next_project_id: AtomicU64,
    pub(crate) mutation: MutationCoordinator,
    pub(crate) data_mutation: DataMutationCoordinator,
    pub(crate) excel_sync: ExcelSyncCoordinator,
    pub(crate) project_init: ProjectInitCoordinator,
}

impl WorkspaceService {
    /// Creates an empty workspace service.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds an allowed filesystem root after resolving symlinks.
    pub fn add_root(
        &self,
        name: impl Into<String>,
        path: impl AsRef<Path>,
    ) -> Result<WorkspaceRoot, WorkspaceError> {
        let name = name.into();
        if name.is_empty() {
            return Err(WorkspaceError::InvalidRootName);
        }
        let path = path
            .as_ref()
            .canonicalize()
            .map_err(|source| WorkspaceError::ResolveRoot {
                path: path.as_ref().to_path_buf(),
                source,
            })?;
        if !path.is_dir() {
            return Err(WorkspaceError::RootNotDirectory(path));
        }
        let root = WorkspaceRoot {
            id: format!("root-{}", stable_root_suffix(&path)),
            name,
            path,
        };
        self.roots
            .write()
            .map_err(|_| WorkspaceError::StatePoisoned)?
            .insert(root.id.clone(), root.clone());
        Ok(root)
    }

    /// Removes roots whose adapter-owned names use the given prefix.
    pub fn remove_roots_with_name_prefix(&self, prefix: &str) -> Result<(), WorkspaceError> {
        self.roots
            .write()
            .map_err(|_| WorkspaceError::StatePoisoned)?
            .retain(|_, root| !root.name.starts_with(prefix));
        Ok(())
    }

    pub(crate) fn root(&self, id: &str) -> Result<WorkspaceRoot, WorkspaceError> {
        self.roots
            .read()
            .map_err(|_| WorkspaceError::StatePoisoned)?
            .get(id)
            .cloned()
            .ok_or_else(|| WorkspaceError::UnknownRoot(id.to_owned()))
    }

    /// Discovers `project.toml` at each root and one non-hidden directory below it.
    pub fn discover_projects(&self) -> Result<Vec<ProjectCandidate>, WorkspaceError> {
        let roots = self
            .roots
            .read()
            .map_err(|_| WorkspaceError::StatePoisoned)?;
        let sessions = self
            .sessions
            .read()
            .map_err(|_| WorkspaceError::StatePoisoned)?;
        let opened = sessions
            .values()
            .map(|session| (session.manifest_path().to_path_buf(), session.id().clone()))
            .collect::<BTreeMap<_, _>>();
        let mut candidates = Vec::new();
        for root in roots.values() {
            for relative_manifest in discover_root_manifests(&root.path)? {
                let manifest_path = root.path.join(&relative_manifest);
                candidates.push(ProjectCandidate {
                    root_id: root.id.clone(),
                    root_name: root.name.clone(),
                    relative_manifest: normalized_relative_path(&relative_manifest),
                    project_id: opened.get(&manifest_path).cloned(),
                });
            }
        }
        candidates.sort_by(|left, right| {
            (&left.root_name, &left.relative_manifest)
                .cmp(&(&right.root_name, &right.relative_manifest))
        });
        Ok(candidates)
    }

    /// Opens a discovered manifest without accepting an arbitrary filesystem path.
    pub fn open_discovered_project(
        &self,
        root_id: &str,
        relative_manifest: &str,
        options: RuntimeOptions,
        trust_project_scripts: bool,
    ) -> Result<Arc<ProjectSession>, WorkspaceError> {
        let (root, canonical) = self.resolve_discovered_manifest(root_id, relative_manifest)?;
        let manifest = crate::ProjectManifest::load(&canonical).map_err(|source| {
            WorkspaceError::OpenProject {
                path: canonical.clone(),
                source,
            }
        })?;
        if let Some(existing) = self
            .sessions
            .read()
            .map_err(|_| WorkspaceError::StatePoisoned)?
            .values()
            .find(|session| session.manifest_path() == canonical)
            .cloned()
        {
            return Ok(existing);
        }
        if !trust_project_scripts
            && (!manifest.parsers.scripts.is_empty() || !manifest.type_mappings.scripts.is_empty())
        {
            return Err(WorkspaceError::UntrustedProjectScripts);
        }
        inspect_project_scripts(&root.path, &canonical, &manifest)?;
        let sequence = self.next_project_id.fetch_add(1, Ordering::Relaxed) + 1;
        let id = ProjectId::generated(sequence);
        self.open_project(id, canonical, options)
    }

    /// Inspects a discovered manifest without loading or executing project scripts.
    pub fn inspect_discovered_project(
        &self,
        root_id: &str,
        relative_manifest: &str,
    ) -> Result<DiscoveredProjectInspection, WorkspaceError> {
        let (root, canonical) = self.resolve_discovered_manifest(root_id, relative_manifest)?;
        let manifest = crate::ProjectManifest::load(&canonical).map_err(|source| {
            WorkspaceError::OpenProject {
                path: canonical.clone(),
                source,
            }
        })?;
        let scripts = inspect_project_scripts(&root.path, &canonical, &manifest)?;
        Ok(DiscoveredProjectInspection {
            root_id: root.id,
            relative_manifest: normalized_relative_path(&validate_relative_manifest(
                relative_manifest,
            )?),
            schema_id: manifest.project.id,
            requires_trust: !scripts.is_empty(),
            scripts,
        })
    }

    fn resolve_discovered_manifest(
        &self,
        root_id: &str,
        relative_manifest: &str,
    ) -> Result<(WorkspaceRoot, PathBuf), WorkspaceError> {
        let root = self
            .roots
            .read()
            .map_err(|_| WorkspaceError::StatePoisoned)?
            .get(root_id)
            .cloned()
            .ok_or_else(|| WorkspaceError::UnknownRoot(root_id.to_owned()))?;
        let relative = validate_relative_manifest(relative_manifest)?;
        let manifest_path = root.path.join(relative);
        let canonical =
            manifest_path
                .canonicalize()
                .map_err(|source| WorkspaceError::ResolveManifest {
                    path: manifest_path.clone(),
                    source,
                })?;
        if !canonical.starts_with(&root.path) {
            return Err(WorkspaceError::ManifestOutsideRoot);
        }
        Ok((root, canonical))
    }

    /// Opens and registers a project as one atomic workspace operation.
    pub fn open_project(
        &self,
        id: ProjectId,
        manifest_path: impl AsRef<Path>,
        options: RuntimeOptions,
    ) -> Result<Arc<ProjectSession>, WorkspaceError> {
        let manifest_path = manifest_path.as_ref();
        let session = ProjectSession::open(id, manifest_path, options).map_err(|source| {
            WorkspaceError::OpenProject {
                path: manifest_path.to_path_buf(),
                source,
            }
        })?;
        self.register(session)
    }

    /// Registers a session, rejecting duplicate project identifiers.
    pub fn register(&self, session: ProjectSession) -> Result<Arc<ProjectSession>, WorkspaceError> {
        let id = session.id().clone();
        let session = Arc::new(session);
        let mut sessions = self
            .sessions
            .write()
            .map_err(|_| WorkspaceError::StatePoisoned)?;
        if sessions.contains_key(&id) {
            return Err(WorkspaceError::DuplicateProject(id));
        }
        sessions.insert(id, Arc::clone(&session));
        Ok(session)
    }

    /// Returns a registered project session.
    pub fn project(&self, id: &ProjectId) -> Result<Arc<ProjectSession>, WorkspaceError> {
        self.sessions
            .read()
            .map_err(|_| WorkspaceError::StatePoisoned)?
            .get(id)
            .cloned()
            .ok_or_else(|| WorkspaceError::UnknownProject(id.clone()))
    }

    /// Lists project identifiers in deterministic order.
    pub fn project_ids(&self) -> Result<Vec<ProjectId>, WorkspaceError> {
        Ok(self
            .sessions
            .read()
            .map_err(|_| WorkspaceError::StatePoisoned)?
            .keys()
            .cloned()
            .collect())
    }
}

/// Application-level workspace failures.
#[derive(Debug, thiserror::Error)]
pub enum WorkspaceError {
    #[error("workspace root name must not be empty")]
    InvalidRootName,
    #[error("failed to resolve workspace root `{path}`")]
    ResolveRoot {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("workspace root `{0}` is not a directory")]
    RootNotDirectory(PathBuf),
    #[error("failed to enumerate workspace root `{path}`")]
    ReadRoot {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("unknown workspace root `{0}`")]
    UnknownRoot(String),
    #[error("project manifest must be a relative path ending in `project.toml`")]
    InvalidRelativeManifest,
    #[error("failed to resolve project manifest `{path}`")]
    ResolveManifest {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("project manifest resolves outside its allowed root")]
    ManifestOutsideRoot,
    #[error("project declares Lua scripts that have not been explicitly trusted")]
    UntrustedProjectScripts,
    #[error("project script `{path}` resolves outside its allowed root")]
    ProjectScriptOutsideRoot { path: PathBuf },
    #[error("failed to read project script `{path}`")]
    ReadProjectScript {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to open project manifest `{path}`")]
    OpenProject {
        path: std::path::PathBuf,
        #[source]
        source: anyhow::Error,
    },
    #[error("project `{0}` is already registered")]
    DuplicateProject(ProjectId),
    #[error("unknown project `{0}`")]
    UnknownProject(ProjectId),
    #[error("workspace state lock is poisoned")]
    StatePoisoned,
}

#[derive(Debug, Clone)]
pub struct WorkspaceRoot {
    id: String,
    name: String,
    path: PathBuf,
}

impl WorkspaceRoot {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn path(&self) -> &Path {
        &self.path
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct ProjectCandidate {
    pub root_id: String,
    pub root_name: String,
    pub relative_manifest: String,
    pub project_id: Option<ProjectId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct DiscoveredProjectInspection {
    pub root_id: String,
    pub relative_manifest: String,
    pub schema_id: String,
    pub requires_trust: bool,
    pub scripts: Vec<ProjectScript>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct ProjectScript {
    pub kind: ProjectScriptKind,
    pub path: String,
    pub digest: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProjectScriptKind {
    Parser,
    TypeMapping,
}

fn inspect_project_scripts(
    allowed_root: &Path,
    manifest_path: &Path,
    manifest: &crate::ProjectManifest,
) -> Result<Vec<ProjectScript>, WorkspaceError> {
    let project_root = manifest_path.parent().unwrap_or_else(|| Path::new("."));
    let mut scripts = manifest
        .parsers
        .scripts
        .iter()
        .map(|path| (ProjectScriptKind::Parser, path))
        .chain(
            manifest
                .type_mappings
                .scripts
                .iter()
                .map(|path| (ProjectScriptKind::TypeMapping, path)),
        )
        .map(|(kind, declared)| {
            let path = project_root.join(declared);
            let canonical =
                path.canonicalize()
                    .map_err(|source| WorkspaceError::ReadProjectScript {
                        path: path.clone(),
                        source,
                    })?;
            if !canonical.starts_with(allowed_root) {
                return Err(WorkspaceError::ProjectScriptOutsideRoot { path });
            }
            let bytes =
                fs::read(&canonical).map_err(|source| WorkspaceError::ReadProjectScript {
                    path: canonical.clone(),
                    source,
                })?;
            let relative = canonical.strip_prefix(allowed_root).map_err(|_| {
                WorkspaceError::ProjectScriptOutsideRoot {
                    path: canonical.clone(),
                }
            })?;
            Ok(ProjectScript {
                kind,
                path: normalized_relative_path(relative),
                digest: format!("sha256:{:x}", Sha256::digest(bytes)),
            })
        })
        .collect::<Result<Vec<_>, WorkspaceError>>()?;
    scripts.sort_by(|left, right| {
        script_kind_rank(left.kind)
            .cmp(&script_kind_rank(right.kind))
            .then_with(|| left.path.cmp(&right.path))
    });
    scripts.dedup();
    Ok(scripts)
}

fn script_kind_rank(kind: ProjectScriptKind) -> u8 {
    match kind {
        ProjectScriptKind::Parser => 0,
        ProjectScriptKind::TypeMapping => 1,
    }
}

fn discover_root_manifests(root: &Path) -> Result<Vec<PathBuf>, WorkspaceError> {
    let mut manifests = BTreeSet::new();
    let direct = root.join("project.toml");
    if direct.is_file() {
        manifests.insert(PathBuf::from("project.toml"));
    }
    let entries = fs::read_dir(root).map_err(|source| WorkspaceError::ReadRoot {
        path: root.to_path_buf(),
        source,
    })?;
    for entry in entries {
        let entry = entry.map_err(|source| WorkspaceError::ReadRoot {
            path: root.to_path_buf(),
            source,
        })?;
        let name = entry.file_name();
        if name.to_string_lossy().starts_with('.') || !entry.path().is_dir() {
            continue;
        }
        let manifest = entry.path().join("project.toml");
        if manifest.is_file() {
            manifests.insert(PathBuf::from(name).join("project.toml"));
        }
    }
    Ok(manifests.into_iter().collect())
}

fn validate_relative_manifest(path: &str) -> Result<PathBuf, WorkspaceError> {
    let path = Path::new(path);
    if path.is_absolute()
        || path.file_name().and_then(|name| name.to_str()) != Some("project.toml")
        || path
            .components()
            .any(|component| !matches!(component, std::path::Component::Normal(_)))
    {
        return Err(WorkspaceError::InvalidRelativeManifest);
    }
    Ok(path.to_path_buf())
}

fn normalized_relative_path(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn stable_root_suffix(path: &Path) -> String {
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(path.as_os_str().as_encoded_bytes());
    digest[..8]
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

impl std::fmt::Display for ProjectId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };

    use super::*;
    use crate::RuntimeOptions;

    static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn session(id: &str) -> ProjectSession {
        let directory = temp_dir(id);
        fs::create_dir_all(&directory).unwrap();
        let project = directory.join("project.toml");
        fs::write(&project, project_manifest(id)).unwrap();
        ProjectSession::open(
            ProjectId::new(id).expect("test project id should be valid"),
            project,
            RuntimeOptions::default(),
        )
        .unwrap()
    }

    #[test]
    fn project_ids_are_listed_deterministically() {
        let workspace = WorkspaceService::new();
        workspace
            .register(session("zeta"))
            .expect("first project should register");
        workspace
            .register(session("alpha"))
            .expect("second project should register");

        let ids = workspace
            .project_ids()
            .expect("workspace state should be readable")
            .into_iter()
            .map(|id| id.as_str().to_owned())
            .collect::<Vec<_>>();
        assert_eq!(ids, ["alpha", "zeta"]);
    }

    #[test]
    fn duplicate_project_ids_are_rejected() {
        let workspace = WorkspaceService::new();
        workspace
            .register(session("game"))
            .expect("first project should register");

        let error = workspace
            .register(session("game"))
            .expect_err("duplicate project should fail");
        assert!(matches!(error, WorkspaceError::DuplicateProject(_)));
    }

    #[test]
    fn open_project_loads_runtime_and_content_revisions() {
        let directory = temp_dir("open");
        fs::create_dir_all(&directory).unwrap();
        let project = directory.join("project.toml");
        fs::write(&project, project_manifest("demo")).unwrap();
        let workspace = WorkspaceService::new();

        let session = workspace
            .open_project(
                ProjectId::new("demo").unwrap(),
                &project,
                RuntimeOptions::default(),
            )
            .unwrap();

        assert_eq!(session.manifest_path(), project.canonicalize().unwrap());
        assert!(session.revision().project.starts_with("sha256:"));
        assert_eq!(session.revision().project.len(), 71);
        let _ = fs::remove_dir_all(directory);
    }

    #[test]
    fn discovers_only_explicit_non_hidden_project_manifests() {
        let directory = temp_dir("discover");
        fs::create_dir_all(directory.join("game")).unwrap();
        fs::create_dir_all(directory.join(".cache")).unwrap();
        fs::create_dir_all(directory.join("nested/deeper")).unwrap();
        fs::write(
            directory.join("game/project.toml"),
            project_manifest("game"),
        )
        .unwrap();
        fs::write(
            directory.join(".cache/project.toml"),
            project_manifest("cache"),
        )
        .unwrap();
        fs::write(
            directory.join("nested/deeper/project.toml"),
            project_manifest("deep"),
        )
        .unwrap();
        let workspace = WorkspaceService::new();
        workspace.add_root("workspace", &directory).unwrap();

        let projects = workspace.discover_projects().unwrap();

        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].relative_manifest, "game/project.toml");
        let _ = fs::remove_dir_all(directory);
    }

    #[test]
    fn open_discovered_rejects_traversal_and_untrusted_scripts() {
        let directory = temp_dir("trust");
        fs::create_dir_all(directory.join("game")).unwrap();
        fs::write(
            directory.join("game/project.toml"),
            format!(
                "{}\n[parsers]\nscripts = [\"parser.lua\"]\n",
                project_manifest("game")
            ),
        )
        .unwrap();
        fs::write(
            directory.join("game/parser.lua"),
            r#"
return {
  parsers = {
    identity = {
      parse = function(cell)
        return cell.text
      end,
    },
  },
}
"#,
        )
        .unwrap();
        let workspace = WorkspaceService::new();
        let root = workspace.add_root("workspace", &directory).unwrap();

        assert!(matches!(
            workspace.open_discovered_project(
                root.id(),
                "../project.toml",
                RuntimeOptions::default(),
                false,
            ),
            Err(WorkspaceError::InvalidRelativeManifest)
        ));
        assert!(matches!(
            workspace.open_discovered_project(
                root.id(),
                "game/project.toml",
                RuntimeOptions::default(),
                false,
            ),
            Err(WorkspaceError::UntrustedProjectScripts)
        ));
        let inspection = workspace
            .inspect_discovered_project(root.id(), "game/project.toml")
            .unwrap();
        assert!(inspection.requires_trust);
        assert_eq!(inspection.scripts.len(), 1);
        assert_eq!(inspection.scripts[0].path, "game/parser.lua");
        assert!(inspection.scripts[0].digest.starts_with("sha256:"));
        workspace
            .open_discovered_project(
                root.id(),
                "game/project.toml",
                RuntimeOptions::default(),
                true,
            )
            .unwrap();
        let _ = fs::remove_dir_all(directory);
    }

    fn project_manifest(id: &str) -> String {
        format!(
            "project = {{ id = \"{id}\" }}\n\
             groups = {{ common = {{ default = true }} }}\n\
             views = {{ default = {{ contract = \"{id}/default\", groups = [\"common\"] }} }}\n"
        )
    }

    fn temp_dir(label: &str) -> PathBuf {
        let nonce = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "sora-workspace-service-{}-{label}-{nonce}",
            std::process::id()
        ))
    }
}
