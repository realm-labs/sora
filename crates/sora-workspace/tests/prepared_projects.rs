use std::{
    env::temp_dir as system_temp_dir,
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use sha2::{Digest, Sha256};
use sora_codegen::type_mapping::TypeMappingContext;
use sora_ir::model::{ConfigIr, TypeIr};
use sora_workspace::{
    PreparedProject, ProjectId, ProjectRuntime, ProjectScriptKind, ProjectSession, RuntimeOptions,
    TableQuery, WorkspaceError, WorkspaceService,
};
use uuid::Uuid;

const PARSER: &str =
    "return { parsers = { approved = { parse = function(cell) return cell.text end } } }";
const MAPPING: &str = r#"return { type_mappings = { {
  target = "csharp", schema_type = "Snapshot", type_name = "ApprovedType",
} } }"#;
const LOADER: &str = r#"return { source_loaders = { approved_format = {
  load = function(source, ctx)
    return { rows = { { id = 1, name = "approved:" .. ctx.read_text("value.txt") } } }
  end,
} } }"#;
const SCRIPTS: [(&str, &str); 3] = [
    ("tools/parser.lua", PARSER),
    ("tools/mapping.lua", MAPPING),
    ("tools/source.lua", LOADER),
];
const MANIFEST: &str = r#"
project = { id = "snapshot" }
groups = { common = { default = true } }
views = { default = { contract = "snapshot/default", groups = ["common"] } }
includes = ["schema.toml"]
[parsers]
scripts = ["tools/parser.lua"]
[type_mappings]
scripts = ["tools/mapping.lua"]
[source_loaders]
scripts = ["tools/source.lua"]
"#;

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let root = system_temp_dir().join(format!("sora-prepared-{}", Uuid::new_v4()));
        fs::create_dir_all(root.join("tools")).unwrap();
        fs::create_dir_all(root.join("data/items")).unwrap();
        fs::write(root.join("project.toml"), MANIFEST).unwrap();
        fs::write(root.join("data/items/value.txt"), "data-a").unwrap();
        fs::write(
            root.join("schema.toml"),
            r#"
[tables.Item]
id = "item"
mode = "map"
key = "id"
source = { file = "items", format = "approved_format" }
[tables.Item.fields]
id = "i32"
name = "string"
"#,
        )
        .unwrap();
        for (path, source) in SCRIPTS {
            fs::write(root.join(path), source).unwrap();
        }
        Self(root)
    }

    fn prepare(&self, workspace: &WorkspaceService) -> PreparedProject {
        let root = workspace.add_root("snapshot", &self.0).unwrap();
        workspace
            .prepare_discovered_project(root.id(), "project.toml")
            .unwrap()
    }

    fn open_local(
        &self,
        workspace: &WorkspaceService,
        options: RuntimeOptions,
    ) -> Arc<ProjectSession> {
        workspace
            .open_project(
                ProjectId::new("local").unwrap(),
                self.0.join("project.toml"),
                options,
            )
            .unwrap()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn assert_approved(session: &ProjectSession) {
    assert_eq!(session.manifest().project.id, "snapshot");
    assert_eq!(
        session.manifest().parsers.scripts,
        [PathBuf::from("tools/parser.lua")]
    );
    assert!(session.runtime().schema_parsers().contains("approved"));
    assert!(session.runtime().cell_parsers().contains("approved"));
    assert!(
        session
            .runtime()
            .source_registry()
            .get("approved_format")
            .is_some()
    );
    let ir = ConfigIr {
        project_id: "snapshot".to_owned(),
        contract_id: "snapshot".to_owned(),
        view: None,
        group_defaults: Default::default(),
        views: Default::default(),
        localization: None,
        enums: Vec::new(),
        structs: Vec::new(),
        unions: Vec::new(),
        tables: Vec::new(),
    };
    let mapping = session
        .runtime()
        .type_mappings()
        .map_type(TypeMappingContext {
            target: "csharp",
            ir: &ir,
            ty: &TypeIr::Struct("Snapshot".to_owned()),
        })
        .unwrap();
    assert_eq!(mapping.type_name, "ApprovedType");
    let dependencies = session.runtime().source_dependencies();
    let loader = dependencies
        .iter()
        .find(|entry| entry.path == Path::new("tools/source.lua"))
        .unwrap();
    assert_eq!(loader.digest, sha256(LOADER));
}

fn sha256(source: &str) -> String {
    format!(
        "sha256:{}",
        Sha256::digest(source.as_bytes())
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    )
}

#[test]
fn prepare_only_captures_sources_and_keeps_inspection_metadata() {
    let fixture = Fixture::new();
    let workspace = WorkspaceService::new();
    for (path, _) in SCRIPTS {
        fs::write(
            fixture.0.join(path),
            "error('must not execute during inspection')",
        )
        .unwrap();
    }
    let prepared = fixture.prepare(&workspace);
    let inspection = prepared.inspection();
    assert!(inspection.requires_trust);
    assert_eq!(inspection.scripts.len(), 3);
    assert_eq!(inspection.scripts[0].kind, ProjectScriptKind::Parser);
    assert_eq!(inspection.scripts[1].kind, ProjectScriptKind::SourceLoader);
    assert_eq!(inspection.scripts[2].kind, ProjectScriptKind::TypeMapping);
    for script in &inspection.scripts {
        assert!(script.path.starts_with("tools/"));
        assert_eq!(
            script.digest,
            sha256("error('must not execute during inspection')")
        );
    }
    assert!(!format!("{prepared:?}").contains("must not execute"));
    let root = workspace.add_root("explicit", &fixture.0).unwrap();
    let explicit = workspace
        .prepare_discovered_project_in_root(&root, "project.toml")
        .unwrap();
    assert_eq!(explicit.inspection(), inspection);
    assert_eq!(
        &workspace
            .inspect_discovered_project(root.id(), "project.toml")
            .unwrap(),
        inspection
    );
}

#[test]
fn prepared_open_executes_captured_scripts_and_ignores_new_manifest_declarations() {
    let fixture = Fixture::new();
    let workspace = WorkspaceService::new();
    let prepared = fixture.prepare(&workspace);
    let cloned = prepared.clone();
    for (path, _) in SCRIPTS {
        fs::write(
            fixture.0.join(path),
            "error('replacement script B executed')",
        )
        .unwrap();
    }
    fs::write(
        fixture.0.join("evil.lua"),
        "error('new manifest declaration executed')",
    )
    .unwrap();
    let malicious = MANIFEST
        .replace("snapshot", "changed")
        .replace("tools/parser.lua\"]", "tools/parser.lua\", \"evil.lua\"]")
        .replace("tools/mapping.lua\"]", "tools/mapping.lua\", \"evil.lua\"]")
        .replace("tools/source.lua\"]", "tools/source.lua\", \"evil.lua\"]");
    fs::write(fixture.0.join("project.toml"), malicious).unwrap();

    let session = workspace
        .open_prepared_project(prepared, RuntimeOptions::default())
        .unwrap();
    assert_approved(&session);
    let query = || {
        session
            .query_table(&TableQuery {
                table: "Item".to_owned(),
                ..Default::default()
            })
            .unwrap()
    };
    assert_eq!(query().rows[0].values["name"], "approved:data-a");
    fs::write(fixture.0.join("data/items/value.txt"), "data-b").unwrap();
    assert_eq!(query().rows[0].values["name"], "approved:data-b");
    assert!(
        session
            .runtime()
            .source_dependencies()
            .iter()
            .any(|entry| entry.path.ends_with("items/value.txt"))
    );
    let reused = workspace
        .open_prepared_project(cloned, RuntimeOptions::default())
        .unwrap();
    assert!(Arc::ptr_eq(&session, &reused));
}

#[test]
fn prepared_open_survives_deleted_scripts_and_root_manifest() {
    let fixture = Fixture::new();
    let workspace = WorkspaceService::new();
    let prepared = fixture.prepare(&workspace);
    for (path, _) in SCRIPTS {
        fs::remove_file(fixture.0.join(path)).unwrap();
    }
    fs::remove_file(fixture.0.join("project.toml")).unwrap();
    let session = workspace
        .open_prepared_project(prepared.clone(), RuntimeOptions::default())
        .unwrap();
    assert_approved(&session);
    let reused = workspace
        .open_prepared_project(prepared, RuntimeOptions::default())
        .unwrap();
    assert!(Arc::ptr_eq(&session, &reused));
}

#[test]
fn existing_sessions_are_compared_to_loaded_bytes_not_current_disk_digests() {
    for (path, original) in SCRIPTS {
        let fixture = Fixture::new();
        let workspace = WorkspaceService::new();
        // B is valid and has the same definitions, but different executable bytes.
        fs::write(fixture.0.join(path), format!("{original}\n-- loaded B")).unwrap();
        let old = fixture.open_local(&workspace, RuntimeOptions::default());
        fs::write(fixture.0.join(path), original).unwrap();
        let approved_a = fixture.prepare(&workspace);
        let error = workspace
            .open_prepared_project(approved_a, RuntimeOptions::default())
            .unwrap_err();
        assert!(
            matches!(error, WorkspaceError::PreparedProjectMismatch(_)),
            "{error:?}"
        );
        assert!(error.to_string().contains("authorized project snapshot"));
        assert!(Arc::ptr_eq(&old, &workspace.project(old.id()).unwrap()));
    }
}

#[test]
fn prepared_open_rejects_changed_manifest_even_when_scripts_match() {
    let fixture = Fixture::new();
    let workspace = WorkspaceService::new();
    fixture.open_local(&workspace, RuntimeOptions::default());
    fs::write(
        fixture.0.join("project.toml"),
        MANIFEST.replace("snapshot", "other"),
    )
    .unwrap();
    assert!(matches!(
        workspace.open_prepared_project(fixture.prepare(&workspace), RuntimeOptions::default()),
        Err(WorkspaceError::PreparedProjectMismatch(_))
    ));
}

#[test]
fn explicit_extra_scripts_are_captured_and_part_of_session_identity() {
    let fixture = Fixture::new();
    let workspace = WorkspaceService::new();
    let extra = fixture.0.join("extra.lua");
    let source = PARSER.replace("approved", "explicit_extra");
    fs::write(&extra, &source).unwrap();
    let options = RuntimeOptions {
        parser_scripts: vec![extra.clone()],
        ..Default::default()
    };
    let prepared = fixture.prepare(&workspace);
    let session = workspace
        .open_prepared_project(prepared.clone(), options.clone())
        .unwrap();
    assert!(session.runtime().cell_parsers().contains("explicit_extra"));
    assert!(matches!(
        workspace.open_prepared_project(prepared.clone(), RuntimeOptions::default()),
        Err(WorkspaceError::PreparedProjectMismatch(_))
    ));
    fs::write(&extra, format!("{source}\n-- changed explicit extra")).unwrap();
    assert!(matches!(
        workspace.open_prepared_project(prepared, options),
        Err(WorkspaceError::PreparedProjectMismatch(_))
    ));
}

#[test]
fn session_reuse_rejects_changed_execution_options() {
    let fixture = Fixture::new();
    let workspace = WorkspaceService::new();
    let prepared = fixture.prepare(&workspace);
    workspace
        .open_prepared_project(prepared.clone(), RuntimeOptions::default())
        .unwrap();
    let mut options = RuntimeOptions::default();
    options.execution.parallel = false;
    assert!(matches!(
        workspace.open_prepared_project(prepared, options),
        Err(WorkspaceError::PreparedProjectMismatch(_))
    ));
}

#[test]
fn session_reuse_preserves_script_order_and_labels() {
    let fixture = Fixture::new();
    fs::write(
        fixture.0.join("tools/second.lua"),
        PARSER.replace("approved", "second"),
    )
    .unwrap();
    let ordered = MANIFEST.replace(
        "[\"tools/parser.lua\"]",
        "[\"tools/parser.lua\", \"tools/second.lua\"]",
    );
    fs::write(fixture.0.join("project.toml"), &ordered).unwrap();
    let workspace = WorkspaceService::new();
    let original = fixture.prepare(&workspace);
    workspace
        .open_prepared_project(original.clone(), RuntimeOptions::default())
        .unwrap();
    fs::write(
        fixture.0.join("project.toml"),
        ordered.replace(
            "\"tools/parser.lua\", \"tools/second.lua\"",
            "\"tools/second.lua\", \"tools/parser.lua\"",
        ),
    )
    .unwrap();
    let reordered = fixture.prepare(&workspace);
    assert_eq!(original.inspection(), reordered.inspection());
    assert!(matches!(
        workspace.open_prepared_project(reordered, RuntimeOptions::default()),
        Err(WorkspaceError::PreparedProjectMismatch(_))
    ));

    fs::copy(
        fixture.0.join("tools/source.lua"),
        fixture.0.join("tools/alias.lua"),
    )
    .unwrap();
    fs::write(
        fixture.0.join("project.toml"),
        ordered.replace("tools/source.lua", "tools/alias.lua"),
    )
    .unwrap();
    assert!(matches!(
        workspace.open_prepared_project(fixture.prepare(&workspace), RuntimeOptions::default()),
        Err(WorkspaceError::PreparedProjectMismatch(_))
    ));
}

#[test]
fn untrusted_boolean_open_never_bypasses_trust_for_an_existing_session() {
    let fixture = Fixture::new();
    let workspace = WorkspaceService::new();
    fixture.open_local(&workspace, RuntimeOptions::default());
    let root = workspace.add_root("snapshot", &fixture.0).unwrap();
    assert!(matches!(
        workspace.open_discovered_project(
            root.id(),
            "project.toml",
            RuntimeOptions::default(),
            false
        ),
        Err(WorkspaceError::UntrustedProjectScripts)
    ));
    assert!(matches!(
        workspace.open_discovered_project_in_root(
            &root,
            "project.toml",
            RuntimeOptions::default(),
            false
        ),
        Err(WorkspaceError::UntrustedProjectScripts)
    ));
    let reused = workspace
        .open_discovered_project_in_root(&root, "project.toml", RuntimeOptions::default(), true)
        .unwrap();
    assert_eq!(reused.id().as_str(), "local");
}

#[test]
fn new_manifest_scripts_cannot_enter_an_initially_scriptless_snapshot() {
    let fixture = Fixture::new();
    fs::write(
        fixture.0.join("project.toml"),
        "project = { id = 'no_scripts' }\n",
    )
    .unwrap();
    let workspace = WorkspaceService::new();
    let prepared = fixture.prepare(&workspace);
    assert!(!prepared.inspection().requires_trust);
    fs::write(
        fixture.0.join("project.toml"),
        "project = { id = 'injected' }\n[parsers]\nscripts = ['evil.lua']\n",
    )
    .unwrap();
    fs::write(fixture.0.join("evil.lua"), "error('must not execute')").unwrap();
    let session = workspace
        .open_prepared_project(prepared, RuntimeOptions::default())
        .unwrap();
    assert_eq!(session.manifest().project.id, "no_scripts");
    assert!(session.manifest().parsers.scripts.is_empty());
}

#[test]
fn ordinary_local_runtime_also_retains_captured_sources() {
    let fixture = Fixture::new();
    let runtime = ProjectRuntime::load(
        Some(&fixture.0.join("project.toml")),
        RuntimeOptions::default(),
    )
    .unwrap();
    for (path, _) in SCRIPTS {
        fs::write(fixture.0.join(path), "error('changed after load')").unwrap();
    }
    assert!(runtime.cell_parsers().contains("approved"));
    let loader = runtime
        .source_dependencies()
        .into_iter()
        .find(|item| item.path == Path::new("tools/source.lua"))
        .unwrap();
    assert_eq!(loader.digest, sha256(LOADER));
}

#[test]
fn prepare_rejects_scripts_outside_the_root_and_invalid_loader_labels() {
    let fixture = Fixture::new();
    let nested = fixture.0.join("nested");
    fs::create_dir_all(&nested).unwrap();
    let workspace = WorkspaceService::new();
    let root = workspace.add_root("nested", &nested).unwrap();
    for category in ["parsers", "type_mappings", "source_loaders"] {
        fs::write(
            nested.join("project.toml"),
            format!(
                "project = {{ id = 'escape' }}\n[{category}]\nscripts = ['../tools/parser.lua']\n"
            ),
        )
        .unwrap();
        let error = workspace
            .prepare_discovered_project(root.id(), "project.toml")
            .unwrap_err();
        assert!(matches!(
            error,
            WorkspaceError::ProjectScriptOutsideRoot { .. }
                | WorkspaceError::InvalidSourceLoaderScriptPath(_)
        ));
    }
}
