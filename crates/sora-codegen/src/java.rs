use std::path::{Path, PathBuf};

use minijinja::context;
use sora_diagnostics::{Result, SoraError};
use sora_ir::model::ConfigIr;

use crate::{
    generator::CodeGenerator,
    model::build_model,
    render::{ensure_dir, render_template, write_file},
};

pub struct JavaCodeGenerator;

impl CodeGenerator for JavaCodeGenerator {
    fn generate(&self, ir: &ConfigIr, out_dir: &Path) -> Result<()> {
        ensure_dir(out_dir)?;
        let model = build_model(ir)?;
        let package_dir = java_package_dir(out_dir, &model.package)?;

        for item in &model.enums {
            let rendered = render_template(
                "java",
                "enum.java.j2",
                context! { package => &model.package, enum => item },
            )?;
            write_file(&package_dir.join(format!("{}.java", item.name)), rendered)?;
        }

        for record in &model.records {
            let rendered = render_template(
                "java",
                "record.java.j2",
                context! { package => &model.package, record => record },
            )?;
            write_file(
                &package_dir.join(format!("{}.java", record.pascal_name)),
                rendered,
            )?;
        }

        for union in &model.unions {
            let rendered = render_template(
                "java",
                "union.java.j2",
                context! { package => &model.package, union => union },
            )?;
            write_file(
                &package_dir.join(format!("{}.java", union.pascal_name)),
                rendered,
            )?;
        }

        let rendered = render_template(
            "java",
            "runtime.java.j2",
            context! { package => &model.package },
        )?;
        write_file(&package_dir.join("Runtime.java"), rendered)?;

        let rendered = render_template("java", "config.java.j2", context! { model => &model })?;
        write_file(&package_dir.join("SoraConfig.java"), rendered)
    }
}

fn java_package_dir(out_dir: &Path, package: &str) -> Result<PathBuf> {
    let mut path = out_dir.to_path_buf();
    for segment in package.split('.') {
        if !is_java_package_segment(segment) {
            return Err(SoraError::InvalidSchema(format!(
                "java package `{package}` must use dot-separated identifier segments"
            )));
        }
        path.push(segment);
    }
    Ok(path)
}

fn is_java_package_segment(segment: &str) -> bool {
    let mut chars = segment.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || first == '_')
        && chars.all(|value| value.is_ascii_alphanumeric() || value == '_')
}
