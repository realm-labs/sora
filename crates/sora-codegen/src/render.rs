use std::{fs, path::Path};

use minijinja::Environment;
use serde::Serialize;
use sora_diagnostics::{Result, SoraError};

pub(crate) fn render_template(
    target: &str,
    file_name: &str,
    ctx: impl Serialize,
) -> Result<String> {
    let template_name = format!("{target}/{file_name}");
    let source = sora_templates::template_source(target, file_name).ok_or_else(|| {
        SoraError::RenderTemplate {
            template: template_name.clone(),
            message: "embedded template not found".to_owned(),
        }
    })?;
    let mut env = Environment::new();
    env.set_keep_trailing_newline(true);
    env.add_template(file_name, source)
        .map_err(|source| SoraError::RenderTemplate {
            template: template_name.clone(),
            message: source.to_string(),
        })?;
    let template = env
        .get_template(file_name)
        .map_err(|source| SoraError::RenderTemplate {
            template: template_name.clone(),
            message: source.to_string(),
        })?;
    template
        .render(ctx)
        .map_err(|source| SoraError::RenderTemplate {
            template: template_name,
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
