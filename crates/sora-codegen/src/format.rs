use std::{
    env, fs,
    io::Read,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::Duration,
};

use sora_diagnostics::{Result, SoraError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatMode {
    Never,
    Auto,
    Required,
}

#[derive(Debug, Clone, Copy)]
pub struct FormatterConfig {
    pub language: &'static str,
    pub command: &'static str,
    pub args: &'static [&'static str],
    pub extensions: &'static [&'static str],
}

impl FormatterConfig {
    pub const fn new(
        language: &'static str,
        command: &'static str,
        args: &'static [&'static str],
        extensions: &'static [&'static str],
    ) -> Self {
        Self {
            language,
            command,
            args,
            extensions,
        }
    }
}

pub fn format_generated_code(
    language: &'static str,
    formatter: Option<FormatterConfig>,
    out_dir: &Path,
    mode: FormatMode,
) -> Result<()> {
    format_generated_code_with_cancellation(language, formatter, out_dir, mode, &|| false)
}

pub fn format_generated_code_with_cancellation(
    language: &'static str,
    formatter: Option<FormatterConfig>,
    out_dir: &Path,
    mode: FormatMode,
    cancelled: &(dyn Fn() -> bool + Sync),
) -> Result<()> {
    if mode == FormatMode::Never {
        return Ok(());
    }

    let Some(formatter) = formatter else {
        return match mode {
            FormatMode::Never | FormatMode::Auto => Ok(()),
            FormatMode::Required => Err(format_error(
                language,
                "<none>",
                "no formatter is configured for this codegen target",
            )),
        };
    };

    if !command_exists(formatter.command) {
        return match mode {
            FormatMode::Never | FormatMode::Auto => Ok(()),
            FormatMode::Required => Err(format_error(
                formatter.language,
                formatter.command,
                "formatter command was not found in PATH",
            )),
        };
    }

    let files = collect_files(out_dir, formatter.extensions)?;
    if files.is_empty() {
        return Ok(());
    }

    if cancelled() {
        return Err(SoraError::OperationCancelled {
            operation: "code formatting",
        });
    }
    let mut child = Command::new(formatter.command)
        .args(formatter.args)
        .args(&files)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|source| {
            format_error(formatter.language, formatter.command, source.to_string())
        })?;
    let stdout = drain_pipe(child.stdout.take());
    let stderr = drain_pipe(child.stderr.take());
    let status = loop {
        if cancelled() {
            let _ = child.kill();
            let _ = child.wait();
            let _ = stdout.join();
            let _ = stderr.join();
            return Err(SoraError::OperationCancelled {
                operation: "code formatting",
            });
        }
        match child.try_wait().map_err(|source| {
            format_error(formatter.language, formatter.command, source.to_string())
        })? {
            Some(status) => break status,
            None => thread::sleep(Duration::from_millis(10)),
        }
    };
    let stdout = stdout.join().unwrap_or_default();
    let stderr = stderr.join().unwrap_or_default();

    if status.success() {
        return Ok(());
    }

    let message = command_output_message(&stdout, &stderr);
    Err(format_error(formatter.language, formatter.command, message))
}

fn drain_pipe(pipe: Option<impl Read + Send + 'static>) -> thread::JoinHandle<Vec<u8>> {
    thread::spawn(move || {
        let mut bytes = Vec::new();
        if let Some(mut pipe) = pipe {
            let _ = pipe.read_to_end(&mut bytes);
        }
        bytes
    })
}

fn collect_files(root: &Path, extensions: &[&str]) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    collect_files_inner(root, extensions, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_files_inner(path: &Path, extensions: &[&str], files: &mut Vec<PathBuf>) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }
    if path.is_file() {
        if has_any_extension(path, extensions) {
            files.push(path.to_path_buf());
        }
        return Ok(());
    }

    for entry in fs::read_dir(path).map_err(|source| SoraError::ReadFile {
        path: path.to_path_buf(),
        source,
    })? {
        let entry = entry.map_err(|source| SoraError::ReadFile {
            path: path.to_path_buf(),
            source,
        })?;
        collect_files_inner(&entry.path(), extensions, files)?;
    }

    Ok(())
}

fn has_any_extension(path: &Path, extensions: &[&str]) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .is_some_and(|value| extensions.contains(&value))
}

fn command_exists(command: &str) -> bool {
    let Some(paths) = env::var_os("PATH") else {
        return false;
    };
    env::split_paths(&paths).any(|path| path.join(command).is_file())
}

fn command_output_message(stdout: &[u8], stderr: &[u8]) -> String {
    let stderr = String::from_utf8_lossy(stderr).trim().to_owned();
    if !stderr.is_empty() {
        return stderr;
    }

    let stdout = String::from_utf8_lossy(stdout).trim().to_owned();
    if !stdout.is_empty() {
        return stdout;
    }

    "formatter exited with a non-zero status".to_owned()
}

fn format_error(
    language: &'static str,
    command: impl Into<String>,
    message: impl Into<String>,
) -> SoraError {
    SoraError::FormatCode {
        language,
        command: command.into(),
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(unix)]
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };
    #[cfg(unix)]
    use std::time::Instant;

    #[test]
    fn auto_skips_missing_formatter() {
        let base = env::temp_dir().join("sora-codegen-format-missing");
        let formatter = FormatterConfig::new("Erlang", "erlfmt", &["-w"], &["erl"]);
        format_generated_code("Erlang", Some(formatter), &base, FormatMode::Auto).unwrap();
    }

    #[test]
    fn required_rejects_unsupported_target() {
        let base = env::temp_dir().join("sora-codegen-format-unsupported");
        let error =
            format_generated_code("Proto schema", None, &base, FormatMode::Required).unwrap_err();
        assert!(error.to_string().contains("no formatter is configured"));
    }

    #[cfg(unix)]
    #[test]
    fn cancellation_terminates_formatter_process() {
        let base =
            env::temp_dir().join(format!("sora-codegen-format-cancel-{}", std::process::id()));
        fs::create_dir_all(&base).unwrap();
        fs::write(base.join("generated.test"), "content").unwrap();
        let cancelled = Arc::new(AtomicBool::new(false));
        let cancel_signal = Arc::clone(&cancelled);
        let cancel_thread = thread::spawn(move || {
            thread::sleep(Duration::from_millis(30));
            cancel_signal.store(true, Ordering::Release);
        });
        let started = Instant::now();

        let error = format_generated_code_with_cancellation(
            "test",
            Some(FormatterConfig::new(
                "test",
                "sh",
                &["-c", "sleep 10"],
                &["test"],
            )),
            &base,
            FormatMode::Required,
            &|| cancelled.load(Ordering::Acquire),
        )
        .unwrap_err();

        cancel_thread.join().unwrap();
        assert!(matches!(
            error,
            SoraError::OperationCancelled {
                operation: "code formatting"
            }
        ));
        assert!(started.elapsed() < Duration::from_secs(2));
        let _ = fs::remove_dir_all(base);
    }
}
