//! Client-provided filesystem roots stay private to one authorization server.
use std::{collections::BTreeMap, path::PathBuf, sync::Arc};

use anyhow::{Result, anyhow, bail};
#[allow(deprecated)]
use rmcp::model::ListRootsResult;
use sora_workspace::{ProjectCandidate, WorkspaceRoot, WorkspaceService};
use url::Url;

const MAX_CLIENT_ROOTS: usize = 64;

#[derive(Debug, Default)]
pub(crate) struct ClientRoots {
    pub(crate) workspace: Arc<WorkspaceService>,
    authorities: BTreeMap<String, WorkspaceRoot>,
}

impl ClientRoots {
    #[allow(deprecated)]
    pub(crate) fn new(result: ListRootsResult) -> Result<Self> {
        if result.roots.len() > MAX_CLIENT_ROOTS {
            bail!("too many client roots (maximum {MAX_CLIENT_ROOTS})");
        }
        let mut roots = Self::default();
        for (index, root) in result.roots.into_iter().enumerate() {
            let path = Url::parse(&root.uri)?
                .to_file_path()
                .map_err(|()| anyhow!("client roots must be local file URLs"))?;
            let authority = roots.workspace.add_root(
                root.name.unwrap_or_else(|| format!("root-{}", index + 1)),
                path,
            )?;
            roots
                .authorities
                .insert(authority.id().to_owned(), authority);
        }
        Ok(roots)
    }

    pub(crate) fn contains(&self, root_id: &str) -> bool {
        self.authorities.contains_key(root_id)
    }

    pub(crate) fn authority(&self, root_id: &str) -> Option<WorkspaceRoot> {
        self.authorities.get(root_id).cloned()
    }

    pub(crate) fn discover(&self, shared: &WorkspaceService) -> Result<Vec<ProjectCandidate>> {
        let mut projects = shared.discover_projects()?;
        for mut project in self.workspace.discover_projects()? {
            if projects.iter().any(|existing| {
                existing.root_id == project.root_id
                    && existing.relative_manifest == project.relative_manifest
            }) {
                continue;
            }
            // A stale or escaping candidate must not hide the other discoverable projects.
            let Ok(manifest) = self.manifest_path(&project.root_id, &project.relative_manifest)
            else {
                continue;
            };
            project.project_id = shared.project_ids()?.into_iter().find(|id| {
                shared
                    .project(id)
                    .is_ok_and(|session| session.manifest_path() == manifest)
            });
            projects.push(project);
        }
        projects.sort_by(|left, right| {
            (&left.root_name, &left.relative_manifest)
                .cmp(&(&right.root_name, &right.relative_manifest))
        });
        Ok(projects)
    }

    fn manifest_path(&self, root_id: &str, relative_manifest: &str) -> Result<PathBuf> {
        let root = self
            .authority(root_id)
            .ok_or_else(|| anyhow!("unknown client root"))?;
        Ok(root.resolve_manifest(relative_manifest)?)
    }
}
