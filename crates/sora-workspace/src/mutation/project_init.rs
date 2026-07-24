use std::{
    collections::BTreeMap,
    fs,
    path::{Component, Path, PathBuf},
    sync::{Mutex, RwLock},
};

use chrono::{DateTime, Duration, Utc};
use schemars::JsonSchema;
use serde::Serialize;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use super::{TransactionReceipt, commit_text_transaction};
use crate::{
    ProjectId, ProjectRevision, RuntimeOptions, WorkspaceService, studio::service::TextFileWrite,
};

const PLAN_TTL_MINUTES: i64 = 10;

/// One file that will be created by a project initialization plan.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct ProjectInitPlanFile {
    pub path: String,
    pub size: usize,
    pub content: String,
}

/// Immutable preview for creating a project inside an allowed root.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct ProjectInitPlan {
    pub plan_id: String,
    pub authorization_context: String,
    pub operation_kind: String,
    pub root_id: String,
    pub relative_directory: String,
    pub package: String,
    pub files: Vec<ProjectInitPlanFile>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

/// Result of atomically creating and opening a project.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct ProjectInitApplyReport {
    pub plan_id: String,
    pub project_id: ProjectId,
    pub revision: ProjectRevision,
    pub files: Vec<String>,
    pub transaction: TransactionReceipt,
}

#[derive(Debug, Clone)]
struct StoredPlan {
    owner: String,
    plan: ProjectInitPlan,
}

#[derive(Debug, Clone)]
struct AppliedInit {
    plan_id: String,
    report: ProjectInitApplyReport,
}

#[derive(Debug, Default)]
pub(crate) struct ProjectInitCoordinator {
    plans: RwLock<BTreeMap<String, StoredPlan>>,
    applies: RwLock<BTreeMap<(String, String), AppliedInit>>,
    write_lock: Mutex<()>,
}

impl WorkspaceService {
    /// Previews a minimal TOML/JSON Sora project scaffold without writing.
    pub fn preview_project_init(
        &self,
        authorization_context: &str,
        root_id: &str,
        relative_directory: &str,
        package: &str,
    ) -> Result<ProjectInitPlan, crate::MutationPlanError> {
        validate_package(package)?;
        let root = self
            .root(root_id)
            .map_err(|error| crate::MutationPlanError::Project(error.to_string()))?;
        let relative = validate_relative_directory(relative_directory)?;
        ensure_empty_target(root.path(), &root.path().join(&relative))?;
        let files = scaffold_files(package);
        let created_at = Utc::now();
        let plan = ProjectInitPlan {
            plan_id: format!("plan:{}", Uuid::new_v4()),
            authorization_context: authorization_fingerprint(authorization_context),
            operation_kind: "project_init".to_owned(),
            root_id: root_id.to_owned(),
            relative_directory: relative_string(&relative),
            package: package.to_owned(),
            files: files
                .iter()
                .map(|(path, content)| ProjectInitPlanFile {
                    path: relative_string(path),
                    size: content.len(),
                    content: content.clone(),
                })
                .collect(),
            created_at,
            expires_at: created_at + Duration::minutes(PLAN_TTL_MINUTES),
        };
        self.project_init
            .insert(authorization_context, plan.clone())?;
        Ok(plan)
    }

    /// Applies a project initialization plan and opens the new project.
    pub fn apply_project_init(
        &self,
        authorization_context: &str,
        plan_id: &str,
        idempotency_key: &str,
    ) -> Result<ProjectInitApplyReport, crate::MutationPlanError> {
        validate_idempotency_key(idempotency_key)?;
        if let Some(report) =
            self.project_init
                .idempotent(authorization_context, plan_id, idempotency_key)?
        {
            return Ok(report);
        }
        let stored = self.project_init.plan(plan_id)?;
        if stored.owner != authorization_context {
            return Err(crate::MutationPlanError::AuthorizationMismatch);
        }
        if stored.plan.expires_at <= Utc::now() {
            self.project_init.remove(plan_id)?;
            return Err(crate::MutationPlanError::ExpiredPlan);
        }
        let _guard = self
            .project_init
            .write_lock
            .lock()
            .map_err(|_| crate::MutationPlanError::WriteLockPoisoned)?;
        let root = self
            .root(&stored.plan.root_id)
            .map_err(|error| crate::MutationPlanError::Project(error.to_string()))?;
        let relative = validate_relative_directory(&stored.plan.relative_directory)?;
        let target = root.path().join(&relative);
        ensure_empty_target(root.path(), &target)?;
        let created_target = !target.exists();
        fs::create_dir_all(&target)
            .map_err(|error| crate::MutationPlanError::Transaction(error.to_string()))?;
        let writes = stored
            .plan
            .files
            .iter()
            .map(|file| TextFileWrite {
                path: target.join(&file.path),
                content: file.content.clone(),
            })
            .collect::<Vec<_>>();
        let project_path = target.join("project.toml");
        let transaction = match commit_text_transaction(&target, &writes, || {
            let input = sora_input_schema::input::SchemaFileInput::new(&project_path);
            sora_core::pipeline::load_schema_ir(&input)?;
            Ok(())
        }) {
            Ok(transaction) => transaction,
            Err(error) => {
                if created_target {
                    let _ = fs::remove_dir_all(&target);
                }
                return Err(crate::MutationPlanError::Transaction(error.to_string()));
            }
        };
        let relative_manifest = format!("{}/project.toml", stored.plan.relative_directory);
        let session = self
            .open_discovered_project(
                &stored.plan.root_id,
                &relative_manifest,
                RuntimeOptions::default(),
                false,
            )
            .map_err(|error| crate::MutationPlanError::Project(error.to_string()))?;
        let report = ProjectInitApplyReport {
            plan_id: plan_id.to_owned(),
            project_id: session.id().clone(),
            revision: session.revision(),
            files: transaction.affected_files.clone(),
            transaction,
        };
        self.project_init.record(
            authorization_context,
            plan_id,
            idempotency_key,
            report.clone(),
        )?;
        self.project_init.remove(plan_id)?;
        Ok(report)
    }
}

impl ProjectInitCoordinator {
    fn insert(&self, owner: &str, plan: ProjectInitPlan) -> Result<(), crate::MutationPlanError> {
        let now = Utc::now();
        let mut plans = self
            .plans
            .write()
            .map_err(|_| crate::MutationPlanError::StatePoisoned)?;
        plans.retain(|_, stored| stored.plan.expires_at > now);
        plans.insert(
            plan.plan_id.clone(),
            StoredPlan {
                owner: owner.to_owned(),
                plan,
            },
        );
        Ok(())
    }

    fn plan(&self, id: &str) -> Result<StoredPlan, crate::MutationPlanError> {
        self.plans
            .read()
            .map_err(|_| crate::MutationPlanError::StatePoisoned)?
            .get(id)
            .cloned()
            .ok_or(crate::MutationPlanError::UnknownPlan)
    }

    fn remove(&self, id: &str) -> Result<(), crate::MutationPlanError> {
        self.plans
            .write()
            .map_err(|_| crate::MutationPlanError::StatePoisoned)?
            .remove(id);
        Ok(())
    }

    fn idempotent(
        &self,
        owner: &str,
        plan_id: &str,
        key: &str,
    ) -> Result<Option<ProjectInitApplyReport>, crate::MutationPlanError> {
        let applies = self
            .applies
            .read()
            .map_err(|_| crate::MutationPlanError::StatePoisoned)?;
        match applies.get(&(owner.to_owned(), key.to_owned())) {
            Some(applied) if applied.plan_id == plan_id => Ok(Some(applied.report.clone())),
            Some(_) => Err(crate::MutationPlanError::IdempotencyConflict),
            None => Ok(None),
        }
    }

    fn record(
        &self,
        owner: &str,
        plan_id: &str,
        key: &str,
        report: ProjectInitApplyReport,
    ) -> Result<(), crate::MutationPlanError> {
        self.applies
            .write()
            .map_err(|_| crate::MutationPlanError::StatePoisoned)?
            .insert(
                (owner.to_owned(), key.to_owned()),
                AppliedInit {
                    plan_id: plan_id.to_owned(),
                    report,
                },
            );
        Ok(())
    }
}

fn scaffold_files(package: &str) -> Vec<(PathBuf, String)> {
    vec![
        (
            PathBuf::from("project.toml"),
            format!(
                r#"package = "{package}"
includes = ["schema/items.toml"]

[build]
default_source_format = "json"
data_root = "data"
schema_lock = "generated/schema.lock"

[[build.codegen]]
target = "rust"
out = "generated/rust"
format = "auto"

[[build.exports]]
format = "binary"
out = "generated/config.sora"
"#
            ),
        ),
        (
            PathBuf::from("schema/items.toml"),
            r#"[[tables]]
name = "Item"
mode = "map"
key = "id"
source = { file = "Item.json", format = "json" }

[[tables.fields]]
name = "id"
type = "i32"

[[tables.fields]]
name = "name"
type = "string"
"#
            .to_owned(),
        ),
        (
            PathBuf::from("data/Item.json"),
            "[{\"id\":1,\"name\":\"Example\"}]\n".to_owned(),
        ),
    ]
}

fn validate_relative_directory(path: &str) -> Result<PathBuf, crate::MutationPlanError> {
    let path = Path::new(path);
    if path.as_os_str().is_empty()
        || path.is_absolute()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(crate::MutationPlanError::Project(
            "project directory must be a non-empty root-relative path".to_owned(),
        ));
    }
    Ok(path.to_path_buf())
}

fn ensure_empty_target(root: &Path, target: &Path) -> Result<(), crate::MutationPlanError> {
    let canonical = nearest_existing_ancestor(target)
        .canonicalize()
        .map_err(|error| crate::MutationPlanError::Project(error.to_string()))?;
    if !canonical.starts_with(root) {
        return Err(crate::MutationPlanError::Project(
            "project directory resolves outside its allowed root".to_owned(),
        ));
    }
    if target.exists() {
        if !target.is_dir() {
            return Err(crate::MutationPlanError::Project(
                "project target exists and is not a directory".to_owned(),
            ));
        }
        let mut entries = fs::read_dir(target)
            .map_err(|error| crate::MutationPlanError::Project(error.to_string()))?;
        if entries.next().is_some() {
            return Err(crate::MutationPlanError::Project(
                "project target directory is not empty".to_owned(),
            ));
        }
    }
    Ok(())
}

fn nearest_existing_ancestor(path: &Path) -> &Path {
    let mut current = path;
    while !current.exists() {
        let Some(parent) = current.parent() else {
            break;
        };
        current = parent;
    }
    current
}

fn validate_package(package: &str) -> Result<(), crate::MutationPlanError> {
    if !package.is_empty()
        && package.len() <= 128
        && package
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
    {
        Ok(())
    } else {
        Err(crate::MutationPlanError::Project(
            "package must contain 1-128 ASCII letters, digits, '.', '-' or '_'".to_owned(),
        ))
    }
}

fn validate_idempotency_key(key: &str) -> Result<(), crate::MutationPlanError> {
    if !key.is_empty() && key.len() <= 128 && key.bytes().all(|byte| byte.is_ascii_graphic()) {
        Ok(())
    } else {
        Err(crate::MutationPlanError::InvalidIdempotencyKey)
    }
}

fn authorization_fingerprint(context: &str) -> String {
    let digest = Sha256::digest(context.as_bytes());
    format!(
        "auth:{}",
        digest
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    )
}

fn relative_string(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
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
    fn project_init_previews_then_creates_a_buildable_project() {
        let root = temp_dir();
        fs::create_dir_all(&root).unwrap();
        let workspace = WorkspaceService::new();
        let allowed = workspace.add_root("tests", &root).unwrap();

        let plan = workspace
            .preview_project_init("test", allowed.id(), "new-game", "com.example.game")
            .unwrap();

        assert!(!root.join("new-game").exists());
        assert_eq!(plan.files.len(), 3);
        let report = workspace
            .apply_project_init("test", &plan.plan_id, "init-1")
            .unwrap();
        let replay = workspace
            .apply_project_init("test", &plan.plan_id, "init-1")
            .unwrap();
        assert_eq!(report.project_id, replay.project_id);
        assert!(root.join("new-game/project.toml").is_file());
        assert!(root.join("new-game/schema/items.toml").is_file());
        assert!(root.join("new-game/data/Item.json").is_file());
        assert!(workspace.project(&report.project_id).is_ok());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn project_init_rejects_non_empty_targets() {
        let root = temp_dir();
        fs::create_dir_all(root.join("occupied")).unwrap();
        fs::write(root.join("occupied/file.txt"), "owned").unwrap();
        let workspace = WorkspaceService::new();
        let allowed = workspace.add_root("tests", &root).unwrap();

        let error = workspace
            .preview_project_init("test", allowed.id(), "occupied", "demo")
            .unwrap_err();

        assert!(error.to_string().contains("not empty"));
        assert_eq!(
            fs::read_to_string(root.join("occupied/file.txt")).unwrap(),
            "owned"
        );
        let _ = fs::remove_dir_all(root);
    }

    fn temp_dir() -> PathBuf {
        let nonce = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "sora-project-init-{}-{time}-{nonce}",
            std::process::id()
        ))
    }
}
