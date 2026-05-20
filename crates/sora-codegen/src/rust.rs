use std::path::Path;

use heck::ToSnakeCase;
use minijinja::context;
use sora_diagnostics::Result;
use sora_ir::model::ConfigIr;

use crate::{
    generator::CodeGenerator,
    model::build_model,
    render::{ensure_dir, render_template, write_file},
};

pub struct RustCodeGenerator;

impl CodeGenerator for RustCodeGenerator {
    fn generate(&self, ir: &ConfigIr, out_dir: &Path) -> Result<()> {
        ensure_dir(out_dir)?;
        let model = build_model(ir)?;

        for item in &model.enums {
            let rendered = render_template("rust", "enum.rs.j2", context! { enum => item })?;
            write_file(
                &out_dir.join(format!("{}.rs", item.name.to_snake_case())),
                rendered,
            )?;
        }

        for record in &model.records {
            let rendered = render_template("rust", "struct.rs.j2", context! { record => record })?;
            write_file(&out_dir.join(format!("{}.rs", record.snake_name)), rendered)?;
        }

        for union in &model.unions {
            let rendered = render_template("rust", "union.rs.j2", context! { union => union })?;
            write_file(&out_dir.join(format!("{}.rs", union.snake_name)), rendered)?;
        }

        let rendered = render_template("rust", "mod.rs.j2", context! { model => &model })?;
        write_file(&out_dir.join("mod.rs"), rendered)?;

        let rendered = render_template("rust", "runtime.rs.j2", context! {})?;
        write_file(&out_dir.join("runtime.rs"), rendered)
    }
}
