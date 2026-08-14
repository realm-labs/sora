# Schema 格式

Sora 将 SCON、TOML、YAML、JSON、Lua 作为能力等价的一等前端；新项目推荐使用 SCON。五种表示都会先 lower 为相同的 `SchemaModule` 和 `ProjectSchema`，因此校验、IR、代码生成、Excel 模板、导出和 schema lock 的语义一致。

| 扩展名 | 格式 | 说明 |
| --- | --- | --- |
| `.scon` | SCON | 推荐，使用简洁的 object block。 |
| `.toml` | TOML | 使用嵌套 named table。 |
| `.yaml`, `.yml` | YAML | 使用 keyed mapping。 |
| `.json` | JSON | 使用 keyed object。 |
| `.lua` | Lua | 返回纯数据 table；有序成员使用单键对象列表。 |

文件扩展名决定前端。Sora 按声明顺序加载 `includes`，五种格式可以互相 include。

## 统一 keyed 模型

声明名只写在 mapping key 上。`enums`、`structs`、`unions`、`tables`、`fields`、`variants`、`indexes`、`localization.sources` 都遵循这一规则，body 内不能重复 `name`。未知属性和旧的 named object 数组都会被拒绝。

推荐的 SCON 写法：

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

组合统一使用 Sora 的 `includes`。原生 SCON `include`、substitution、interpolation、object spread 和 array spread 会被拒绝，并报告文件、行、列。

## TOML

```toml
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
```

## YAML / JSON

两者都使用以声明名为 key 的 mapping/object：

```yaml
enums:
  ItemType: [Weapon, Armor]
tables:
  Item:
    mode: map
    key: id
    fields:
      id: i32
      name: { type: string, length: [2, 32] }
```

```json
{
  "enums": { "ItemType": ["Weapon", "Armor"] },
  "tables": {
    "Item": {
      "mode": "map",
      "key": "id",
      "fields": {
        "id": "i32",
        "name": { "type": "string", "length": [2, 32] }
      }
    }
  }
}
```

## Lua

Lua 文件必须返回一个纯数据 table。fields 和 union variants 有语义顺序，因此用有序的单键 table 列表表示：

```lua
return {
  enums = { ItemType = { "Weapon", "Armor" } },
  tables = {
    Item = {
      mode = "map",
      key = "id",
      fields = {
        { id = "i32" },
        { name = { type = "string", length = { 2, 32 } } },
      },
    },
  },
}
```

enum values、fields、union variants 保留作者顺序；其他命名集合按名称规范化排序，保证跨格式结果确定。

## 简写

- enum 可直接写值数组；需要 `groups` 或 alias 时展开为 `{ values, groups, aliases }`。alias mapping 的方向是“外部别名 -> canonical value”。
- field 可直接写类型字符串；存在约束、parser、comment、default、groups 或 `from` 时展开。
- 无选项 parser 写成字符串，例如 `parser = "tuple"`；有选项时写含 `kind` 的对象。
- 非 unique index 可直接写字段数组；`unique = true` 时使用详细对象。

项目级 `build`、`parsers`、`type_mappings` 保持原有数据模型，五种格式均可编码。
