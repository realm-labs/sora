# Quick Start

This guide builds a minimal item table, generates an Excel template, exports a runtime bundle, and generates Rust code that can load it.

Install the CLI from source during development:

```bash
cargo install --path crates/sora-cli
```

## 1. Create a Project

The fastest path is to scaffold the same minimal project:

```bash
sora init --out my-config --schema-format toml
cd my-config
```

`--schema-format` accepts `toml`, `yaml`, `json`, or `lua`. The scaffold creates this layout:

| Path | Who edits it | Purpose |
| --- | --- | --- |
| `project.toml` | You | Project entry point, build outputs, default data location. |
| `schema/items.toml` | You | Schema for the `Item` table. |
| `data/Item.xlsx` | Designers or tools | Editable row data. |
| `generated/` | Sora | Schema lock, Excel templates, generated code, exported data. |

The rest of this section shows the generated files so you can understand the project shape. `project.toml` looks like this:

```toml
package = "game_config"
includes = ["schema/items.toml"]

[build]
default_source_format = "xlsx"
data_root = "data"
schema_lock = "generated/schema.lock"
excel_templates = "generated/excel"

[[build.codegen]]
target = "rust"
out = "generated/rust"
format = "auto"

[[build.exports]]
format = "binary"
out = "generated/config.sora"
```

In this file, `default_source_format = "xlsx"` means table sources default to Excel. `data_root = "data"` means `Item.xlsx` is read from `data/Item.xlsx` during export and build. The `binary` export writes the runtime bundle that Rust code will load because Rust defaults to `runtime_format = "sora"`.

Create `schema/items.toml`:

```toml
[[enums]]
name = "ItemType"
values = ["Weapon", "Armor", "Material", "Consumable"]

[[tables]]
name = "Item"
mode = "map"
key = "id"

[tables.source]
format = "xlsx"
file = "Item.xlsx"
sheet = "Item"

[[tables.fields]]
name = "id"
type = "i32"
comment = "Item id"

[[tables.fields]]
name = "name"
type = "string"
comment = "Display name"

[[tables.fields]]
name = "item_type"
type = "enum<ItemType>"
comment = "Item category"

[[tables.fields]]
name = "max_stack"
type = "i32"
default = "1"
range = [1, 9999]
comment = "Stack limit"
```

## 2. Generate the Excel Template

The workbook header is generated from the schema:

```bash
sora excel-template --project project.toml --out generated/excel
```

This creates `generated/excel/Item.xlsx`. Copy it to `data/Item.xlsx` and fill rows below the generated header:

| id | name | item_type | max_stack |
| --- | --- | --- | --- |
| 1001 | Iron Sword | Weapon | 1 |
| 2001 | Health Potion | Consumable | 99 |

## 3. Check, Export, and Generate

Validate the schema without reading row data:

```bash
sora check --project project.toml
```

Run every output declared in `[build]`. This also loads and validates source data before writing exports:

```bash
sora build --project project.toml
```

Or run the steps separately:

```bash
sora gen --target rust --project project.toml --out generated/rust

sora export \
  --format binary \
  --default-source-format xlsx \
  --project project.toml \
  --data-root data \
  --out generated/config.sora
```

## 4. Next Steps

Read [First Config](tutorial/first-config.md) for the same example with the generated runtime usage, or inspect `examples/showcase/project.toml` for a larger multi-language setup.
