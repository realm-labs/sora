use std::path::Path;

use minijinja::context;
use sora_diagnostics::Result;

use crate::{
    ecmascript::{EcmaScriptModel, EcmaScriptOptionsView, EcmaScriptTarget, ecmascript_barrels},
    generator::{CodeGenerator, CodegenContext, runtime_format_name},
    model::build_base_model,
    options::TypeScriptCodegenOptions,
    render::{ensure_dir, render_template, write_file},
};

pub struct TypeScriptCodeGenerator;
crate::impl_test_codegen_generate!(TypeScriptCodeGenerator, "typescript");

impl CodeGenerator for TypeScriptCodeGenerator {
    fn generate(&self, context: CodegenContext<'_>, out_dir: &Path) -> Result<()> {
        let ir = context.ir;
        let codegen_options = context.options::<TypeScriptCodegenOptions>()?;
        ensure_dir(out_dir)?;
        let runtime_format = runtime_format_name(codegen_options.runtime_format);

        let options = EcmaScriptOptionsView::new(
            EcmaScriptTarget::TypeScript,
            codegen_options.enum_repr,
            false,
        );
        let model = EcmaScriptModel::from_base_model(
            context.target,
            ir,
            build_base_model(ir)?,
            context.type_mappings,
        );

        for item in &model.enums {
            let rendered = render_template(
                "typescript",
                "enum.ts.j2",
                context! { enum => item, options => &options, runtime_format => runtime_format },
            )?;
            write_file(&out_dir.join(format!("{}.ts", item.module_path)), rendered)?;
        }

        for record in &model.records {
            let rendered = render_template(
                "typescript",
                "record.ts.j2",
                context! { record => record, options => &options, runtime_format => runtime_format },
            )?;
            write_file(
                &out_dir.join(format!("{}.ts", record.module_path)),
                rendered,
            )?;
        }

        for union in &model.unions {
            let rendered = render_template(
                "typescript",
                "union.ts.j2",
                context! { union => union, options => &options, runtime_format => runtime_format },
            )?;
            write_file(&out_dir.join(format!("{}.ts", union.module_path)), rendered)?;
        }

        let rendered = render_template(
            "typescript",
            "runtime.ts.j2",
            context! { runtime_format => runtime_format },
        )?;
        write_file(&out_dir.join("sora_runtime.ts"), rendered)?;

        let rendered = render_template(
            "typescript",
            "config.ts.j2",
            context! { model => &model, options => &options, runtime_format => runtime_format },
        )?;
        write_file(&out_dir.join("sora_config.ts"), rendered)?;

        for (module_path, rendered) in ecmascript_barrels(&model.modules, options.import_ext) {
            write_file(&out_dir.join(format!("{module_path}.ts")), rendered)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::options::{EnumRepr, RuntimeFormat, TypeScriptCodegenOptions};
    use sora_ir::{model::ConfigIr, normalize::normalize_schema};
    use sora_schema::model::ProjectSchema;
    use std::{
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };

    static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn generates_typescript_files() {
        let ir = example_ir();
        let base = temp_dir();

        TypeScriptCodeGenerator.generate(&ir, &base).unwrap();

        let item = std::fs::read_to_string(base.join("item.ts")).unwrap();
        let item_type = std::fs::read_to_string(base.join("item_type.ts")).unwrap();
        let action = std::fs::read_to_string(base.join("action.ts")).unwrap();
        let runtime = std::fs::read_to_string(base.join("sora_runtime.ts")).unwrap();
        let config = std::fs::read_to_string(base.join("sora_config.ts")).unwrap();
        let index = std::fs::read_to_string(base.join("index.ts")).unwrap();

        assert!(item.contains("export interface Item"));
        assert!(item.contains("largeId: bigint;"));
        assert!(item.contains("itemType: ItemType;"));
        assert!(item.contains("import type { ItemType } from \"./item_type.js\";"));
        assert!(
            item.contains(
                "import { collectItemTypeTextKeys, decodeItemType, decodeItemTypeValue } from \"./item_type.js\";"
            )
        );
        assert!(item.contains("largeId: reader.readI64()"));
        assert!(item_type.contains("export type ItemType ="));
        assert!(item_type.contains("\"Weapon\""));
        assert!(action.contains("export type Action ="));
        assert!(action.contains("type: \"AddItem\""));
        assert!(runtime.contains("readI64(): bigint"));
        assert!(item.contains("export class ItemTable"));
        assert!(item.contains("get(key: number): Item | undefined"));
        assert!(item.contains("getByName(name: string): Item | undefined"));
        assert!(item.contains("findByItemType(itemType: ItemType): Item[]"));
        assert!(!config.contains("export class ItemTable"));
        assert!(config.contains("static fromSource(source: SoraTableSource): SoraConfig"));
        assert!(runtime.contains("export interface SoraConfigTable"));
        assert!(runtime.contains("export interface SoraTableSource"));
        assert!(index.contains("export * from \"./sora_config.js\";"));
        assert!(index.ends_with('\n'));

        let _ = std::fs::remove_dir_all(base);
    }

    #[test]
    fn typescript_enum_integer_option_changes_api() {
        let ir = example_ir();
        let base = temp_dir();

        TypeScriptCodeGenerator
            .generate_with_options(
                &ir,
                TypeScriptCodegenOptions {
                    enum_repr: EnumRepr::Integer,
                    ..Default::default()
                },
                &base,
            )
            .unwrap();

        let item_type = std::fs::read_to_string(base.join("item_type.ts")).unwrap();
        assert!(item_type.contains("export type ItemType ="));
        assert!(item_type.contains("| 10"));
        assert!(item_type.contains("| 20"));
        assert!(item_type.contains("Weapon: 10"));
        assert!(item_type.contains("return id as ItemType;"));

        let _ = std::fs::remove_dir_all(base);
    }

    #[test]
    fn typescript_supports_export_runtime_formats() {
        for (format, parse_fn, decode_fn) in [
            (RuntimeFormat::Json, "static parseJson", "decodeItemValue"),
            (RuntimeFormat::Cbor, "static parseCbor", "decodeItemValue"),
            (
                RuntimeFormat::SoraProtobuf,
                "static parseProtobuf",
                "decodeItemValue",
            ),
        ] {
            let ir = example_ir();
            let base = temp_dir();

            TypeScriptCodeGenerator
                .generate_with_options(
                    &ir,
                    TypeScriptCodegenOptions {
                        runtime_format: format,
                        ..Default::default()
                    },
                    &base,
                )
                .unwrap();

            let config = std::fs::read_to_string(base.join("sora_config.ts")).unwrap();
            let runtime = std::fs::read_to_string(base.join("sora_runtime.ts")).unwrap();
            let item = std::fs::read_to_string(base.join("item.ts")).unwrap();
            let item_type = std::fs::read_to_string(base.join("item_type.ts")).unwrap();

            assert!(!config.contains("SoraValueBundle"));
            assert!(!config.contains(parse_fn));
            assert!(runtime.contains(parse_fn));
            assert!(config.contains("fromSource(source: SoraTableSource)"));
            assert!(config.contains(decode_fn));
            assert!(item.contains("decodeItemValue"));
            assert!(item.contains("object.get(\"id\")"));
            assert!(item_type.contains("decodeItemTypeValue"));
            if format == RuntimeFormat::Cbor {
                assert!(runtime.contains("from \"cbor-x\""));
            }
            if format == RuntimeFormat::SoraProtobuf {
                assert!(runtime.contains("from \"protobufjs\""));
                assert!(runtime.contains("new protobuf.Type(\"Bundle\")"));
            }

            let _ = std::fs::remove_dir_all(base);
        }
    }

    #[test]
    fn generates_namespaced_typescript_modules() {
        let schema: ProjectSchema = toml::from_str(
            r#"
project = { id = "game_config" }
groups = { common = { default = true } }
views = { default = { contract = "game_config/default", groups = ["common"] } }

[[structs]]
name = "common.Reward"
[[structs.fields]]
name = "amount"
type = "i32"

[[structs]]
name = "quests.Reward"
[[structs.fields]]
name = "quest_id"
type = "string"

[[tables]]
id = "items.item"
name = "items.Item"
mode = "map"
key = "id"
[[tables.fields]]
name = "id"
type = "string"
[[tables.fields]]
name = "reward"
type = "struct<common.Reward>"
[[tables.fields]]
name = "quest_reward"
type = "struct<quests.Reward>"
"#,
        )
        .unwrap();
        let ir = normalize_schema(schema).unwrap();
        let base = temp_dir();
        TypeScriptCodeGenerator.generate(&ir, &base).unwrap();

        let item = std::fs::read_to_string(base.join("items/item.ts")).unwrap();
        let root_index = std::fs::read_to_string(base.join("index.ts")).unwrap();
        let items_index = std::fs::read_to_string(base.join("items/index.ts")).unwrap();
        let config = std::fs::read_to_string(base.join("sora_config.ts")).unwrap();
        assert!(item.contains("from \"../sora_runtime.js\""));
        assert!(item.contains("from \"./../common/reward.js\""));
        assert!(item.contains("Reward as CommonReward"));
        assert!(item.contains("Reward as QuestsReward"));
        assert!(item.contains("readonly reward: CommonReward"));
        assert!(item.contains("readonly questReward: QuestsReward"));
        assert!(config.contains("ItemTable as ItemsItemTable"));
        assert!(config.contains("from \"./items/item.js\""));
        assert!(config.contains("itemsItem(): ItemsItemTable"));
        assert!(root_index.contains("export * as items from \"./items/index.js\";"));
        assert!(items_index.contains("export * from \"./item.js\";"));
        let _ = std::fs::remove_dir_all(base);
    }

    fn example_ir() -> ConfigIr {
        let schema: ProjectSchema = toml::from_str(
            r#"
project = { id = "game_config" }
groups = { common = { default = true } }
views = { default = { contract = "game_config/default", groups = ["common"] } }

[[enums]]
name = "ItemType"
values = [{ id = 10, name = "Weapon" }, { id = 20, name = "Armor" }]

[[unions]]
name = "Action"

[[unions.variants]]
name = "AddItem"

[[unions.variants.fields]]
name = "item_id"
type = "i32"

[[tables]]
id = "item"
name = "Item"
mode = "map"
key = "id"

[[tables.fields]]
name = "id"
type = "i32"

[[tables.fields]]
name = "name"
type = "string"

[[tables.fields]]
name = "item_type"
type = "enum<ItemType>"

[[tables.fields]]
name = "large_id"
type = "i64"

[[tables.fields]]
name = "action"
type = "union<Action>"

[[tables.indexes]]
name = "by_name"
fields = ["name"]
unique = true

[[tables.indexes]]
name = "by_item_type"
fields = ["item_type"]
"#,
        )
        .unwrap();

        normalize_schema(schema).unwrap()
    }

    fn temp_dir() -> PathBuf {
        let unique = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("sora-typescript-codegen-test-{unique}"))
    }
}
