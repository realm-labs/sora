# Schema 格式

Sora 将 SCON、TOML、YAML、JSON、Lua 作为能力等价的一等 Schema 前端；新项目推荐使用 SCON。五种表示都会转换为相同的 Schema 模型，因此校验、代码生成、Excel 模板、导出和 schema lock 的语义一致。

| 扩展名 | 格式 | 说明 |
| --- | --- | --- |
| `.scon` | SCON | 推荐；使用简洁的 object block。 |
| `.toml` | TOML | 使用嵌套 named table。 |
| `.yaml`, `.yml` | YAML | 使用 keyed mapping。 |
| `.json` | JSON | 使用 keyed object；文件本身不支持注释。 |
| `.lua` | Lua | 返回纯数据 table；有序成员使用单键 table 列表。 |

文件扩展名决定使用哪个前端。`includes` 路径相对于声明它的文件解析，Sora 按声明顺序加载；五种格式可以自由互相 include。

## 项目根文件与 include module

项目根文件必须包含 `project`，并且至少声明一个 group 和一个 view；还可以包含 `codegen`、`localization` 和构建配置。include module 不能包含这些仅限根文件的属性；它通常包含 `namespace`、`imports`、类型与表声明，以及更多 `includes`。

下面是五种格式下等价的最小项目根文件。选择与你的项目文件扩展名对应的代码块，并把 `items` 的扩展名改成实际使用的 module 格式。

### SCON 项目根文件

```scon
project { id = "game_config" }
groups { common { default = true } }
views { default { contract = "game_config/default", groups = ["common"] } }
includes = ["schema/common.scon", "schema/items.scon"]
```

### TOML 项目根文件

```toml
project = { id = "game_config" }
groups = { common = { default = true } }
views = { default = { contract = "game_config/default", groups = ["common"] } }
includes = ["schema/common.scon", "schema/items.toml"]
```

### YAML 项目根文件

```yaml
project: { id: game_config }
groups:
  common: { default: true }
views:
  default: { contract: game_config/default, groups: [common] }
includes: [schema/common.scon, schema/items.yaml]
```

### JSON 项目根文件

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

### Lua 项目根文件

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

后面的示例共用下面这个 module，使 `common` import 能解析到真实声明：

```scon
# schema/common.scon
namespace = "game.common"

structs {
  Reward {
    fields { code = "string" }
  }
}
```

## 五种格式的等价完整 module

下面五份 `items` module 声明的是完全相同的 Schema，覆盖 namespace 与 import、enum 及枚举值注释、alias、struct、union、ref、约束、parser、source 和 index。

声明名写在 mapping key 上，body 内不要重复写 `name`。`comment` 是会进入 schema lock 和生成代码文档的 Schema 元数据；目前支持 enum、enum value 和各种 field。

enum alias `SSR = "Epic"` 表示源数据可以写 `SSR`，Sora 会将其归一化为规范枚举值 `Epic`；它不会额外生成一个 `SSR` 枚举成员。

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

Lua Schema 文件必须返回一个纯数据 table。文件系统、进程、package 和 debug 库不可用。enum value、field 和 union variant 有语义顺序，因此 Lua 要把有序命名集合写成单键 table 的列表。

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

## 名称解析

在这些示例中，本地声明 `Item` 的规范名称是 `game.items.Item`。

- `enum<Rarity>` 和 `ref<Item.id>` 在当前 `game.items` namespace 中解析。
- `struct<common.Reward>` 先通过 `common` import 展开为 `game.common.Reward`。
- 不以 import alias 开头的 dotted name 被当作项目绝对限定名。
- `ref<game.items.Item.id>` 的最后一段是字段名，前面的全部内容是表名。

namespace 段、import alias 和声明名必须是 ASCII 标识符。空 namespace 表示项目根命名空间。

## 统一 keyed 模型与顺序

`enums`、`structs`、`unions`、`tables`、`fields`、`variants`、`indexes` 和 `localization.sources` 都用声明名作为 mapping key。未知属性和旧的 named object 数组写法会被拒绝。

enum value、field 和 union variant 保留作者顺序，其他命名集合会按名称规范化，以保证输出确定。Lua 必须像上面的示例一样，用单键 table 列表表示有序命名集合；普通 Lua table 的 key 遍历顺序不能表达声明顺序。

组合统一使用 Sora 的 `includes` 数组。原生 SCON `include`、substitution、interpolation、object spread 和 array spread 会被拒绝，以确保五种格式的语义一致。

## 简写规则

- enum 可以直接写字符串数组；需要 `values`、`groups`、`aliases` 或 `comment` 时使用详细形式。alias mapping 的方向是“外部别名 -> canonical value”。
- field 可以直接写类型字符串；存在 `groups`、`comment`、`default`、`range`、`length`、`parser` 或仅 table field 支持的 `from` 时使用详细形式。
- 无选项 parser 可以写成字符串，例如 `parser = "json"`；有选项时写成包含 `kind` 和字符串值选项的对象。
- 非 unique index 可以直接写字段数组；unique index 写成 `{ fields, unique = true }`。
- `groups` 可以是单个 group 字符串，也可以是字符串列表。

各属性的完整语义见 [Enum、Struct 与 Union](enums-structs-unions.md)、[Tables](tables.md)、[Types](types.md)、[Cell Parsers](parsers.md)、[References and Derived Fields](references.md)、[Localization](../localization.md)和[项目配置](../project-config.md)。
