use std::path::Path;

use minijinja::context;
use sora_diagnostics::Result;
use sora_ir::model::ConfigIr;

use crate::{
    ecmascript::{EcmaScriptModel, EcmaScriptOptionsView, EcmaScriptTarget},
    generator::{CodeGenerator, runtime_format_name},
    model::build_base_model,
    render::{ensure_dir, render_template, write_file},
};

pub struct TypeScriptCodeGenerator;

impl CodeGenerator for TypeScriptCodeGenerator {
    fn generate(&self, ir: &ConfigIr, out_dir: &Path) -> Result<()> {
        ensure_dir(out_dir)?;
        let runtime_format = runtime_format_name(ir.codegen.typescript.runtime_format);

        let options = EcmaScriptOptionsView::new(
            EcmaScriptTarget::TypeScript,
            ir.codegen.typescript.enum_repr,
            false,
        );
        let model = EcmaScriptModel::from_base_model(ir, build_base_model(ir)?);

        for item in &model.enums {
            let rendered = render_template(
                "typescript",
                "enum.ts.j2",
                context! { enum => item, options => &options, runtime_format => runtime_format },
            )?;
            write_file(&out_dir.join(format!("{}.ts", item.snake_name)), rendered)?;
        }

        for record in &model.records {
            let rendered = render_template(
                "typescript",
                "record.ts.j2",
                context! { record => record, options => &options, runtime_format => runtime_format },
            )?;
            write_file(&out_dir.join(format!("{}.ts", record.snake_name)), rendered)?;
        }

        for union in &model.unions {
            let rendered = render_template(
                "typescript",
                "union.ts.j2",
                context! { union => union, options => &options, runtime_format => runtime_format },
            )?;
            write_file(&out_dir.join(format!("{}.ts", union.snake_name)), rendered)?;
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

        let rendered = render_template(
            "typescript",
            "index.ts.j2",
            context! { model => &model, options => &options },
        )?;
        write_file(&out_dir.join("index.ts"), rendered)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sora_ir::{model::EnumReprIr, normalize::normalize_schema};
    use sora_schema::model::SchemaFile;
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
                "import { decodeItemType, decodeItemTypeValue } from \"./item_type.js\";"
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
        let mut ir = example_ir();
        ir.codegen.typescript.enum_repr = EnumReprIr::Integer;
        let base = temp_dir();

        TypeScriptCodeGenerator.generate(&ir, &base).unwrap();

        let item_type = std::fs::read_to_string(base.join("item_type.ts")).unwrap();
        assert!(item_type.contains("export type ItemType ="));
        assert!(item_type.contains("| 0"));
        assert!(item_type.contains("| 1"));
        assert!(item_type.contains("Weapon: 0"));
        assert!(item_type.contains("return ordinal as ItemType;"));

        let _ = std::fs::remove_dir_all(base);
    }

    #[test]
    fn typescript_supports_export_runtime_formats() {
        for (format, parse_fn, decode_fn) in [
            (
                sora_ir::model::RuntimeFormatIr::Json,
                "static parseJson",
                "decodeItemValue",
            ),
            (
                sora_ir::model::RuntimeFormatIr::Cbor,
                "static parseCbor",
                "decodeItemValue",
            ),
            (
                sora_ir::model::RuntimeFormatIr::SoraProtobuf,
                "static parseProtobuf",
                "decodeItemValue",
            ),
        ] {
            let mut ir = example_ir();
            ir.codegen.typescript.runtime_format = format;
            let base = temp_dir();

            TypeScriptCodeGenerator.generate(&ir, &base).unwrap();

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
            if format == sora_ir::model::RuntimeFormatIr::Cbor {
                assert!(runtime.contains("from \"cbor-x\""));
            }
            if format == sora_ir::model::RuntimeFormatIr::SoraProtobuf {
                assert!(runtime.contains("from \"protobufjs\""));
                assert!(runtime.contains("new protobuf.Type(\"Bundle\")"));
            }

            let _ = std::fs::remove_dir_all(base);
        }
    }

    fn example_ir() -> ConfigIr {
        let schema: SchemaFile = toml::from_str(
            r#"
package = "game_config"

[[enums]]
name = "ItemType"
values = ["Weapon", "Armor"]

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
required = true

[[tables.fields]]
name = "name"
type = "string"
required = true

[[tables.fields]]
name = "item_type"
type = "enum<ItemType>"
required = true

[[tables.fields]]
name = "large_id"
type = "i64"
required = true

[[tables.fields]]
name = "action"
type = "union<Action>"
required = true

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
