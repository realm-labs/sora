use std::{
    fs,
    path::{Path, PathBuf},
};

use sora_diagnostics::{Result, SoraError};

pub(crate) fn create_dir_all(path: &Path) -> Result<()> {
    fs::create_dir_all(path).map_err(|source| SoraError::CreateDir {
        path: path.to_path_buf(),
        source,
    })
}

pub(crate) fn write_file(path: PathBuf, content: impl AsRef<[u8]>) -> Result<()> {
    fs::write(&path, content).map_err(|source| SoraError::WriteFile { path, source })
}
