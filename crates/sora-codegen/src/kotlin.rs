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

        for union in &model.unions {
            let rendered = render_template(
                "kotlin",
                "union.kt.j2",
                context! { package => &model.package, union => union },
            )?;
            write_file(&out_dir.join(format!("{}.kt", union.pascal_name)), rendered)?;
        }

        let rendered = render_template(
            "kotlin",
            "runtime.kt.j2",
            context! { package => &model.package },
        )?;
        write_file(&out_dir.join("Runtime.kt"), rendered)?;

        let rendered = render_template("kotlin", "config.kt.j2", context! { model => &model })?;
        write_file(&out_dir.join("SoraConfig.kt"), rendered)?;

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
    use std::{
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };

    static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn generates_rust_and_kotlin_files() {
        let ir = example_ir();
        let base = temp_dir();
        let rust_out = base.join("rust");
        let kotlin_out = base.join("kotlin");

        RustCodeGenerator.generate(&ir, &rust_out).unwrap();
        KotlinCodeGenerator.generate(&ir, &kotlin_out).unwrap();

        let rust_item = std::fs::read_to_string(rust_out.join("item.rs")).unwrap();
        let rust_item_type = std::fs::read_to_string(rust_out.join("item_type.rs")).unwrap();
        let rust_action = std::fs::read_to_string(rust_out.join("action.rs")).unwrap();
        let rust_runtime = std::fs::read_to_string(rust_out.join("runtime.rs")).unwrap();
        let rust_mod = std::fs::read_to_string(rust_out.join("mod.rs")).unwrap();
        let kotlin_item = std::fs::read_to_string(kotlin_out.join("Item.kt")).unwrap();
        let kotlin_action = std::fs::read_to_string(kotlin_out.join("Action.kt")).unwrap();
        let kotlin_runtime = std::fs::read_to_string(kotlin_out.join("Runtime.kt")).unwrap();
        let kotlin_config = std::fs::read_to_string(kotlin_out.join("SoraConfig.kt")).unwrap();

        assert!(rust_item.contains("pub struct Item"));
        assert!(rust_item.contains("pub item_type: ItemType"));
        assert!(rust_item.contains("pub action: Action"));
        assert!(rust_item.contains("impl super::runtime::SoraDecode for Item"));
        assert!(!rust_item.contains("impl std::fmt::Display for Item"));
        assert!(!rust_item_type.contains("impl std::fmt::Display for ItemType"));
        assert!(rust_action.contains("pub enum Action"));
        assert!(rust_action.contains("AddItem {"));
        assert!(rust_action.contains("impl super::runtime::SoraDecode for Action"));
        assert!(!rust_action.contains("impl std::fmt::Display for Action"));
        assert!(rust_runtime.contains("pub struct SoraBundle"));
        assert!(rust_mod.contains("pub struct SoraConfig"));
        assert!(rust_mod.contains("from_bytes"));
        assert!(rust_mod.contains("pub type SoraMap<K, V> = std::collections::HashMap<K, V>;"));
        assert!(rust_mod.contains("pub trait SoraTable: std::any::Any + Send + Sync"));
        assert!(rust_mod.contains("tables: SoraMap<&'static str, Box<dyn SoraTable>>"));
        assert!(rust_mod.contains("pub struct ItemTable(SoraMap<i32, item::Item>)"));
        assert!(rust_mod.contains("impl SoraTable for ItemTable"));
        assert!(rust_mod.contains("fn key(&self) -> Option<&'static str>"));
        assert!(rust_mod.contains("Some(\"id\")"));
        assert!(rust_mod.contains("pub fn tables(&self) -> impl Iterator<Item = &dyn SoraTable>"));
        assert!(rust_mod.contains("impl std::ops::Deref for ItemTable"));
        assert!(
            rust_mod.contains("tables.insert(\"Item\", Box::new(ItemTable::decode(&bundle)?));")
        );
        assert!(rust_mod.contains("|row| row.id"));
        assert!(
            rust_mod.contains("fn table<T: SoraTable + 'static>(&self, name: &'static str) -> &T")
        );
        assert!(!rust_mod.contains("as_any"));
        assert!(rust_mod.contains("let table: &dyn std::any::Any = table.as_ref();"));
        assert!(rust_mod.contains("table.downcast_ref::<T>()"));
        assert!(rust_mod.contains("pub fn item(&self) -> &ItemTable"));
        assert!(rust_mod.contains("pub fn get(&self, key: i32) -> Option<&item::Item>"));
        assert!(!rust_mod.contains("pub fn get_item"));
        assert!(!rust_mod.contains("pub fn iter_item"));
        assert!(!rust_mod.contains("decode_singleton_table"));
        assert!(kotlin_item.contains("data class Item"));
        assert!(kotlin_item.contains("val itemType: ItemType"));
        assert!(kotlin_item.contains("val action: Action"));
        assert!(kotlin_action.contains("sealed class Action"));
        assert!(kotlin_action.contains("data class AddItem"));
        assert!(kotlin_action.contains("fun decode(reader: SoraReader): Action"));
        assert!(kotlin_item.contains("fun decode(reader: SoraReader): Item"));
        assert!(kotlin_runtime.contains("class SoraBundle"));
        assert!(kotlin_config.contains("data class SoraConfig"));
        assert!(kotlin_config.contains("val item: Map<Int, Item>"));
        assert!(kotlin_config.contains("fun getItem(key: Int): Item? = item[key]"));
        assert!(kotlin_config.contains("fun itemValues(): Collection<Item> = item.values"));
        assert!(kotlin_config.contains("fun fromBytes(bytes: ByteArray): SoraConfig"));

        let _ = std::fs::remove_dir_all(base);
    }

    #[test]
    fn rust_config_api_respects_table_modes() {
        let ir = table_mode_ir();
        let base = temp_dir();
        let rust_out = base.join("rust");

        RustCodeGenerator.generate(&ir, &rust_out).unwrap();

        let rust_mod = std::fs::read_to_string(rust_out.join("mod.rs")).unwrap();
        assert!(rust_mod.contains("pub struct ItemTable(SoraMap<i32, item::Item>)"));
        assert!(rust_mod.contains("pub struct SettingsTable(settings::Settings)"));
        assert!(rust_mod.contains("pub fn item(&self) -> &ItemTable"));
        assert!(rust_mod.contains("pub fn settings(&self) -> &SettingsTable"));
        assert!(!rust_mod.contains("pub fn settings_row"));
        assert!(rust_mod.contains("decode_singleton_table"));

        let _ = std::fs::remove_dir_all(base);
    }

    #[test]
    fn rust_config_api_can_use_fx_hash_map() {
        let mut ir = example_ir();
        ir.codegen.rust.map_type = sora_ir::model::RustMapTypeIr::FxHashMap;
        let base = temp_dir();
        let rust_out = base.join("rust");

        RustCodeGenerator.generate(&ir, &rust_out).unwrap();

        let rust_mod = std::fs::read_to_string(rust_out.join("mod.rs")).unwrap();
        assert!(rust_mod.contains("pub type SoraMap<K, V> = rustc_hash::FxHashMap<K, V>;"));
        assert!(rust_mod.contains("pub struct ItemTable(SoraMap<i32, item::Item>)"));
        assert!(rust_mod.contains("tables: SoraMap<&'static str, Box<dyn SoraTable>>"));

        let _ = std::fs::remove_dir_all(base);
    }

    fn example_ir() -> ConfigIr {
        let schema: SchemaFile = toml::from_str(
            r#"
package = "game_config"

[[enums]]
name = "ItemType"
values = ["Weapon", "Armor", "Material", "Consumable"]

[[unions]]
name = "Action"

[[unions.variants]]
name = "AddItem"

[[unions.variants.fields]]
name = "item_id"
type = "i32"

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

[[tables.fields]]
name = "action"
type = "union<Action>"
required = true
comment = "Action"
"#,
        )
        .unwrap();

        normalize_schema(schema).unwrap()
    }

    fn table_mode_ir() -> ConfigIr {
        let schema: SchemaFile = toml::from_str(
            r#"
package = "game_config"

[[tables]]
name = "Item"
mode = "map"
key = "id"

[[tables.fields]]
name = "id"
type = "i32"
required = true

[[tables]]
name = "Settings"
mode = "singleton"

[[tables.fields]]
name = "version"
type = "string"
required = true
"#,
        )
        .unwrap();

        normalize_schema(schema).unwrap()
    }

    fn temp_dir() -> PathBuf {
        let unique = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("sora-codegen-test-{unique}"))
    }
}
