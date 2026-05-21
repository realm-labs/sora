use std::{fs, path::Path};

use anyhow::{Context, Result};
pub(crate) fn clean_xlsx_files(path: &Path) -> Result<()> {
    for entry in
        fs::read_dir(path).with_context(|| format!("failed to read `{}`", path.display()))?
    {
        let entry = entry?;
        let candidate = entry.path();
        if candidate
            .extension()
            .is_some_and(|extension| extension == "xlsx")
        {
            fs::remove_file(&candidate)
                .with_context(|| format!("failed to remove `{}`", candidate.display()))?;
        }
    }
    Ok(())
}

pub(crate) fn clean_dir(path: &Path) -> Result<()> {
    if path.exists() {
        fs::remove_dir_all(path)
            .with_context(|| format!("failed to remove `{}`", path.display()))?;
    }
    Ok(())
}

pub(crate) fn clean_file(path: &Path) -> Result<()> {
    if path.exists() {
        fs::remove_file(path).with_context(|| format!("failed to remove `{}`", path.display()))?;
    }
    Ok(())
}
