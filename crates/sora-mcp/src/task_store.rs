use std::{
    collections::BTreeMap,
    sync::Mutex,
    time::{Duration, Instant},
};

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use rmcp::model::{ErrorData as McpError, ListTasksResult, Task, TaskStatus};
use serde_json::Value;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

const DEFAULT_TTL_MS: u64 = 5 * 60 * 1_000;
const MIN_TTL_MS: u64 = 1_000;
const MAX_TTL_MS: u64 = 10 * 60 * 1_000;
const POLL_INTERVAL_MS: u64 = 250;
const MAX_ACTIVE_TASKS_PER_OWNER: usize = 8;
const MAX_RETAINED_TASKS: usize = 64;
const PAGE_SIZE: usize = 20;
const CURSOR_PREFIX: &str = "sora-task-v1:";

#[derive(Debug)]
pub(crate) struct TaskStore {
    state: Mutex<TaskStoreState>,
}

#[derive(Debug, Default)]
struct TaskStoreState {
    next_sequence: u64,
    records: BTreeMap<String, TaskRecord>,
}

#[derive(Debug)]
struct TaskRecord {
    owner: String,
    project_id: Option<String>,
    sequence: u64,
    task: Task,
    cancellation: CancellationToken,
    result: Option<Result<Value, McpError>>,
    expires_at: Option<Instant>,
}

#[derive(Debug)]
pub(crate) struct CreatedTask {
    pub task: Task,
    pub cancellation: CancellationToken,
}

impl Default for TaskStore {
    fn default() -> Self {
        Self {
            state: Mutex::new(TaskStoreState::default()),
        }
    }
}

impl TaskStore {
    pub(crate) fn create(
        &self,
        owner: &str,
        project_id: Option<String>,
        requested_ttl_ms: Option<u64>,
    ) -> Result<CreatedTask, McpError> {
        let mut state = self.lock()?;
        prune(&mut state);
        let active = state
            .records
            .values()
            .filter(|record| {
                record.owner == owner
                    && matches!(
                        record.task.status,
                        TaskStatus::Working | TaskStatus::InputRequired
                    )
            })
            .count();
        if active >= MAX_ACTIVE_TASKS_PER_OWNER {
            return Err(McpError::invalid_request(
                format!("too many active tasks; at most {MAX_ACTIVE_TASKS_PER_OWNER} are allowed"),
                None,
            ));
        }

        evict_terminal_records(&mut state);
        let task_id = Uuid::new_v4().to_string();
        let timestamp = rmcp::task_manager::current_timestamp();
        let ttl_ms = requested_ttl_ms
            .unwrap_or(DEFAULT_TTL_MS)
            .clamp(MIN_TTL_MS, MAX_TTL_MS);
        let task = Task::new(
            task_id.clone(),
            TaskStatus::Working,
            timestamp.clone(),
            timestamp,
        )
        .with_status_message("Task accepted")
        .with_ttl(ttl_ms)
        .with_poll_interval(POLL_INTERVAL_MS);
        let cancellation = CancellationToken::new();
        let sequence = state.next_sequence;
        state.next_sequence = state.next_sequence.saturating_add(1);
        state.records.insert(
            task_id,
            TaskRecord {
                owner: owner.to_owned(),
                project_id,
                sequence,
                task: task.clone(),
                cancellation: cancellation.clone(),
                result: None,
                expires_at: None,
            },
        );
        Ok(CreatedTask { task, cancellation })
    }

    pub(crate) fn finish(
        &self,
        owner: &str,
        task_id: &str,
        result: Result<Value, McpError>,
    ) -> Result<Task, McpError> {
        let mut state = self.lock()?;
        let record = owned_record_mut(&mut state, owner, task_id)?;
        let result_is_error = match &result {
            Ok(value) => value
                .get("isError")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            Err(_) => true,
        };
        record.result = Some(result);
        record.task.status = if record.cancellation.is_cancelled() && result_is_error {
            TaskStatus::Cancelled
        } else if result_is_error {
            TaskStatus::Failed
        } else {
            TaskStatus::Completed
        };
        record.task.status_message = Some(
            match record.task.status {
                TaskStatus::Completed => "Task completed",
                TaskStatus::Failed => "Task failed",
                TaskStatus::Cancelled => "Task cancelled",
                TaskStatus::Working | TaskStatus::InputRequired => unreachable!(),
                _ => "Task finished",
            }
            .to_owned(),
        );
        record.task.last_updated_at = rmcp::task_manager::current_timestamp();
        record.expires_at =
            Some(Instant::now() + Duration::from_millis(record.task.ttl.unwrap_or(DEFAULT_TTL_MS)));
        Ok(record.task.clone())
    }

    pub(crate) fn list(
        &self,
        owner: &str,
        cursor: Option<&str>,
    ) -> Result<ListTasksResult, McpError> {
        let offset = decode_cursor(cursor)?;
        let mut state = self.lock()?;
        prune(&mut state);
        let mut records = state
            .records
            .values()
            .filter(|record| record.owner == owner)
            .collect::<Vec<_>>();
        records.sort_by_key(|record| record.sequence);
        if offset > records.len() {
            return Err(McpError::invalid_params(
                "task cursor is out of range",
                None,
            ));
        }
        let tasks = records
            .iter()
            .skip(offset)
            .take(PAGE_SIZE)
            .map(|record| record.task.clone())
            .collect::<Vec<_>>();
        let next_offset = offset + tasks.len();
        let mut result = ListTasksResult::new(tasks);
        if next_offset < records.len() {
            result.next_cursor = Some(encode_cursor(next_offset));
        }
        Ok(result)
    }

    pub(crate) fn get(&self, owner: &str, task_id: &str) -> Result<Task, McpError> {
        let mut state = self.lock()?;
        prune(&mut state);
        Ok(owned_record(&state, owner, task_id)?.task.clone())
    }

    pub(crate) fn get_for_project(
        &self,
        owner: &str,
        project_id: &str,
        task_id: &str,
    ) -> Result<Task, McpError> {
        let mut state = self.lock()?;
        prune(&mut state);
        let record = owned_record(&state, owner, task_id)?;
        if record.project_id.as_deref() != Some(project_id) {
            return Err(task_not_found());
        }
        Ok(record.task.clone())
    }

    pub(crate) fn result(
        &self,
        owner: &str,
        task_id: &str,
    ) -> Result<Result<Value, McpError>, McpError> {
        let mut state = self.lock()?;
        prune(&mut state);
        let record = owned_record(&state, owner, task_id)?;
        match record.task.status {
            TaskStatus::Working | TaskStatus::InputRequired => {
                Err(McpError::invalid_request("task result is not ready", None))
            }
            TaskStatus::Cancelled => Err(McpError::invalid_request("task was cancelled", None)),
            TaskStatus::Completed | TaskStatus::Failed => record
                .result
                .clone()
                .ok_or_else(|| McpError::internal_error("task result is missing", None)),
            _ => Err(McpError::invalid_request("task result is not ready", None)),
        }
    }

    pub(crate) fn cancel(&self, owner: &str, task_id: &str) -> Result<Task, McpError> {
        let mut state = self.lock()?;
        prune(&mut state);
        let record = owned_record_mut(&mut state, owner, task_id)?;
        match record.task.status {
            TaskStatus::Working | TaskStatus::InputRequired => {
                record.cancellation.cancel();
                record.task.status = TaskStatus::Working;
                record.task.status_message = Some("Cancellation requested".to_owned());
                record.task.last_updated_at = rmcp::task_manager::current_timestamp();
            }
            TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Cancelled => {}
            _ => {}
        }
        Ok(record.task.clone())
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, TaskStoreState>, McpError> {
        self.state
            .lock()
            .map_err(|_| McpError::internal_error("task store lock poisoned", None))
    }
}

fn owned_record<'a>(
    state: &'a TaskStoreState,
    owner: &str,
    task_id: &str,
) -> Result<&'a TaskRecord, McpError> {
    state
        .records
        .get(task_id)
        .filter(|record| record.owner == owner)
        .ok_or_else(task_not_found)
}

fn owned_record_mut<'a>(
    state: &'a mut TaskStoreState,
    owner: &str,
    task_id: &str,
) -> Result<&'a mut TaskRecord, McpError> {
    state
        .records
        .get_mut(task_id)
        .filter(|record| record.owner == owner)
        .ok_or_else(task_not_found)
}

fn prune(state: &mut TaskStoreState) {
    let now = Instant::now();
    state
        .records
        .retain(|_, record| record.expires_at.is_none_or(|expires_at| expires_at > now));
}

fn evict_terminal_records(state: &mut TaskStoreState) {
    while state.records.len() >= MAX_RETAINED_TASKS {
        let oldest_terminal = state
            .records
            .iter()
            .filter(|(_, record)| {
                matches!(
                    record.task.status,
                    TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Cancelled
                )
            })
            .min_by_key(|(_, record)| record.sequence)
            .map(|(task_id, _)| task_id.clone());
        let Some(task_id) = oldest_terminal else {
            break;
        };
        state.records.remove(&task_id);
    }
}

fn encode_cursor(offset: usize) -> String {
    URL_SAFE_NO_PAD.encode(format!("{CURSOR_PREFIX}{offset}"))
}

fn decode_cursor(cursor: Option<&str>) -> Result<usize, McpError> {
    let Some(cursor) = cursor else {
        return Ok(0);
    };
    let bytes = URL_SAFE_NO_PAD
        .decode(cursor)
        .map_err(|_| McpError::invalid_params("invalid task cursor", None))?;
    let value = std::str::from_utf8(&bytes)
        .map_err(|_| McpError::invalid_params("invalid task cursor", None))?;
    value
        .strip_prefix(CURSOR_PREFIX)
        .ok_or_else(|| McpError::invalid_params("invalid task cursor", None))?
        .parse()
        .map_err(|_| McpError::invalid_params("invalid task cursor", None))
}

fn task_not_found() -> McpError {
    McpError::invalid_params("task not found", None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tasks_are_owner_and_project_bound() {
        let store = TaskStore::default();
        let created = store
            .create("owner-a", Some("project-a".to_owned()), None)
            .expect("create task");

        assert!(store.get("owner-b", &created.task.task_id).is_err());
        assert!(
            store
                .get_for_project("owner-a", "project-b", &created.task.task_id)
                .is_err()
        );
        assert_eq!(
            store
                .get_for_project("owner-a", "project-a", &created.task.task_id)
                .expect("project task")
                .status,
            TaskStatus::Working
        );
    }

    #[test]
    fn cancellation_becomes_terminal_after_failed_work_stops() {
        let store = TaskStore::default();
        let created = store.create("owner", None, None).expect("create task");
        let cancelling = store
            .cancel("owner", &created.task.task_id)
            .expect("cancel task");

        assert_eq!(cancelling.status, TaskStatus::Working);
        assert_eq!(
            cancelling.status_message.as_deref(),
            Some("Cancellation requested")
        );
        assert!(created.cancellation.is_cancelled());

        let task = store
            .finish(
                "owner",
                &created.task.task_id,
                Ok(serde_json::json!({"isError": true})),
            )
            .expect("finish task");

        assert_eq!(task.status, TaskStatus::Cancelled);
    }

    #[test]
    fn successful_work_that_passed_the_commit_point_remains_completed() {
        let store = TaskStore::default();
        let created = store.create("owner", None, None).expect("create task");
        store
            .cancel("owner", &created.task.task_id)
            .expect("cancel task");

        let task = store
            .finish(
                "owner",
                &created.task.task_id,
                Ok(serde_json::json!({"isError": false})),
            )
            .expect("finish task");

        assert_eq!(task.status, TaskStatus::Completed);
    }

    #[test]
    fn malformed_cursors_are_rejected() {
        let store = TaskStore::default();
        assert!(store.list("owner", Some("not-base64!")).is_err());
    }
}
