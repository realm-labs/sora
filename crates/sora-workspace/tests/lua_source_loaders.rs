use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use sora_workspace::{
    BuildRequest, DataOperation, ProjectId, ProjectSession, RowSelector, RuntimeOptions,
    TableQuery, WorkspaceService, build_project,
};

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

#[test]
fn project_loader_reads_multiple_files_in_stable_order_and_returns_nested_values() {
    let root = project(loader_script());
    write_json(
        &root.join("data/items/z.json"),
        r#"{"id":2,"name":"z","tags":[],"props":{"rank":2},"target":1}"#,
    );
    write_json(
        &root.join("data/items/a.json"),
        r#"{"id":1,"name":"a","tags":["x","y"],"props":{"rank":1},"target":1}"#,
    );
    let workspace = WorkspaceService::new();
    let session = workspace
        .open_project(
            ProjectId::new("custom-source").unwrap(),
            root.join("project.toml"),
            RuntimeOptions::default(),
        )
        .unwrap();

    let report = session
        .query_table(&TableQuery {
            table: "Item".to_owned(),
            limit: Some(10),
            ..TableQuery::default()
        })
        .unwrap_or_else(|error| panic!("{error:#}; root={:?}", error.root_cause()));

    assert_eq!(report.rows.len(), 2);
    assert_eq!(report.rows[0].values["name"], serde_json::json!("a"));
    assert_eq!(report.rows[0].values["tags"], serde_json::json!(["x", "y"]));
    assert_eq!(
        report.rows[0].values["props"],
        serde_json::json!({"rank": 1})
    );
    assert_eq!(report.rows[1].values["name"], serde_json::json!("z"));

    let build = build_project(
        BuildRequest {
            project: root.join("project.toml"),
            default_source_format: None,
            data_root: None,
            view: None,
            include_schema_lock: false,
            include_excel_templates: false,
            include_codegen: false,
            include_exports: true,
            targets: Vec::new(),
            export_formats: Vec::new(),
            clean: true,
        },
        session.runtime(),
    )
    .unwrap();
    assert_eq!(build.artifacts.len(), 1);
    assert!(root.join("generated/config.json").is_file());

    let dependencies = session.runtime().source_dependencies();
    assert!(
        dependencies
            .iter()
            .any(|item| item.path == Path::new("tools/source_loaders.lua"))
    );
    assert!(
        dependencies
            .iter()
            .any(|item| item.path.ends_with("items/a.json"))
    );
    assert!(
        dependencies
            .iter()
            .all(|item| item.digest.starts_with("sha256:"))
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn declared_extension_infers_a_custom_file_format() {
    let root = project(
        r#"
return { source_loaders = { custom_format = {
  extensions = { "custom" },
  load = function(source, ctx)
    return { rows = { ctx.json_decode(ctx.read_text(".")) } }
  end,
} } }
"#,
    );
    let schema_path = root.join("schema/items.toml");
    let schema = fs::read_to_string(&schema_path).unwrap().replace(
        "source = { file = \"items\", format = \"custom_format\" }",
        "source = { file = \"item.custom\" }",
    );
    fs::write(schema_path, schema).unwrap();
    write_json(
        &root.join("data/item.custom"),
        r#"{"id":1,"name":"inferred","tags":[],"props":{"rank":1},"target":1}"#,
    );
    let workspace = WorkspaceService::new();
    let session = workspace
        .open_project(
            ProjectId::new("custom-extension").unwrap(),
            root.join("project.toml"),
            RuntimeOptions::default(),
        )
        .unwrap();
    let report = session
        .query_table(&TableQuery {
            table: "Item".to_owned(),
            ..TableQuery::default()
        })
        .unwrap();
    assert_eq!(report.rows[0].values["name"], serde_json::json!("inferred"));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn json_nulls_and_explicit_null_sentinel_survive_loader_tables() {
    let root = project(
        r#"
return { source_loaders = { custom_format = {
  load = function(source, ctx)
    local row = ctx.json_decode(ctx.read_text("item.json"))
    if row.palette["."] ~= ctx.null then
      ctx.error({ message = "JSON null palette entry was not preserved" })
    end
    row.palette = ctx.array({
      ctx.array({ ".", row.palette["."] }),
      ctx.array({ "G", row.palette["G"] }),
    })
    row.explicit_null = ctx.null
    return { rows = { row } }
  end,
} } }
"#,
    );
    let schema_path = root.join("schema/items.toml");
    let schema = fs::read_to_string(&schema_path).unwrap().replace(
        "[[tables.fields]]\nname = \"target\"\ntype = \"ref<Target.id>\"",
        r#"[[tables.fields]]
name = "target"
type = "ref<Target.id>"
[[tables.fields]]
name = "null_field"
type = "optional<string>"
[[tables.fields]]
name = "nullable_values"
type = "list<optional<string>>"
[[tables.fields]]
name = "palette"
type = "map<string,optional<string>>"
[[tables.fields]]
name = "explicit_null"
type = "optional<string>""#,
    );
    fs::write(schema_path, schema).unwrap();
    write_json(
        &root.join("data/items/item.json"),
        r#"{
  "id": 1,
  "name": "layout",
  "tags": [],
  "props": {"rank": 1},
  "target": 1,
  "null_field": null,
  "nullable_values": ["first", null, "third"],
  "palette": {".": null, "G": "terrain.grass"}
}"#,
    );

    let workspace = WorkspaceService::new();
    let session = workspace
        .open_project(
            ProjectId::new("custom-null").unwrap(),
            root.join("project.toml"),
            RuntimeOptions::default(),
        )
        .unwrap();
    let report = session
        .query_table(&TableQuery {
            table: "Item".to_owned(),
            ..TableQuery::default()
        })
        .unwrap();
    let row = &report.rows[0].values;
    assert_eq!(row["null_field"], serde_json::Value::Null);
    assert_eq!(
        row["nullable_values"],
        serde_json::json!(["first", null, "third"])
    );
    assert_eq!(
        row["palette"],
        serde_json::json!([[".", null], ["G", "terrain.grass"]])
    );
    assert_eq!(row["explicit_null"], serde_json::Value::Null);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn sora_still_validates_refs_from_custom_loader_output() {
    let root = project(loader_script());
    write_json(
        &root.join("data/items/item.json"),
        r#"{"id":1,"name":"bad","tags":[],"props":{"rank":1},"target":999}"#,
    );
    let workspace = WorkspaceService::new();
    let session = workspace
        .open_project(
            ProjectId::new("custom-ref").unwrap(),
            root.join("project.toml"),
            RuntimeOptions::default(),
        )
        .unwrap();
    let error = session
        .query_table(&TableQuery {
            table: "Item".to_owned(),
            ..TableQuery::default()
        })
        .unwrap_err();
    assert!(format!("{:?}", error.root_cause()).contains("MissingReference"));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn source_loader_cannot_override_builtin_or_duplicate_a_format() {
    let builtin = project(
        r#"
return { source_loaders = { json = { load = function() return { rows = {} } end } } }
"#,
    );
    let error = match sora_workspace::ProjectRuntime::load(
        Some(&builtin.join("project.toml")),
        RuntimeOptions::default(),
    ) {
        Ok(_) => panic!("built-in format override must fail"),
        Err(error) => error,
    };
    assert!(error.to_string().contains("already registered"));

    let builtin_extension = project(
        r#"
return { source_loaders = { custom_format = {
  extensions = { "json" },
  load = function() return { rows = {} } end,
} } }
"#,
    );
    let error = match sora_workspace::ProjectRuntime::load(
        Some(&builtin_extension.join("project.toml")),
        RuntimeOptions::default(),
    ) {
        Ok(_) => panic!("built-in extension override must fail"),
        Err(error) => error,
    };
    assert!(
        error
            .to_string()
            .contains("source extension `json` is already registered by format `json`")
    );

    let duplicate = project(loader_script());
    fs::write(duplicate.join("tools/duplicate.lua"), loader_script()).unwrap();
    let project_path = duplicate.join("project.toml");
    let manifest = fs::read_to_string(&project_path).unwrap().replace(
        "scripts = [\"tools/source_loaders.lua\"]",
        "scripts = [\"tools/source_loaders.lua\", \"tools/duplicate.lua\"]",
    );
    fs::write(&project_path, manifest).unwrap();
    let error = match sora_workspace::ProjectRuntime::load(
        Some(&project_path),
        RuntimeOptions::default(),
    ) {
        Ok(_) => panic!("duplicate loader format must fail"),
        Err(error) => error,
    };
    assert!(
        error
            .to_string()
            .contains("duplicate Lua source loader format")
    );
    let _ = fs::remove_dir_all(builtin);
    let _ = fs::remove_dir_all(builtin_extension);
    let _ = fs::remove_dir_all(duplicate);
}

#[test]
fn dependencies_are_replaced_for_each_source_load() {
    let root = project(
        r#"
return { source_loaders = { custom_format = {
  load = function(source, ctx)
    local selected = ctx.read_text("selected.txt")
    return { rows = { ctx.json_decode(ctx.read_text(selected)) } }
  end,
} } }
"#,
    );
    fs::write(root.join("data/items/selected.txt"), "a.json").unwrap();
    write_json(
        &root.join("data/items/a.json"),
        r#"{"id":1,"name":"a","tags":[],"props":{"rank":1},"target":1}"#,
    );
    write_json(
        &root.join("data/items/b.json"),
        r#"{"id":1,"name":"b","tags":[],"props":{"rank":1},"target":1}"#,
    );
    let workspace = WorkspaceService::new();
    let session = workspace
        .open_project(
            ProjectId::new("dependency-scope").unwrap(),
            root.join("project.toml"),
            RuntimeOptions::default(),
        )
        .unwrap();
    session
        .query_table(&TableQuery {
            table: "Item".to_owned(),
            ..TableQuery::default()
        })
        .unwrap();
    assert!(
        session
            .runtime()
            .source_dependencies()
            .iter()
            .any(|dependency| dependency.path.ends_with("items/a.json"))
    );

    fs::write(root.join("data/items/selected.txt"), "b.json").unwrap();
    session
        .query_table(&TableQuery {
            table: "Item".to_owned(),
            ..TableQuery::default()
        })
        .unwrap();
    let dependencies = session.runtime().source_dependencies();
    assert!(
        dependencies
            .iter()
            .any(|dependency| dependency.path.ends_with("items/b.json"))
    );
    assert!(
        dependencies
            .iter()
            .all(|dependency| !dependency.path.ends_with("items/a.json"))
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn capability_rejects_traversal_and_external_symlinks() {
    let traversal = project(
        r#"
return { source_loaders = { custom_format = {
  load = function(source, ctx)
    ctx.read_text("../outside.json")
    return { rows = {} }
  end,
} } }
"#,
    );
    let error = query_error(&traversal, "traversal");
    assert!(error.contains("cannot contain `..`"));

    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        let symlink_root = project(loader_script());
        let outside = symlink_root.join("outside.json");
        write_json(&outside, r#"{"id":1}"#);
        symlink(&outside, symlink_root.join("data/items/escape.json")).unwrap();
        let error = query_error(&symlink_root, "symlink");
        assert!(error.contains("symlink escapes"));
        let _ = fs::remove_dir_all(symlink_root);
    }
    let _ = fs::remove_dir_all(traversal);
}

#[test]
fn lua_diagnostics_cancellation_and_instruction_budget_are_stable() {
    let diagnostic = project(
        r#"
return { source_loaders = { custom_format = {
  load = function(source, ctx)
    ctx.error({ path = "bad/input.custom", line = 7, column = 3, field = "id", message = "bad record" })
  end,
} } }
"#,
    );
    let error = query_error(&diagnostic, "diagnostic");
    assert!(error.contains("bad/input.custom"));
    assert!(error.contains("bad record"));
    let workspace = WorkspaceService::new();
    let session = workspace
        .open_project(
            ProjectId::new("diagnostic-structured").unwrap(),
            diagnostic.join("project.toml"),
            RuntimeOptions::default(),
        )
        .unwrap();
    let report = session.validate_data(&Default::default());
    assert!(!report.ok);
    assert_eq!(report.diagnostics[0].span.as_ref().unwrap().start_line, 7);
    assert_eq!(
        report.diagnostics[0]
            .entity
            .as_ref()
            .unwrap()
            .field
            .as_deref(),
        Some("id")
    );

    let budget = project(
        r#"
return { source_loaders = { custom_format = {
  load = function() while true do end end,
} } }
"#,
    );
    let error = query_error(&budget, "budget");
    assert!(error.contains("instruction budget exceeded"));

    let cancelled = project(loader_script());
    let runtime = sora_workspace::ProjectRuntime::load(
        Some(&cancelled.join("project.toml")),
        RuntimeOptions::default(),
    )
    .unwrap();
    runtime.execution().cancel();
    assert!(runtime.execution().is_cancelled());
    let workspace = WorkspaceService::new();
    let session = workspace
        .open_project(
            ProjectId::new("cancelled-load").unwrap(),
            cancelled.join("project.toml"),
            RuntimeOptions::default(),
        )
        .unwrap();
    session.runtime().execution().cancel();
    let error = session
        .query_table(&TableQuery {
            table: "Item".to_owned(),
            ..TableQuery::default()
        })
        .unwrap_err();
    assert!(format!("{error:#}").contains("cancelled"));
    let _ = fs::remove_dir_all(diagnostic);
    let _ = fs::remove_dir_all(budget);
    let _ = fs::remove_dir_all(cancelled);
}

#[test]
fn loader_script_and_input_changes_affect_revision_and_custom_sources_are_read_only() {
    let root = project(loader_script());
    write_json(
        &root.join("data/items/item.json"),
        r#"{"id":1,"name":"a","tags":[],"props":{"rank":1},"target":1}"#,
    );
    let workspace = WorkspaceService::new();
    let id = ProjectId::new("custom-revision").unwrap();
    let session = workspace
        .open_project(
            id.clone(),
            root.join("project.toml"),
            RuntimeOptions::default(),
        )
        .unwrap();
    let first = session.revision();
    let error = workspace
        .preview_data_mutation(
            &id,
            "test",
            &first.schema,
            &first.data,
            vec![DataOperation::UpdateFields {
                table: "Item".to_owned(),
                selector: RowSelector::Map {
                    key: serde_json::json!(1),
                },
                fields: BTreeMap::from([("name".to_owned(), serde_json::json!("changed"))]),
            }],
        )
        .unwrap_err();
    assert!(format!("{error:#}").contains("data source format `custom_format` is not mutable"));

    write_json(
        &root.join("data/items/item.json"),
        r#"{"id":1,"name":"b","tags":[],"props":{"rank":1},"target":1}"#,
    );
    let after_input = ProjectSession::open(
        ProjectId::new("custom-revision-input").unwrap(),
        root.join("project.toml"),
        RuntimeOptions::default(),
    )
    .unwrap()
    .revision();
    assert_ne!(first.data, after_input.data);

    fs::write(
        root.join("tools/source_loaders.lua"),
        format!("{}\n-- revision change\n", loader_script()),
    )
    .unwrap();
    let after_script = ProjectSession::open(
        ProjectId::new("custom-revision-script").unwrap(),
        root.join("project.toml"),
        RuntimeOptions::default(),
    )
    .unwrap()
    .revision();
    assert_ne!(after_input.data, after_script.data);
    let _ = fs::remove_dir_all(root);
}

fn query_error(root: &Path, id: &str) -> String {
    let workspace = WorkspaceService::new();
    let session = workspace
        .open_project(
            ProjectId::new(id).unwrap(),
            root.join("project.toml"),
            RuntimeOptions::default(),
        )
        .unwrap();
    let error = session
        .query_table(&TableQuery {
            table: "Item".to_owned(),
            ..TableQuery::default()
        })
        .unwrap_err();
    format!("{error:#}")
}

fn project(script: &str) -> PathBuf {
    let root = temp_dir();
    fs::create_dir_all(root.join("schema")).unwrap();
    fs::create_dir_all(root.join("data/items")).unwrap();
    fs::create_dir_all(root.join("tools")).unwrap();
    fs::write(
        root.join("project.toml"),
        r#"
project = { id = "custom_sources" }
groups = { common = { default = true } }
views = { default = { contract = "custom_sources/default", groups = ["common"] } }
includes = ["schema/items.toml"]

[source_loaders]
scripts = ["tools/source_loaders.lua"]

[build]
data_root = "data"

[[build.exports]]
format = "json"
out = "generated/config.json"
"#,
    )
    .unwrap();
    fs::write(
        root.join("schema/items.toml"),
        r#"
[[tables]]
id = "target"
name = "Target"
mode = "map"
key = "id"
source = { file = "targets.json", format = "json" }
[[tables.fields]]
name = "id"
type = "i32"

[[structs]]
name = "Props"
[[structs.fields]]
name = "rank"
type = "i32"

[[tables]]
id = "item"
name = "Item"
mode = "map"
key = "id"
source = { file = "items", format = "custom_format" }
[[tables.fields]]
name = "id"
type = "i32"
[[tables.fields]]
name = "name"
type = "string"
[[tables.fields]]
name = "tags"
type = "list<string>"
[[tables.fields]]
name = "props"
type = "struct<Props>"
[[tables.fields]]
name = "target"
type = "ref<Target.id>"
"#,
    )
    .unwrap();
    fs::write(root.join("data/targets.json"), "[{\"id\":1}]\n").unwrap();
    fs::write(root.join("tools/source_loaders.lua"), script).unwrap();
    root
}

fn loader_script() -> &'static str {
    r#"
return {
  source_loaders = {
    custom_format = {
      extensions = { "custom" },
      load = function(source, ctx)
        if io ~= nil or os ~= nil or package ~= nil or debug ~= nil or
           require ~= nil or dofile ~= nil or loadfile ~= nil or
           math.random ~= nil or math.randomseed ~= nil then
          ctx.error({ message = "unsafe Lua capability was exposed" })
        end
        local rows = {}
        for _, entry in ipairs(ctx.list(".")) do
          if entry.kind == "file" then
            local value = ctx.json_decode(ctx.read_text(entry.path))
            table.insert(rows, value)
          end
        end
        return { rows = rows }
      end,
    },
  },
}
"#
}

fn write_json(path: &Path, content: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

fn temp_dir() -> PathBuf {
    let nonce = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "sora-lua-source-loader-{}-{nonce}",
        std::process::id()
    ))
}
