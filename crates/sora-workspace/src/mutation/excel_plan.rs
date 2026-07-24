use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::{
        Arc, RwLock,
        atomic::{AtomicBool, Ordering},
    },
};

use chrono::{DateTime, Duration, Utc};
use schemars::JsonSchema;
use serde::Serialize;
use sha2::{Digest, Sha256};
use sora_excel::sync::{ExcelSyncReport, ExcelTemplateSync};
use uuid::Uuid;

use super::{DataFileChange, FileWrite, TransactionReceipt, commit_file_transaction};
use crate::{Diagnostic, ProjectId, ProjectRevision, ProjectSession, WorkspaceService};

const PLAN_TTL_MINUTES: i64 = 10;
const MAX_ACTIVE_PLANS: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExcelSyncPhase {
    ResolveProject,
    ValidateRevision,
    LoadSchema,
    InspectWorkbooks,
    StageWorkbooks,
    RenderWorkbooks,
    Diff,
    Commit,
    ValidateData,
    RefreshRevision,
    Complete,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct ExcelSyncProgress {
    pub phase: ExcelSyncPhase,
    pub completed: usize,
    pub total: usize,
}

type ProgressCallback = dyn Fn(ExcelSyncProgress) + Send + Sync;

/// Cooperative cancellation and progress reporting for an Excel synchronization.
#[derive(Clone, Default)]
pub struct ExcelSyncControl {
    cancelled: Arc<AtomicBool>,
    progress: Option<Arc<ProgressCallback>>,
}

impl ExcelSyncControl {
    pub fn on_progress(
        mut self,
        progress: impl Fn(ExcelSyncProgress) + Send + Sync + 'static,
    ) -> Self {
        self.progress = Some(Arc::new(progress));
        self
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
    }

    fn checkpoint(&self, phase: ExcelSyncPhase) -> Result<(), ExcelSyncPlanError> {
        if self.is_cancelled() {
            return Err(ExcelSyncPlanError::OperationCancelled);
        }
        self.report(phase);
        if self.is_cancelled() {
            return Err(ExcelSyncPlanError::OperationCancelled);
        }
        Ok(())
    }

    fn report(&self, phase: ExcelSyncPhase) {
        if let Some(progress) = &self.progress {
            progress(ExcelSyncProgress {
                phase,
                completed: excel_phase_index(phase),
                total: 11,
            });
        }
    }
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct ExcelSyncPlan {
    pub plan_id: String,
    pub project_id: ProjectId,
    pub authorization_context: String,
    pub operation_kind: String,
    pub input_revisions: ProjectRevision,
    pub workbook_changes: Vec<ExcelSyncWorkbookChange>,
    pub file_changes: Vec<DataFileChange>,
    pub diagnostics: Vec<Diagnostic>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct ExcelSyncWorkbookChange {
    pub path: String,
    pub created: bool,
    pub sheets: Vec<ExcelSyncSheetChange>,
    pub preserved_sheets: Vec<String>,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct ExcelSyncSheetChange {
    pub sheet: String,
    pub created: bool,
    pub changed: bool,
    pub rows: usize,
    pub added_columns: Vec<String>,
    pub legacy_columns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct ExcelSyncApplyReport {
    pub plan_id: String,
    pub project_id: ProjectId,
    pub previous_revision: ProjectRevision,
    pub revision: ProjectRevision,
    pub workbook_changes: Vec<ExcelSyncWorkbookChange>,
    pub transaction: TransactionReceipt,
}

#[derive(Debug, thiserror::Error)]
pub enum ExcelSyncPlanError {
    #[error("unknown Excel sync plan")]
    UnknownPlan,
    #[error("Excel sync plan has expired")]
    ExpiredPlan,
    #[error("Excel sync plan belongs to a different authorization context")]
    AuthorizationMismatch,
    #[error("Excel sync plan belongs to a different project")]
    ProjectMismatch,
    #[error("expected schema revision does not match the current project")]
    SchemaRevisionConflict,
    #[error("expected data revision does not match the current project")]
    DataRevisionConflict,
    #[error("idempotency key must contain 1-128 printable ASCII characters")]
    InvalidIdempotencyKey,
    #[error("idempotency key was already used for another plan")]
    IdempotencyConflict,
    #[error("workspace Excel sync state lock is poisoned")]
    StatePoisoned,
    #[error("project write lock is poisoned")]
    WriteLockPoisoned,
    #[error("failed to access project: {0}")]
    Project(String),
    #[error("failed to render Excel sync: {0}")]
    Rendering(String),
    #[error("Excel sync transaction failed: {0}")]
    Transaction(String),
    #[error("project revision refresh failed: {0}")]
    Revision(String),
    #[error("Excel synchronization was cancelled")]
    OperationCancelled,
}

#[derive(Debug, Clone)]
struct StoredPlan {
    owner: String,
    plan: ExcelSyncPlan,
}

#[derive(Debug, Clone)]
struct IdempotentApply {
    plan_id: String,
    report: ExcelSyncApplyReport,
}

#[derive(Debug, Default)]
pub(crate) struct ExcelSyncCoordinator {
    plans: RwLock<BTreeMap<String, StoredPlan>>,
    applies: RwLock<BTreeMap<(String, ProjectId, String), IdempotentApply>>,
}

struct RenderedSync {
    writes: Vec<FileWrite>,
    workbook_changes: Vec<ExcelSyncWorkbookChange>,
}

impl WorkspaceService {
    pub fn preview_excel_sync(
        &self,
        project_id: &ProjectId,
        authorization_context: &str,
        expected_schema_revision: &str,
        expected_data_revision: &str,
    ) -> Result<ExcelSyncPlan, ExcelSyncPlanError> {
        self.preview_excel_sync_with_control(
            project_id,
            authorization_context,
            expected_schema_revision,
            expected_data_revision,
            &ExcelSyncControl::default(),
        )
    }

    pub fn preview_excel_sync_with_control(
        &self,
        project_id: &ProjectId,
        authorization_context: &str,
        expected_schema_revision: &str,
        expected_data_revision: &str,
        control: &ExcelSyncControl,
    ) -> Result<ExcelSyncPlan, ExcelSyncPlanError> {
        control.checkpoint(ExcelSyncPhase::ResolveProject)?;
        let session = self
            .project(project_id)
            .map_err(|error| ExcelSyncPlanError::Project(error.to_string()))?;
        control.checkpoint(ExcelSyncPhase::ValidateRevision)?;
        let revision = session.revision();
        ensure_expected_revisions(&revision, expected_schema_revision, expected_data_revision)?;
        let rendered = render_sync(&session, control)?;
        let file_changes = file_changes(project_root(&session), &rendered.writes)?;
        control.checkpoint(ExcelSyncPhase::Diff)?;
        let created_at = Utc::now();
        let plan = ExcelSyncPlan {
            plan_id: format!("plan:{}", Uuid::new_v4()),
            project_id: project_id.clone(),
            authorization_context: authorization_fingerprint(authorization_context),
            operation_kind: "excel_sync".to_owned(),
            input_revisions: revision,
            workbook_changes: rendered.workbook_changes,
            file_changes,
            diagnostics: Vec::new(),
            created_at,
            expires_at: created_at + Duration::minutes(PLAN_TTL_MINUTES),
        };
        self.excel_sync
            .insert(authorization_context, plan.clone())?;
        control.report(ExcelSyncPhase::Complete);
        Ok(plan)
    }

    pub fn apply_excel_sync(
        &self,
        project_id: &ProjectId,
        authorization_context: &str,
        plan_id: &str,
        idempotency_key: &str,
    ) -> Result<ExcelSyncApplyReport, ExcelSyncPlanError> {
        self.apply_excel_sync_with_control(
            project_id,
            authorization_context,
            plan_id,
            idempotency_key,
            &ExcelSyncControl::default(),
        )
    }

    pub fn apply_excel_sync_with_control(
        &self,
        project_id: &ProjectId,
        authorization_context: &str,
        plan_id: &str,
        idempotency_key: &str,
        control: &ExcelSyncControl,
    ) -> Result<ExcelSyncApplyReport, ExcelSyncPlanError> {
        control.checkpoint(ExcelSyncPhase::ResolveProject)?;
        validate_idempotency_key(idempotency_key)?;
        if let Some(report) = self.excel_sync.idempotent(
            authorization_context,
            project_id,
            plan_id,
            idempotency_key,
        )? {
            return Ok(report);
        }
        let stored = self.excel_sync.plan(plan_id)?;
        if stored.owner != authorization_context {
            return Err(ExcelSyncPlanError::AuthorizationMismatch);
        }
        if &stored.plan.project_id != project_id {
            return Err(ExcelSyncPlanError::ProjectMismatch);
        }
        if stored.plan.expires_at <= Utc::now() {
            self.excel_sync.remove(plan_id)?;
            return Err(ExcelSyncPlanError::ExpiredPlan);
        }
        let session = self
            .project(project_id)
            .map_err(|error| ExcelSyncPlanError::Project(error.to_string()))?;
        let _guard = session
            .write_lock
            .lock()
            .map_err(|_| ExcelSyncPlanError::WriteLockPoisoned)?;
        let current = session
            .refresh_revision()
            .map_err(|error| ExcelSyncPlanError::Revision(error.to_string()))?;
        control.checkpoint(ExcelSyncPhase::ValidateRevision)?;
        ensure_expected_revisions(
            &current,
            &stored.plan.input_revisions.schema,
            &stored.plan.input_revisions.data,
        )?;
        let rendered = render_sync(&session, control)?;
        control.checkpoint(ExcelSyncPhase::Commit)?;
        let receipt = commit_file_transaction(project_root(&session), &rendered.writes, || {
            control.report(ExcelSyncPhase::ValidateData);
            session.validated_data().map(|_| ())
        })
        .map_err(|error| ExcelSyncPlanError::Transaction(error.to_string()))?;
        control.report(ExcelSyncPhase::RefreshRevision);
        let revision = session
            .refresh_revision()
            .map_err(|error| ExcelSyncPlanError::Revision(error.to_string()))?;
        let report = ExcelSyncApplyReport {
            plan_id: plan_id.to_owned(),
            project_id: project_id.clone(),
            previous_revision: current,
            revision,
            workbook_changes: rendered.workbook_changes,
            transaction: receipt,
        };
        self.excel_sync.record(
            authorization_context,
            project_id,
            plan_id,
            idempotency_key,
            report.clone(),
        )?;
        self.excel_sync.remove(plan_id)?;
        self.data_mutation
            .invalidate_project(project_id)
            .map_err(|error| ExcelSyncPlanError::Project(error.to_string()))?;
        control.report(ExcelSyncPhase::Complete);
        Ok(report)
    }
}

impl ExcelSyncCoordinator {
    fn insert(&self, owner: &str, plan: ExcelSyncPlan) -> Result<(), ExcelSyncPlanError> {
        let now = Utc::now();
        let mut plans = self
            .plans
            .write()
            .map_err(|_| ExcelSyncPlanError::StatePoisoned)?;
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

    fn plan(&self, id: &str) -> Result<StoredPlan, ExcelSyncPlanError> {
        self.plans
            .read()
            .map_err(|_| ExcelSyncPlanError::StatePoisoned)?
            .get(id)
            .cloned()
            .ok_or(ExcelSyncPlanError::UnknownPlan)
    }

    fn remove(&self, id: &str) -> Result<(), ExcelSyncPlanError> {
        self.plans
            .write()
            .map_err(|_| ExcelSyncPlanError::StatePoisoned)?
            .remove(id);
        Ok(())
    }

    pub(crate) fn invalidate_project(
        &self,
        project_id: &ProjectId,
    ) -> Result<(), ExcelSyncPlanError> {
        self.plans
            .write()
            .map_err(|_| ExcelSyncPlanError::StatePoisoned)?
            .retain(|_, stored| &stored.plan.project_id != project_id);
        Ok(())
    }

    fn idempotent(
        &self,
        owner: &str,
        project_id: &ProjectId,
        plan_id: &str,
        key: &str,
    ) -> Result<Option<ExcelSyncApplyReport>, ExcelSyncPlanError> {
        let applies = self
            .applies
            .read()
            .map_err(|_| ExcelSyncPlanError::StatePoisoned)?;
        match applies.get(&(owner.to_owned(), project_id.clone(), key.to_owned())) {
            Some(record) if record.plan_id == plan_id => Ok(Some(record.report.clone())),
            Some(_) => Err(ExcelSyncPlanError::IdempotencyConflict),
            None => Ok(None),
        }
    }

    fn record(
        &self,
        owner: &str,
        project_id: &ProjectId,
        plan_id: &str,
        key: &str,
        report: ExcelSyncApplyReport,
    ) -> Result<(), ExcelSyncPlanError> {
        self.applies
            .write()
            .map_err(|_| ExcelSyncPlanError::StatePoisoned)?
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

fn render_sync(
    session: &ProjectSession,
    control: &ExcelSyncControl,
) -> Result<RenderedSync, ExcelSyncPlanError> {
    let project_root = project_root(session);
    let data_root = session.data_root();
    control.checkpoint(ExcelSyncPhase::LoadSchema)?;
    let ir = session
        .normalized_schema()
        .map_err(|error| ExcelSyncPlanError::Rendering(error.to_string()))?;
    control.checkpoint(ExcelSyncPhase::InspectWorkbooks)?;
    let preview = ExcelTemplateSync
        .preview(&ir, &data_root)
        .map_err(|error| ExcelSyncPlanError::Rendering(error.to_string()))?;
    let stage_root = project_root
        .join(".sora")
        .join("excel-staging")
        .join(Uuid::new_v4().to_string());
    let result = render_sync_in_stage(&ir, &data_root, &stage_root, preview, control);
    let cleanup_error = fs::remove_dir_all(&stage_root)
        .err()
        .filter(|error| error.kind() != std::io::ErrorKind::NotFound);
    match (result, cleanup_error) {
        (Ok(rendered), None) => Ok(rendered),
        (Ok(_), Some(error)) => Err(ExcelSyncPlanError::Rendering(format!(
            "failed to remove Excel staging directory: {error}"
        ))),
        (Err(error), _) => Err(error),
    }
}

fn render_sync_in_stage(
    ir: &sora_ir::model::ConfigIr,
    data_root: &Path,
    stage_root: &Path,
    preview: ExcelSyncReport,
    control: &ExcelSyncControl,
) -> Result<RenderedSync, ExcelSyncPlanError> {
    control.checkpoint(ExcelSyncPhase::StageWorkbooks)?;
    fs::create_dir_all(stage_root)
        .map_err(|error| ExcelSyncPlanError::Rendering(error.to_string()))?;
    for workbook in preview.workbooks {
        control.checkpoint(ExcelSyncPhase::StageWorkbooks)?;
        if !workbook.path.exists() {
            continue;
        }
        let relative = bounded_relative(data_root, &workbook.path)?;
        let staged = stage_root.join(relative);
        if let Some(parent) = staged.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| ExcelSyncPlanError::Rendering(error.to_string()))?;
        }
        fs::copy(&workbook.path, staged)
            .map_err(|error| ExcelSyncPlanError::Rendering(error.to_string()))?;
    }
    control.checkpoint(ExcelSyncPhase::RenderWorkbooks)?;
    let report = ExcelTemplateSync
        .write(ir, stage_root)
        .map_err(|error| ExcelSyncPlanError::Rendering(error.to_string()))?;
    let mut writes = Vec::new();
    let mut workbook_changes = Vec::new();
    for workbook in report.workbooks {
        control.checkpoint(ExcelSyncPhase::Diff)?;
        let relative = bounded_relative(stage_root, &workbook.path)?;
        let target = data_root.join(&relative);
        if workbook.written {
            writes.push(FileWrite {
                path: target,
                content: Some(
                    fs::read(&workbook.path)
                        .map_err(|error| ExcelSyncPlanError::Rendering(error.to_string()))?,
                ),
            });
        }
        workbook_changes.push(ExcelSyncWorkbookChange {
            path: relative_path(&relative),
            created: workbook.created,
            sheets: workbook
                .sheets
                .into_iter()
                .map(|sheet| ExcelSyncSheetChange {
                    sheet: sheet.sheet,
                    created: sheet.created,
                    changed: sheet.changed,
                    rows: sheet.rows,
                    added_columns: sheet.added_columns,
                    legacy_columns: sheet.legacy_columns,
                })
                .collect(),
            preserved_sheets: workbook.preserved_sheets,
        });
    }
    Ok(RenderedSync {
        writes,
        workbook_changes,
    })
}

fn bounded_relative<'a>(root: &'a Path, path: &'a Path) -> Result<PathBuf, ExcelSyncPlanError> {
    path.strip_prefix(root)
        .map(Path::to_path_buf)
        .map_err(|_| ExcelSyncPlanError::Rendering("workbook escaped data root".to_owned()))
}

fn file_changes(
    root: &Path,
    writes: &[FileWrite],
) -> Result<Vec<DataFileChange>, ExcelSyncPlanError> {
    writes
        .iter()
        .map(|write| {
            let current = match fs::read(&write.path) {
                Ok(content) => content,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => Vec::new(),
                Err(error) => return Err(ExcelSyncPlanError::Rendering(error.to_string())),
            };
            let path = write.path.strip_prefix(root).map_err(|_| {
                ExcelSyncPlanError::Rendering("workbook escaped project root".to_owned())
            })?;
            let next = write.content.as_deref().unwrap_or_default();
            Ok(DataFileChange {
                path: relative_path(path),
                previous_size: current.len(),
                next_size: next.len(),
                previous_digest: digest(&current),
                next_digest: digest(next),
            })
        })
        .collect()
}

fn ensure_expected_revisions(
    revision: &ProjectRevision,
    expected_schema: &str,
    expected_data: &str,
) -> Result<(), ExcelSyncPlanError> {
    if revision.schema != expected_schema {
        return Err(ExcelSyncPlanError::SchemaRevisionConflict);
    }
    if revision.data != expected_data {
        return Err(ExcelSyncPlanError::DataRevisionConflict);
    }
    Ok(())
}

fn validate_idempotency_key(key: &str) -> Result<(), ExcelSyncPlanError> {
    if !key.is_empty() && key.len() <= 128 && key.bytes().all(|byte| byte.is_ascii_graphic()) {
        Ok(())
    } else {
        Err(ExcelSyncPlanError::InvalidIdempotencyKey)
    }
}

fn authorization_fingerprint(context: &str) -> String {
    format!("auth:{}", hex_digest(context.as_bytes()))
}

fn digest(bytes: &[u8]) -> String {
    format!("sha256:{}", hex_digest(bytes))
}

fn hex_digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn relative_path(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn project_root(session: &ProjectSession) -> &Path {
    session
        .manifest_path()
        .parent()
        .unwrap_or_else(|| Path::new("."))
}

fn excel_phase_index(phase: ExcelSyncPhase) -> usize {
    match phase {
        ExcelSyncPhase::ResolveProject => 1,
        ExcelSyncPhase::ValidateRevision => 2,
        ExcelSyncPhase::LoadSchema => 3,
        ExcelSyncPhase::InspectWorkbooks => 4,
        ExcelSyncPhase::StageWorkbooks => 5,
        ExcelSyncPhase::RenderWorkbooks => 6,
        ExcelSyncPhase::Diff => 7,
        ExcelSyncPhase::Commit => 8,
        ExcelSyncPhase::ValidateData => 9,
        ExcelSyncPhase::RefreshRevision => 10,
        ExcelSyncPhase::Complete => 11,
    }
}
