use std::{
    fs,
    path::{Path, PathBuf},
};

use sora_diagnostics::{Result, SoraError};

pub(crate) fn ensure_workbook_path_is_bounded(root: &Path, path: &Path) -> Result<()> {
    let relative = path
        .strip_prefix(root)
        .map_err(|_| SoraError::ExcelTemplate {
            path: path.to_path_buf(),
            message: "workbook path is outside the configured root".to_owned(),
        })?;
    if relative.as_os_str().is_empty()
        || relative.components().any(|component| {
            matches!(
                component,
                std::path::Component::ParentDir
                    | std::path::Component::RootDir
                    | std::path::Component::Prefix(_)
            )
        })
    {
        return Err(SoraError::ExcelTemplate {
            path: path.to_path_buf(),
            message: "workbook path contains unsafe traversal".to_owned(),
        });
    }

    let resolved_root = resolve_through_existing_ancestor(root)?;
    let resolved_path = resolve_through_existing_ancestor(path)?;
    if !resolved_path.starts_with(&resolved_root) {
        return Err(SoraError::ExcelTemplate {
            path: path.to_path_buf(),
            message: "workbook path resolves outside the configured root".to_owned(),
        });
    }
    Ok(())
}

pub(crate) fn create_workbook_parent(root: &Path, path: &Path) -> Result<()> {
    ensure_workbook_path_is_bounded(root, path)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| SoraError::CreateDir {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    ensure_workbook_path_is_bounded(root, path)
}

fn resolve_through_existing_ancestor(path: &Path) -> Result<PathBuf> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|source| SoraError::ReadFile {
                path: path.to_path_buf(),
                source,
            })?
            .join(path)
    };
    let existing = absolute
        .ancestors()
        .find(|ancestor| ancestor.exists())
        .ok_or_else(|| SoraError::ExcelTemplate {
            path: path.to_path_buf(),
            message: "workbook path has no existing ancestor".to_owned(),
        })?;
    let mut resolved = fs::canonicalize(existing).map_err(|source| SoraError::ReadFile {
        path: existing.to_path_buf(),
        source,
    })?;
    let unresolved = absolute
        .strip_prefix(existing)
        .expect("an ancestor must be a path prefix");
    for component in unresolved.components() {
        match component {
            std::path::Component::Normal(segment) => resolved.push(segment),
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir
            | std::path::Component::RootDir
            | std::path::Component::Prefix(_) => {
                return Err(SoraError::ExcelTemplate {
                    path: path.to_path_buf(),
                    message: "workbook path contains unsafe unresolved traversal".to_owned(),
                });
            }
        }
    }
    Ok(resolved)
}
