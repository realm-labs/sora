use std::{collections::BTreeMap, fs, path::Path, sync::RwLock};

use chrono::{DateTime, Duration, Utc};
use schemars::JsonSchema;
use serde::Serialize;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use super::{
    SchemaOperation, TransactionReceipt, commit_text_transaction, execute_schema_operations,
};
use crate::{
    Diagnostic, ProjectId, ProjectRevision, ProjectSession, WorkspaceService,
    studio::{StudioSchema, service::TextFileWrite},
};

const PLAN_TTL_MINUTES: i64 = 10;
const MAX_ACTIVE_PLANS: usize = 32;

/// One project-relative textual change in a schema mutation plan.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct TextFileDiff {
    pub path: String,
    pub diff: String,
    pub previous_size: usize,
    pub next_size: usize,
}

/// Immutable preview returned before a schema mutation can be applied.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct SchemaMutationPlan {
    pub plan_id: String,
    pub project_id: ProjectId,
    pub authorization_context: String,
    pub operation_kind: String,
    pub normalized_operations: Vec<SchemaOperation>,
    pub input_revisions: ProjectRevision,
    pub text_diffs: Vec<TextFileDiff>,
    pub diagnostics: Vec<Diagnostic>,
    pub affected_files: Vec<String>,
    pub affected_entities: Vec<String>,
    pub affected_tables: Vec<String>,
    pub affected_build_targets: Vec<String>,
    pub requires_data_migration: bool,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

/// Result of atomically applying a schema mutation plan.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct SchemaApplyReport {
    pub plan_id: String,
    pub project_id: ProjectId,
    pub previous_revision: ProjectRevision,
    pub revision: ProjectRevision,
    pub affected_files: Vec<String>,
    pub affected_entities: Vec<String>,
    pub affected_tables: Vec<String>,
    pub requires_data_migration: bool,
    pub transaction: TransactionReceipt,
}

/// Stable plan lifecycle and mutation application failures.
#[derive(Debug, thiserror::Error)]
pub enum MutationPlanError {
    #[error("unknown schema mutation plan")]
    UnknownPlan,
    #[error("schema mutation plan has expired")]
    ExpiredPlan,
    #[error("schema mutation plan belongs to a different authorization context")]
    AuthorizationMismatch,
    #[error("schema mutation plan belongs to a different project")]
    ProjectMismatch,
    #[error("expected schema revision does not match the current project")]
    SchemaRevisionConflict,
    #[error("expected manifest revision does not match the current project")]
    ManifestRevisionConflict,
    #[error("idempotency key must contain 1-128 printable ASCII characters")]
    InvalidIdempotencyKey,
    #[error("idempotency key was already used for another plan")]
    IdempotencyConflict,
    #[error("workspace mutation state lock is poisoned")]
    StatePoisoned,
    #[error("project write lock is poisoned")]
    WriteLockPoisoned,
    #[error("failed to access project: {0}")]
    Project(String),
    #[error("schema operation failed: {0}")]
    Operation(String),
    #[error("schema validation failed: {0}")]
    Validation(String),
    #[error("schema rendering failed: {0}")]
    Rendering(String),
    #[error("schema transaction failed: {0}")]
    Transaction(String),
    #[error("project revision refresh failed: {0}")]
    Revision(String),
}

#[derive(Debug, Clone)]
struct StoredSchemaPlan {
    owner: String,
    plan: SchemaMutationPlan,
}

#[derive(Debug, Clone)]
struct IdempotentApply {
    plan_id: String,
    report: SchemaApplyReport,
}

#[derive(Debug, Default)]
pub(crate) struct MutationCoordinator {
    schema_plans: RwLock<BTreeMap<String, StoredSchemaPlan>>,
    applies: RwLock<BTreeMap<(String, ProjectId, String), IdempotentApply>>,
}

impl WorkspaceService {
    /// Previews an ordered schema operation batch without writing files.
    pub fn preview_schema_mutation(
        &self,
        project_id: &ProjectId,
        authorization_context: &str,
        expected_schema_revision: &str,
        expected_manifest_revision: &str,
        operations: Vec<SchemaOperation>,
    ) -> Result<SchemaMutationPlan, MutationPlanError> {
        let session = self
            .project(project_id)
            .map_err(|error| MutationPlanError::Project(error.to_string()))?;
        let revision = session.revision();
        ensure_expected_revisions(
            &revision,
            expected_schema_revision,
            expected_manifest_revision,
        )?;
        let base = load_valid_schema(&session)?;
        let execution = execute_schema_operations(&base, &operations)
            .map_err(|error| MutationPlanError::Operation(error.to_string()))?;
        crate::studio::service::validate_studio_schema_with_parsers(
            &execution.schema,
            session.runtime().schema_parsers(),
        )
        .map_err(|error| MutationPlanError::Validation(error.to_string()))?;
        let writes = crate::studio::service::render_studio_schema_writes(
            session.manifest_path(),
            &execution.schema,
        )
        .map_err(|error| MutationPlanError::Rendering(error.to_string()))?;
        let root = project_root(&session);
        let text_diffs = text_diffs(root, &writes)?;
        let affected_files = text_diffs.iter().map(|diff| diff.path.clone()).collect();
        let inspection = session
            .inspect()
            .map_err(|error| MutationPlanError::Project(error.to_string()))?;
        let created_at = Utc::now();
        let plan = SchemaMutationPlan {
            plan_id: format!("plan:{}", Uuid::new_v4()),
            project_id: project_id.clone(),
            authorization_context: authorization_fingerprint(authorization_context),
            operation_kind: "schema".to_owned(),
            normalized_operations: operations,
            input_revisions: revision,
            text_diffs,
            diagnostics: Vec::new(),
            affected_files,
            affected_entities: execution.affected_entities.into_iter().collect(),
            affected_tables: execution.affected_tables.into_iter().collect(),
            affected_build_targets: inspection
                .build_outputs
                .into_iter()
                .map(|output| format!("{}:{}", output.kind, output.name))
                .collect(),
            requires_data_migration: execution.requires_data_migration,
            created_at,
            expires_at: created_at + Duration::minutes(PLAN_TTL_MINUTES),
        };
        self.mutation
            .insert_schema_plan(authorization_context, plan.clone())?;
        Ok(plan)
    }

    /// Applies an unexpired schema plan under a project-scoped write lock.
    pub fn apply_schema_mutation(
        &self,
        project_id: &ProjectId,
        authorization_context: &str,
        plan_id: &str,
        idempotency_key: &str,
    ) -> Result<SchemaApplyReport, MutationPlanError> {
        validate_idempotency_key(idempotency_key)?;
        if let Some(report) = self.mutation.idempotent_result(
            authorization_context,
            project_id,
            plan_id,
            idempotency_key,
        )? {
            return Ok(report);
        }
        let stored = self.mutation.schema_plan(plan_id)?;
        if stored.owner != authorization_context {
            return Err(MutationPlanError::AuthorizationMismatch);
        }
        if &stored.plan.project_id != project_id {
            return Err(MutationPlanError::ProjectMismatch);
        }
        if stored.plan.expires_at <= Utc::now() {
            self.mutation.remove_schema_plan(plan_id)?;
            return Err(MutationPlanError::ExpiredPlan);
        }
        let session = self
            .project(project_id)
            .map_err(|error| MutationPlanError::Project(error.to_string()))?;
        let _write_guard = session
            .write_lock
            .lock()
            .map_err(|_| MutationPlanError::WriteLockPoisoned)?;
        let current = session
            .refresh_revision()
            .map_err(|error| MutationPlanError::Revision(error.to_string()))?;
        ensure_expected_revisions(
            &current,
            &stored.plan.input_revisions.schema,
            &stored.plan.input_revisions.manifest,
        )?;
        let base = load_valid_schema(&session)?;
        let execution = execute_schema_operations(&base, &stored.plan.normalized_operations)
            .map_err(|error| MutationPlanError::Operation(error.to_string()))?;
        crate::studio::service::validate_studio_schema_with_parsers(
            &execution.schema,
            session.runtime().schema_parsers(),
        )
        .map_err(|error| MutationPlanError::Validation(error.to_string()))?;
        let writes = crate::studio::service::render_studio_schema_writes(
            session.manifest_path(),
            &execution.schema,
        )
        .map_err(|error| MutationPlanError::Rendering(error.to_string()))?;
        let next_schema = execution.schema.clone();
        let parser_registry = session.runtime().schema_parsers();
        let receipt = commit_text_transaction(project_root(&session), &writes, || {
            crate::studio::service::validate_studio_schema_with_parsers(
                &next_schema,
                parser_registry,
            )?;
            let loaded = session.load_studio_schema();
            if loaded.ok {
                Ok(())
            } else {
                let message = loaded
                    .diagnostics
                    .into_iter()
                    .map(|diagnostic| diagnostic.message)
                    .collect::<Vec<_>>()
                    .join("; ");
                anyhow::bail!("reloaded schema is invalid: {message}")
            }
        })
        .map_err(|error| MutationPlanError::Transaction(error.to_string()))?;
        let revision = session
            .refresh_revision()
            .map_err(|error| MutationPlanError::Revision(error.to_string()))?;
        let report = SchemaApplyReport {
            plan_id: stored.plan.plan_id.clone(),
            project_id: project_id.clone(),
            previous_revision: current,
            revision,
            affected_files: receipt.affected_files.clone(),
            affected_entities: stored.plan.affected_entities.clone(),
            affected_tables: stored.plan.affected_tables.clone(),
            requires_data_migration: stored.plan.requires_data_migration,
            transaction: receipt,
        };
        self.mutation.record_apply(
            authorization_context,
            project_id,
            plan_id,
            idempotency_key,
            report.clone(),
        )?;
        self.mutation.invalidate_project_plans(project_id, None)?;
        Ok(report)
    }
}

impl MutationCoordinator {
    fn insert_schema_plan(
        &self,
        owner: &str,
        plan: SchemaMutationPlan,
    ) -> Result<(), MutationPlanError> {
        let now = Utc::now();
        let mut plans = self
            .schema_plans
            .write()
            .map_err(|_| MutationPlanError::StatePoisoned)?;
        plans.retain(|_, stored| stored.plan.expires_at > now);
        let mut owned = plans
            .iter()
            .filter(|(_, stored)| {
                stored.owner == owner && stored.plan.project_id == plan.project_id
            })
            .map(|(id, stored)| (id.clone(), stored.plan.created_at))
            .collect::<Vec<_>>();
        owned.sort_by_key(|(_, created)| *created);
        let remove_count = owned
            .len()
            .saturating_add(1)
            .saturating_sub(MAX_ACTIVE_PLANS);
        for (id, _) in owned.into_iter().take(remove_count) {
            plans.remove(&id);
        }
        plans.insert(
            plan.plan_id.clone(),
            StoredSchemaPlan {
                owner: owner.to_owned(),
                plan,
            },
        );
        Ok(())
    }

    fn schema_plan(&self, plan_id: &str) -> Result<StoredSchemaPlan, MutationPlanError> {
        self.schema_plans
            .read()
            .map_err(|_| MutationPlanError::StatePoisoned)?
            .get(plan_id)
            .cloned()
            .ok_or(MutationPlanError::UnknownPlan)
    }

    fn remove_schema_plan(&self, plan_id: &str) -> Result<(), MutationPlanError> {
        self.schema_plans
            .write()
            .map_err(|_| MutationPlanError::StatePoisoned)?
            .remove(plan_id);
        Ok(())
    }

    fn invalidate_project_plans(
        &self,
        project_id: &ProjectId,
        keep: Option<&str>,
    ) -> Result<(), MutationPlanError> {
        self.schema_plans
            .write()
            .map_err(|_| MutationPlanError::StatePoisoned)?
            .retain(|id, stored| {
                &stored.plan.project_id != project_id || keep == Some(id.as_str())
            });
        Ok(())
    }

    fn idempotent_result(
        &self,
        owner: &str,
        project_id: &ProjectId,
        plan_id: &str,
        key: &str,
    ) -> Result<Option<SchemaApplyReport>, MutationPlanError> {
        let map_key = (owner.to_owned(), project_id.clone(), key.to_owned());
        let applies = self
            .applies
            .read()
            .map_err(|_| MutationPlanError::StatePoisoned)?;
        match applies.get(&map_key) {
            Some(record) if record.plan_id == plan_id => Ok(Some(record.report.clone())),
            Some(_) => Err(MutationPlanError::IdempotencyConflict),
            None => Ok(None),
        }
    }

    fn record_apply(
        &self,
        owner: &str,
        project_id: &ProjectId,
        plan_id: &str,
        key: &str,
        report: SchemaApplyReport,
    ) -> Result<(), MutationPlanError> {
        self.applies
            .write()
            .map_err(|_| MutationPlanError::StatePoisoned)?
            .insert(
                (owner.to_owned(), project_id.clone(), key.to_owned()),
                IdempotentApply {
                    plan_id: plan_id.to_owned(),
                    report,
                },
            );
        Ok(())
    }
}

fn load_valid_schema(session: &ProjectSession) -> Result<StudioSchema, MutationPlanError> {
    let response = session.load_studio_schema();
    if !response.ok {
        return Err(MutationPlanError::Validation(
            response
                .diagnostics
                .into_iter()
                .map(|diagnostic| diagnostic.message)
                .collect::<Vec<_>>()
                .join("; "),
        ));
    }
    response
        .schema
        .ok_or_else(|| MutationPlanError::Validation("schema graph is unavailable".to_owned()))
}

fn ensure_expected_revisions(
    revision: &ProjectRevision,
    expected_schema: &str,
    expected_manifest: &str,
) -> Result<(), MutationPlanError> {
    if revision.schema != expected_schema {
        return Err(MutationPlanError::SchemaRevisionConflict);
    }
    if revision.manifest != expected_manifest {
        return Err(MutationPlanError::ManifestRevisionConflict);
    }
    Ok(())
}

fn text_diffs(
    root: &Path,
    writes: &[TextFileWrite],
) -> Result<Vec<TextFileDiff>, MutationPlanError> {
    writes
        .iter()
        .map(|write| {
            let current = match fs::read_to_string(&write.path) {
                Ok(content) => content,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
                Err(error) => return Err(MutationPlanError::Rendering(error.to_string())),
            };
            let path = write
                .path
                .strip_prefix(root)
                .map_err(|_| {
                    MutationPlanError::Rendering(
                        "rendered schema path is outside project root".to_owned(),
                    )
                })?
                .components()
                .map(|component| component.as_os_str().to_string_lossy())
                .collect::<Vec<_>>()
                .join("/");
            Ok(TextFileDiff {
                path,
                diff: crate::studio::diff::simple_diff(&current, &write.content),
                previous_size: current.len(),
                next_size: write.content.len(),
            })
        })
        .collect()
}

fn project_root(session: &ProjectSession) -> &Path {
    session
        .manifest_path()
        .parent()
        .unwrap_or_else(|| Path::new("."))
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

fn validate_idempotency_key(key: &str) -> Result<(), MutationPlanError> {
    if !key.is_empty() && key.len() <= 128 && key.bytes().all(|byte| byte.is_ascii_graphic()) {
        Ok(())
    } else {
        Err(MutationPlanError::InvalidIdempotencyKey)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    use crate::{FieldDefinition, FieldOwner, FieldOwnerKind, RuntimeOptions};

    use super::*;

    static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn preview_is_read_only_and_apply_is_atomic_and_idempotent() {
        let root = temp_project();
        let project = root.join("project.toml");
        let workspace = WorkspaceService::new();
        let id = ProjectId::new("demo").unwrap();
        let session = workspace
            .open_project(id.clone(), &project, RuntimeOptions::default())
            .unwrap();
        let before = fs::read_to_string(root.join("schema.toml")).unwrap();
        let revision = session.revision();
        let operations = vec![SchemaOperation::AddField {
            owner: FieldOwner {
                kind: FieldOwnerKind::Table,
                name: "Item".to_owned(),
                variant: None,
            },
            field: FieldDefinition {
                name: "name".to_owned(),
                ty: "string".to_owned(),
                scope: "all".to_owned(),
                parser: None,
                comment: None,
                default: None,
                range: None,
                length: None,
            },
        }];

        let plan = workspace
            .preview_schema_mutation(
                &id,
                "test-owner",
                &revision.schema,
                &revision.manifest,
                operations,
            )
            .unwrap();

        assert_eq!(
            fs::read_to_string(root.join("schema.toml")).unwrap(),
            before
        );
        assert!(
            plan.text_diffs
                .iter()
                .any(|diff| diff.diff.contains("+name = \"name\""))
        );
        let first = workspace
            .apply_schema_mutation(&id, "test-owner", &plan.plan_id, "request-1")
            .unwrap();
        let replay = workspace
            .apply_schema_mutation(&id, "test-owner", &plan.plan_id, "request-1")
            .unwrap();
        assert_eq!(first.revision, replay.revision);
        assert_ne!(first.previous_revision.schema, first.revision.schema);
        assert!(
            fs::read_to_string(root.join("schema.toml"))
                .unwrap()
                .contains("name = \"name\"")
        );
        assert!(
            fs::read_to_string(root.join("schema.toml"))
                .unwrap()
                .contains("[[tables.indexes]]")
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn apply_rejects_revision_conflicts_without_writing() {
        let root = temp_project();
        let project = root.join("project.toml");
        let workspace = WorkspaceService::new();
        let id = ProjectId::new("demo").unwrap();
        let session = workspace
            .open_project(id.clone(), &project, RuntimeOptions::default())
            .unwrap();
        let revision = session.revision();
        let plan = workspace
            .preview_schema_mutation(
                &id,
                "test-owner",
                &revision.schema,
                &revision.manifest,
                vec![SchemaOperation::SetTableScope {
                    table: "Item".to_owned(),
                    scope: "client".to_owned(),
                }],
            )
            .unwrap();
        fs::write(
            root.join("schema.toml"),
            format!(
                "{}\n# external change\n",
                fs::read_to_string(root.join("schema.toml")).unwrap()
            ),
        )
        .unwrap();

        let error = workspace
            .apply_schema_mutation(&id, "test-owner", &plan.plan_id, "request-2")
            .unwrap_err();

        assert!(matches!(error, MutationPlanError::SchemaRevisionConflict));
        assert!(
            fs::read_to_string(root.join("schema.toml"))
                .unwrap()
                .contains("# external change")
        );
        let _ = fs::remove_dir_all(root);
    }

    fn temp_project() -> PathBuf {
        let nonce = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "sora-schema-plan-{}-{time}-{nonce}",
            std::process::id()
        ));
        fs::create_dir_all(&root).unwrap();
        fs::write(
            root.join("project.toml"),
            "package = \"demo\"\nincludes = [\"schema.toml\"]\n",
        )
        .unwrap();
        fs::write(
            root.join("schema.toml"),
            r#"[[tables]]
name = "Item"
mode = "map"
key = "id"

[[tables.fields]]
name = "id"
type = "i32"

[[tables.indexes]]
name = "by_id"
fields = ["id"]
unique = true
"#,
        )
        .unwrap();
        root
    }
}
