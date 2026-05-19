use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::Serialize;
use sora_diagnostics::{Result, SoraError};

pub(crate) fn deterministic_json_bytes(value: &impl Serialize) -> Result<Vec<u8>> {
    serde_json::to_vec(value).map_err(SoraError::SerializeData)
}

pub(crate) fn create_dir_all(path: &Path) -> Result<()> {
    fs::create_dir_all(path).map_err(|source| SoraError::CreateDir {
        path: path.to_path_buf(),
        source,
    })
}

pub(crate) fn write_file(path: PathBuf, content: impl AsRef<[u8]>) -> Result<()> {
    fs::write(&path, content).map_err(|source| SoraError::WriteFile { path, source })
}
