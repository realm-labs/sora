use super::*;
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, SystemTime},
};

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, PartialEq, Eq)]
struct GeneratedFileSnapshot {
    bytes: Vec<u8>,
    modified: SystemTime,
    #[cfg(unix)]
    identity: (u64, u64),
}

#[test]
fn identical_build_preserves_generated_files() {
    assert_identical_rebuild_preserves_generated_files(false);
}

#[test]
fn identical_clean_build_preserves_generated_files() {
    assert_identical_rebuild_preserves_generated_files(true);
}

#[test]
fn rebuild_updates_only_outputs_with_different_final_bytes() {
    let base = temp_dir();
    let project = write_project(&base);
    run_build(&project, false, &test_context()).unwrap();
    let before = snapshot_generated_files(&base);

    wait_for_distinct_mtime();
    let source = base.join("data/items.toml");
    let content = fs::read_to_string(&source)
        .unwrap()
        .replace("Iron Sword", "Steel Sword");
    fs::write(source, content).unwrap();
    run_build(&project, false, &test_context()).unwrap();
    let after = snapshot_generated_files(&base);

    assert_eq!(
        before.keys().collect::<Vec<_>>(),
        after.keys().collect::<Vec<_>>()
    );
    assert!(
        before
            .iter()
            .any(|(path, snapshot)| snapshot.bytes != after[path].bytes),
        "changing source data must change at least one generated artifact"
    );
    for (path, previous) in &before {
        let current = &after[path];
        if previous.bytes == current.bytes {
            assert_file_snapshot_unchanged(path, previous, current);
        }
    }

    let _ = fs::remove_dir_all(base);
}

#[test]
fn clean_removes_stale_files_and_empty_directories_without_rewriting_survivors() {
    let base = temp_dir();
    let project = write_project(&base);
    let schema_path = base.join("schema/items.toml");
    let second_table = r#"

[[tables]]
id = "skill"
name = "Skill"
mode = "map"
key = "id"

[tables.source]
file = "skills.toml"

[[tables.fields]]
name = "id"
type = "i32"

[[tables.fields]]
name = "name"
type = "string"
"#;
    let schema = fs::read_to_string(&schema_path).unwrap();
    fs::write(&schema_path, format!("{schema}{second_table}")).unwrap();
    fs::write(
        base.join("data/skills.toml"),
        r#"
[[rows]]
id = 2001
name = "Slash"
"#,
    )
    .unwrap();
    run_build(&project, true, &test_context()).unwrap();
    let before = snapshot_generated_files(&base);
    assert!(base.join("generated/rust/skill.rs").exists());
    assert!(base.join("generated/excel/Skill.xlsx").exists());
    assert!(base.join("generated/debug-json/Skill.json").exists());
    let stale_dir = base.join("generated/rust/obsolete/nested");
    fs::create_dir_all(&stale_dir).unwrap();
    fs::write(stale_dir.join("old.rs"), "stale").unwrap();

    wait_for_distinct_mtime();
    fs::write(&schema_path, schema).unwrap();
    run_build(&project, true, &test_context()).unwrap();
    let after = snapshot_generated_files(&base);

    assert!(!base.join("generated/rust/skill.rs").exists());
    assert!(!base.join("generated/excel/Skill.xlsx").exists());
    assert!(!base.join("generated/debug-json/Skill.json").exists());
    assert!(!base.join("generated/rust/obsolete").exists());
    for (path, previous) in &before {
        if let Some(current) = after.get(path)
            && previous.bytes == current.bytes
        {
            assert_file_snapshot_unchanged(path, previous, current);
        }
    }

    let _ = fs::remove_dir_all(base);
}

#[test]
fn failed_clean_build_preserves_the_last_complete_output() {
    let base = temp_dir();
    let project = write_project(&base);
    run_build(&project, true, &test_context()).unwrap();
    let before = snapshot_generated_files(&base);
    fs::write(
        base.join("data/items.toml"),
        r#"
[[rows]]
id = 1001
name = "Broken"
item_type = "MissingVariant"
"#,
    )
    .unwrap();

    wait_for_distinct_mtime();
    assert!(run_build(&project, true, &test_context()).is_err());
    let after = snapshot_generated_files(&base);

    assert_snapshots_unchanged(&before, &after);

    let _ = fs::remove_dir_all(base);
}

#[test]
fn parallel_rebuild_preserves_identical_generated_files() {
    let base = temp_dir();
    let project = write_project(&base);
    let context = sora_workspace::ProjectRuntime::load(
        None,
        sora_workspace::RuntimeOptions {
            execution: sora_execution::ExecutionOptions {
                parallel: true,
                jobs: Some(4),
            },
            ..Default::default()
        },
    )
    .unwrap();
    run_build(&project, true, &context).unwrap();
    let before = snapshot_generated_files(&base);

    wait_for_distinct_mtime();
    run_build(&project, true, &context).unwrap();
    let after = snapshot_generated_files(&base);

    assert_snapshots_unchanged(&before, &after);
    assert!(
        !base.join(".sora/build-staging").exists()
            || fs::read_dir(base.join(".sora/build-staging"))
                .unwrap()
                .next()
                .is_none()
    );

    let _ = fs::remove_dir_all(base);
}

fn assert_identical_rebuild_preserves_generated_files(clean: bool) {
    let base = temp_dir();
    let project = write_project(&base);
    run_build(&project, clean, &test_context()).unwrap();
    let before = snapshot_generated_files(&base);

    wait_for_distinct_mtime();
    run_build(&project, clean, &test_context()).unwrap();
    let after = snapshot_generated_files(&base);

    assert_snapshots_unchanged(&before, &after);
    let _ = fs::remove_dir_all(base);
}

fn run_build(
    project: &Path,
    clean: bool,
    context: &sora_workspace::ProjectRuntime,
) -> anyhow::Result<()> {
    run(
        BuildArgs {
            project: project.to_path_buf(),
            default_source_format: None,
            data_root: None,
            view: None,
            target: Vec::new(),
            clean,
        },
        context,
    )
}

fn snapshot_generated_files(base: &Path) -> BTreeMap<PathBuf, GeneratedFileSnapshot> {
    let root = base.join("generated");
    let mut paths = Vec::new();
    collect_generated_files(&root, &mut paths);
    paths.sort();
    paths
        .into_iter()
        .map(|path| {
            let metadata = fs::metadata(&path).unwrap();
            let relative = path.strip_prefix(&root).unwrap().to_path_buf();
            (
                relative,
                GeneratedFileSnapshot {
                    bytes: fs::read(&path).unwrap(),
                    modified: metadata.modified().unwrap(),
                    #[cfg(unix)]
                    identity: file_identity(&metadata),
                },
            )
        })
        .collect()
}

fn assert_snapshots_unchanged(
    before: &BTreeMap<PathBuf, GeneratedFileSnapshot>,
    after: &BTreeMap<PathBuf, GeneratedFileSnapshot>,
) {
    assert_eq!(
        before.keys().collect::<Vec<_>>(),
        after.keys().collect::<Vec<_>>(),
        "generated artifact set changed"
    );
    for (path, previous) in before {
        assert_file_snapshot_unchanged(path, previous, &after[path]);
    }
}

fn assert_file_snapshot_unchanged(
    path: &Path,
    previous: &GeneratedFileSnapshot,
    current: &GeneratedFileSnapshot,
) {
    assert!(
        previous.bytes == current.bytes,
        "generated artifact bytes changed: {}",
        path.display()
    );
    assert_eq!(
        previous.modified,
        current.modified,
        "generated artifact mtime changed: {}",
        path.display()
    );
    #[cfg(unix)]
    assert_eq!(
        previous.identity,
        current.identity,
        "generated artifact identity changed: {}",
        path.display()
    );
}

fn collect_generated_files(root: &Path, paths: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(root).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            collect_generated_files(&path, paths);
        } else {
            paths.push(path);
        }
    }
}

#[cfg(unix)]
fn file_identity(metadata: &fs::Metadata) -> (u64, u64) {
    use std::os::unix::fs::MetadataExt;
    (metadata.dev(), metadata.ino())
}

fn wait_for_distinct_mtime() {
    std::thread::sleep(Duration::from_millis(1_100));
}

#[test]
fn build_command_generates_configured_outputs() {
    let base = temp_dir();
    let project = write_project(&base);

    run(
        BuildArgs {
            project: project.clone(),
            default_source_format: None,
            data_root: None,
            view: None,
            target: Vec::new(),
            clean: false,
        },
        &test_context(),
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
            view: None,
            target: vec!["rs".to_owned()],
            clean: true,
        },
        &test_context(),
    )
    .unwrap();

    assert!(base.join("generated/rust/item.rs").exists());
    assert!(!rust_stale.exists());
    assert!(kotlin_stale.exists());
    assert!(!base.join("generated/kotlin/game_config/Item.kt").exists());

    let _ = fs::remove_dir_all(base);
}

#[test]
fn build_command_accepts_yaml_project_manifest() {
    let base = temp_dir();
    let schema_dir = base.join("schema");
    fs::create_dir_all(&schema_dir).unwrap();
    let project = base.join("project.yaml");
    fs::write(
        &project,
        r#"
project: { id: game_config }
groups: { common: { default: true } }
views: { default: { contract: game_config/default, groups: [common] } }
includes:
  - schema/items.yaml
build:
  schema_lock: generated/schema.lock
  excel_templates: generated/excel
"#,
    )
    .unwrap();
    fs::write(
        schema_dir.join("items.yaml"),
        r#"
enums:
  - name: ItemType
    values: [{ id: 0, name: Weapon }, { id: 1, name: Armor }]
tables:
  - id: item
    name: Item
    mode: map
    key: id
    source:
      file: Item.xlsx
    fields:
      - name: id
        type: i32
      - name: item_type
        type: enum<ItemType>
"#,
    )
    .unwrap();

    run(
        BuildArgs {
            project: project.clone(),
            default_source_format: None,
            data_root: None,
            view: None,
            target: Vec::new(),
            clean: false,
        },
        &test_context(),
    )
    .unwrap();

    assert!(base.join("generated/schema.lock").exists());
    assert!(base.join("generated/excel/Item.xlsx").exists());

    let _ = fs::remove_dir_all(base);
}

#[test]
fn build_command_accepts_json_project_manifest() {
    let base = temp_dir();
    let schema_dir = base.join("schema");
    fs::create_dir_all(&schema_dir).unwrap();
    let project = base.join("project.json");
    fs::write(
        &project,
        r#"
{
  "project": { "id": "game_config" },
  "groups": { "common": { "default": true } },
  "views": {
    "default": { "contract": "game_config/default", "groups": ["common"] }
  },
  "includes": ["schema/items.json"],
  "build": {
    "schema_lock": "generated/schema.lock",
    "excel_templates": "generated/excel"
  }
}
"#,
    )
    .unwrap();
    fs::write(
        schema_dir.join("items.json"),
        r#"
{
  "enums": [
    { "name": "ItemType", "values": [{ "id": 0, "name": "Weapon" }, { "id": 1, "name": "Armor" }] }
  ],
  "tables": [
    {
      "id": "item",
      "name": "Item",
      "mode": "map",
      "key": "id",
      "source": { "file": "Item.xlsx" },
      "fields": [
        { "name": "id", "type": "i32" },
        { "name": "item_type", "type": "enum<ItemType>" }
      ]
    }
  ]
}
"#,
    )
    .unwrap();

    run(
        BuildArgs {
            project: project.clone(),
            default_source_format: None,
            data_root: None,
            view: None,
            target: Vec::new(),
            clean: false,
        },
        &test_context(),
    )
    .unwrap();

    assert!(base.join("generated/schema.lock").exists());
    assert!(base.join("generated/excel/Item.xlsx").exists());

    let _ = fs::remove_dir_all(base);
}

#[test]
fn build_command_accepts_lua_project_manifest() {
    let base = temp_dir();
    let schema_dir = base.join("schema");
    fs::create_dir_all(&schema_dir).unwrap();
    let project = base.join("project.lua");
    fs::write(
        &project,
        r#"
return {
  project = { id = "game_config" },
  groups = { common = { default = true } },
  views = {
    default = { contract = "game_config/default", groups = { "common" } },
  },
  includes = { "schema/items.lua" },
  build = {
    schema_lock = "generated/schema.lock",
    excel_templates = "generated/excel",
  },
}
"#,
    )
    .unwrap();
    fs::write(
        schema_dir.join("items.lua"),
        r#"
return {
  enums = {
    { name = "ItemType", values = { { id = 0, name = "Weapon" }, { id = 1, name = "Armor" } } },
  },
  tables = {
    {
      id = "item",
      name = "Item",
      mode = "map",
      key = "id",
      source = { file = "Item.xlsx" },
      fields = {
        { name = "id", type = "i32" },
        { name = "item_type", type = "enum<ItemType>" },
      },
    },
  },
}
"#,
    )
    .unwrap();

    run(
        BuildArgs {
            project: project.clone(),
            default_source_format: None,
            data_root: None,
            view: None,
            target: Vec::new(),
            clean: false,
        },
        &test_context(),
    )
    .unwrap();

    assert!(base.join("generated/schema.lock").exists());
    assert!(base.join("generated/excel/Item.xlsx").exists());

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
            view: None,
            target: vec!["rust".to_owned()],
            clean: false,
        },
        &test_context(),
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
            view: None,
            target: vec!["dart".to_owned()],
            clean: false,
        },
        &test_context(),
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
project = { id = "game_config" }
groups = { common = { default = true } }
includes = ["schema/items.toml"]

[views.default]
contract = "game_config/default"
groups = ["common"]

[views.default.bindings.kotlin]
package = "game_config"

[views.default.bindings.scala]
package = "game_config"

[views.default.bindings.c]
prefix = "sora"

[views.default.bindings.cpp]
namespace = "sora"

[views.default.bindings.proto-schema]
package = "game_config"

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
values = [{ id = 0, name = "Weapon" }, { id = 1, name = "Armor" }]

[[tables]]
id = "item"
name = "Item"
mode = "map"
key = "id"

[tables.source]
file = "items.toml"

[[tables.fields]]
name = "id"
type = "i32"

[[tables.fields]]
name = "name"
type = "string"

[[tables.fields]]
name = "item_type"
type = "enum<ItemType>"
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

fn test_context() -> sora_workspace::ProjectRuntime {
    sora_workspace::ProjectRuntime::load(None, sora_workspace::RuntimeOptions::default()).unwrap()
}
