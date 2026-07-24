use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use sha2::Digest;
use sora_data::model::{ConfigData, RowData, TableData, Value};
use sora_excel::generator::ExcelTemplateGenerator;
use sora_input_schema::input::SchemaFileInput;
use sora_workspace::{
    DataOperation, ProjectId, RowSelector, RuntimeOptions, WorkspaceService,
    execute_data_operations,
};

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

#[test]
fn map_singleton_and_list_selectors_enforce_identity() {
    let schema: sora_schema::model::SchemaFile = toml::from_str(
        r#"
package = "selectors"

[[tables]]
name = "MapTable"
mode = "map"
key = "id"
source = { file = "map.json", format = "json" }
[[tables.fields]]
name = "id"
type = "i32"
[[tables.fields]]
name = "name"
type = "string"

[[tables]]
name = "SingletonTable"
mode = "singleton"
source = { file = "singleton.json", format = "json" }
[[tables.fields]]
name = "value"
type = "string"

[[tables]]
name = "ListTable"
mode = "list"
source = { file = "list.json", format = "json" }
[[tables.fields]]
name = "value"
type = "string"
"#,
    )
    .unwrap();
    let ir = sora_ir::normalize::normalize_schema(schema).unwrap();
    let data = ConfigData {
        tables: vec![
            table(
                "MapTable",
                row([("id", Value::Integer(1)), ("name", text("old"))]),
            ),
            table("SingletonTable", row([("value", text("old"))])),
            table("ListTable", row([("value", text("old"))])),
        ],
    };
    let list_hash = row_hash(&data.tables[2].rows[0]);
    let operations = vec![
        DataOperation::UpdateFields {
            table: "MapTable".to_owned(),
            selector: RowSelector::Map {
                key: serde_json::json!(1),
            },
            fields: BTreeMap::from([("name".to_owned(), serde_json::json!("map"))]),
        },
        DataOperation::UpdateFields {
            table: "SingletonTable".to_owned(),
            selector: RowSelector::Singleton,
            fields: BTreeMap::from([("value".to_owned(), serde_json::json!("singleton"))]),
        },
        DataOperation::UpdateFields {
            table: "ListTable".to_owned(),
            selector: RowSelector::List {
                index: 0,
                expected_row_hash: list_hash,
            },
            fields: BTreeMap::from([("value".to_owned(), serde_json::json!("list"))]),
        },
    ];

    let execution = execute_data_operations(&ir, &data, &operations).unwrap();

    assert_eq!(execution.data.tables[0].rows[0].values["name"], text("map"));
    assert_eq!(
        execution.data.tables[1].rows[0].values["value"],
        text("singleton")
    );
    assert_eq!(
        execution.data.tables[2].rows[0].values["value"],
        text("list")
    );
}

#[test]
fn preview_and_apply_roundtrip_every_mutable_file_format() {
    for format in ["json", "yaml", "toml", "csv", "xlsx"] {
        let root = project(format);
        let project = root.join("project.toml");
        let workspace = WorkspaceService::new();
        let id = ProjectId::new(format!("format-{format}")).unwrap();
        let session = workspace
            .open_project(id.clone(), &project, RuntimeOptions::default())
            .unwrap();
        let source = source_path(&root, format);
        let before = fs::read(&source).unwrap();
        let revision = session.revision();

        let plan = workspace
            .preview_data_mutation(
                &id,
                "test",
                &revision.schema,
                &revision.data,
                vec![DataOperation::UpdateFields {
                    table: "Item".to_owned(),
                    selector: RowSelector::Map {
                        key: serde_json::json!(1),
                    },
                    fields: BTreeMap::from([(
                        "name".to_owned(),
                        serde_json::json!(format!("updated-{format}")),
                    )]),
                }],
            )
            .unwrap();

        assert_eq!(fs::read(&source).unwrap(), before, "{format} preview wrote");
        assert_eq!(plan.file_changes.len(), 1);
        let report = workspace
            .apply_data_mutation(&id, "test", &plan.plan_id, &format!("apply-{format}"))
            .unwrap();
        let replay = workspace
            .apply_data_mutation(&id, "test", &plan.plan_id, &format!("apply-{format}"))
            .unwrap();
        assert_eq!(report.revision, replay.revision);
        let (_, data) = session.validated_data().unwrap();
        assert_eq!(
            data.tables[0].rows[0].values["name"],
            text(&format!("updated-{format}"))
        );
        fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn xlsx_mutation_preserves_styles_legacy_columns_and_other_sheets() {
    let root = project("xlsx");
    let path = source_path(&root, "xlsx");
    let mut workbook = umya_spreadsheet::reader::xlsx::read(&path).unwrap();
    let sheet = workbook.get_sheet_by_name_mut("Item").unwrap();
    sheet.get_style_mut((3, 8)).set_background_color("FF00FF00");
    sheet.get_cell_mut((4, 3)).set_value("legacy");
    sheet.get_cell_mut((4, 8)).set_value("keep-me");
    let other = workbook.new_sheet("Notes").unwrap();
    other.get_cell_mut("A1").set_value("untouched");
    umya_spreadsheet::writer::xlsx::write(&workbook, &path).unwrap();
    let before_style = workbook
        .get_sheet_by_name("Item")
        .unwrap()
        .get_style((3, 8))
        .clone();
    let workspace = WorkspaceService::new();
    let id = ProjectId::new("xlsx-preserve").unwrap();
    let session = workspace
        .open_project(
            id.clone(),
            root.join("project.toml"),
            RuntimeOptions::default(),
        )
        .unwrap();
    let revision = session.revision();
    let plan = workspace
        .preview_data_mutation(
            &id,
            "test",
            &revision.schema,
            &revision.data,
            vec![DataOperation::UpdateFields {
                table: "Item".to_owned(),
                selector: RowSelector::Map {
                    key: serde_json::json!(1),
                },
                fields: BTreeMap::from([("name".to_owned(), serde_json::json!("updated"))]),
            }],
        )
        .unwrap();
    workspace
        .apply_data_mutation(&id, "test", &plan.plan_id, "xlsx-preserve")
        .unwrap();

    let workbook = umya_spreadsheet::reader::xlsx::read(&path).unwrap();
    assert_eq!(
        workbook
            .get_sheet_by_name("Item")
            .unwrap()
            .get_style((3, 8)),
        &before_style
    );
    assert_eq!(
        workbook
            .get_sheet_by_name("Item")
            .unwrap()
            .get_value((4, 8)),
        "keep-me"
    );
    assert_eq!(
        workbook.get_sheet_by_name("Notes").unwrap().get_value("A1"),
        "untouched"
    );
    fs::remove_dir_all(root).unwrap();
}

fn project(format: &str) -> PathBuf {
    let root = temp_dir(format);
    fs::create_dir_all(root.join("data")).unwrap();
    fs::write(
        root.join("project.toml"),
        "package = \"formats\"\nincludes = [\"schema.toml\"]\n\n[build]\ndata_root = \"data\"\n",
    )
    .unwrap();
    let file = match format {
        "yaml" => "items.yaml",
        "xlsx" => "items.xlsx",
        _ => match format {
            "json" => "items.json",
            "toml" => "items.toml",
            "csv" => "items.csv",
            _ => unreachable!(),
        },
    };
    fs::write(
        root.join("schema.toml"),
        format!(
            r#"[[tables]]
name = "Item"
mode = "map"
key = "id"
source = {{ file = "{file}", format = "{format}", sheet = "Item" }}

[[tables.fields]]
name = "id"
type = "i32"

[[tables.fields]]
name = "name"
type = "string"
"#
        ),
    )
    .unwrap();
    match format {
        "json" => fs::write(
            root.join("data/items.json"),
            "[{\"id\":1,\"name\":\"old\"}]\n",
        )
        .unwrap(),
        "yaml" => fs::write(root.join("data/items.yaml"), "- id: 1\n  name: old\n").unwrap(),
        "toml" => fs::write(
            root.join("data/items.toml"),
            "[[rows]]\nid = 1\nname = \"old\"\n",
        )
        .unwrap(),
        "csv" => fs::write(root.join("data/items.csv"), "id,name\n1,old\n").unwrap(),
        "xlsx" => {
            let ir = sora_core::pipeline::load_schema_ir(&SchemaFileInput::new(
                root.join("project.toml"),
            ))
            .unwrap();
            ExcelTemplateGenerator
                .generate_with_rows(&ir, &root.join("data"), |_| {
                    vec![vec!["1".to_owned(), "old".to_owned()]]
                })
                .unwrap();
        }
        _ => unreachable!(),
    }
    root
}

fn source_path(root: &Path, format: &str) -> PathBuf {
    let extension = if format == "yaml" { "yaml" } else { format };
    root.join("data").join(format!("items.{extension}"))
}

fn table(name: &str, row: RowData) -> TableData {
    TableData {
        name: name.to_owned(),
        rows: vec![row],
    }
}

fn row<const N: usize>(values: [(&str, Value); N]) -> RowData {
    RowData {
        values: values
            .into_iter()
            .map(|(name, value)| (name.to_owned(), value))
            .collect(),
    }
}

fn text(value: &str) -> Value {
    Value::String(value.to_owned())
}

fn row_hash(row: &RowData) -> String {
    let natural = row
        .values
        .iter()
        .map(|(key, value)| {
            let value = match value {
                Value::String(value) => serde_json::Value::String(value.clone()),
                _ => unreachable!(),
            };
            (key.clone(), value)
        })
        .collect::<BTreeMap<_, _>>();
    let digest = sha2::Sha256::digest(serde_json::to_vec(&natural).unwrap());
    format!(
        "row:{}",
        digest
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    )
}

fn temp_dir(label: &str) -> PathBuf {
    let nonce = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "sora-data-mutation-{label}-{}-{nonce}",
        std::process::id()
    ))
}
