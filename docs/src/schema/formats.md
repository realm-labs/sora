# Schema Formats

Sora supports SCON, TOML, YAML, JSON, and Lua as equal schema frontends. SCON is recommended for new projects. Every frontend lowers to the same schema model, so validation, code generation, Excel templates, exports, and schema locks have the same semantics.

| Extension | Format | Notes |
| --- | --- | --- |
| `.scon` | SCON | Recommended; concise object blocks. |
| `.toml` | TOML | Uses nested named tables. |
| `.yaml`, `.yml` | YAML | Uses keyed mappings. |
| `.json` | JSON | Uses keyed objects and does not allow source comments. |
| `.lua` | Lua | Returns a data-only table; ordered members use singleton entries. |

The file extension selects the frontend. `includes` are resolved relative to the file that declares them, loaded in declaration order, and may freely mix all five formats.

## Project root and included modules

The project root must contain `project` and declare at least one group and one view. It may also contain `codegen`, `localization`, and build configuration. An included module must not contain those root-only properties; it normally contains `namespace`, `imports`, declarations, and further `includes`.

This is the same minimal project root in every format. Choose the block matching the extension of your project file and change `items` to the extension of the module you want to use.

### SCON project root

```scon
project { id = "game_config" }
groups { common { default = true } }
views { default { contract = "game_config/default", groups = ["common"] } }
includes = ["schema/common.scon", "schema/items.scon"]
```

### TOML project root

```toml
project = { id = "game_config" }
groups = { common = { default = true } }
views = { default = { contract = "game_config/default", groups = ["common"] } }
includes = ["schema/common.scon", "schema/items.toml"]
```

### YAML project root

```yaml
project: { id: game_config }
groups:
  common: { default: true }
views:
  default: { contract: game_config/default, groups: [common] }
includes: [schema/common.scon, schema/items.yaml]
```

### JSON project root

```json
{
  "project": { "id": "game_config" },
  "groups": { "common": { "default": true } },
  "views": {
    "default": { "contract": "game_config/default", "groups": ["common"] }
  },
  "includes": ["schema/common.scon", "schema/items.json"]
}
```

### Lua project root

```lua
return {
  project = { id = "game_config" },
  groups = { common = { default = true } },
  views = {
    default = { contract = "game_config/default", groups = { "common" } },
  },
  includes = { "schema/common.scon", "schema/items.lua" },
}
```

The examples below use this shared module so that the `common` import resolves to a real declaration:

```scon
# schema/common.scon
namespace = "game.common"

structs {
  Reward {
    fields { code = "string" }
  }
}
```

## Equivalent complete modules

Each of the following `items` modules declares exactly the same schema. They demonstrate namespaces and imports, enum and enum-value comments, aliases, structs, unions, references, constraints, parsers, sources, and indexes.

Declaration names are mapping keys; do not repeat a `name` property inside a declaration body. The `comment` property is schema metadata used in schema locks and generated documentation. It is supported on enums, enum values, and fields.

The enum alias `SSR = "Epic"` means source data may use `SSR`, which Sora normalizes to the canonical enum value `Epic`. It does not create an additional generated enum member.

### SCON

```scon
namespace = "game.items"
imports { common = "game.common" }

enums {
  Rarity {
    comment = "Item rarity"
    values = [
      { id = 0, name = "Common", comment = "Common item" },
      { id = 1, name = "Epic", comment = "Rare item" },
    ]
    aliases { SSR = "Epic" }
  }
}

structs {
  StatBonus {
    fields {
      attack {
        type = "i32"
        comment = "Flat attack bonus"
        range = [0, 9999]
      }
    }
  }
}

unions {
  Grant {
    tag = "kind"
    variants {
      Item {
        fields {
          item_id = "ref<Item.id>"
          count { type = "u16", default = "1" }
        }
      }
      Currency {
        fields {
          amount { type = "i32", range = [1, 999999] }
        }
      }
    }
  }
}

tables {
  Item {
    mode = "map"
    key = "id"
    source { format = "xlsx", file = "Items.xlsx", sheet = "Item" }
    fields {
      id { type = "u32", comment = "Stable item id", range = [1, 999999] }
      name { type = "string", length = [1, 64] }
      rarity = "enum<Rarity>"
      bonus { type = "optional<struct<StatBonus>>", parser = "json" }
      reward { type = "optional<struct<common.Reward>>", parser = "json" }
      tags {
        type = "list<string>"
        parser = { kind = "split", separator = "|" }
      }
    }
    indexes {
      by_name { fields = ["name"], unique = true }
      by_rarity = ["rarity"]
    }
  }
}
```

### TOML

```toml
namespace = "game.items"
imports = { common = "game.common" }

[enums.Rarity]
comment = "Item rarity"
values = [
  { id = 0, name = "Common", comment = "Common item" },
  { id = 1, name = "Epic", comment = "Rare item" },
]
aliases = { SSR = "Epic" }

[structs.StatBonus.fields]
attack = { type = "i32", comment = "Flat attack bonus", range = [0, 9999] }

[unions.Grant]
tag = "kind"

[unions.Grant.variants.Item.fields]
item_id = "ref<Item.id>"
count = { type = "u16", default = "1" }

[unions.Grant.variants.Currency.fields]
amount = { type = "i32", range = [1, 999999] }

[tables.Item]
mode = "map"
key = "id"

[tables.Item.source]
format = "xlsx"
file = "Items.xlsx"
sheet = "Item"

[tables.Item.fields]
id = { type = "u32", comment = "Stable item id", range = [1, 999999] }
name = { type = "string", length = [1, 64] }
rarity = "enum<Rarity>"
bonus = { type = "optional<struct<StatBonus>>", parser = "json" }
reward = { type = "optional<struct<common.Reward>>", parser = "json" }
tags = { type = "list<string>", parser = { kind = "split", separator = "|" } }

[tables.Item.indexes]
by_name = { fields = ["name"], unique = true }
by_rarity = ["rarity"]
```

### YAML

```yaml
namespace: game.items
imports:
  common: game.common

enums:
  Rarity:
    comment: Item rarity
    values:
      - { id: 0, name: Common, comment: Common item }
      - { id: 1, name: Epic, comment: Rare item }
    aliases:
      SSR: Epic

structs:
  StatBonus:
    fields:
      attack:
        type: i32
        comment: Flat attack bonus
        range: [0, 9999]

unions:
  Grant:
    tag: kind
    variants:
      Item:
        fields:
          item_id: "ref<Item.id>"
          count: { type: u16, default: "1" }
      Currency:
        fields:
          amount: { type: i32, range: [1, 999999] }

tables:
  Item:
    mode: map
    key: id
    source: { format: xlsx, file: Items.xlsx, sheet: Item }
    fields:
      id: { type: u32, comment: Stable item id, range: [1, 999999] }
      name: { type: string, length: [1, 64] }
      rarity: "enum<Rarity>"
      bonus: { type: "optional<struct<StatBonus>>", parser: json }
      reward: { type: "optional<struct<common.Reward>>", parser: json }
      tags:
        type: "list<string>"
        parser: { kind: split, separator: "|" }
    indexes:
      by_name: { fields: [name], unique: true }
      by_rarity: [rarity]
```

### JSON

```json
{
  "namespace": "game.items",
  "imports": { "common": "game.common" },
  "enums": {
    "Rarity": {
      "comment": "Item rarity",
      "values": [
        { "id": 0, "name": "Common", "comment": "Common item" },
        { "id": 1, "name": "Epic", "comment": "Rare item" }
      ],
      "aliases": { "SSR": "Epic" }
    }
  },
  "structs": {
    "StatBonus": {
      "fields": {
        "attack": {
          "type": "i32",
          "comment": "Flat attack bonus",
          "range": [0, 9999]
        }
      }
    }
  },
  "unions": {
    "Grant": {
      "tag": "kind",
      "variants": {
        "Item": {
          "fields": {
            "item_id": "ref<Item.id>",
            "count": { "type": "u16", "default": "1" }
          }
        },
        "Currency": {
          "fields": {
            "amount": { "type": "i32", "range": [1, 999999] }
          }
        }
      }
    }
  },
  "tables": {
    "Item": {
      "mode": "map",
      "key": "id",
      "source": { "format": "xlsx", "file": "Items.xlsx", "sheet": "Item" },
      "fields": {
        "id": {
          "type": "u32",
          "comment": "Stable item id",
          "range": [1, 999999]
        },
        "name": { "type": "string", "length": [1, 64] },
        "rarity": "enum<Rarity>",
        "bonus": { "type": "optional<struct<StatBonus>>", "parser": "json" },
        "reward": {
          "type": "optional<struct<common.Reward>>",
          "parser": "json"
        },
        "tags": {
          "type": "list<string>",
          "parser": { "kind": "split", "separator": "|" }
        }
      },
      "indexes": {
        "by_name": { "fields": ["name"], "unique": true },
        "by_rarity": ["rarity"]
      }
    }
  }
}
```

### Lua

Lua schema files must return one data-only table. Filesystem, process, package, and debug libraries are unavailable. Enum values, fields, and union variants have semantic order, so Lua represents ordered named collections as lists of singleton tables.

```lua
return {
  namespace = "game.items",
  imports = { common = "game.common" },
  enums = {
    Rarity = {
      comment = "Item rarity",
      values = {
        { id = 0, name = "Common", comment = "Common item" },
        { id = 1, name = "Epic", comment = "Rare item" },
      },
      aliases = { SSR = "Epic" },
    },
  },
  structs = {
    StatBonus = {
      fields = {
        {
          attack = {
            type = "i32",
            comment = "Flat attack bonus",
            range = { 0, 9999 },
          },
        },
      },
    },
  },
  unions = {
    Grant = {
      tag = "kind",
      variants = {
        {
          Item = {
            fields = {
              { item_id = "ref<Item.id>" },
              { count = { type = "u16", default = "1" } },
            },
          },
        },
        {
          Currency = {
            fields = {
              { amount = { type = "i32", range = { 1, 999999 } } },
            },
          },
        },
      },
    },
  },
  tables = {
    Item = {
      mode = "map",
      key = "id",
      source = { format = "xlsx", file = "Items.xlsx", sheet = "Item" },
      fields = {
        {
          id = {
            type = "u32",
            comment = "Stable item id",
            range = { 1, 999999 },
          },
        },
        { name = { type = "string", length = { 1, 64 } } },
        { rarity = "enum<Rarity>" },
        { bonus = { type = "optional<struct<StatBonus>>", parser = "json" } },
        {
          reward = {
            type = "optional<struct<common.Reward>>",
            parser = "json",
          },
        },
        {
          tags = {
            type = "list<string>",
            parser = { kind = "split", separator = "|" },
          },
        },
      },
      indexes = {
        by_name = { fields = { "name" }, unique = true },
        by_rarity = { "rarity" },
      },
    },
  },
}
```

## Name resolution

In the examples, local declaration `Item` has the canonical name `game.items.Item`.

- `enum<Rarity>` and `ref<Item.id>` resolve in the current `game.items` namespace.
- `struct<common.Reward>` expands the `common` import to `game.common.Reward`.
- A dotted name that does not start with an import alias is treated as an absolute project-qualified name.
- In `ref<game.items.Item.id>`, the final segment is the field and everything before it is the table name.

Namespace segments, import aliases, and declaration names must be ASCII identifiers. The empty namespace is the project root namespace.

## Shared keyed model and ordering

`enums`, `structs`, `unions`, `tables`, `fields`, `variants`, `indexes`, and `localization.sources` use declaration names as mapping keys. Unknown properties and the former array-of-named-objects representation are rejected.

Enum values, fields, and union variants preserve author order. Other named collections are normalized by name for deterministic output. Lua must use singleton-table lists for ordered named collections, as shown above; ordinary Lua table key iteration does not preserve declaration order.

Use Sora's `includes` array for composition. Native SCON `include`, substitutions, interpolation, and object or array spread are deliberately rejected so all formats retain identical semantics.

## Shorthands

- An enum can be a string array. Use the detailed `{ values, groups, aliases, comment }` form when needed. Alias mappings are `external alias -> canonical value`.
- A field can be a type string. Use the detailed form when it has `groups`, `comment`, `default`, `range`, `length`, `parser`, or table-only `from`.
- A parser without options can be a string such as `parser = "json"`. With options it is an object containing `kind` and string-valued options.
- A non-unique index can be a field array. Use `{ fields, unique = true }` for a unique index.
- `groups` accepts either one group string or a list of group strings.

For the complete property semantics, see [Enums, Structs, and Unions](enums-structs-unions.md), [Tables](tables.md), [Types](types.md), [Cell Parsers](parsers.md), [References and Derived Fields](references.md), [Localization](../localization.md), and [Project Configuration](../project-config.md).
