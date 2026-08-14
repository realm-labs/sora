# Schema Formats

Sora supports SCON, TOML, YAML, JSON, and Lua as equal schema frontends. SCON is the recommended format for new projects. Every frontend lowers to the same `SchemaModule` and `ProjectSchema`, so validation, IR, code generation, Excel templates, exports, and schema locks have the same semantics.

| Extension | Format | Notes |
| --- | --- | --- |
| `.scon` | SCON | Recommended; concise object blocks. |
| `.toml` | TOML | Uses nested named tables. |
| `.yaml`, `.yml` | YAML | Uses keyed mappings. |
| `.json` | JSON | Uses keyed objects. |
| `.lua` | Lua | Returns a data-only table; ordered members use singleton entries. |

The extension selects the frontend. `includes` are loaded by Sora in declaration order and may freely mix all five formats.

## Shared keyed model

Names are always mapping keys. A declaration body must not repeat a `name` property. `enums`, `structs`, `unions`, `tables`, `fields`, `variants`, `indexes`, and `localization.sources` all follow this rule. Unknown properties and the former array-of-named-objects representation are rejected.

SCON supports the most direct form:

```scon
project { id = "game_config" }
includes = ["schema/items.scon"]

enums {
  ItemType = ["Weapon", "Armor"]
}

tables {
  Item {
    mode = "map"
    key = "id"
    fields {
      id = "i32"
      name {
        type = "string"
        length = [2, 32]
      }
    }
    indexes {
      by_name {
        fields = ["name"]
        unique = true
      }
    }
  }
}
```

Use Sora's `includes` array for composition. Native SCON `include`, substitutions, interpolation, and object or array spread are deliberately rejected with source locations so all formats retain identical semantics.

## TOML

```toml
project = { id = "game_config" }
includes = ["schema/items.toml"]

[enums]
ItemType = ["Weapon", "Armor"]

[tables.Item]
mode = "map"
key = "id"

[tables.Item.fields]
id = "i32"

[tables.Item.fields.name]
type = "string"
length = [2, 32]

[tables.Item.indexes.by_name]
fields = ["name"]
unique = true
```

## YAML

```yaml
project: { id: game_config }
includes: [schema/items.yaml]
enums:
  ItemType: [Weapon, Armor]
tables:
  Item:
    mode: map
    key: id
    fields:
      id: i32
      name:
        type: string
        length: [2, 32]
    indexes:
      by_name:
        fields: [name]
        unique: true
```

## JSON

```json
{
  "project": { "id": "game_config" },
  "includes": ["schema/items.json"],
  "enums": { "ItemType": ["Weapon", "Armor"] },
  "tables": {
    "Item": {
      "mode": "map",
      "key": "id",
      "fields": {
        "id": "i32",
        "name": { "type": "string", "length": [2, 32] }
      },
      "indexes": {
        "by_name": { "fields": ["name"], "unique": true }
      }
    }
  }
}
```

## Lua

Lua files must return one data table. Libraries with filesystem, process, or debug access are unavailable. Fields and union variants have semantic order, so Lua represents them as an ordered list of singleton keyed tables.

```lua
return {
  project = { id = "game_config" },
  includes = { "schema/items.lua" },
  enums = {
    ItemType = { "Weapon", "Armor" },
  },
  tables = {
    Item = {
      mode = "map",
      key = "id",
      fields = {
        { id = "i32" },
        { name = { type = "string", length = { 2, 32 } } },
      },
      indexes = {
        by_name = { fields = { "name" }, unique = true },
      },
    },
  },
}
```

Enum values, fields, and union variants preserve author order. Other named collections are normalized by name for deterministic output across frontends.

## Shorthands

- An enum can be a value array. Expand it to `{ values, groups, aliases }` when needed. Alias mappings are `external alias -> canonical value`.
- A field can be a type string. Expand it when it has constraints, a parser, comments, defaults, groups, or `from`.
- A parser without options is a string such as `parser = "tuple"`; with options it is an object containing `kind` and its options.
- A non-unique index can be a field array. Use an object to set `unique = true`.

Project-only `build`, `parsers`, `source_loaders`, and `type_mappings` retain their existing data model and are available in all five formats.
