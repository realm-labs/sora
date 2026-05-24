use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use crate::{
    diff::simple_diff,
    model::{StudioField, StudioNode, StudioNodeKind, StudioSchema, StudioSummary},
    render::{parse_parser, push_field, render_schema_module},
    service::{
        load_studio_schema, preview_studio_schema, project_text_with_schema_files,
        write_studio_schema,
    },
};

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

#[test]
fn returns_partial_graph_for_validation_error() {
    let base = temp_dir();
    let project = write_project(
        &base,
        r#"
[[tables]]
name = "Item"
mode = "map"
key = "missing_id"

[[tables.fields]]
name = "id"
type = "i32"
"#,
    );

    let response = load_studio_schema(&project);

    assert!(!response.ok);
    assert_eq!(
        response
            .schema
            .as_ref()
            .unwrap()
            .nodes
            .iter()
            .find(|node| node.id == "table:Item")
            .map(|node| node.name.as_str()),
        Some("Item")
    );
    assert_eq!(
        response
            .diagnostics
            .iter()
            .find(|diagnostic| diagnostic.message.contains("key field `missing_id`"))
            .and_then(|diagnostic| diagnostic.target_id.as_deref()),
        Some("table:Item")
    );

    let _ = fs::remove_dir_all(base);
}

#[test]
fn returns_raw_graph_for_normalization_error() {
    let base = temp_dir();
    let project = write_project(
        &base,
        r#"
[[tables]]
name = "Item"
mode = "list"

[[tables.fields]]
name = "tags"
type = "set<string>"
parser = { kind = "unknown_parser" }
"#,
    );

    let response = load_studio_schema(&project);

    assert!(!response.ok);
    let schema = response.schema.as_ref().unwrap();
    let item = schema
        .nodes
        .iter()
        .find(|node| node.id == "table:Item")
        .unwrap();
    assert_eq!(item.fields[0].name, "tags");
    assert_eq!(item.fields[0].parser.as_deref(), Some("unknown_parser"));
    assert!(
        response
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("unsupported parser"))
    );

    let _ = fs::remove_dir_all(base);
}

#[test]
fn renders_editable_table_and_field_settings() {
    let schema = StudioSchema {
        package: "game_config".to_owned(),
        sources: vec!["schema/items.toml".to_owned()],
        summary: StudioSummary {
            enums: 0,
            structs: 0,
            unions: 0,
            tables: 2,
            edges: 0,
        },
        nodes: vec![
            StudioNode {
                id: "table:Item".to_owned(),
                name: "Item".to_owned(),
                kind: StudioNodeKind::Table,
                source: "schema/items.toml".to_owned(),
                scope: "client".to_owned(),
                subtitle: "map table, 2 fields".to_owned(),
                fields: vec![
                    StudioField {
                        name: "id".to_owned(),
                        ty: "i32".to_owned(),
                        scope: "all".to_owned(),
                        parser: None,
                        comment: None,
                        default: None,
                        range: None,
                        length: None,
                        source: None,
                    },
                    StudioField {
                        name: "price".to_owned(),
                        ty: "struct<ResourceCost>".to_owned(),
                        scope: "client".to_owned(),
                        parser: Some("columns (prefix=\"\")".to_owned()),
                        comment: Some("Expanded price".to_owned()),
                        default: Some("0".to_owned()),
                        range: Some([1, 10]),
                        length: Some([1, 3]),
                        source: Some(
                            "PriceRow: item_id -> id, field=value, order_by=seq".to_owned(),
                        ),
                    },
                ],
                metadata: BTreeMap::from([
                    ("mode".to_owned(), "map".to_owned()),
                    ("key".to_owned(), "id".to_owned()),
                    ("source".to_owned(), "Core.xlsx".to_owned()),
                    ("sheet".to_owned(), "Item".to_owned()),
                ]),
            },
            StudioNode {
                id: "table:PriceRow".to_owned(),
                name: "PriceRow".to_owned(),
                kind: StudioNodeKind::Table,
                source: "schema/items.toml".to_owned(),
                scope: "all".to_owned(),
                subtitle: "list table, 0 fields".to_owned(),
                fields: Vec::new(),
                metadata: BTreeMap::from([
                    ("mode".to_owned(), "list".to_owned()),
                    ("fields".to_owned(), "0".to_owned()),
                ]),
            },
        ],
        edges: Vec::new(),
    };

    let rendered = render_schema_module(&schema);

    assert!(rendered.contains("scope = \"client\""));
    assert!(rendered.contains("source = { file = \"Core.xlsx\", sheet = \"Item\" }"));
    assert!(rendered.contains("parser = { kind = \"columns\", prefix = \"\" }"));
    assert!(rendered.contains("default = \"0\""));
    assert!(rendered.contains("range = [1, 10]"));
    assert!(rendered.contains("length = [1, 3]"));
    assert!(rendered.contains(
        "from = { table = \"PriceRow\", parent_key = \"id\", child_key = \"item_id\", field = \"value\", order_by = \"seq\" }"
    ));
}

#[test]
fn does_not_render_key_for_non_map_table() {
    let schema = StudioSchema {
        package: "game_config".to_owned(),
        sources: vec!["schema/items.toml".to_owned()],
        summary: StudioSummary {
            enums: 0,
            structs: 0,
            unions: 0,
            tables: 1,
            edges: 0,
        },
        nodes: vec![StudioNode {
            id: "table:Drop".to_owned(),
            name: "Drop".to_owned(),
            kind: StudioNodeKind::Table,
            source: "schema/items.toml".to_owned(),
            scope: "all".to_owned(),
            subtitle: "list table, 1 field".to_owned(),
            fields: vec![StudioField {
                name: "id".to_owned(),
                ty: "i32".to_owned(),
                scope: "all".to_owned(),
                parser: None,
                comment: None,
                default: None,
                range: None,
                length: None,
                source: None,
            }],
            metadata: BTreeMap::from([
                ("mode".to_owned(), "list".to_owned()),
                ("key".to_owned(), "id".to_owned()),
            ]),
        }],
        edges: Vec::new(),
    };

    let rendered = render_schema_module(&schema);

    assert!(rendered.contains("mode = \"list\""));
    assert!(!rendered.contains("key = \"id\""));
}

#[test]
fn renders_comma_parser_separator() {
    let parser = parse_parser(&Some("tuple (separator=\",\")".to_owned())).unwrap();
    assert_eq!(
        parser.options.get("separator").map(String::as_str),
        Some(",")
    );

    let mut out = String::new();
    push_field(
        &mut out,
        &StudioField {
            name: "budget".to_owned(),
            ty: "struct<ComplexBudget>".to_owned(),
            scope: "all".to_owned(),
            parser: Some("tuple (separator=\",\")".to_owned()),
            comment: None,
            default: None,
            range: None,
            length: None,
            source: None,
        },
    );

    assert!(out.contains("parser = { kind = \"tuple\", separator = \",\" }"));
}

#[test]
fn previews_rendered_schema_without_writing_include() {
    let base = temp_dir();
    let project = write_project(
        &base,
        r#"
[[enums]]
name = "Rarity"
values = ["Common"]
"#,
    );
    let mut schema = load_studio_schema(&project).schema.unwrap();
    schema.package = "edited_config".to_owned();

    let preview = preview_studio_schema(&project, &schema);
    let current = fs::read_to_string(base.join("schema/items.toml")).unwrap();
    let current_project = fs::read_to_string(&project).unwrap();
    let target = format!("{} + 1 schema files", project.display());

    assert!(preview.ok);
    assert_eq!(preview.target.as_deref(), Some(target.as_str()));
    assert!(
        preview
            .diff
            .unwrap()
            .contains("+package = \"edited_config\"")
    );
    assert!(!current.contains("edited_config"));
    assert!(!current_project.contains("edited_config"));

    let _ = fs::remove_dir_all(base);
}

#[test]
fn simple_diff_reports_no_changes() {
    assert_eq!(simple_diff("same\n", "same\n"), "No changes.");
}

#[test]
fn simple_diff_keeps_insertions_tight() {
    let diff = simple_diff("a\nb\nc\nd\n", "a\nb\nx\nc\nd\n");

    assert!(diff.contains("+x\n"));
    assert!(diff.contains(" c\n"));
    assert!(!diff.contains("-c\n+d\n"));
}

#[test]
fn project_package_preview_preserves_existing_format_when_unchanged() {
    let base = temp_dir();
    let project = write_project(
        &base,
        r#"
[[enums]]
name = "Rarity"
values = ["Common"]
"#,
    );
    let project_text = fs::read_to_string(&project).unwrap();

    let rendered =
        project_text_with_schema_files(&project, "game_config", &["schema/items.toml".to_owned()])
            .unwrap();

    assert_eq!(rendered, project_text);

    let _ = fs::remove_dir_all(base);
}

#[test]
fn preview_preserves_existing_schema_node_order() {
    let base = temp_dir();
    let project = write_project(
        &base,
        r#"
[[structs]]
name = "Early"

[[unions]]
name = "Choice"

[[unions.variants]]
name = "A"

[[structs]]
name = "Late"

[[structs.fields]]
name = "value"
type = "i32"
"#,
    );
    let mut schema = load_studio_schema(&project).schema.unwrap();
    let late = schema
        .nodes
        .iter_mut()
        .find(|node| node.id == "struct:Late")
        .unwrap();
    late.fields[0].comment = Some("edited".to_owned());

    let preview = preview_studio_schema(&project, &schema);
    let content = preview.content.unwrap();
    let early = content.find("name = \"Early\"").unwrap();
    let union = content.find("name = \"Choice\"").unwrap();
    let late = content.find("name = \"Late\"").unwrap();

    assert!(early < union);
    assert!(union < late);
    assert!(preview.diff.unwrap().contains("+comment = \"edited\""));

    let _ = fs::remove_dir_all(base);
}

#[test]
fn save_writes_nodes_to_their_schema_sources() {
    let base = temp_dir();
    let schema_dir = base.join("schema");
    fs::create_dir_all(&schema_dir).unwrap();
    let project = base.join("project.toml");
    fs::write(
        &project,
        r#"
package = "game_config"
includes = ["schema/items.toml", "schema/quests.toml"]
"#,
    )
    .unwrap();
    fs::write(
        schema_dir.join("items.toml"),
        r#"
[[tables]]
name = "Item"
mode = "map"
key = "id"

[[tables.fields]]
name = "id"
type = "i32"
"#,
    )
    .unwrap();
    fs::write(
        schema_dir.join("quests.toml"),
        r#"
[[tables]]
name = "Quest"
mode = "map"
key = "id"

[[tables.fields]]
name = "id"
type = "i32"
"#,
    )
    .unwrap();
    let mut schema = load_studio_schema(&project).schema.unwrap();
    assert_eq!(schema.sources, ["schema/items.toml", "schema/quests.toml"]);
    assert_eq!(
        schema
            .nodes
            .iter()
            .find(|node| node.id == "table:Quest")
            .map(|node| node.source.as_str()),
        Some("schema/quests.toml")
    );
    schema
        .nodes
        .iter_mut()
        .find(|node| node.id == "table:Quest")
        .unwrap()
        .fields
        .push(StudioField {
            name: "name".to_owned(),
            ty: "string".to_owned(),
            scope: "all".to_owned(),
            parser: None,
            comment: None,
            default: None,
            range: None,
            length: None,
            source: None,
        });

    write_studio_schema(&project, &schema).unwrap();

    let items = fs::read_to_string(schema_dir.join("items.toml")).unwrap();
    let quests = fs::read_to_string(schema_dir.join("quests.toml")).unwrap();
    assert!(!items.contains("name = \"name\""));
    assert!(quests.contains("name = \"name\""));

    let _ = fs::remove_dir_all(base);
}

#[test]
fn save_creates_new_schema_include() {
    let base = temp_dir();
    let project = write_project(
        &base,
        r#"
[[enums]]
name = "Rarity"
values = ["Common"]
"#,
    );
    let mut schema = load_studio_schema(&project).schema.unwrap();
    schema.sources.push("schema/new_items.toml".to_owned());

    write_studio_schema(&project, &schema).unwrap();

    let project_text = fs::read_to_string(&project).unwrap();
    let new_schema = fs::read_to_string(base.join("schema/new_items.toml")).unwrap();
    assert!(project_text.contains(r#"includes = ["schema/items.toml", "schema/new_items.toml"]"#));
    assert!(new_schema.contains("# Generated by Sora Studio."));

    let _ = fs::remove_dir_all(base);
}

#[test]
fn save_rejects_removed_schema_include_that_still_owns_nodes() {
    let base = temp_dir();
    let project = write_project(
        &base,
        r#"
[[enums]]
name = "Rarity"
values = ["Common"]
"#,
    );
    let mut schema = load_studio_schema(&project).schema.unwrap();
    schema.sources = vec!["schema/other.toml".to_owned()];

    let error = write_studio_schema(&project, &schema).unwrap_err();

    assert!(
        error
            .to_string()
            .contains("belongs to unknown schema file `schema/items.toml`")
    );

    let _ = fs::remove_dir_all(base);
}

#[test]
fn save_updates_project_package() {
    let base = temp_dir();
    let project = write_project(
        &base,
        r#"
[[enums]]
name = "Rarity"
values = ["Common"]
"#,
    );
    let mut schema = load_studio_schema(&project).schema.unwrap();
    schema.package = "edited_config".to_owned();

    write_studio_schema(&project, &schema).unwrap();

    let project_text = fs::read_to_string(&project).unwrap();
    assert!(project_text.contains("package = \"edited_config\""));

    let _ = fs::remove_dir_all(base);
}

fn write_project(base: &Path, schema_text: &str) -> PathBuf {
    let schema_dir = base.join("schema");
    fs::create_dir_all(&schema_dir).unwrap();
    let project = base.join("project.toml");
    fs::write(
        &project,
        r#"
package = "game_config"
includes = ["schema/items.toml"]
"#,
    )
    .unwrap();
    fs::write(schema_dir.join("items.toml"), schema_text).unwrap();
    project
}

fn temp_dir() -> PathBuf {
    let id = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("sora-studio-test-{}-{id}", std::process::id()))
}
