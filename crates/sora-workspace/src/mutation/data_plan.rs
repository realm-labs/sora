use std::{collections::BTreeMap, fs, path::Path, sync::RwLock};

use chrono::{DateTime, Duration, Utc};
use schemars::JsonSchema;
use serde::Serialize;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use super::{
    DataOperation, DataSourceImpact, FileWrite, RowChange, TransactionReceipt,
    commit_file_transaction, execute_data_operations,
};
use crate::{
    Diagnostic, ProjectId, ProjectRevision, ProjectSession, WorkspaceService,
    mutation::data::{load_raw_project_data, render_data_writes, validate_mutated_data},
};

const PLAN_TTL_MINUTES: i64 = 10;
const MAX_ACTIVE_PLANS: usize = 32;

/// One physical file change produced by a data preview.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct DataFileChange {
    pub path: String,
    pub previous_size: usize,
    pub next_size: usize,
    pub previous_digest: String,
    pub next_digest: String,
}

/// Immutable, authorization- and revision-bound data mutation plan.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct DataMutationPlan {
    pub plan_id: String,
    pub project_id: ProjectId,
    pub authorization_context: String,
    pub operation_kind: String,
    pub normalized_operations: Vec<DataOperation>,
    pub input_revisions: ProjectRevision,
    pub row_changes: Vec<RowChange>,
    pub source_impacts: Vec<DataSourceImpact>,
    pub file_changes: Vec<DataFileChange>,
    pub diagnostics: Vec<Diagnostic>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

/// Result of atomically applying a data mutation plan.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct DataApplyReport {
    pub plan_id: String,
    pub project_id: ProjectId,
    pub previous_revision: ProjectRevision,
    pub revision: ProjectRevision,
    pub row_changes: Vec<RowChange>,
    pub source_impacts: Vec<DataSourceImpact>,
    pub transaction: TransactionReceipt,
}

/// Stable data plan lifecycle and apply failures.
#[derive(Debug, thiserror::Error)]
pub enum DataPlanError {
    #[error("unknown data mutation plan")]
    UnknownPlan,
    #[error("data mutation plan has expired")]
    ExpiredPlan,
    #[error("data mutation plan belongs to a different authorization context")]
    AuthorizationMismatch,
    #[error("data mutation plan belongs to a different project")]
    ProjectMismatch,
    #[error("expected schema revision does not match the current project")]
    SchemaRevisionConflict,
    #[error("expected data revision does not match the current project")]
    DataRevisionConflict,
    #[error("idempotency key must contain 1-128 printable ASCII characters")]
    InvalidIdempotencyKey,
    #[error("idempotency key was already used for another plan")]
    IdempotencyConflict,
    #[error("workspace data mutation state lock is poisoned")]
    StatePoisoned,
    #[error("project write lock is poisoned")]
    WriteLockPoisoned,
    #[error("failed to access project: {0}")]
    Project(String),
    #[error("data operation failed: {0}")]
    Operation(String),
    #[error("data validation failed: {0}")]
    Validation(String),
    #[error("data source rendering failed: {0}")]
    Rendering(String),
    #[error("data transaction failed: {0}")]
    Transaction(String),
    #[error("project revision refresh failed: {0}")]
    Revision(String),
}

#[derive(Debug, Clone)]
struct StoredPlan {
    owner: String,
    plan: DataMutationPlan,
}

#[derive(Debug, Clone)]
struct IdempotentApply {
    plan_id: String,
    report: DataApplyReport,
}

#[derive(Debug, Default)]
pub(crate) struct DataMutationCoordinator {
    plans: RwLock<BTreeMap<String, StoredPlan>>,
    applies: RwLock<BTreeMap<(String, ProjectId, String), IdempotentApply>>,
}

impl WorkspaceService {
    /// Previews an ordered data operation batch without modifying source files.
    pub fn preview_data_mutation(
        &self,
        project_id: &ProjectId,
        authorization_context: &str,
        expected_schema_revision: &str,
        expected_data_revision: &str,
        operations: Vec<DataOperation>,
    ) -> Result<DataMutationPlan, DataPlanError> {
        if operations.is_empty() {
            return Err(DataPlanError::Operation(
                "data operation batch must not be empty".to_owned(),
            ));
        }
        let session = self
            .project(project_id)
            .map_err(|error| DataPlanError::Project(error.to_string()))?;
        let revision = session.revision();
        ensure_expected_revisions(&revision, expected_schema_revision, expected_data_revision)?;
        let (ir, base) = load_raw_project_data(&session)
            .map_err(|error| DataPlanError::Validation(error.to_string()))?;
        let execution = execute_data_operations(&ir, &base, &operations)
            .map_err(|error| DataPlanError::Operation(error.to_string()))?;
        validate_mutated_data(&session, &ir, &execution.data)
            .map_err(|error| DataPlanError::Validation(error.to_string()))?;
        let (writes, source_impacts) =
            render_data_writes(&session, &ir, &execution.data, &execution.affected_tables)
                .map_err(|error| DataPlanError::Rendering(error.to_string()))?;
        let file_changes = file_changes(project_root(&session), &writes)?;
        let created_at = Utc::now();
        let plan = DataMutationPlan {
            plan_id: format!("plan:{}", Uuid::new_v4()),
            project_id: project_id.clone(),
            authorization_context: authorization_fingerprint(authorization_context),
            operation_kind: "data".to_owned(),
            normalized_operations: operations,
            input_revisions: revision,
            row_changes: execution.changes,
            source_impacts,
            file_changes,
            diagnostics: Vec::new(),
            created_at,
            expires_at: created_at + Duration::minutes(PLAN_TTL_MINUTES),
        };
        self.data_mutation
            .insert(authorization_context, plan.clone())?;
        Ok(plan)
    }

    /// Applies an unexpired data plan under the project write lock.
    pub fn apply_data_mutation(
        &self,
        project_id: &ProjectId,
        authorization_context: &str,
        plan_id: &str,
        idempotency_key: &str,
    ) -> Result<DataApplyReport, DataPlanError> {
        validate_idempotency_key(idempotency_key)?;
        if let Some(report) = self.data_mutation.idempotent(
            authorization_context,
            project_id,
            plan_id,
            idempotency_key,
        )? {
            return Ok(report);
        }
        let stored = self.data_mutation.plan(plan_id)?;
        if stored.owner != authorization_context {
            return Err(DataPlanError::AuthorizationMismatch);
        }
        if &stored.plan.project_id != project_id {
            return Err(DataPlanError::ProjectMismatch);
        }
        if stored.plan.expires_at <= Utc::now() {
            self.data_mutation.remove(plan_id)?;
            return Err(DataPlanError::ExpiredPlan);
        }
        let session = self
            .project(project_id)
            .map_err(|error| DataPlanError::Project(error.to_string()))?;
        let _guard = session
            .write_lock
            .lock()
            .map_err(|_| DataPlanError::WriteLockPoisoned)?;
        let current = session
            .refresh_revision()
            .map_err(|error| DataPlanError::Revision(error.to_string()))?;
        ensure_expected_revisions(
            &current,
            &stored.plan.input_revisions.schema,
            &stored.plan.input_revisions.data,
        )?;
        let (ir, base) = load_raw_project_data(&session)
            .map_err(|error| DataPlanError::Validation(error.to_string()))?;
        let execution = execute_data_operations(&ir, &base, &stored.plan.normalized_operations)
            .map_err(|error| DataPlanError::Operation(error.to_string()))?;
        validate_mutated_data(&session, &ir, &execution.data)
            .map_err(|error| DataPlanError::Validation(error.to_string()))?;
        let (writes, source_impacts) =
            render_data_writes(&session, &ir, &execution.data, &execution.affected_tables)
                .map_err(|error| DataPlanError::Rendering(error.to_string()))?;
        let receipt = commit_file_transaction(project_root(&session), &writes, || {
            session.validated_data().map(|_| ())
        })
        .map_err(|error| DataPlanError::Transaction(error.to_string()))?;
        let revision = session
            .refresh_revision()
            .map_err(|error| DataPlanError::Revision(error.to_string()))?;
        let report = DataApplyReport {
            plan_id: plan_id.to_owned(),
            project_id: project_id.clone(),
            previous_revision: current,
            revision,
            row_changes: execution.changes,
            source_impacts,
            transaction: receipt,
        };
        self.data_mutation.record(
            authorization_context,
            project_id,
            plan_id,
            idempotency_key,
            report.clone(),
        )?;
        self.data_mutation.invalidate_project(project_id)?;
        Ok(report)
    }
}

impl DataMutationCoordinator {
    fn insert(&self, owner: &str, plan: DataMutationPlan) -> Result<(), DataPlanError> {
        let now = Utc::now();
        let mut plans = self
            .plans
            .write()
            .map_err(|_| DataPlanError::StatePoisoned)?;
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
            StoredPlan {
                owner: owner.to_owned(),
                plan,
            },
        );
        Ok(())
    }

    fn plan(&self, id: &str) -> Result<StoredPlan, DataPlanError> {
        self.plans
            .read()
            .map_err(|_| DataPlanError::StatePoisoned)?
            .get(id)
            .cloned()
            .ok_or(DataPlanError::UnknownPlan)
    }

    fn remove(&self, id: &str) -> Result<(), DataPlanError> {
        self.plans
            .write()
            .map_err(|_| DataPlanError::StatePoisoned)?
            .remove(id);
        Ok(())
    }

    fn invalidate_project(&self, project_id: &ProjectId) -> Result<(), DataPlanError> {
        self.plans
            .write()
            .map_err(|_| DataPlanError::StatePoisoned)?
            .retain(|_, stored| &stored.plan.project_id != project_id);
        Ok(())
    }

    fn idempotent(
        &self,
        owner: &str,
        project_id: &ProjectId,
        plan_id: &str,
        key: &str,
    ) -> Result<Option<DataApplyReport>, DataPlanError> {
        let applies = self
            .applies
            .read()
            .map_err(|_| DataPlanError::StatePoisoned)?;
        match applies.get(&(owner.to_owned(), project_id.clone(), key.to_owned())) {
            Some(record) if record.plan_id == plan_id => Ok(Some(record.report.clone())),
            Some(_) => Err(DataPlanError::IdempotencyConflict),
            None => Ok(None),
        }
    }

    fn record(
        &self,
        owner: &str,
        project_id: &ProjectId,
        plan_id: &str,
        key: &str,
        report: DataApplyReport,
    ) -> Result<(), DataPlanError> {
        self.applies
            .write()
            .map_err(|_| DataPlanError::StatePoisoned)?
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

fn ensure_expected_revisions(
    revision: &ProjectRevision,
    expected_schema: &str,
    expected_data: &str,
) -> Result<(), DataPlanError> {
    if revision.schema != expected_schema {
        return Err(DataPlanError::SchemaRevisionConflict);
    }
    if revision.data != expected_data {
        return Err(DataPlanError::DataRevisionConflict);
    }
    Ok(())
}

fn file_changes(root: &Path, writes: &[FileWrite]) -> Result<Vec<DataFileChange>, DataPlanError> {
    writes
        .iter()
        .map(|write| {
            let current = match fs::read(&write.path) {
                Ok(content) => content,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => Vec::new(),
                Err(error) => return Err(DataPlanError::Rendering(error.to_string())),
            };
            let path = write
                .path
                .strip_prefix(root)
                .map_err(|_| {
                    DataPlanError::Rendering(
                        "rendered data path is outside project root".to_owned(),
                    )
                })?
                .components()
                .map(|component| component.as_os_str().to_string_lossy())
                .collect::<Vec<_>>()
                .join("/");
            Ok(DataFileChange {
                path,
                previous_size: current.len(),
                next_size: write.content.len(),
                previous_digest: digest(&current),
                next_digest: digest(&write.content),
            })
        })
        .collect()
}

fn digest(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    format!(
        "sha256:{}",
        digest
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    )
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

fn validate_idempotency_key(key: &str) -> Result<(), DataPlanError> {
    if !key.is_empty() && key.len() <= 128 && key.bytes().all(|byte| byte.is_ascii_graphic()) {
        Ok(())
    } else {
        Err(DataPlanError::InvalidIdempotencyKey)
    }
}

fn project_root(session: &ProjectSession) -> &Path {
    session
        .manifest_path()
        .parent()
        .unwrap_or_else(|| Path::new("."))
}
