use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use sora_excel::generator::ExcelTemplateGenerator;
use sora_input_schema::input::ProjectSchemaInput;

use crate::args::{InitArgs, SchemaFormatArg};

pub fn run(args: InitArgs) -> Result<()> {
    let layout = InitLayout::new(&args.out, args.schema_format);
    ensure_can_write(&layout, args.force)?;

    fs::create_dir_all(&layout.schema_dir).with_context(|| {
        format!(
            "failed to create schema directory `{}`",
            layout.schema_dir.display()
        )
    })?;
    fs::create_dir_all(&layout.data_dir).with_context(|| {
        format!(
            "failed to create data directory `{}`",
            layout.data_dir.display()
        )
    })?;
    fs::create_dir_all(&layout.generated_dir).with_context(|| {
        format!(
            "failed to create generated directory `{}`",
            layout.generated_dir.display()
        )
    })?;

    write_text_file(
        &layout.project_file,
        project_template(args.schema_format),
        args.force,
    )?;
    write_text_file(
        &layout.schema_file,
        schema_template(args.schema_format),
        args.force,
    )?;
    write_sample_data(&layout, args.force)?;
    ensure_backup_ignores(&args.out.join(".gitignore"))?;

    Ok(())
}

const BACKUP_GITIGNORE_ENTRIES: [&str; 2] = [".sora/", ".sora-backup/"];

struct InitLayout {
    schema_dir: PathBuf,
    data_dir: PathBuf,
    generated_dir: PathBuf,
    project_file: PathBuf,
    schema_file: PathBuf,
    data_file: PathBuf,
}

impl InitLayout {
    fn new(root: &Path, format: SchemaFormatArg) -> Self {
        let extension = format.extension();
        let schema_dir = root.join("schema");
        let data_dir = root.join("data");
        let generated_dir = root.join("generated");
        Self {
            project_file: root.join(format!("project.{extension}")),
            schema_file: schema_dir.join(format!("items.{extension}")),
            data_file: data_dir.join("Item.xlsx"),
            schema_dir,
            data_dir,
            generated_dir,
        }
    }

    fn files(&self) -> [&Path; 3] {
        [&self.project_file, &self.schema_file, &self.data_file]
    }
}

fn ensure_can_write(layout: &InitLayout, force: bool) -> Result<()> {
    if force {
        return Ok(());
    }

    let existing = layout
        .files()
        .into_iter()
        .find(|path| path.exists())
        .map(Path::to_path_buf);
    if let Some(path) = existing {
        bail!(
            "`{}` already exists; pass --force to overwrite scaffold files",
            path.display()
        );
    }

    Ok(())
}

fn write_text_file(path: &Path, content: &'static str, force: bool) -> Result<()> {
    if path.exists() && !force {
        bail!(
            "`{}` already exists; pass --force to overwrite scaffold files",
            path.display()
        );
    }
    fs::write(path, content).with_context(|| format!("failed to write `{}`", path.display()))
}

fn ensure_backup_ignores(path: &Path) -> Result<()> {
    let mut content = if path.exists() {
        fs::read_to_string(path).with_context(|| format!("failed to read `{}`", path.display()))?
    } else {
        String::new()
    };
    let mut changed = false;
    for entry in BACKUP_GITIGNORE_ENTRIES {
        if !content.lines().any(|line| line.trim() == entry) {
            if !content.is_empty() && !content.ends_with('\n') {
                content.push('\n');
            }
            content.push_str(entry);
            content.push('\n');
            changed = true;
        }
    }
    if changed {
        fs::write(path, content)
            .with_context(|| format!("failed to write `{}`", path.display()))?;
    }
    Ok(())
}

fn write_sample_data(layout: &InitLayout, force: bool) -> Result<()> {
    if layout.data_file.exists() && !force {
        bail!(
            "`{}` already exists; pass --force to overwrite scaffold files",
            layout.data_file.display()
        );
    }

    let input = ProjectSchemaInput::new(&layout.project_file);
    let ir = sora_core::pipeline::load_schema_ir(&input).with_context(|| {
        format!(
            "failed to load generated project `{}`",
            layout.project_file.display()
        )
    })?;
    ExcelTemplateGenerator
        .generate_with_rows(&ir, &layout.data_dir, |table| {
            if table.name == "Item" {
                vec![
                    vec![
                        "1001".to_owned(),
                        "Iron Sword".to_owned(),
                        "Weapon".to_owned(),
                        "1".to_owned(),
                    ],
                    vec![
                        "1002".to_owned(),
                        "Health Potion".to_owned(),
                        "Consumable".to_owned(),
                        "20".to_owned(),
                    ],
                ]
            } else {
                Vec::new()
            }
        })
        .with_context(|| {
            format!(
                "failed to write sample Excel data into `{}`",
                layout.data_dir.display()
            )
        })
}

impl SchemaFormatArg {
    fn extension(self) -> &'static str {
        match self {
            Self::Scon => "scon",
            Self::Toml => "toml",
            Self::Yaml => "yaml",
            Self::Json => "json",
            Self::Lua => "lua",
        }
    }
}

fn project_template(format: SchemaFormatArg) -> &'static str {
    match format {
        SchemaFormatArg::Scon => {
            r#"project { id = "game_config" }
groups { common { default = true } }
views {
  default {
    contract = "game_config/default"
    groups = ["common"]
  }
}
includes = ["schema/items.scon"]

codegen {
  rust {
    crate { name = "game-config" }
  }
}

build {
  default_source_format = "xlsx"
  data_root = "data"
  view = "default"
  schema_lock = "generated/schema.lock"
  excel_templates = "generated/excel"
  codegen = [
    { target = "rust", out = "generated/rust", format = "auto" },
  ]
  exports = [
    { format = "binary", out = "generated/config.sora" },
  ]
}
"#
        }
        SchemaFormatArg::Toml => {
            r#"project = { id = "game_config" }
groups = { common = { default = true } }
views = { default = { contract = "game_config/default", groups = ["common"] } }
includes = ["schema/items.toml"]

[codegen.rust.crate]
name = "game-config"

[build]
default_source_format = "xlsx"
data_root = "data"
view = "default"
schema_lock = "generated/schema.lock"
excel_templates = "generated/excel"

[[build.codegen]]
target = "rust"
out = "generated/rust"
format = "auto"

[[build.exports]]
format = "binary"
out = "generated/config.sora"
"#
        }
        SchemaFormatArg::Yaml => {
            r#"project: { id: game_config }
groups: { common: { default: true } }
views: { default: { contract: game_config/default, groups: [common] } }
includes:
  - schema/items.yaml
codegen:
  rust:
    crate: { name: game-config }

build:
  default_source_format: xlsx
  data_root: data
  view: default
  schema_lock: generated/schema.lock
  excel_templates: generated/excel
  codegen:
    - target: rust
      out: generated/rust
      format: auto
  exports:
    - format: binary
      out: generated/config.sora
"#
        }
        SchemaFormatArg::Json => {
            r#"{
  "project": { "id": "game_config" },
  "groups": { "common": { "default": true } },
  "views": {
    "default": { "contract": "game_config/default", "groups": ["common"] }
  },
  "includes": ["schema/items.json"],
  "codegen": {
    "rust": {
      "crate": { "name": "game-config" }
    }
  },
  "build": {
    "default_source_format": "xlsx",
    "data_root": "data",
    "view": "default",
    "schema_lock": "generated/schema.lock",
    "excel_templates": "generated/excel",
    "codegen": [
      { "target": "rust", "out": "generated/rust", "format": "auto" }
    ],
    "exports": [
      { "format": "binary", "out": "generated/config.sora" }
    ]
  }
}
"#
        }
        SchemaFormatArg::Lua => {
            r#"return {
  project = { id = "game_config" },
  groups = { common = { default = true } },
  views = {
    default = { contract = "game_config/default", groups = { "common" } },
  },
  includes = { "schema/items.lua" },
  codegen = {
    rust = {
      crate = { name = "game-config" },
    },
  },
  build = {
    default_source_format = "xlsx",
    data_root = "data",
    view = "default",
    schema_lock = "generated/schema.lock",
    excel_templates = "generated/excel",
    codegen = {
      { target = "rust", out = "generated/rust", format = "auto" },
    },
    exports = {
      { format = "binary", out = "generated/config.sora" },
    },
  },
}
"#
        }
    }
}

fn schema_template(format: SchemaFormatArg) -> &'static str {
    match format {
        SchemaFormatArg::Scon => {
            r#"enums {
  ItemType = ["Weapon", "Armor", "Material", "Consumable"]
}

tables {
  Item {
    id = "item"
    mode = "map"
    key = "id"
    source {
      file = "Item.xlsx"
      sheet = "Item"
    }
    fields {
      id {
        type = "i32"
        comment = "Item id"
      }
      name {
        type = "string"
        comment = "Display name"
      }
      item_type {
        type = "enum<ItemType>"
        comment = "Item category"
      }
      max_stack {
        type = "i32"
        default = "1"
        range = [1, 9999]
        comment = "Stack limit"
      }
    }
  }
}
"#
        }
        SchemaFormatArg::Toml => {
            r#"[enums]
ItemType = ["Weapon", "Armor", "Material", "Consumable"]

[tables.Item]
id = "item"
mode = "map"
key = "id"

[tables.Item.source]
file = "Item.xlsx"
sheet = "Item"

[tables.Item.fields.id]
type = "i32"
comment = "Item id"

[tables.Item.fields.name]
type = "string"
comment = "Display name"

[tables.Item.fields.item_type]
type = "enum<ItemType>"
comment = "Item category"

[tables.Item.fields.max_stack]
type = "i32"
default = "1"
range = [1, 9999]
comment = "Stack limit"
"#
        }
        SchemaFormatArg::Yaml => {
            r#"enums:
  ItemType: [Weapon, Armor, Material, Consumable]

tables:
  Item:
    id: item
    mode: map
    key: id
    source:
      file: Item.xlsx
      sheet: Item
    fields:
      id:
        type: i32
        comment: Item id
      name:
        type: string
        comment: Display name
      item_type:
        type: enum<ItemType>
        comment: Item category
      max_stack:
        type: i32
        default: "1"
        range: [1, 9999]
        comment: Stack limit
"#
        }
        SchemaFormatArg::Json => {
            r#"{
  "enums": {
    "ItemType": ["Weapon", "Armor", "Material", "Consumable"]
  },
  "tables": {
    "Item": {
      "id": "item",
      "mode": "map",
      "key": "id",
      "source": {
        "file": "Item.xlsx",
        "sheet": "Item"
      },
      "fields": {
        "id": { "type": "i32", "comment": "Item id" },
        "name": { "type": "string", "comment": "Display name" },
        "item_type": { "type": "enum<ItemType>", "comment": "Item category" },
        "max_stack": {
          "type": "i32",
          "default": "1",
          "range": [1, 9999],
          "comment": "Stack limit"
        }
      }
    }
  }
}
"#
        }
        SchemaFormatArg::Lua => {
            r#"return {
  enums = {
    ItemType = { "Weapon", "Armor", "Material", "Consumable" },
  },
  tables = {
    Item = {
      id = "item",
      mode = "map",
      key = "id",
      source = {
        file = "Item.xlsx",
        sheet = "Item",
      },
      fields = {
        { id = { type = "i32", comment = "Item id" } },
        { name = { type = "string", comment = "Display name" } },
        { item_type = { type = "enum<ItemType>", comment = "Item category" } },
        { max_stack = {
          type = "i32",
          default = "1",
          range = { 1, 9999 },
          comment = "Stack limit",
        } },
      },
    },
  },
}
"#
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{args::BuildArgs, build};
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn init_generates_buildable_toml_project() {
        let base = temp_dir();

        run(InitArgs {
            out: base.clone(),
            schema_format: SchemaFormatArg::Toml,
            force: false,
        })
        .unwrap();

        assert!(base.join("project.toml").exists());
        assert!(base.join("schema/items.toml").exists());
        assert!(base.join("data/Item.xlsx").exists());
        let gitignore = fs::read_to_string(base.join(".gitignore")).unwrap();
        assert!(gitignore.lines().any(|line| line == ".sora/"));
        assert!(gitignore.lines().any(|line| line == ".sora-backup/"));

        build::run(
            BuildArgs {
                project: base.join("project.toml"),
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
        assert!(base.join("generated/rust/Cargo.toml").exists());
        assert!(base.join("generated/rust/src/lib.rs").exists());
        assert!(base.join("generated/rust/src/item.rs").exists());
        assert!(base.join("generated/config.sora").exists());

        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn init_supports_all_schema_formats() {
        for format in [
            SchemaFormatArg::Scon,
            SchemaFormatArg::Toml,
            SchemaFormatArg::Yaml,
            SchemaFormatArg::Json,
            SchemaFormatArg::Lua,
        ] {
            let base = temp_dir();
            run(InitArgs {
                out: base.clone(),
                schema_format: format,
                force: false,
            })
            .unwrap();
            let project = base.join(format!("project.{}", format.extension()));
            let input = ProjectSchemaInput::new(project);
            sora_core::pipeline::check_schema(&input).unwrap();
            sora_core::pipeline::generate_code(&input, "rust", &base.join("generated/rust"))
                .unwrap();
            assert!(base.join("data/Item.xlsx").exists());
            assert!(base.join("generated/rust/Cargo.toml").exists());
            assert!(base.join("generated/rust/src/lib.rs").exists());

            let _ = fs::remove_dir_all(base);
        }
    }

    #[test]
    fn init_rejects_existing_scaffold_files_without_force() {
        let base = temp_dir();
        fs::create_dir_all(&base).unwrap();
        fs::write(base.join("project.toml"), "existing").unwrap();

        let error = run(InitArgs {
            out: base.clone(),
            schema_format: SchemaFormatArg::Toml,
            force: false,
        })
        .unwrap_err();
        assert!(error.to_string().contains("already exists"));

        run(InitArgs {
            out: base.clone(),
            schema_format: SchemaFormatArg::Toml,
            force: true,
        })
        .unwrap();
        assert!(base.join("schema/items.toml").exists());

        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn init_preserves_existing_gitignore_entries() {
        let base = temp_dir();
        fs::create_dir_all(&base).unwrap();
        fs::write(base.join(".gitignore"), "custom-output/\n").unwrap();

        run(InitArgs {
            out: base.clone(),
            schema_format: SchemaFormatArg::Toml,
            force: false,
        })
        .unwrap();

        let gitignore = fs::read_to_string(base.join(".gitignore")).unwrap();
        assert!(gitignore.lines().any(|line| line == "custom-output/"));
        assert!(gitignore.lines().any(|line| line == ".sora/"));
        assert!(gitignore.lines().any(|line| line == ".sora-backup/"));

        let _ = fs::remove_dir_all(base);
    }

    fn temp_dir() -> PathBuf {
        let unique = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("sora-init-test-{}-{unique}", std::process::id()))
    }

    fn test_context() -> sora_workspace::ProjectRuntime {
        sora_workspace::ProjectRuntime::load(None, sora_workspace::RuntimeOptions::default())
            .unwrap()
    }
}
