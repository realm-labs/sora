use std::path::{Path, PathBuf};

pub fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("crate should live under workspace/crates")
        .to_path_buf()
}

pub fn templates_dir() -> PathBuf {
    workspace_root().join("templates")
}

pub fn target_templates_dir(target: &str) -> PathBuf {
    templates_dir().join(target)
}
