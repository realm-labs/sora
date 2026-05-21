use super::*;
use std::sync::atomic::{AtomicU64, Ordering};

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

#[test]
fn build_command_generates_configured_outputs() {
    let base = temp_dir();
    let project = write_project(&base);

    run(
        BuildArgs {
            project: project.clone(),
            default_source_format: None,
            data_root: None,
            scope: None,
            target: Vec::new(),
            clean: false,
        },
        &ExecutionContext::default(),
    )
    .unwrap();

    assert!(base.join("generated/schema.lock").exists());
    assert!(base.join("generated/excel/Item.xlsx").exists());
    assert!(base.join("generated/rust/item.rs").exists());
    assert!(base.join("generated/lua/item.lua").exists());
    assert!(base.join("generated/c/item.h").exists());
    assert!(base.join("generated/c/sora_config.h").exists());
    assert!(base.join("generated/cpp/item.hpp").exists());
    assert!(base.join("generated/cpp/sora_config.hpp").exists());
    assert!(base.join("generated/typescript/item.ts").exists());
    assert!(base.join("generated/javascript/item.js").exists());
    assert!(base.join("generated/javascript/item.d.ts").exists());
    assert!(base.join("generated/erlang/item.erl").exists());
    assert!(base.join("generated/python/item.py").exists());
    assert!(base.join("generated/python/sora_config.py").exists());
    assert!(base.join("generated/dart/item.dart").exists());
    assert!(base.join("generated/dart/sora_config.dart").exists());
    assert!(base.join("generated/godot/item.gd").exists());
    assert!(base.join("generated/godot/sora_config.gd").exists());
    assert!(base.join("generated/proto/sora_config.proto").exists());
    assert!(base.join("generated/config.sora").exists());
    assert!(base.join("generated/config.json").exists());
    assert!(base.join("generated/config.sora.pb").exists());
    assert!(base.join("generated/config.pb").exists());
    assert!(base.join("generated/config.cbor").exists());
    assert!(base.join("generated/debug-json/Item.json").exists());

    let _ = fs::remove_dir_all(base);
}

#[test]
fn build_command_can_filter_codegen_targets() {
    let base = temp_dir();
    let project = write_project(&base);
    let rust_stale = base.join("generated/rust/stale.txt");
    let kotlin_stale = base.join("generated/kotlin/stale.txt");
    fs::create_dir_all(rust_stale.parent().unwrap()).unwrap();
    fs::create_dir_all(kotlin_stale.parent().unwrap()).unwrap();
    fs::write(&rust_stale, "stale").unwrap();
    fs::write(&kotlin_stale, "stale").unwrap();

    run(
        BuildArgs {
            project: project.clone(),
            default_source_format: None,
            data_root: None,
            scope: None,
            target: vec![BuildTarget::Rust],
            clean: true,
        },
        &ExecutionContext::default(),
    )
    .unwrap();

    assert!(base.join("generated/rust/item.rs").exists());
    assert!(!rust_stale.exists());
    assert!(kotlin_stale.exists());
    assert!(!base.join("generated/kotlin/game_config/Item.kt").exists());

    let _ = fs::remove_dir_all(base);
}

#[test]
fn build_command_rejects_missing_runtime_export() {
    let base = temp_dir();
    let project = write_project(&base);
    let content = fs::read_to_string(&project).unwrap().replace(
        r#"
[[build.exports]]
format = "binary"
out = "generated/config.sora"
"#,
        "",
    );
    fs::write(&project, content).unwrap();

    let error = run(
        BuildArgs {
            project: project.clone(),
            default_source_format: None,
            data_root: None,
            scope: None,
            target: vec![BuildTarget::Rust],
            clean: false,
        },
        &ExecutionContext::default(),
    )
    .unwrap_err();

    assert!(
        error
            .to_string()
            .contains("rust codegen uses runtime_format `sora` and requires a `binary` export")
    );

    let _ = fs::remove_dir_all(base);
}

#[test]
fn build_command_rejects_unsupported_runtime_format() {
    let base = temp_dir();
    let project = write_project(&base);
    let content = fs::read_to_string(&project).unwrap().replace(
        r#"[codegen.dart]
runtime_format = "json""#,
        r#"[codegen.dart]
runtime_format = "sora""#,
    );
    fs::write(&project, content).unwrap();

    let error = run(
        BuildArgs {
            project: project.clone(),
            default_source_format: None,
            data_root: None,
            scope: None,
            target: vec![BuildTarget::Dart],
            clean: false,
        },
        &ExecutionContext::default(),
    )
    .unwrap_err();

    assert!(
        error
            .to_string()
            .contains("dart codegen runtime_format `sora` is not supported")
    );

    let _ = fs::remove_dir_all(base);
}

fn write_project(base: &Path) -> PathBuf {
    let data_dir = base.join("data");
    let schema_dir = base.join("schema");
    fs::create_dir_all(&data_dir).unwrap();
    fs::create_dir_all(&schema_dir).unwrap();

    let project = base.join("project.toml");
    fs::write(
        &project,
        r#"
package = "game_config"
includes = ["schema/items.toml"]

[build]
default_source_format = "toml"
data_root = "data"
schema_lock = "generated/schema.lock"
excel_templates = "generated/excel"

[[build.codegen]]
target = "rust"
out = "generated/rust"
format = "auto"

[[build.codegen]]
target = "kotlin"
out = "generated/kotlin"

[[build.codegen]]
target = "scala"
out = "generated/scala"

[[build.codegen]]
target = "lua"
out = "generated/lua"

[[build.codegen]]
target = "c"
out = "generated/c"

[[build.codegen]]
target = "cpp"
out = "generated/cpp"

[[build.codegen]]
target = "typescript"
out = "generated/typescript"

[[build.codegen]]
target = "javascript"
out = "generated/javascript"

[[build.codegen]]
target = "erlang"
out = "generated/erlang"

[[build.codegen]]
target = "python"
out = "generated/python"
format = "auto"

[[build.codegen]]
target = "dart"
out = "generated/dart"

[[build.codegen]]
target = "godot"
out = "generated/godot"

[[build.codegen]]
target = "proto-schema"
out = "generated/proto"

[[build.exports]]
format = "binary"
out = "generated/config.sora"

[[build.exports]]
format = "json"
out = "generated/config.json"

[[build.exports]]
format = "sora-protobuf"
out = "generated/config.sora.pb"

[[build.exports]]
format = "proto"
out = "generated/config.pb"

[[build.exports]]
format = "cbor"
out = "generated/config.cbor"

[[build.exports]]
format = "json-debug"
out = "generated/debug-json"

[codegen.dart]
runtime_format = "json"

[codegen.godot]
runtime_format = "json"
"#,
    )
    .unwrap();

    fs::write(
        schema_dir.join("items.toml"),
        r#"
[[enums]]
name = "ItemType"
values = ["Weapon", "Armor"]

[[tables]]
name = "Item"
mode = "map"
key = "id"

[tables.source]
file = "items.toml"

[[tables.fields]]
name = "id"
type = "i32"
key = true
required = true

[[tables.fields]]
name = "name"
type = "string"
required = true

[[tables.fields]]
name = "item_type"
type = "enum<ItemType>"
required = true
"#,
    )
    .unwrap();

    fs::write(
        data_dir.join("items.toml"),
        r#"
[[rows]]
id = 1001
name = "Iron Sword"
item_type = "Weapon"
"#,
    )
    .unwrap();

    project
}

fn temp_dir() -> PathBuf {
    let unique = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("sora-cli-build-test-{unique}"))
}
