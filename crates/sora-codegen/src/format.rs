use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

use sora_diagnostics::{Result, SoraError};

use crate::target::CodegenTarget;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatMode {
    Never,
    Auto,
    Required,
}

pub fn format_generated_code(
    target: CodegenTarget,
    out_dir: &Path,
    mode: FormatMode,
) -> Result<()> {
    if mode == FormatMode::Never {
        return Ok(());
    }

    let Some(formatter) = Formatter::for_target(target) else {
        return match mode {
            FormatMode::Never | FormatMode::Auto => Ok(()),
            FormatMode::Required => Err(format_error(
                target.language_name(),
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

    let files = collect_files(out_dir, formatter.extension)?;
    if files.is_empty() {
        return Ok(());
    }

    let output = Command::new(formatter.command)
        .args(formatter.args)
        .args(&files)
        .output()
        .map_err(|source| {
            format_error(formatter.language, formatter.command, source.to_string())
        })?;

    if output.status.success() {
        return Ok(());
    }

    let message = command_output_message(&output.stdout, &output.stderr);
    Err(format_error(formatter.language, formatter.command, message))
}

struct Formatter {
    language: &'static str,
    command: &'static str,
    args: &'static [&'static str],
    extension: &'static str,
}

impl Formatter {
    fn for_target(target: CodegenTarget) -> Option<Self> {
        match target {
            CodegenTarget::Rust => Some(Self {
                language: "Rust",
                command: "rustfmt",
                args: &[],
                extension: "rs",
            }),
            CodegenTarget::Go => Some(Self {
                language: "Go",
                command: "gofmt",
                args: &["-w"],
                extension: "go",
            }),
            CodegenTarget::Erlang => Some(Self {
                language: "Erlang",
                command: "erlfmt",
                args: &["-w"],
                extension: "erl",
            }),
            CodegenTarget::Python => Some(Self {
                language: "Python",
                command: "black",
                args: &["--quiet"],
                extension: "py",
            }),
            CodegenTarget::Kotlin
            | CodegenTarget::CSharp
            | CodegenTarget::Java
            | CodegenTarget::TypeScript
            | CodegenTarget::JavaScript
            | CodegenTarget::Lua
            | CodegenTarget::Proto => None,
        }
    }
}

fn collect_files(root: &Path, extension: &str) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    collect_files_inner(root, extension, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_files_inner(path: &Path, extension: &str, files: &mut Vec<PathBuf>) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }
    if path.is_file() {
        if has_extension(path, extension) {
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
        collect_files_inner(&entry.path(), extension, files)?;
    }

    Ok(())
}

fn has_extension(path: &Path, extension: &str) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value == extension)
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

impl CodegenTarget {
    fn language_name(self) -> &'static str {
        match self {
            CodegenTarget::Rust => "Rust",
            CodegenTarget::Kotlin => "Kotlin",
            CodegenTarget::CSharp => "C#",
            CodegenTarget::Java => "Java",
            CodegenTarget::Go => "Go",
            CodegenTarget::TypeScript => "TypeScript",
            CodegenTarget::JavaScript => "JavaScript",
            CodegenTarget::Erlang => "Erlang",
            CodegenTarget::Lua => "Lua",
            CodegenTarget::Proto => "Proto",
            CodegenTarget::Python => "Python",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_skips_missing_formatter() {
        let base = env::temp_dir().join("sora-codegen-format-missing");
        format_generated_code(CodegenTarget::Erlang, &base, FormatMode::Auto).unwrap();
    }

    #[test]
    fn required_rejects_unsupported_target() {
        let base = env::temp_dir().join("sora-codegen-format-unsupported");
        let error =
            format_generated_code(CodegenTarget::Proto, &base, FormatMode::Required).unwrap_err();
        assert!(error.to_string().contains("no formatter is configured"));
    }
}
