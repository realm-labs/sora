# Schema 格式

Sora schema 文件可以写成 TOML、YAML、JSON 或 Lua。这些格式都会加载到同一个 schema model，后续生成的 IR、代码、Excel 模板、导出文件和 schema lock 都一致。

文件扩展名决定 parser：

| 扩展名 | 格式 |
| --- | --- |
| `.toml` | TOML |
| `.yaml`、`.yml` | YAML |
| `.json` | JSON |
| `.lua` | Lua |

include 文件按自己的扩展名解析，所以 YAML 项目可以 include TOML、JSON 或 Lua module，任意受支持的项目格式都可以混用受支持的 module 格式。

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

Lua schema 文件必须 `return` 一个 table。这个 table 使用和 TOML/YAML/JSON 形状一致的字段名。Lua schema loader 面向数据配置；`package`、`io`、`os` 和 `debug` 不可用。

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

## 项目 Build 配置

项目文件里的 `build` 也可以使用 YAML、JSON 或 Lua：

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
