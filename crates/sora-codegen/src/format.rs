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

    let files = collect_files(out_dir, formatter.extensions)?;
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
    extensions: &'static [&'static str],
}

impl Formatter {
    fn for_target(target: CodegenTarget) -> Option<Self> {
        match target {
            CodegenTarget::Rust => Some(Self {
                language: "Rust",
                command: "rustfmt",
                args: &[],
                extensions: &["rs"],
            }),
            CodegenTarget::Go => Some(Self {
                language: "Go",
                command: "gofmt",
                args: &["-w"],
                extensions: &["go"],
            }),
            CodegenTarget::Erlang => Some(Self {
                language: "Erlang",
                command: "erlfmt",
                args: &["-w"],
                extensions: &["erl"],
            }),
            CodegenTarget::Python => Some(Self {
                language: "Python",
                command: "black",
                args: &["--quiet"],
                extensions: &["py"],
            }),
            CodegenTarget::C => Some(Self {
                language: "C",
                command: "clang-format",
                args: &["-i"],
                extensions: &["h", "c"],
            }),
            CodegenTarget::Cpp => Some(Self {
                language: "C++",
                command: "clang-format",
                args: &["-i"],
                extensions: &["hpp"],
            }),
            CodegenTarget::Scala => Some(Self {
                language: "Scala",
                command: "scalafmt",
                args: &[],
                extensions: &["scala"],
            }),
            CodegenTarget::Kotlin
            | CodegenTarget::CSharp
            | CodegenTarget::Java
            | CodegenTarget::Dart
            | CodegenTarget::Godot
            | CodegenTarget::TypeScript
            | CodegenTarget::JavaScript
            | CodegenTarget::Lua
            | CodegenTarget::ProtoSchema => None,
        }
    }
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

impl CodegenTarget {
    fn language_name(self) -> &'static str {
        match self {
            CodegenTarget::Rust => "Rust",
            CodegenTarget::Kotlin => "Kotlin",
            CodegenTarget::CSharp => "C#",
            CodegenTarget::Java => "Java",
            CodegenTarget::Scala => "Scala",
            CodegenTarget::Go => "Go",
            CodegenTarget::Dart => "Dart",
            CodegenTarget::Godot => "Godot",
            CodegenTarget::C => "C",
            CodegenTarget::Cpp => "C++",
            CodegenTarget::TypeScript => "TypeScript",
            CodegenTarget::JavaScript => "JavaScript",
            CodegenTarget::Erlang => "Erlang",
            CodegenTarget::Lua => "Lua",
            CodegenTarget::ProtoSchema => "Proto schema",
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
        let error = format_generated_code(CodegenTarget::ProtoSchema, &base, FormatMode::Required)
            .unwrap_err();
        assert!(error.to_string().contains("no formatter is configured"));
    }
}
