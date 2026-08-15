use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use super::{
    diff::simple_diff,
    model::{StudioField, StudioNode, StudioNodeKind, StudioSchema, StudioSummary},
    render::{parse_parser, render_schema_module},
    service::{
        TextFileWrite, load_studio_schema, load_studio_schema_view, preview_studio_schema,
        project_text_with_schema_files, save_studio_schema, write_studio_schema,
    },
};
use crate::mutation::commit_text_transaction;

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

#[test]
fn studio_preserves_enum_and_value_comments() {
    let base = temp_dir();
    let project = write_project(
        &base,
        r#"
[enums.ItemType]
comment = "Item category"
values = [
  { id = 0, name = "Weapon", comment = "Weapon item" },
  { id = 1, name = "Armor", comment = "Armor item" },
]
"#,
    );

    let response = load_studio_schema(&project);
    assert!(response.ok, "{:?}", response.diagnostics);
    let schema = response.schema.unwrap();
    let item_type = schema
        .nodes
        .iter()
        .find(|node| node.id == "enum:ItemType")
        .unwrap();
    assert_eq!(
        item_type.metadata.get("comment").map(String::as_str),
        Some("Item category")
    );
    assert_eq!(item_type.fields[0].comment.as_deref(), Some("Weapon item"));

    let rendered = render_schema_module(&schema);
    assert!(
        rendered.contains("comment = \"Item category\""),
        "{rendered}"
    );
    assert!(rendered.contains("comment = \"Weapon item\""), "{rendered}");

    let _ = fs::remove_dir_all(base);
}

#[test]
fn studio_preserves_module_namespace_and_imports() {
    let base = temp_dir();
    let project = write_project(
        &base,
        r#"
namespace = "game.items"
imports = { common = "game.common" }

[tables.Item]
id = "game.items.item"
mode = "map"
key = "id"

[tables.Item.fields]
id = "string"
"#,
    );

    let schema = load_studio_schema(&project).schema.unwrap();
    assert!(
        schema
            .nodes
            .iter()
            .any(|node| node.name == "game.items.Item")
    );
    write_studio_schema(&project, &schema).unwrap();

    let rendered = fs::read_to_string(base.join("schema/items.toml")).unwrap();
    assert!(
        rendered.contains("namespace = \"game.items\""),
        "{rendered}"
    );
    assert!(rendered.contains("common = \"game.common\""), "{rendered}");
    assert!(rendered.contains("[tables.Item]"), "{rendered}");
    assert!(
        !rendered.contains("tables.\"game.items.Item\""),
        "{rendered}"
    );
    assert!(load_studio_schema(&project).ok);

    let _ = fs::remove_dir_all(base);
}

#[test]
fn returns_partial_graph_for_validation_error() {
    let base = temp_dir();
    let project = write_project(
        &base,
        r#"
[tables.Item]
id = "item"
mode = "map"
key = "missing_id"

[tables.Item.fields]
id = "i32"
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
[tables.Item]
id = "item"
mode = "list"

[tables.Item.fields.tags]
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
        project_id: "game_config".to_owned(),
        groups: std::collections::BTreeMap::from([
            ("client".to_owned(), false),
            ("common".to_owned(), true),
        ]),
        views: std::collections::BTreeMap::from([(
            "default".to_owned(),
            serde_json::json!({
                "contract": "game_config/default",
                "groups": ["common", "client"],
            }),
        )]),
        sources: vec!["schema/items.toml".to_owned()],
        module_namespaces: std::collections::BTreeMap::new(),
        module_imports: std::collections::BTreeMap::new(),
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
                groups: vec!["client".to_owned()],
                subtitle: "map table, 2 fields".to_owned(),
                fields: vec![
                    StudioField {
                        name: "id".to_owned(),
                        ty: "i32".to_owned(),
                        enum_value_id: None,
                        groups: Vec::new(),
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
                        enum_value_id: None,
                        groups: vec!["client".to_owned()],
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
                aliases: Vec::new(),
                indexes: Vec::new(),
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
                groups: Vec::new(),
                subtitle: "list table, 0 fields".to_owned(),
                fields: Vec::new(),
                aliases: Vec::new(),
                indexes: Vec::new(),
                metadata: BTreeMap::from([
                    ("mode".to_owned(), "list".to_owned()),
                    ("fields".to_owned(), "0".to_owned()),
                ]),
            },
        ],
        edges: Vec::new(),
    };

    let rendered = render_schema_module(&schema);

    assert!(rendered.contains("groups = \"client\""), "{rendered}");
    assert!(rendered.contains("source = {"));
    assert!(rendered.contains("file = \"Core.xlsx\""));
    assert!(rendered.contains("sheet = \"Item\""));
    assert!(rendered.contains("kind = \"columns\""));
    assert!(rendered.contains("prefix = \"\""));
    assert!(rendered.contains("default = \"0\""));
    assert!(rendered.contains("range = ["));
    assert!(rendered.contains("length = ["));
    assert!(rendered.contains("from = {"));
    assert!(rendered.contains("table = \"PriceRow\""));
    assert!(rendered.contains("parent_key = \"id\""));
    assert!(rendered.contains("child_key = \"item_id\""));
    assert!(rendered.contains("field = \"value\""));
    assert!(rendered.contains("order_by = \"seq\""));
}

#[test]
fn does_not_render_key_for_non_map_table() {
    let schema = StudioSchema {
        project_id: "game_config".to_owned(),
        groups: std::collections::BTreeMap::from([("common".to_owned(), true)]),
        views: std::collections::BTreeMap::from([(
            "default".to_owned(),
            serde_json::json!({
                "contract": "game_config/default",
                "groups": ["common"],
            }),
        )]),
        sources: vec!["schema/items.toml".to_owned()],
        module_namespaces: std::collections::BTreeMap::new(),
        module_imports: std::collections::BTreeMap::new(),
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
            groups: Vec::new(),
            subtitle: "list table, 1 field".to_owned(),
            fields: vec![StudioField {
                name: "id".to_owned(),
                ty: "i32".to_owned(),
                enum_value_id: None,
                groups: Vec::new(),
                parser: None,
                comment: None,
                default: None,
                range: None,
                length: None,
                source: None,
            }],
            aliases: Vec::new(),
            indexes: Vec::new(),
            metadata: BTreeMap::from([
                ("mode".to_owned(), "list".to_owned()),
                ("key".to_owned(), "id".to_owned()),
            ]),
        }],
        edges: Vec::new(),
    };

    let rendered = render_schema_module(&schema);

    assert!(rendered.contains("mode = \"list\""), "{rendered}");
    assert!(!rendered.contains("key = \"id\""));
}

#[test]
fn renders_comma_parser_separator() {
    let parser = parse_parser(&Some("tuple (separator=\",\")".to_owned())).unwrap();
    assert_eq!(
        parser.options.get("separator").map(String::as_str),
        Some(",")
    );

    assert_eq!(parser.kind, "tuple");
}

#[test]
fn previews_rendered_schema_without_writing_include() {
    let base = temp_dir();
    let project = write_project(
        &base,
        r#"
[enums]
Rarity = ["Common"]
"#,
    );
    let mut schema = load_studio_schema(&project).schema.unwrap();
    schema.project_id = "edited_config".to_owned();

    let preview = preview_studio_schema(&project, &schema);
    let current = fs::read_to_string(base.join("schema/items.toml")).unwrap();
    let current_project = fs::read_to_string(&project).unwrap();
    let target = format!("{} + 1 schema files", project.display());

    assert!(preview.ok);
    assert_eq!(preview.target.as_deref(), Some(target.as_str()));
    assert!(preview.diff.unwrap().contains("+id = \"edited_config\""));
    assert!(!current.contains("edited_config"));
    assert!(!current_project.contains("edited_config"));

    let _ = fs::remove_dir_all(base);
}

#[test]
fn preserves_explicit_enum_value_ids() {
    let base = temp_dir();
    let project = write_project(
        &base,
        r#"
[enums.Rarity]
values = [{ id = 42, name = "Common" }]

[enums.Rarity.aliases]
common = "Common"
"#,
    );
    let schema = load_studio_schema(&project).schema.unwrap();
    let enum_node = schema
        .nodes
        .iter()
        .find(|node| node.id == "enum:Rarity")
        .unwrap();

    assert_eq!(enum_node.fields[0].enum_value_id, Some(42));
    assert_eq!(enum_node.aliases[0].alias, "common");
    let rendered = render_schema_module(&schema);
    assert!(rendered.contains("Rarity"));
    assert!(rendered.contains("id = 42"));
    assert!(rendered.contains("common = \"Common\""));

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
fn project_identity_preview_preserves_existing_format_when_unchanged() {
    let base = temp_dir();
    let project = write_project(
        &base,
        r#"
[enums]
Rarity = ["Common"]
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
[structs.Early]

[unions.Choice]

[unions.Choice.variants.A]

[structs.Late]

[structs.Late.fields.value]
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
    let early = content.find("[structs.Early]").unwrap();
    let union = content.find("[unions.Choice]").unwrap();
    let late = content.find("[structs.Late]").unwrap();

    assert!(early < late);
    assert!(late < union);
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
project = { id = "game_config" }
groups = { common = { default = true } }
views = { default = { contract = "game_config/default", groups = ["common"] } }
includes = ["schema/items.toml", "schema/quests.toml"]
"#,
    )
    .unwrap();
    fs::write(
        schema_dir.join("items.toml"),
        r#"
[tables.Item]
id = "item"
mode = "map"
key = "id"

[tables.Item.fields]
id = "i32"
"#,
    )
    .unwrap();
    fs::write(
        schema_dir.join("quests.toml"),
        r#"
[tables.Quest]
id = "quest"
mode = "map"
key = "id"

[tables.Quest.fields]
id = "i32"
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
            enum_value_id: None,
            groups: Vec::new(),
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
    assert!(!items.contains("[tables.Quest.fields.name]"));
    assert!(quests.contains("[tables.Quest.fields.name]"));
    assert!(quests.contains("type = \"string\""));

    let _ = fs::remove_dir_all(base);
}

#[test]
fn save_rejects_project_declarations_that_studio_cannot_persist() {
    let base = temp_dir();
    let project = write_project(
        &base,
        r#"
[tables.Item]
id = "item"
mode = "list"
"#,
    );
    let original = fs::read_to_string(&project).unwrap();

    let mut schema = load_studio_schema(&project).schema.unwrap();
    schema.groups.insert("server".to_owned(), false);
    let error = write_studio_schema(&project, &schema).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("does not edit project group declarations")
    );
    assert_eq!(fs::read_to_string(&project).unwrap(), original);

    let mut schema = load_studio_schema(&project).schema.unwrap();
    schema.views.remove("default");
    let error = write_studio_schema(&project, &schema).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("does not edit project view declarations")
    );
    assert_eq!(fs::read_to_string(&project).unwrap(), original);

    let _ = fs::remove_dir_all(base);
}

#[test]
fn save_creates_new_schema_include() {
    let base = temp_dir();
    let project = write_project(
        &base,
        r#"
[enums]
Rarity = ["Common"]
"#,
    );
    let mut schema = load_studio_schema(&project).schema.unwrap();
    schema.sources.push("schema/new_items.toml".to_owned());

    write_studio_schema(&project, &schema).unwrap();

    let project_text = fs::read_to_string(&project).unwrap();
    let new_schema = fs::read_to_string(base.join("schema/new_items.toml")).unwrap();
    let project_value: toml::Table = project_text.parse().unwrap();
    assert_eq!(
        project_value
            .get("includes")
            .and_then(toml::Value::as_array)
            .unwrap()
            .iter()
            .map(|value| value.as_str().unwrap())
            .collect::<Vec<_>>(),
        ["schema/items.toml", "schema/new_items.toml"]
    );
    assert_eq!(new_schema, "");

    let _ = fs::remove_dir_all(base);
}

#[test]
fn save_rejects_removed_schema_include_that_still_owns_nodes() {
    let base = temp_dir();
    let project = write_project(
        &base,
        r#"
[enums]
Rarity = ["Common"]
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
fn save_edits_mixed_schema_include_formats() {
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
  - schema/items.toml
  - schema/quests.yaml
  - schema/rewards.json
  - schema/events.lua
build:
  data_root: data
"#,
    )
    .unwrap();
    fs::write(
        schema_dir.join("items.toml"),
        r#"
[tables.Item]
id = "item"
mode = "map"
key = "id"

[tables.Item.fields]
id = "i32"
"#,
    )
    .unwrap();
    fs::write(
        schema_dir.join("quests.yaml"),
        r#"
tables:
  Quest:
    id: quest
    mode: map
    key: id
    fields:
      id: i32
"#,
    )
    .unwrap();
    fs::write(
        schema_dir.join("rewards.json"),
        r#"
{
  "enums": { "Rarity": ["Common"] }
}
"#,
    )
    .unwrap();
    fs::write(
        schema_dir.join("events.lua"),
        r#"
return {
  structs = {
    EventPayload = { fields = { { value = "string" } } },
  },
}
"#,
    )
    .unwrap();

    let mut schema = load_studio_schema(&project).schema.unwrap();
    assert_eq!(
        schema.sources,
        [
            "schema/items.toml",
            "schema/quests.yaml",
            "schema/rewards.json",
            "schema/events.lua"
        ]
    );
    let quest = schema
        .nodes
        .iter_mut()
        .find(|node| node.id == "table:Quest")
        .unwrap();
    assert_eq!(quest.source, "schema/quests.yaml");
    quest.fields.push(StudioField {
        name: "title".to_owned(),
        ty: "string".to_owned(),
        enum_value_id: None,
        groups: Vec::new(),
        parser: None,
        comment: Some("Quest title".to_owned()),
        default: None,
        range: None,
        length: None,
        source: None,
    });
    schema.sources.push("schema/new.lua".to_owned());

    write_studio_schema(&project, &schema).unwrap();

    let project_text = fs::read_to_string(&project).unwrap();
    let quests = fs::read_to_string(schema_dir.join("quests.yaml")).unwrap();
    let rewards = fs::read_to_string(schema_dir.join("rewards.json")).unwrap();
    let events = fs::read_to_string(schema_dir.join("events.lua")).unwrap();
    let new_lua = fs::read_to_string(schema_dir.join("new.lua")).unwrap();
    assert!(project_text.contains("- schema/new.lua"));
    assert!(project_text.contains("data_root: data"));
    assert!(quests.contains("Quest title"));
    assert!(rewards.contains("\"enums\""));
    assert!(events.starts_with("return "));
    assert_eq!(new_lua, "return {\n}\n");
    assert!(load_studio_schema(&project).ok);

    let _ = fs::remove_dir_all(base);
}

#[test]
fn save_updates_project_identity() {
    let base = temp_dir();
    let project = write_project(
        &base,
        r#"
[enums]
Rarity = ["Common"]
"#,
    );
    let mut schema = load_studio_schema(&project).schema.unwrap();
    schema.project_id = "edited_config".to_owned();

    write_studio_schema(&project, &schema).unwrap();

    let project_text = fs::read_to_string(&project).unwrap();
    assert!(project_text.contains("id = \"edited_config\""));

    let _ = fs::remove_dir_all(base);
}

#[test]
fn save_updates_project_identity_with_multiline_includes() {
    let base = temp_dir();
    let schema_dir = base.join("schema");
    fs::create_dir_all(&schema_dir).unwrap();
    let project = base.join("project.toml");
    fs::write(
        &project,
        r#"
project = { id = "game_config" }
groups = { common = { default = true } }
views = { default = { contract = "game_config/default", groups = ["common"] } }
includes = [
    "schema/items.toml",
]

[build]
data_root = "data"
"#,
    )
    .unwrap();
    fs::write(
        schema_dir.join("items.toml"),
        r#"
[enums]
Rarity = ["Common"]
"#,
    )
    .unwrap();
    let mut schema = load_studio_schema(&project).schema.unwrap();
    schema.project_id = "edited_config".to_owned();

    let response = save_studio_schema(&project, &schema);
    let project_text = fs::read_to_string(&project).unwrap();

    assert!(response.ok, "{:?}", response.diagnostics);
    assert!(project_text.contains("id = \"edited_config\""));
    assert!(project_text.contains("[build]"));
    assert!(load_studio_schema(&project).ok);

    let _ = fs::remove_dir_all(base);
}

#[test]
fn showcase_project_roundtrips_through_studio_save() {
    let base = temp_dir();
    let project = copy_showcase_project(&base);
    let mut schema = load_studio_schema(&project).schema.unwrap();
    let node_count = schema.nodes.len();

    schema.project_id = "com.sora.showcase.edited".to_owned();
    let response = save_studio_schema(&project, &schema);
    let reloaded = load_studio_schema(&project);

    assert!(response.ok, "{:?}", response.diagnostics);
    assert!(reloaded.ok, "{:?}", reloaded.diagnostics);
    assert_eq!(reloaded.schema.unwrap().nodes.len(), node_count);
    assert!(
        fs::read_to_string(&project)
            .unwrap()
            .contains("id = \"com.sora.showcase.edited\"")
    );

    let _ = fs::remove_dir_all(base);
}

#[test]
fn showcase_views_use_the_shared_schema_projection() {
    let base = temp_dir();
    let project = copy_showcase_project(&base);

    let client = load_studio_schema_view(&project, "client");
    let server = load_studio_schema_view(&project, "server");

    assert!(client.ok, "{:?}", client.diagnostics);
    assert!(server.ok, "{:?}", server.diagnostics);
    let client = client.schema.unwrap();
    let server = server.schema.unwrap();
    assert_eq!(client.summary.tables, 33);
    assert_eq!(server.summary.tables, 34);
    assert!(
        !client
            .nodes
            .iter()
            .any(|node| node.name == "MaintenanceWindow")
    );
    assert!(
        server
            .nodes
            .iter()
            .any(|node| node.name == "MaintenanceWindow")
    );

    let _ = fs::remove_dir_all(base);
}

#[test]
fn transactional_text_write_keeps_existing_files_on_prepare_failure() {
    let base = temp_dir();
    fs::create_dir_all(&base).unwrap();
    let existing = base.join("existing.toml");
    let blocked = base.join("blocked");
    fs::write(&existing, "original").unwrap();
    fs::write(&blocked, "not a directory").unwrap();

    let error = commit_text_transaction(
        &base,
        &[
            TextFileWrite {
                path: existing.clone(),
                content: "changed".to_owned(),
            },
            TextFileWrite {
                path: blocked.join("new.toml"),
                content: "new".to_owned(),
            },
        ],
        || Ok(()),
    )
    .unwrap_err();

    assert!(!error.to_string().is_empty());
    assert_eq!(fs::read_to_string(&existing).unwrap(), "original");
    assert!(
        fs::read_dir(&base)
            .unwrap()
            .filter_map(Result::ok)
            .all(|entry| !entry.file_name().to_string_lossy().contains("sora-studio"))
    );

    let _ = fs::remove_dir_all(base);
}

fn write_project(base: &Path, schema_text: &str) -> PathBuf {
    let schema_dir = base.join("schema");
    fs::create_dir_all(&schema_dir).unwrap();
    let project = base.join("project.toml");
    fs::write(
        &project,
        r#"
project = { id = "game_config" }
groups = { common = { default = true } }
views = { default = { contract = "game_config/default", groups = ["common"] } }
includes = ["schema/items.toml"]
"#,
    )
    .unwrap();
    fs::write(schema_dir.join("items.toml"), schema_text).unwrap();
    project
}

fn copy_showcase_project(base: &Path) -> PathBuf {
    let showcase = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("examples/showcase");
    let schema_dir = base.join("schema");
    let views_dir = base.join("views");
    fs::create_dir_all(&schema_dir).unwrap();
    fs::create_dir_all(&views_dir).unwrap();
    fs::copy(showcase.join("project.scon"), base.join("project.scon")).unwrap();
    for file in [
        "core.scon",
        "items.scon",
        "combat.scon",
        "quests.scon",
        "system.scon",
        "events.scon",
    ] {
        fs::copy(showcase.join("schema").join(file), schema_dir.join(file)).unwrap();
    }
    for file in ["full.toml", "client.toml", "server.toml"] {
        fs::copy(showcase.join("views").join(file), views_dir.join(file)).unwrap();
    }
    base.join("project.scon")
}

fn temp_dir() -> PathBuf {
    let id = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("sora-studio-test-{}-{id}", std::process::id()))
}
