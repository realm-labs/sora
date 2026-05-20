use std::path::Path;

use heck::ToSnakeCase;
use minijinja::context;
use sora_diagnostics::{Result, SoraError};
use sora_ir::model::ConfigIr;

use crate::{
    generator::CodeGenerator,
    model::build_model,
    render::{ensure_dir, render_template, write_file},
};

pub struct GoCodeGenerator;

impl CodeGenerator for GoCodeGenerator {
    fn generate(&self, ir: &ConfigIr, out_dir: &Path) -> Result<()> {
        ensure_dir(out_dir)?;
        let model = build_model(ir)?;
        let package = go_package_name(&model.package)?;

        for item in &model.enums {
            let rendered = render_template(
                "go",
                "enum.go.j2",
                context! { package => &package, enum => item },
            )?;
            write_file(
                &out_dir.join(format!("{}.go", item.name.to_snake_case())),
                rendered,
            )?;
        }

        for record in &model.records {
            let rendered = render_template(
                "go",
                "record.go.j2",
                context! { package => &package, record => record },
            )?;
            write_file(&out_dir.join(format!("{}.go", record.snake_name)), rendered)?;
        }

        for union in &model.unions {
            let rendered = render_template(
                "go",
                "union.go.j2",
                context! { package => &package, union => union },
            )?;
            write_file(&out_dir.join(format!("{}.go", union.snake_name)), rendered)?;
        }

        let rendered = render_template("go", "runtime.go.j2", context! { package => &package })?;
        write_file(&out_dir.join("runtime.go"), rendered)?;

        let rendered = render_template(
            "go",
            "config.go.j2",
            context! { package => &package, model => &model },
        )?;
        write_file(&out_dir.join("config.go"), rendered)
    }
}

fn go_package_name(package: &str) -> Result<String> {
    let Some(segment) = package.rsplit('.').next() else {
        return Err(SoraError::InvalidSchema(
            "go package must not be empty".to_owned(),
        ));
    };
    let package = segment.to_snake_case();
    if !is_go_package_name(&package) {
        return Err(SoraError::InvalidSchema(format!(
            "go package `{package}` must be a valid identifier"
        )));
    }
    Ok(package)
}

fn is_go_package_name(package: &str) -> bool {
    let mut chars = package.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || first == '_')
        && chars.all(|value| value.is_ascii_alphanumeric() || value == '_')
}
