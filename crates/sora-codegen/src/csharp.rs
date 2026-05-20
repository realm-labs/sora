use std::path::Path;

use minijinja::context;
use sora_diagnostics::Result;
use sora_ir::model::ConfigIr;

use crate::{
    generator::CodeGenerator,
    model::build_model,
    render::{ensure_dir, render_template, write_file},
};

pub struct CSharpCodeGenerator;

impl CodeGenerator for CSharpCodeGenerator {
    fn generate(&self, ir: &ConfigIr, out_dir: &Path) -> Result<()> {
        ensure_dir(out_dir)?;
        let model = build_model(ir)?;

        for item in &model.enums {
            let rendered = render_template(
                "csharp",
                "enum.cs.j2",
                context! { namespace => &model.package, enum => item },
            )?;
            write_file(&out_dir.join(format!("{}.cs", item.name)), rendered)?;
        }

        for record in &model.records {
            let rendered = render_template(
                "csharp",
                "record.cs.j2",
                context! { namespace => &model.package, record => record },
            )?;
            write_file(
                &out_dir.join(format!("{}.cs", record.pascal_name)),
                rendered,
            )?;
        }

        for union in &model.unions {
            let rendered = render_template(
                "csharp",
                "union.cs.j2",
                context! { namespace => &model.package, union => union },
            )?;
            write_file(&out_dir.join(format!("{}.cs", union.pascal_name)), rendered)?;
        }

        let rendered = render_template(
            "csharp",
            "runtime.cs.j2",
            context! { namespace => &model.package },
        )?;
        write_file(&out_dir.join("Runtime.cs"), rendered)?;

        let rendered = render_template("csharp", "config.cs.j2", context! { model => &model })?;
        write_file(&out_dir.join("SoraConfig.cs"), rendered)
    }
}
