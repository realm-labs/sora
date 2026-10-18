use std::{
    collections::BTreeMap,
    future::Future,
    sync::{
        Arc, Mutex, MutexGuard,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

use rmcp::{
    ErrorData as McpError,
    model::{CallToolResult, DetailedTask, JsonObject, Task, TaskPayload, TaskStatus},
    task_manager::current_timestamp,
};
use serde::Serialize;
use serde_json::{Value, to_value};
use tokio::{
    pin, select,
    time::{Instant as TokioInstant, sleep_until},
};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

const EXECUTION_TTL: Duration = Duration::from_secs(300);
const RESULT_RETENTION: Duration = Duration::from_secs(60);
const POLL_INTERVAL_MS: u64 = 250;
const MAX_ACTIVE_TASKS_PER_OWNER: usize = 8;
const MAX_RETAINED_TASKS: usize = 64;

#[derive(Debug)]
pub(crate) struct TaskStore {
    records: Mutex<BTreeMap<String, TaskRecord>>,
    execution_ttl: Duration,
    result_retention: Duration,
}

#[derive(Debug)]
struct TaskRecord {
    owner: String,
    project_id: Option<String>,
    created: Instant,
    deadline: Instant,
    task: DetailedTask,
    cancellation: CancellationToken,
    retain_until: Option<Instant>,
}

#[derive(Debug)]
pub(crate) struct CreatedTask {
    pub task: Task,
    pub cancellation: CancellationToken,
    pub deadline: Instant,
}

impl CreatedTask {
    /// The deadline signals cancellation, but never drops a worker that may be
    /// inside an atomic commit. Final state comes from the worker's real result.
    pub(crate) async fn run<T>(&self, work: impl Future<Output = T>) -> T {
        pin!(work);
        select! {
            result = &mut work => result,
            _ = sleep_until(TokioInstant::from_std(self.deadline)) => {
                self.cancellation.cancel();
                work.await
            }
        }
    }
}

/// Only a worker's cancellation checkpoint can confirm that work really stopped.
/// A cancelled token alone cannot distinguish cancellation from a completed commit.
#[derive(Debug, Clone, Default)]
pub(crate) struct TaskExecutionState {
    cancellation_observed: Arc<AtomicBool>,
}

impl TaskExecutionState {
    pub(crate) fn mark_cancellation_observed(&self) {
        self.cancellation_observed.store(true, Ordering::Release);
    }

    fn cancellation_observed(&self) -> bool {
        self.cancellation_observed.load(Ordering::Acquire)
    }
}

impl Default for TaskStore {
    fn default() -> Self {
        Self {
            records: Mutex::new(BTreeMap::new()),
            execution_ttl: EXECUTION_TTL,
            result_retention: RESULT_RETENTION,
        }
    }
}

impl TaskStore {
    pub(crate) fn create(
        &self,
        owner: &str,
        project_id: Option<String>,
    ) -> Result<CreatedTask, McpError> {
        let mut records = self.lock()?;
        sweep(&mut records, Instant::now());
        if records
            .values()
            .filter(|record| record.owner == owner && !record.task.status().is_terminal())
            .count()
            >= MAX_ACTIVE_TASKS_PER_OWNER
        {
            return Err(McpError::invalid_request(
                format!("too many active tasks; at most {MAX_ACTIVE_TASKS_PER_OWNER} are allowed"),
                None,
            ));
        }
        if records.len() >= MAX_RETAINED_TASKS {
            let oldest_terminal = records
                .iter()
                .filter(|(_, record)| record.task.status().is_terminal())
                .min_by_key(|(_, record)| record.created)
                .map(|(id, _)| id.clone());
            if let Some(id) = oldest_terminal {
                records.remove(&id);
            } else {
                return Err(McpError::invalid_request(
                    "task store capacity reached",
                    None,
                ));
            }
        }

        let task_id = Uuid::new_v4().to_string();
        let timestamp = current_timestamp();
        let task = Task::new(
            task_id.clone(),
            TaskStatus::Working,
            timestamp.clone(),
            timestamp,
        )
        .with_status_message("Task accepted")
        .with_ttl_ms(self.execution_ttl.as_millis() as u64)
        .with_poll_interval_ms(POLL_INTERVAL_MS);
        let cancellation = CancellationToken::new();
        let created = Instant::now();
        let deadline = created + self.execution_ttl;
        records.insert(
            task_id,
            TaskRecord {
                owner: owner.to_owned(),
                project_id,
                created,
                deadline,
                task: DetailedTask::new(task.clone(), TaskPayload::Working),
                cancellation: cancellation.clone(),
                retain_until: None,
            },
        );
        Ok(CreatedTask {
            task,
            cancellation,
            deadline,
        })
    }

    pub(crate) fn finish(
        &self,
        owner: &str,
        task_id: &str,
        result: Result<CallToolResult, McpError>,
        execution: &TaskExecutionState,
    ) -> Result<DetailedTask, McpError> {
        let mut records = self.lock()?;
        let record = owned_record_mut(&mut records, owner, task_id)?;
        if record.task.status().is_terminal() {
            return Ok(record.task.clone());
        }
        let failed = result
            .as_ref()
            .map_or(true, |result| result.is_error == Some(true));
        let payload = if execution.cancellation_observed() && failed {
            TaskPayload::Cancelled
        } else {
            match result {
                Ok(result) => TaskPayload::Completed {
                    result: json_object(result)?,
                },
                Err(error) => TaskPayload::Failed {
                    error: json_object(error)?,
                },
            }
        };
        let mut task = record.task.task.clone();
        task.status_message = Some(
            match &payload {
                TaskPayload::Completed { .. } => "Task completed",
                TaskPayload::Failed { .. } => "Task failed",
                TaskPayload::Cancelled => "Task cancelled",
                _ => unreachable!(),
            }
            .to_owned(),
        );
        task.last_updated_at = current_timestamp();
        // A result has its own observation window, independent of the execution
        // deadline. Update ttlMs so clients can still use a just-finished task.
        let now = Instant::now();
        task.ttl_ms =
            Some((now.duration_since(record.created) + self.result_retention).as_millis() as u64);
        record.task = DetailedTask::new(task, payload);
        record.retain_until = Some(now + self.result_retention);
        Ok(record.task.clone())
    }

    pub(crate) fn get(&self, owner: &str, task_id: &str) -> Result<DetailedTask, McpError> {
        let mut records = self.lock()?;
        sweep(&mut records, Instant::now());
        Ok(owned_record(&records, owner, task_id)?.task.clone())
    }

    pub(crate) fn get_for_project(
        &self,
        owner: &str,
        project_id: &str,
        task_id: &str,
    ) -> Result<DetailedTask, McpError> {
        let mut records = self.lock()?;
        sweep(&mut records, Instant::now());
        let record = owned_record(&records, owner, task_id)?;
        if record.project_id.as_deref() != Some(project_id) {
            return Err(task_not_found());
        }
        Ok(record.task.clone())
    }

    pub(crate) fn cancel(&self, owner: &str, task_id: &str) -> Result<(), McpError> {
        let mut records = self.lock()?;
        sweep(&mut records, Instant::now());
        request_cancellation(
            owned_record_mut(&mut records, owner, task_id)?,
            "Cancellation requested",
        );
        Ok(())
    }

    /// Preserve authorization-local state while a task is running or its result
    /// remains observable. Fail closed on a poisoned lock rather than evict it.
    pub(crate) fn has_retained_tasks(&self) -> bool {
        let Ok(mut records) = self.records.lock() else {
            return true;
        };
        sweep(&mut records, Instant::now());
        !records.is_empty()
    }

    fn lock(&self) -> Result<MutexGuard<'_, BTreeMap<String, TaskRecord>>, McpError> {
        self.records
            .lock()
            .map_err(|_| McpError::internal_error("task store lock poisoned", None))
    }
}

fn json_object(value: impl Serialize) -> Result<JsonObject, McpError> {
    match to_value(value) {
        Ok(Value::Object(object)) => Ok(object),
        _ => Err(McpError::internal_error(
            "failed to serialize task payload",
            None,
        )),
    }
}

fn owned_record<'a>(
    records: &'a BTreeMap<String, TaskRecord>,
    owner: &str,
    id: &str,
) -> Result<&'a TaskRecord, McpError> {
    records
        .get(id)
        .filter(|record| record.owner == owner)
        .ok_or_else(task_not_found)
}

fn owned_record_mut<'a>(
    records: &'a mut BTreeMap<String, TaskRecord>,
    owner: &str,
    id: &str,
) -> Result<&'a mut TaskRecord, McpError> {
    records
        .get_mut(id)
        .filter(|record| record.owner == owner)
        .ok_or_else(task_not_found)
}

fn request_cancellation(record: &mut TaskRecord, message: &str) {
    if !record.task.status().is_terminal() && !record.cancellation.is_cancelled() {
        record.cancellation.cancel();
        record.task.task.status_message = Some(message.to_owned());
        record.task.task.last_updated_at = current_timestamp();
    }
}

fn sweep(records: &mut BTreeMap<String, TaskRecord>, now: Instant) {
    records.retain(|_, record| record.retain_until.is_none_or(|until| until > now));
    for record in records.values_mut() {
        if now >= record.deadline {
            request_cancellation(record, "Execution deadline reached; cancellation requested");
        }
    }
}

fn task_not_found() -> McpError {
    McpError::invalid_params("task not found", None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tokio::time::{sleep, timeout};

    fn result(is_error: bool) -> CallToolResult {
        if is_error {
            CallToolResult::structured_error(json!({"ok": false}))
        } else {
            CallToolResult::structured(json!({"ok": true}))
        }
    }

    #[test]
    fn tasks_are_owner_and_project_bound() {
        let store = TaskStore::default();
        let task = store.create("a", Some("project-a".into())).unwrap().task;
        assert!(store.get("b", &task.task_id).is_err());
        assert!(store.cancel("b", &task.task_id).is_err());
        assert!(
            store
                .finish(
                    "b",
                    &task.task_id,
                    Ok(result(false)),
                    &TaskExecutionState::default()
                )
                .is_err()
        );
        assert!(
            store
                .get_for_project("a", "project-b", &task.task_id)
                .is_err()
        );
        assert_eq!(
            store
                .get_for_project("a", "project-a", &task.task_id)
                .unwrap()
                .status(),
            TaskStatus::Working
        );
    }

    #[test]
    fn success_and_business_errors_are_completed_with_the_original_result() {
        for is_error in [false, true] {
            let store = TaskStore::default();
            let task = store.create("a", None).unwrap().task;
            let task = store
                .finish(
                    "a",
                    &task.task_id,
                    Ok(result(is_error)),
                    &TaskExecutionState::default(),
                )
                .unwrap();
            assert_eq!(task.status(), TaskStatus::Completed);
            let TaskPayload::Completed { result } = task.payload else {
                panic!("completed payload")
            };
            assert_eq!(result["isError"], is_error);
            assert_eq!(result["structuredContent"]["ok"], !is_error);
        }
    }

    #[test]
    fn cancellation_request_does_not_mask_an_unrelated_failure() {
        let store = TaskStore::default();
        let task = store.create("a", None).unwrap().task;
        store.cancel("a", &task.task_id).unwrap();
        assert_eq!(
            store.get("a", &task.task_id).unwrap().status(),
            TaskStatus::Working
        );
        let task = store
            .finish(
                "a",
                &task.task_id,
                Err(McpError::internal_error("unrelated failure", None)),
                &TaskExecutionState::default(),
            )
            .unwrap();
        assert_eq!(task.status(), TaskStatus::Failed);
        let TaskPayload::Failed { error } = task.payload else {
            panic!("error payload")
        };
        assert_eq!(error["message"], "unrelated failure");
    }

    #[test]
    fn unobserved_cancellation_preserves_a_business_error_as_completed() {
        let store = TaskStore::default();
        let task = store.create("a", None).unwrap().task;
        store.cancel("a", &task.task_id).unwrap();
        let task = store
            .finish(
                "a",
                &task.task_id,
                Ok(result(true)),
                &TaskExecutionState::default(),
            )
            .unwrap();
        assert_eq!(task.status(), TaskStatus::Completed);
        let TaskPayload::Completed { result } = task.payload else {
            panic!("tool error payload")
        };
        assert_eq!(result["isError"], true);
    }

    #[test]
    fn observed_cancellation_becomes_cancelled_after_work_stops() {
        let store = TaskStore::default();
        let task = store.create("a", None).unwrap().task;
        store.cancel("a", &task.task_id).unwrap();
        let execution = TaskExecutionState::default();
        execution.mark_cancellation_observed();
        assert_eq!(
            store
                .finish("a", &task.task_id, Ok(result(true)), &execution)
                .unwrap()
                .status(),
            TaskStatus::Cancelled
        );
    }

    #[test]
    fn successful_work_that_passed_the_commit_point_remains_completed() {
        let store = TaskStore::default();
        let task = store.create("a", None).unwrap().task;
        store.cancel("a", &task.task_id).unwrap();
        let execution = TaskExecutionState::default();
        execution.mark_cancellation_observed();
        assert_eq!(
            store
                .finish("a", &task.task_id, Ok(result(false)), &execution)
                .unwrap()
                .status(),
            TaskStatus::Completed
        );
    }

    #[test]
    fn active_limit_is_per_owner_and_total_capacity_is_strict_across_owners() {
        let store = TaskStore::default();
        for _ in 0..MAX_ACTIVE_TASKS_PER_OWNER {
            store.create("a", None).unwrap();
        }
        assert!(store.create("a", None).is_err());
        for index in MAX_ACTIVE_TASKS_PER_OWNER..MAX_RETAINED_TASKS {
            store.create(&format!("owner-{index}"), None).unwrap();
        }
        assert!(store.create("another-owner", None).is_err());
        assert_eq!(store.records.lock().unwrap().len(), MAX_RETAINED_TASKS);
    }

    #[test]
    fn terminal_records_can_be_evicted_without_evicting_live_work() {
        let store = TaskStore::default();
        let oldest = store.create("done", None).unwrap().task;
        store
            .finish(
                "done",
                &oldest.task_id,
                Ok(result(false)),
                &TaskExecutionState::default(),
            )
            .unwrap();
        for index in 1..MAX_RETAINED_TASKS {
            store.create(&format!("owner-{index}"), None).unwrap();
        }
        store.create("new-owner", None).unwrap();
        assert!(store.get("done", &oldest.task_id).is_err());
        assert_eq!(store.records.lock().unwrap().len(), MAX_RETAINED_TASKS);
    }

    #[test]
    fn ttl_requests_cooperative_cancellation_without_fabricating_a_terminal_state() {
        let store = TaskStore {
            execution_ttl: Duration::ZERO,
            ..TaskStore::default()
        };
        let created = store.create("a", None).unwrap();
        let task = store.get("a", &created.task.task_id).unwrap();
        assert!(created.cancellation.is_cancelled());
        assert_eq!(task.status(), TaskStatus::Working);
        assert_eq!(task.task.ttl_ms, Some(0));
        assert!(store.has_retained_tasks());
        let execution = TaskExecutionState::default();
        execution.mark_cancellation_observed();
        assert_eq!(
            store
                .finish("a", &created.task.task_id, Ok(result(true)), &execution)
                .unwrap()
                .status(),
            TaskStatus::Cancelled
        );
    }

    #[tokio::test]
    async fn execution_deadline_cancels_the_worker_without_client_polling() {
        let store = TaskStore {
            execution_ttl: Duration::from_millis(5),
            ..TaskStore::default()
        };
        let created = store.create("a", None).unwrap();
        let execution = TaskExecutionState::default();
        let outcome = timeout(
            Duration::from_secs(1),
            created.run(async {
                created.cancellation.cancelled().await;
                execution.mark_cancellation_observed();
                Ok(result(true))
            }),
        )
        .await
        .expect("deadline must wake the worker without a poll");
        assert_eq!(
            store
                .finish("a", &created.task.task_id, outcome, &execution)
                .unwrap()
                .status(),
            TaskStatus::Cancelled
        );
    }

    #[tokio::test]
    async fn deadline_waits_for_commit_instead_of_aborting_or_fabricating_cancellation() {
        let store = TaskStore {
            execution_ttl: Duration::from_millis(5),
            ..TaskStore::default()
        };
        let created = store.create("a", None).unwrap();
        let committed = AtomicBool::new(false);
        let outcome = created
            .run(async {
                created.cancellation.cancelled().await;
                sleep(Duration::from_millis(5)).await;
                committed.store(true, Ordering::Release);
                Ok(result(false))
            })
            .await;
        assert!(committed.load(Ordering::Acquire));
        let task = store
            .finish(
                "a",
                &created.task.task_id,
                outcome,
                &TaskExecutionState::default(),
            )
            .unwrap();
        assert_eq!(task.status(), TaskStatus::Completed);
        assert!(task.task.ttl_ms.unwrap() >= RESULT_RETENTION.as_millis() as u64);
    }

    #[test]
    fn finished_results_have_a_separate_retention_window() {
        let store = TaskStore {
            result_retention: Duration::ZERO,
            ..TaskStore::default()
        };
        let task = store.create("a", None).unwrap().task;
        store
            .finish(
                "a",
                &task.task_id,
                Ok(result(false)),
                &TaskExecutionState::default(),
            )
            .unwrap();
        assert!(store.get("a", &task.task_id).is_err());
        assert!(!store.has_retained_tasks());
    }

    #[test]
    fn terminal_status_is_immutable() {
        let store = TaskStore::default();
        let task = store.create("a", None).unwrap().task;
        let done = store
            .finish(
                "a",
                &task.task_id,
                Ok(result(false)),
                &TaskExecutionState::default(),
            )
            .unwrap();
        store.cancel("a", &task.task_id).unwrap();
        assert_eq!(
            store
                .finish(
                    "a",
                    &task.task_id,
                    Err(McpError::internal_error("late", None)),
                    &TaskExecutionState::default()
                )
                .unwrap(),
            done
        );
    }
}
