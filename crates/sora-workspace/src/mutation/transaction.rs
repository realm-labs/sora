use std::{
    fs::{self, File},
    io::Write,
    path::{Component, Path, PathBuf},
};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::studio::service::TextFileWrite;

const STATE_DIRECTORY: &str = ".sora";
const TRANSACTION_DIRECTORY: &str = "transactions";
const BACKUP_DIRECTORY: &str = "backups";

#[derive(Debug, Clone)]
pub(crate) struct FileWrite {
    pub path: PathBuf,
    pub content: Vec<u8>,
}

/// Information retained after a committed filesystem transaction.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct TransactionReceipt {
    pub transaction_id: String,
    pub backup_id: String,
    pub affected_files: Vec<String>,
}

/// Stable transaction and recovery failures.
#[derive(Debug, thiserror::Error)]
pub enum TransactionError {
    #[error("transaction target `{0}` resolves outside the project")]
    TargetOutsideProject(PathBuf),
    #[error("transaction target cannot modify Sora's internal state directory")]
    InternalStateTarget,
    #[error("failed to {action} `{path}`")]
    Io {
        action: &'static str,
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to encode transaction journal")]
    EncodeJournal(#[source] serde_json::Error),
    #[error("failed to decode transaction journal `{path}`")]
    DecodeJournal {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("post-commit validation failed: {0}")]
    PostValidation(String),
    #[error("transaction rollback failed after `{cause}`: {rollback}")]
    Rollback { cause: String, rollback: String },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum JournalPhase {
    Prepared,
    Committing,
    Committed,
}

#[derive(Debug, Serialize, Deserialize)]
struct TransactionJournal {
    version: u32,
    transaction_id: String,
    phase: JournalPhase,
    entries: Vec<JournalEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
struct JournalEntry {
    target: String,
    staged: String,
    backup: Option<String>,
}

/// Commits rendered text files with a persistent journal, recoverable backups,
/// rollback, and post-write validation.
pub(crate) fn commit_text_transaction<F>(
    project_root: &Path,
    writes: &[TextFileWrite],
    validate: F,
) -> Result<TransactionReceipt, TransactionError>
where
    F: FnOnce() -> anyhow::Result<()>,
{
    let writes = writes
        .iter()
        .map(|write| FileWrite {
            path: write.path.clone(),
            content: write.content.as_bytes().to_vec(),
        })
        .collect::<Vec<_>>();
    commit_file_transaction(project_root, &writes, validate)
}

pub(crate) fn commit_file_transaction<F>(
    project_root: &Path,
    writes: &[FileWrite],
    validate: F,
) -> Result<TransactionReceipt, TransactionError>
where
    F: FnOnce() -> anyhow::Result<()>,
{
    let root = canonical_directory(project_root)?;
    let id = Uuid::new_v4().to_string();
    let state_root = root.join(STATE_DIRECTORY);
    let transaction_root = state_root.join(TRANSACTION_DIRECTORY).join(&id);
    let backup_root = state_root.join(BACKUP_DIRECTORY).join(&id);
    create_dir_all(&transaction_root)?;
    create_dir_all(&backup_root)?;

    let mut journal = TransactionJournal {
        version: 1,
        transaction_id: id.clone(),
        phase: JournalPhase::Prepared,
        entries: Vec::with_capacity(writes.len()),
    };
    for (index, write) in writes.iter().enumerate() {
        let target = bounded_target(&root, &write.path)?;
        let relative = target
            .strip_prefix(&root)
            .map_err(|_| TransactionError::TargetOutsideProject(target.clone()))?;
        if relative
            .components()
            .next()
            .is_some_and(|component| component.as_os_str() == STATE_DIRECTORY)
        {
            return Err(TransactionError::InternalStateTarget);
        }
        let staged = transaction_root.join(format!("{index}.stage"));
        write_durable(&staged, &write.content)?;
        let backup = if target.exists() {
            let backup = backup_root.join(format!("{index}.backup"));
            copy_file(&target, &backup)?;
            Some(relative_string(backup.strip_prefix(&root).map_err(
                |_| TransactionError::TargetOutsideProject(backup.clone()),
            )?))
        } else {
            None
        };
        journal.entries.push(JournalEntry {
            target: relative_string(relative),
            staged: relative_string(
                staged
                    .strip_prefix(&root)
                    .map_err(|_| TransactionError::TargetOutsideProject(staged.clone()))?,
            ),
            backup,
        });
    }

    let journal_path = state_root
        .join(TRANSACTION_DIRECTORY)
        .join(format!("{id}.json"));
    write_journal(&journal_path, &journal)?;
    journal.phase = JournalPhase::Committing;
    write_journal(&journal_path, &journal)?;

    for entry in &journal.entries {
        let target = root.join(&entry.target);
        let staged = root.join(&entry.staged);
        let result = (|| {
            if let Some(parent) = target.parent() {
                create_dir_all(parent)?;
            }
            replace_file(&staged, &target)
        })();
        if let Err(error) = result {
            return rollback_failure(&root, &journal_path, &journal, error.to_string());
        }
    }

    if let Err(error) = validate() {
        return rollback_failure(
            &root,
            &journal_path,
            &journal,
            TransactionError::PostValidation(error.to_string()).to_string(),
        );
    }

    journal.phase = JournalPhase::Committed;
    write_journal(&journal_path, &journal)?;
    remove_file_if_exists(&journal_path)?;
    remove_dir_all_if_exists(&transaction_root)?;
    Ok(TransactionReceipt {
        transaction_id: format!("txn:{id}"),
        backup_id: format!("backup:{id}"),
        affected_files: journal
            .entries
            .iter()
            .map(|entry| entry.target.clone())
            .collect(),
    })
}

/// Recovers incomplete transactions before a project is loaded.
pub(crate) fn recover_transactions(project_root: &Path) -> Result<(), TransactionError> {
    let root = canonical_directory(project_root)?;
    let journal_root = root.join(STATE_DIRECTORY).join(TRANSACTION_DIRECTORY);
    if !journal_root.exists() {
        return Ok(());
    }
    let mut paths = read_dir_files(&journal_root)?;
    paths.sort();
    for path in paths {
        if path.extension().and_then(|extension| extension.to_str()) != Some("json") {
            continue;
        }
        let bytes = read_file(&path)?;
        let journal: TransactionJournal =
            serde_json::from_slice(&bytes).map_err(|source| TransactionError::DecodeJournal {
                path: path.clone(),
                source,
            })?;
        if journal.phase != JournalPhase::Committed {
            rollback(&root, &journal)?;
        }
        remove_file_if_exists(&path)?;
        remove_dir_all_if_exists(
            &root
                .join(STATE_DIRECTORY)
                .join(TRANSACTION_DIRECTORY)
                .join(&journal.transaction_id),
        )?;
    }
    Ok(())
}

fn rollback_failure<T>(
    root: &Path,
    journal_path: &Path,
    journal: &TransactionJournal,
    cause: String,
) -> Result<T, TransactionError> {
    match rollback(root, journal) {
        Ok(()) => {
            remove_file_if_exists(journal_path)?;
            Err(TransactionError::PostValidation(cause))
        }
        Err(error) => Err(TransactionError::Rollback {
            cause,
            rollback: error.to_string(),
        }),
    }
}

fn rollback(root: &Path, journal: &TransactionJournal) -> Result<(), TransactionError> {
    for entry in journal.entries.iter().rev() {
        let target = root.join(&entry.target);
        if let Some(backup) = &entry.backup {
            let backup = root.join(backup);
            let restore = target.with_extension(format!(
                "{}.sora-restore",
                target
                    .extension()
                    .and_then(|extension| extension.to_str())
                    .unwrap_or_default()
            ));
            copy_file(&backup, &restore)?;
            replace_file(&restore, &target)?;
        } else if target.exists() {
            remove_file_if_exists(&target)?;
        }
    }
    Ok(())
}

fn bounded_target(root: &Path, path: &Path) -> Result<PathBuf, TransactionError> {
    if path.exists() {
        let canonical = path.canonicalize().map_err(|source| TransactionError::Io {
            action: "resolve transaction target",
            path: path.to_path_buf(),
            source,
        })?;
        if canonical.starts_with(root) {
            return Ok(canonical);
        }
        return Err(TransactionError::TargetOutsideProject(canonical));
    }
    let parent = path.parent().unwrap_or(root);
    let mut existing = parent;
    let mut suffix = Vec::new();
    while !existing.exists() {
        let name = existing
            .file_name()
            .ok_or_else(|| TransactionError::TargetOutsideProject(path.to_path_buf()))?;
        suffix.push(name.to_os_string());
        existing = existing
            .parent()
            .ok_or_else(|| TransactionError::TargetOutsideProject(path.to_path_buf()))?;
    }
    let mut parent = existing
        .canonicalize()
        .map_err(|source| TransactionError::Io {
            action: "resolve transaction target parent",
            path: existing.to_path_buf(),
            source,
        })?;
    if !parent.starts_with(root) {
        return Err(TransactionError::TargetOutsideProject(parent));
    }
    for component in suffix.into_iter().rev() {
        parent.push(component);
    }
    let name = path
        .file_name()
        .ok_or_else(|| TransactionError::TargetOutsideProject(path.to_path_buf()))?;
    Ok(parent.join(name))
}

fn canonical_directory(path: &Path) -> Result<PathBuf, TransactionError> {
    let path = path.canonicalize().map_err(|source| TransactionError::Io {
        action: "resolve project root",
        path: path.to_path_buf(),
        source,
    })?;
    if path.is_dir() {
        Ok(path)
    } else {
        Err(TransactionError::TargetOutsideProject(path))
    }
}

fn write_journal(path: &Path, journal: &TransactionJournal) -> Result<(), TransactionError> {
    if let Some(parent) = path.parent() {
        create_dir_all(parent)?;
    }
    let bytes = serde_json::to_vec_pretty(journal).map_err(TransactionError::EncodeJournal)?;
    let temporary = path.with_extension("json.tmp");
    write_durable(&temporary, &bytes)?;
    replace_file(&temporary, path)
}

fn write_durable(path: &Path, bytes: &[u8]) -> Result<(), TransactionError> {
    let mut file = File::create(path).map_err(|source| TransactionError::Io {
        action: "create file",
        path: path.to_path_buf(),
        source,
    })?;
    file.write_all(bytes)
        .map_err(|source| TransactionError::Io {
            action: "write file",
            path: path.to_path_buf(),
            source,
        })?;
    file.sync_all().map_err(|source| TransactionError::Io {
        action: "sync file",
        path: path.to_path_buf(),
        source,
    })
}

fn replace_file(source: &Path, target: &Path) -> Result<(), TransactionError> {
    match fs::rename(source, target) {
        Ok(()) => Ok(()),
        Err(first) if target.exists() => {
            fs::remove_file(target).map_err(|source| TransactionError::Io {
                action: "remove replaced file",
                path: target.to_path_buf(),
                source,
            })?;
            fs::rename(source, target).map_err(|source| TransactionError::Io {
                action: "replace file",
                path: target.to_path_buf(),
                source: std::io::Error::new(
                    source.kind(),
                    format!("initial rename failed with `{first}`; retry failed with `{source}`"),
                ),
            })
        }
        Err(source) => Err(TransactionError::Io {
            action: "replace file",
            path: target.to_path_buf(),
            source,
        }),
    }
}

fn copy_file(source: &Path, target: &Path) -> Result<(), TransactionError> {
    if let Some(parent) = target.parent() {
        create_dir_all(parent)?;
    }
    fs::copy(source, target)
        .map(|_| ())
        .map_err(|source_error| TransactionError::Io {
            action: "copy file",
            path: source.to_path_buf(),
            source: source_error,
        })
}

fn create_dir_all(path: &Path) -> Result<(), TransactionError> {
    fs::create_dir_all(path).map_err(|source| TransactionError::Io {
        action: "create directory",
        path: path.to_path_buf(),
        source,
    })
}

fn remove_file_if_exists(path: &Path) -> Result<(), TransactionError> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(source) => Err(TransactionError::Io {
            action: "remove file",
            path: path.to_path_buf(),
            source,
        }),
    }
}

fn remove_dir_all_if_exists(path: &Path) -> Result<(), TransactionError> {
    match fs::remove_dir_all(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(source) => Err(TransactionError::Io {
            action: "remove directory",
            path: path.to_path_buf(),
            source,
        }),
    }
}

fn read_file(path: &Path) -> Result<Vec<u8>, TransactionError> {
    fs::read(path).map_err(|source| TransactionError::Io {
        action: "read file",
        path: path.to_path_buf(),
        source,
    })
}

fn read_dir_files(path: &Path) -> Result<Vec<PathBuf>, TransactionError> {
    fs::read_dir(path)
        .map_err(|source| TransactionError::Io {
            action: "read directory",
            path: path.to_path_buf(),
            source,
        })?
        .map(|entry| {
            entry
                .map(|entry| entry.path())
                .map_err(|source| TransactionError::Io {
                    action: "read directory entry",
                    path: path.to_path_buf(),
                    source,
                })
        })
        .collect()
}

fn relative_string(path: &Path) -> String {
    path.components()
        .filter_map(|component| match component {
            Component::Normal(value) => Some(value.to_string_lossy().into_owned()),
            _ => None,
        })
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
    fn failed_post_validation_rolls_back_all_targets() {
        let root = temp_dir();
        fs::create_dir_all(&root).unwrap();
        let first = root.join("first.toml");
        let second = root.join("second.toml");
        fs::write(&first, "old-first").unwrap();
        fs::write(&second, "old-second").unwrap();

        let result = commit_text_transaction(
            &root,
            &[
                TextFileWrite {
                    path: first.clone(),
                    content: "new-first".to_owned(),
                },
                TextFileWrite {
                    path: second.clone(),
                    content: "new-second".to_owned(),
                },
            ],
            || anyhow::bail!("invalid schema"),
        );

        assert!(result.is_err());
        assert_eq!(fs::read_to_string(first).unwrap(), "old-first");
        assert_eq!(fs::read_to_string(second).unwrap(), "old-second");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn recovery_rolls_back_a_committing_journal() {
        let root = temp_dir();
        fs::create_dir_all(&root).unwrap();
        let target = root.join("schema.toml");
        fs::write(&target, "old").unwrap();
        let id = "interrupted";
        let state = root.join(".sora");
        let backup = state.join("backups").join(id).join("0.backup");
        fs::create_dir_all(backup.parent().unwrap()).unwrap();
        fs::copy(&target, &backup).unwrap();
        fs::write(&target, "partial").unwrap();
        let journal_path = state.join("transactions").join(format!("{id}.json"));
        fs::create_dir_all(journal_path.parent().unwrap()).unwrap();
        let journal = TransactionJournal {
            version: 1,
            transaction_id: id.to_owned(),
            phase: JournalPhase::Committing,
            entries: vec![JournalEntry {
                target: "schema.toml".to_owned(),
                staged: ".sora/transactions/interrupted/0.stage".to_owned(),
                backup: Some(".sora/backups/interrupted/0.backup".to_owned()),
            }],
        };
        fs::write(&journal_path, serde_json::to_vec(&journal).unwrap()).unwrap();

        recover_transactions(&root).unwrap();

        assert_eq!(fs::read_to_string(target).unwrap(), "old");
        assert!(!journal_path.exists());
        let _ = fs::remove_dir_all(root);
    }

    fn temp_dir() -> PathBuf {
        let nonce = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "sora-workspace-transaction-{}-{time}-{nonce}",
            std::process::id()
        ))
    }
}
