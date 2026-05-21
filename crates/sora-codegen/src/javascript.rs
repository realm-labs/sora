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

pub struct JavaScriptCodeGenerator;

impl CodeGenerator for JavaScriptCodeGenerator {
    fn generate(&self, ir: &ConfigIr, out_dir: &Path) -> Result<()> {
        ensure_dir(out_dir)?;
        let runtime_format = runtime_format_name(ir.codegen.javascript.runtime_format);

        let options = EcmaScriptOptionsView::new(
            EcmaScriptTarget::JavaScript,
            ir.codegen.javascript.enum_repr,
            ir.codegen.javascript.emit_dts,
        );
        let model = EcmaScriptModel::from_base_model(ir, build_base_model(ir)?);

        for item in &model.enums {
            let rendered = render_template(
                "javascript",
                "enum.js.j2",
                context! { enum => item, options => &options, runtime_format => runtime_format },
            )?;
            write_file(&out_dir.join(format!("{}.js", item.snake_name)), rendered)?;
            if options.emit_dts {
                let rendered = render_template(
                    "javascript",
                    "enum.d.ts.j2",
                    context! { enum => item, options => &options, runtime_format => runtime_format },
                )?;
                write_file(&out_dir.join(format!("{}.d.ts", item.snake_name)), rendered)?;
            }
        }

        for record in &model.records {
            let rendered = render_template(
                "javascript",
                "record.js.j2",
                context! { record => record, options => &options, runtime_format => runtime_format },
            )?;
            write_file(&out_dir.join(format!("{}.js", record.snake_name)), rendered)?;
            if options.emit_dts {
                let rendered = render_template(
                    "javascript",
                    "record.d.ts.j2",
                    context! { record => record, options => &options, runtime_format => runtime_format },
                )?;
                write_file(
                    &out_dir.join(format!("{}.d.ts", record.snake_name)),
                    rendered,
                )?;
            }
        }

        for union in &model.unions {
            let rendered = render_template(
                "javascript",
                "union.js.j2",
                context! { union => union, options => &options, runtime_format => runtime_format },
            )?;
            write_file(&out_dir.join(format!("{}.js", union.snake_name)), rendered)?;
            if options.emit_dts {
                let rendered = render_template(
                    "javascript",
                    "union.d.ts.j2",
                    context! { union => union, options => &options, runtime_format => runtime_format },
                )?;
                write_file(
                    &out_dir.join(format!("{}.d.ts", union.snake_name)),
                    rendered,
                )?;
            }
        }

        let rendered = render_template(
            "javascript",
            "runtime.js.j2",
            context! { runtime_format => runtime_format },
        )?;
        write_file(&out_dir.join("sora_runtime.js"), rendered)?;
        if options.emit_dts {
            let rendered = render_template(
                "javascript",
                "runtime.d.ts.j2",
                context! { runtime_format => runtime_format },
            )?;
            write_file(&out_dir.join("sora_runtime.d.ts"), rendered)?;
        }

        let rendered = render_template(
            "javascript",
            "config.js.j2",
            context! { model => &model, options => &options, runtime_format => runtime_format },
        )?;
        write_file(&out_dir.join("sora_config.js"), rendered)?;
        if options.emit_dts {
            let rendered = render_template(
                "javascript",
                "config.d.ts.j2",
                context! { model => &model, options => &options, runtime_format => runtime_format },
            )?;
            write_file(&out_dir.join("sora_config.d.ts"), rendered)?;
        }

        let rendered = render_template(
            "javascript",
            "index.js.j2",
            context! { model => &model, options => &options },
        )?;
        write_file(&out_dir.join("index.js"), rendered)?;
        if options.emit_dts {
            let rendered = render_template(
                "javascript",
                "index.d.ts.j2",
                context! { model => &model, options => &options },
            )?;
            write_file(&out_dir.join("index.d.ts"), rendered)?;
        }

        let rendered = render_template("javascript", "package.json.j2", context! {})?;
        write_file(&out_dir.join("package.json"), rendered)
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
    fn generates_javascript_files_with_declarations() {
        let ir = example_ir();
        let base = temp_dir();

        JavaScriptCodeGenerator.generate(&ir, &base).unwrap();

        let item = std::fs::read_to_string(base.join("item.js")).unwrap();
        let item_dts = std::fs::read_to_string(base.join("item.d.ts")).unwrap();
        let runtime = std::fs::read_to_string(base.join("sora_runtime.js")).unwrap();
        let config = std::fs::read_to_string(base.join("sora_config.js")).unwrap();
        let config_dts = std::fs::read_to_string(base.join("sora_config.d.ts")).unwrap();
        let package = std::fs::read_to_string(base.join("package.json")).unwrap();

        assert!(item.contains("export function decodeItem(reader)"));
        assert!(item.contains("largeId: reader.readI64()"));
        assert!(item.contains("import { decodeItemType } from \"./item_type.js\";"));
        assert!(item_dts.contains("export interface Item"));
        assert!(item_dts.contains("largeId: bigint;"));
        assert!(runtime.contains("readI64()"));
        assert!(item.contains("export class ItemTable"));
        assert!(item.contains("getByName(name)"));
        assert!(item.contains("findByItemType(itemType)"));
        assert!(!config.contains("export class ItemTable"));
        assert!(item_dts.contains("export declare class ItemTable"));
        assert!(!config_dts.contains("export declare class ItemTable"));
        assert!(
            config_dts.contains("static fromBytes(bytes: Uint8Array | ArrayBuffer): SoraConfig;")
        );
        assert!(config_dts.contains("import type { ItemTable } from \"./item.js\";"));
        assert!(package.contains("\"type\": \"module\""));

        let _ = std::fs::remove_dir_all(base);
    }

    #[test]
    fn javascript_can_skip_declarations_and_use_integer_enums() {
        let mut ir = example_ir();
        ir.codegen.javascript.emit_dts = false;
        ir.codegen.javascript.enum_repr = EnumReprIr::Integer;
        let base = temp_dir();

        JavaScriptCodeGenerator.generate(&ir, &base).unwrap();

        let item_type = std::fs::read_to_string(base.join("item_type.js")).unwrap();
        assert!(item_type.contains("Weapon: 0"));
        assert!(item_type.contains("return ordinal;"));
        assert!(!base.join("item_type.d.ts").exists());

        let _ = std::fs::remove_dir_all(base);
    }

    #[test]
    fn javascript_supports_export_runtime_formats() {
        for (format, parse_fn, decode_fn) in [
            (
                sora_ir::model::RuntimeFormatIr::Json,
                "SoraValueBundle.parseJson(bytes)",
                "decodeItemValue",
            ),
            (
                sora_ir::model::RuntimeFormatIr::Cbor,
                "SoraValueBundle.parseCbor(bytes)",
                "decodeItemValue",
            ),
            (
                sora_ir::model::RuntimeFormatIr::SoraProtobuf,
                "SoraValueBundle.parseProtobuf(bytes)",
                "decodeItemValue",
            ),
        ] {
            let mut ir = example_ir();
            ir.codegen.javascript.runtime_format = format;
            let base = temp_dir();

            JavaScriptCodeGenerator.generate(&ir, &base).unwrap();

            let config = std::fs::read_to_string(base.join("sora_config.js")).unwrap();
            let runtime = std::fs::read_to_string(base.join("sora_runtime.js")).unwrap();
            let runtime_dts = std::fs::read_to_string(base.join("sora_runtime.d.ts")).unwrap();
            let item = std::fs::read_to_string(base.join("item.js")).unwrap();
            let item_dts = std::fs::read_to_string(base.join("item.d.ts")).unwrap();

            assert!(config.contains("SoraValueBundle"));
            assert!(config.contains(parse_fn));
            assert!(config.contains(decode_fn));
            assert!(item.contains("decodeItemValue"));
            assert!(item.contains("object.get(\"id\")"));
            assert!(item_dts.contains("decodeItemValue(value: SoraValue)"));
            assert!(runtime_dts.contains("export declare class SoraValueBundle"));
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
        std::env::temp_dir().join(format!("sora-javascript-codegen-test-{unique}"))
    }
}
