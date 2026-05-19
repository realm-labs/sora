use std::path::Path;

use minijinja::context;
use sora_diagnostics::Result;
use sora_ir::model::ConfigIr;

use crate::{
    generator::CodeGenerator,
    model::build_model,
    render::{ensure_dir, render_template, write_file},
};

pub struct KotlinCodeGenerator;

impl CodeGenerator for KotlinCodeGenerator {
    fn generate(&self, ir: &ConfigIr, out_dir: &Path) -> Result<()> {
        ensure_dir(out_dir)?;
        let model = build_model(ir)?;

        for item in &model.enums {
            let rendered = render_template(
                "kotlin",
                "enum.kt.j2",
                context! { package => &model.package, enum => item },
            )?;
            write_file(&out_dir.join(format!("{}.kt", item.name)), rendered)?;
        }

        for record in &model.records {
            let rendered = render_template(
                "kotlin",
                "data_class.kt.j2",
                context! { package => &model.package, record => record },
            )?;
            write_file(
                &out_dir.join(format!("{}.kt", record.pascal_name)),
                rendered,
            )?;
        }

        let rendered = render_template("kotlin", "package.kt.j2", context! { model => &model })?;
        write_file(&out_dir.join("Package.kt"), rendered)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rust::RustCodeGenerator;
    use sora_ir::{model::ConfigIr, normalize::normalize_schema};
    use sora_schema::model::SchemaFile;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn generates_rust_and_kotlin_files() {
        let ir = example_ir();
        let base = temp_dir();
        let rust_out = base.join("rust");
        let kotlin_out = base.join("kotlin");

        RustCodeGenerator.generate(&ir, &rust_out).unwrap();
        KotlinCodeGenerator.generate(&ir, &kotlin_out).unwrap();

        let rust_item = std::fs::read_to_string(rust_out.join("item.rs")).unwrap();
        let kotlin_item = std::fs::read_to_string(kotlin_out.join("Item.kt")).unwrap();

        assert!(rust_item.contains("pub struct Item"));
        assert!(rust_item.contains("pub item_type: ItemType"));
        assert!(kotlin_item.contains("data class Item"));
        assert!(kotlin_item.contains("val itemType: ItemType"));

        let _ = std::fs::remove_dir_all(base);
    }

    fn example_ir() -> ConfigIr {
        let schema: SchemaFile = toml::from_str(
            r#"
package = "game_config"

[[enums]]
name = "ItemType"
values = ["Weapon", "Armor", "Material", "Consumable"]

[[tables]]
name = "Item"
mode = "map"
key = "id"

[[tables.fields]]
name = "id"
type = "i32"
key = true
required = true
comment = "Item id"

[[tables.fields]]
name = "item_type"
type = "enum<ItemType>"
required = true
comment = "Item type"
"#,
        )
        .unwrap();

        normalize_schema(schema).unwrap()
    }

    fn temp_dir() -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("sora-codegen-test-{unique}"))
    }
}
