use std::{
    collections::BTreeMap,
    sync::{Arc, RwLock},
};

use crate::{ProjectId, ProjectSession};

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
    use std::path::PathBuf;

    use super::*;
    use crate::ProjectRevision;

    fn session(id: &str) -> ProjectSession {
        ProjectSession::new(
            ProjectId::new(id).expect("test project id should be valid"),
            PathBuf::from(format!("{id}/project.toml")),
            ProjectRevision {
                project: format!("sha256:{id}-project"),
                manifest: format!("sha256:{id}-manifest"),
                schema: format!("sha256:{id}-schema"),
                data: format!("sha256:{id}-data"),
            },
        )
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
}
