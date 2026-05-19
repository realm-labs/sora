use std::{fs, path::Path};

use minijinja::Environment;
use serde::Serialize;
use sora_diagnostics::{Result, SoraError};

pub(crate) fn render_template(
    target: &str,
    file_name: &str,
    ctx: impl Serialize,
) -> Result<String> {
    let path = sora_templates::target_templates_dir(target).join(file_name);
    let source = fs::read_to_string(&path).map_err(|source| SoraError::ReadFile {
        path: path.clone(),
        source,
    })?;
    let mut env = Environment::new();
    env.add_template(file_name, &source)
        .map_err(|source| SoraError::RenderTemplate {
            template: file_name.to_owned(),
            message: source.to_string(),
        })?;
    let template = env
        .get_template(file_name)
        .map_err(|source| SoraError::RenderTemplate {
            template: file_name.to_owned(),
            message: source.to_string(),
        })?;
    template
        .render(ctx)
        .map_err(|source| SoraError::RenderTemplate {
            template: file_name.to_owned(),
            message: source.to_string(),
        })
}

pub(crate) fn ensure_dir(path: &Path) -> Result<()> {
    fs::create_dir_all(path).map_err(|source| SoraError::CreateDir {
        path: path.to_path_buf(),
        source,
    })
}

pub(crate) fn write_file(path: &Path, content: String) -> Result<()> {
    if let Some(parent) = path.parent() {
        ensure_dir(parent)?;
    }
    fs::write(path, content).map_err(|source| SoraError::WriteFile {
        path: path.to_path_buf(),
        source,
    })
}
