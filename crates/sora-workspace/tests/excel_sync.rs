use std::{
    fs,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
};

use sora_excel::generator::ExcelTemplateGenerator;
use sora_workspace::{
    ExcelSyncControl, ExcelSyncPhase, ExcelSyncPlanError, ProjectId, RuntimeOptions,
    WorkspaceService,
};

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

#[test]
fn excel_sync_preview_is_read_only_and_apply_is_revision_bound_and_idempotent() {
    let nonce = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!(
        "sora-workspace-excel-sync-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir_all(root.join("data")).unwrap();
    fs::write(
        root.join("project.toml"),
        r#"
project = { id = "demo" }
groups = { common = { default = true } }
views = { default = { contract = "demo/default", groups = ["common"] } }
includes = ["schema.toml"]

[build]
data_root = "data"
"#,
    )
    .unwrap();
    fs::write(root.join("schema.toml"), schema(false)).unwrap();
    let bootstrap = WorkspaceService::new();
    let bootstrap_session = bootstrap
        .open_project(
            ProjectId::new("bootstrap").unwrap(),
            root.join("project.toml"),
            RuntimeOptions::default(),
        )
        .unwrap();
    ExcelTemplateGenerator
        .generate(
            &bootstrap_session.normalized_schema().unwrap(),
            &root.join("data"),
        )
        .unwrap();
    fs::write(root.join("schema.toml"), schema(true)).unwrap();

    let workspace = WorkspaceService::new();
    let project = ProjectId::new("demo").unwrap();
    let session = workspace
        .open_project(
            project.clone(),
            root.join("project.toml"),
            RuntimeOptions::default(),
        )
        .unwrap();
    let revision = session.revision();
    let workbook = root.join("data/items.xlsx");
    let original = fs::read(&workbook).unwrap();

    let cancelled_control = ExcelSyncControl::default();
    let canceller = cancelled_control.clone();
    let cancelled_control = cancelled_control.on_progress(move |progress| {
        if progress.phase == ExcelSyncPhase::StageWorkbooks {
            canceller.cancel();
        }
    });
    let error = workspace
        .preview_excel_sync_with_control(
            &project,
            "owner",
            &revision.schema,
            &revision.data,
            &cancelled_control,
        )
        .unwrap_err();
    assert!(matches!(error, ExcelSyncPlanError::OperationCancelled));
    assert_eq!(fs::read(&workbook).unwrap(), original);

    let phases = Arc::new(Mutex::new(Vec::new()));
    let captured_phases = Arc::clone(&phases);
    let control = ExcelSyncControl::default().on_progress(move |progress| {
        captured_phases.lock().unwrap().push(progress.phase);
    });
    let plan = workspace
        .preview_excel_sync_with_control(
            &project,
            "owner",
            &revision.schema,
            &revision.data,
            &control,
        )
        .unwrap();

    assert_eq!(fs::read(&workbook).unwrap(), original);
    assert_eq!(plan.file_changes.len(), 1);
    assert_eq!(plan.workbook_changes[0].sheets[0].added_columns, ["name"]);
    assert_eq!(
        phases.lock().unwrap().last(),
        Some(&ExcelSyncPhase::Complete)
    );

    let report = workspace
        .apply_excel_sync(&project, "owner", &plan.plan_id, "excel-sync-1")
        .unwrap();
    assert_ne!(fs::read(&workbook).unwrap(), original);
    assert_ne!(report.revision.data, revision.data);
    let repeated = workspace
        .apply_excel_sync(&project, "owner", &plan.plan_id, "excel-sync-1")
        .unwrap();
    assert_eq!(repeated.revision, report.revision);

    let _ = fs::remove_dir_all(root);
}

fn schema(include_name: bool) -> String {
    format!(
        r#"
[tables.Item]
id = "item"
mode = "map"
key = "id"
source = {{ file = "items.xlsx", format = "xlsx", sheet = "Item" }}

[tables.Item.fields]
id = "i32"
{}
"#,
        if include_name {
            r#"
name = "optional<string>"
"#
        } else {
            ""
        }
    )
}
