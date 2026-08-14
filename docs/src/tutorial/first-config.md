# First Config

This tutorial creates a small item configuration table. The same pattern scales to larger game data: define the schema, generate an editable workbook, fill rows, export a runtime bundle, and generate code.

## Project Layout

```text
project.scon
schema/items.scon
data/Item.xlsx
generated/
```

## Project Manifest

```scon
project { id = "game_config" }
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
  codegen = [{ target = "rust", out = "generated/rust", format = "auto" }]
  exports = [{ format = "binary", out = "generated/config.sora" }]
}
```

`schema_lock` captures the normalized schema, `excel_templates` writes workbooks with generated headers, `build.codegen` declares language output, and `build.exports` declares runtime data output.

## Schema

```scon
enums { ItemType = ["Weapon", "Armor", "Material", "Consumable"] }

tables {
  Item {
    mode = "map"
    key = "id"
    source { format = "xlsx" file = "Item.xlsx" sheet = "Item" }
    fields {
      id { type = "i32" comment = "Item id" }
      name { type = "string" comment = "Display name" }
      item_type { type = "enum<ItemType>" comment = "Item category" }
      max_stack { type = "i32" default = "1" range = [1, 9999] comment = "Stack limit" }
    }
  }
}
```

This table uses `mode = "map"`, so the generated runtime exposes keyed lookup by `id`.

## Excel Template

Generate a workbook:

```bash
sora excel-template --project project.scon --out generated/excel
```

The generated sheet has metadata rows above the editable data area:

| #field | id | name | item_type | max_stack |
| --- | --- | --- | --- | --- |
| #type | i32 | string | `enum<ItemType>` | i32 |
| #input | key |  |  | range=1..9999 |
| #desc | Item id | Display name | Item category | Stack limit |

Rows start after the generated header:

| id | name | item_type | max_stack |
| --- | --- | --- | --- |
| 1001 | Iron Sword | Weapon | 1 |
| 2001 | Health Potion | Consumable | 99 |

Copy the workbook to `data/Item.xlsx` after generating it, or point your source file at the generated location during experiments.

## Build

Run the configured outputs:

```bash
sora build --project project.scon
```

Expected artifacts:

- `generated/schema.lock`
- `generated/excel/Item.xlsx`
- `generated/rust/Cargo.toml`
- `generated/rust/src/lib.rs`
- `generated/config.sora`

Use `sora check --project project.scon` when you only want schema validation.
