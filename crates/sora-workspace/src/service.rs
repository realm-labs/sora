use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    fs,
    io::Error as IoError,
    path::{Component, Path, PathBuf},
    sync::{
        Arc, RwLock,
        atomic::{AtomicU64, Ordering},
    },
};

use anyhow::Error as AnyhowError;
use schemars::JsonSchema;
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::{
    ProjectId, ProjectManifest, ProjectRuntime, ProjectSession, RuntimeOptions,
    mutation::{
        DataMutationCoordinator, ExcelSyncCoordinator, MutationCoordinator, ProjectInitCoordinator,
        recover_transactions,
    },
    project::capture_manifest,
    scripts::{ScriptSources, capture_project_scripts},
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
        let root = self.root(root_id)?;
        self.open_discovered_project_in_root(
            &root,
            relative_manifest,
            options,
            trust_project_scripts,
        )
    }

    /// Opens a project using an explicit root authority without registering that root.
    pub fn open_discovered_project_in_root(
        &self,
        root: &WorkspaceRoot,
        relative_manifest: &str,
        options: RuntimeOptions,
        trust_project_scripts: bool,
    ) -> Result<Arc<ProjectSession>, WorkspaceError> {
        let prepared = self.prepare_discovered_project_in_root(root, relative_manifest)?;
        if prepared.inspection().requires_trust && !trust_project_scripts {
            return Err(WorkspaceError::UntrustedProjectScripts);
        }
        self.open_prepared_project(prepared, options)
    }

    /// Inspects a discovered manifest without loading or executing project scripts.
    pub fn inspect_discovered_project(
        &self,
        root_id: &str,
        relative_manifest: &str,
    ) -> Result<DiscoveredProjectInspection, WorkspaceError> {
        Ok(self
            .prepare_discovered_project(root_id, relative_manifest)?
            .inspection)
    }

    /// Captures a manifest and all declared Lua sources without evaluating extensions.
    pub fn prepare_discovered_project(
        &self,
        root_id: &str,
        relative_manifest: &str,
    ) -> Result<PreparedProject, WorkspaceError> {
        let root = self.root(root_id)?;
        self.prepare_discovered_project_in_root(&root, relative_manifest)
    }

    /// Captures a project using an explicit root authority, without registering it.
    pub fn prepare_discovered_project_in_root(
        &self,
        root: &WorkspaceRoot,
        relative_manifest: &str,
    ) -> Result<PreparedProject, WorkspaceError> {
        let manifest_path = root.resolve_manifest(relative_manifest)?;
        let (manifest, manifest_source) =
            capture_manifest(&manifest_path).map_err(|source| WorkspaceError::OpenProject {
                path: manifest_path.clone(),
                source,
            })?;
        let (scripts, metadata) =
            capture_project_scripts(&manifest_path, &manifest, Some(&root.path))?;
        let inspection = DiscoveredProjectInspection {
            root_id: root.id.clone(),
            relative_manifest: normalized_relative_path(&validate_relative_manifest(
                relative_manifest,
            )?),
            schema_id: manifest.project.id.clone(),
            requires_trust: !metadata.is_empty(),
            scripts: metadata,
        };
        Ok(PreparedProject {
            manifest_path,
            manifest,
            manifest_source,
            scripts,
            inspection,
        })
    }

    /// Opens exactly the captured project sources. Calling this authorizes that snapshot.
    /// Explicit extra scripts in `options` are separately supplied, trusted local inputs.
    pub fn open_prepared_project(
        &self,
        prepared: PreparedProject,
        options: RuntimeOptions,
    ) -> Result<Arc<ProjectSession>, WorkspaceError> {
        let path = prepared.manifest_path.clone();
        let open_error = |source| WorkspaceError::OpenProject {
            path: path.clone(),
            source,
        };
        let scripts = prepared
            .scripts
            .with_extras(&options.parser_scripts, &options.type_mapping_scripts)
            .map_err(open_error)?;
        let check_existing = |session: &ProjectSession| -> Result<(), WorkspaceError> {
            if session
                .runtime()
                .matches_snapshot(&prepared.manifest, &scripts, &options)
                .map_err(open_error)?
            {
                Ok(())
            } else {
                Err(WorkspaceError::PreparedProjectMismatch(path.clone()))
            }
        };
        {
            let sessions = self
                .sessions
                .read()
                .map_err(|_| WorkspaceError::StatePoisoned)?;
            if let Some(existing) = sessions
                .values()
                .find(|session| session.manifest_path() == path)
            {
                check_existing(existing)?;
                return Ok(Arc::clone(existing));
            }
        }
        if let Some(project_root) = path.parent() {
            recover_transactions(project_root).map_err(|source| open_error(source.into()))?;
        }
        let sequence = self.next_project_id.fetch_add(1, Ordering::Relaxed) + 1;
        let runtime = ProjectRuntime::load_from_sources(
            Some(prepared.manifest.clone()),
            scripts.clone(),
            options.clone(),
        )
        .map_err(open_error)?;
        let session = Arc::new(
            ProjectSession::open_snapshot(
                ProjectId::generated(sequence),
                prepared.manifest_path,
                prepared.manifest.clone(),
                &prepared.manifest_source,
                runtime,
            )
            .map_err(open_error)?,
        );
        let mut sessions = self
            .sessions
            .write()
            .map_err(|_| WorkspaceError::StatePoisoned)?;
        // Another opener may have registered this path while registries were loading.
        if let Some(existing) = sessions
            .values()
            .find(|session| session.manifest_path() == path)
        {
            check_existing(existing)?;
            return Ok(Arc::clone(existing));
        }
        if sessions.contains_key(session.id()) {
            return Err(WorkspaceError::DuplicateProject(session.id().clone()));
        }
        sessions.insert(session.id().clone(), Arc::clone(&session));
        Ok(session)
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
        source: IoError,
    },
    #[error("workspace root `{0}` is not a directory")]
    RootNotDirectory(PathBuf),
    #[error("failed to enumerate workspace root `{path}`")]
    ReadRoot {
        path: PathBuf,
        #[source]
        source: IoError,
    },
    #[error("unknown workspace root `{0}`")]
    UnknownRoot(String),
    #[error(
        "project manifest must be a relative path named project.scon, project.toml, project.yaml, project.yml, project.json, or project.lua"
    )]
    InvalidRelativeManifest,
    #[error("failed to resolve project manifest `{path}`")]
    ResolveManifest {
        path: PathBuf,
        #[source]
        source: IoError,
    },
    #[error("project manifest resolves outside its allowed root")]
    ManifestOutsideRoot,
    #[error("project declares Lua scripts that have not been explicitly trusted")]
    UntrustedProjectScripts,
    #[error(
        "existing session for `{0}` does not match the authorized project snapshot or runtime options; close it or use a new workspace"
    )]
    PreparedProjectMismatch(PathBuf),
    #[error("Lua source loader script paths must stay relative to the project: `{0}`")]
    InvalidSourceLoaderScriptPath(PathBuf),
    #[error("project script `{path}` resolves outside its allowed root")]
    ProjectScriptOutsideRoot { path: PathBuf },
    #[error("failed to read project script `{path}`")]
    ReadProjectScript {
        path: PathBuf,
        #[source]
        source: IoError,
    },
    #[error("failed to open project manifest `{path}`")]
    OpenProject {
        path: PathBuf,
        #[source]
        source: AnyhowError,
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

    /// Resolves a manifest within this root, rejecting traversal and escaping symlinks.
    pub fn resolve_manifest(&self, relative_manifest: &str) -> Result<PathBuf, WorkspaceError> {
        let relative = validate_relative_manifest(relative_manifest)?;
        let manifest_path = self.path.join(relative);
        let canonical =
            manifest_path
                .canonicalize()
                .map_err(|source| WorkspaceError::ResolveManifest {
                    path: manifest_path,
                    source,
                })?;
        if !canonical.starts_with(&self.path) {
            return Err(WorkspaceError::ManifestOutsideRoot);
        }
        Ok(canonical)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct ProjectCandidate {
    pub root_id: String,
    pub root_name: String,
    pub relative_manifest: String,
    pub project_id: Option<ProjectId>,
}

/// Immutable manifest and executable sources awaiting caller authorization.
/// Cloning preserves the captured bytes; no filesystem reads or Lua evaluation occur.
#[derive(Clone)]
pub struct PreparedProject {
    manifest_path: PathBuf,
    manifest: ProjectManifest,
    manifest_source: Arc<str>,
    scripts: ScriptSources,
    inspection: DiscoveredProjectInspection,
}

impl PreparedProject {
    pub fn inspection(&self) -> &DiscoveredProjectInspection {
        &self.inspection
    }
}

impl Debug for PreparedProject {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        formatter
            .debug_struct("PreparedProject")
            .field("manifest_path", &self.manifest_path)
            .field("inspection", &self.inspection)
            .finish_non_exhaustive()
    }
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
    SourceLoader,
    TypeMapping,
}

fn discover_root_manifests(root: &Path) -> Result<Vec<PathBuf>, WorkspaceError> {
    let mut manifests = BTreeSet::new();
    for name in PROJECT_MANIFEST_NAMES {
        if root.join(name).is_file() {
            manifests.insert(PathBuf::from(name));
        }
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
        for manifest_name in PROJECT_MANIFEST_NAMES {
            if entry.path().join(manifest_name).is_file() {
                manifests.insert(PathBuf::from(&name).join(manifest_name));
            }
        }
    }
    Ok(manifests.into_iter().collect())
}

fn validate_relative_manifest(path: &str) -> Result<PathBuf, WorkspaceError> {
    let path = Path::new(path);
    if path.is_absolute()
        || !path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| PROJECT_MANIFEST_NAMES.contains(&name))
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(WorkspaceError::InvalidRelativeManifest);
    }
    Ok(path.to_path_buf())
}

const PROJECT_MANIFEST_NAMES: &[&str] = &[
    "project.scon",
    "project.toml",
    "project.yaml",
    "project.yml",
    "project.json",
    "project.lua",
];

fn normalized_relative_path(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn stable_root_suffix(path: &Path) -> String {
    let digest = Sha256::digest(path.as_os_str().as_encoded_bytes());
    digest[..8]
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

impl Display for ProjectId {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        formatter.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use std::{env::temp_dir as system_temp_dir, process::id as process_id};
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
                "{}\n[parsers]\nscripts = [\"parser.lua\"]\n[source_loaders]\nscripts = [\"source_loader.lua\"]\n",
                project_manifest("game")
            ),
        )
        .unwrap();
        fs::write(
            directory.join("game/source_loader.lua"),
            r#"
return {
  source_loaders = {
    custom_format = {
      load = function()
        return { rows = {} }
      end,
    },
  },
}
"#,
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
        assert_eq!(inspection.scripts.len(), 2);
        assert_eq!(inspection.scripts[0].path, "game/parser.lua");
        assert!(inspection.scripts[0].digest.starts_with("sha256:"));
        assert_eq!(inspection.scripts[1].kind, ProjectScriptKind::SourceLoader);
        assert_eq!(inspection.scripts[1].path, "game/source_loader.lua");
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
        system_temp_dir().join(format!(
            "sora-workspace-service-{}-{label}-{nonce}",
            process_id()
        ))
    }
}
