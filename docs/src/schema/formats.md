# Schema Formats

Sora schema files can be written as TOML, YAML, JSON, or Lua. All formats load into the same schema model and produce the same IR, generated code, Excel templates, exports, and schema locks.

The file extension selects the parser:

| Extension | Format |
| --- | --- |
| `.toml` | TOML |
| `.yaml`, `.yml` | YAML |
| `.json` | JSON |
| `.lua` | Lua |

Includes are parsed by their own file extension, so a YAML project can include TOML, JSON, or Lua modules, and any supported project format can mix supported module formats.

## TOML

```toml
package = "game_config"
includes = ["schema/items.toml"]

[[enums]]
name = "ItemType"
values = ["Weapon", "Armor"]

[[tables]]
name = "Item"
mode = "map"
key = "id"

[[tables.fields]]
name = "id"
type = "i32"
```

## YAML

```yaml
package: game_config
includes:
  - schema/items.yaml

enums:
  - name: ItemType
    values: [Weapon, Armor]

tables:
  - name: Item
    mode: map
    key: id
    fields:
      - name: id
        type: i32
```

## JSON

```json
{
  "package": "game_config",
  "includes": ["schema/items.json"],
  "enums": [
    { "name": "ItemType", "values": ["Weapon", "Armor"] }
  ],
  "tables": [
    {
      "name": "Item",
      "mode": "map",
      "key": "id",
      "fields": [
        { "name": "id", "type": "i32" }
      ]
    }
  ]
}
```

## Lua

Lua schema files must return one table. The returned table uses the same field names as the TOML/YAML/JSON shapes. Lua schema loading is data-oriented; `package`, `io`, `os`, and `debug` are not available.

```lua
return {
  package = "game_config",
  includes = { "schema/items.lua" },

  enums = {
    { name = "ItemType", values = { "Weapon", "Armor" } },
  },

  tables = {
    {
      name = "Item",
      mode = "map",
      key = "id",
      fields = {
        { name = "id", type = "i32" },
      },
    },
  },
}
```

## Project Build Config

The project file can also use YAML, JSON, or Lua for `build`:

```yaml
package: game_config
includes:
  - schema/items.yaml

build:
  default_source_format: xlsx
  data_root: data
  schema_lock: generated/schema.lock
  excel_templates: generated/excel
  codegen:
    - target: rust
      out: generated/rust
      format: auto
  exports:
    - format: binary
      out: generated/config.sora
```

```json
{
  "package": "game_config",
  "includes": ["schema/items.json"],
  "build": {
    "default_source_format": "xlsx",
    "data_root": "data",
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
```

```lua
return {
  package = "game_config",
  includes = { "schema/items.lua" },
  build = {
    default_source_format = "xlsx",
    data_root = "data",
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
```
