use std::{
    collections::BTreeMap,
    path::Path,
    sync::{Arc, RwLock},
};

use crate::{ProjectId, ProjectSession, RuntimeOptions};

/// Registry and coordination point for opened Sora projects.
#[derive(Debug, Default)]
pub struct WorkspaceService {
    sessions: RwLock<BTreeMap<ProjectId, Arc<ProjectSession>>>,
}

impl WorkspaceService {
    /// Creates an empty workspace service.
    pub fn new() -> Self {
        Self::default()
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
        fs::write(&project, format!("package = \"{id}\"\n")).unwrap();
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
        fs::write(&project, "package = \"demo\"\n").unwrap();
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

    fn temp_dir(label: &str) -> PathBuf {
        let nonce = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "sora-workspace-service-{}-{label}-{nonce}",
            std::process::id()
        ))
    }
}
