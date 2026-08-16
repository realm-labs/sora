# 引用和派生字段

引用让一张表指向另一张表的主键。派生字段则从另一张表匹配行，并把值复制或组装到当前表。

| 功能 | 源数据里存什么 | 运行时模型得到什么 |
| --- | --- | --- |
| `ref<Item.id>` | 目标行 id，例如 `1001`。 | id 值，或目标语言专用包装类型。 |
| `from = { ... }` | 数据仍然放在子表行里。 | 父行得到复制或嵌套出来的字段。 |

当关系本身应该保留为 id 时，用 `ref`。当导出数据希望直接带有嵌套字段时，用 `from`。

`ref` 的目标表必须是 `mode = "map"`，被引用字段必须是那张表的 `key`。

## 引用

```toml
[[tables.fields]]
name = "required_item"
type = "ref<Item.id>"
```

Sora 会校验每个值都指向被引用表中存在的行。

引用在源数据里仍然是普通值。生成的运行时代码可以根据目标语言，把它暴露为 key 值或目标语言专用的包装类型。

引用可以放在容器里，例如 `list<ref<Item.id>>`、`set<ref<Item.id>>` 或 `optional<ref<Item.id>>`。内部的 `ref` 仍然必须指向主键。

### Rust 强类型表主键

Rust codegen 会为每个 primitive `map` 表主键生成由目标表拥有的 nominal newtype。若 `Item` 的主键是 `id: u32`，生成类型为 `ItemId`；`Item.id` 和所有 `ref<Item.id>` 都使用同一个类型，包括容器、struct 和 union 内部的引用。另一张同样以 `u32` 为主键的表会得到不同的 Rust 类型，因此两种 id 不能误传。

包装类型使用 `#[repr(transparent)]` 和透明 Serde，所以 bundle、JSON、schema lock 等数据格式仍存储原始标量。生成 API 通过 `from_raw` 和 `raw`/`as_str` 显式转换，不公开 tuple 字段，也不会通过 `Deref` 暴露底层 primitive。string/text 主键表还会生成明确的 `get_str(&str)` 查询方法。

enum 主键直接复用生成的 enum。若某张表的主键本身是 `ref<Other.id>`，它会复用 `Other` 的最终 key type，不会再嵌套一层包装。带命名空间的表会把 key type 放在所属表的模块内，不建立根级 re-export。

这是 Rust 的标准生成语义，没有兼容开关。其他语言生成器目前仍保持各自现有的 key 表示。

## 派生字段

派生字段不是从当前表的单元格读取，而是从另一张表中按 key 匹配行后生成。

这样可以让可编辑数据保持范式化，同时让生成的运行时模型暴露更方便的嵌套值。例如任务奖励可以拆成两张表：

`Quest`：

| id | name |
| --- | --- |
| 1001 | First Quest |
| 1002 | Second Quest |

`QuestReward`：

| quest_id | sort_order | item_id | count |
| --- | --- | --- | --- |
| 1001 | 1 | 2001 | 10 |
| 1001 | 2 | 2002 | 1 |
| 1002 | 1 | 2003 | 5 |

运行时如果希望 `Quest` 里直接有 `rewards: list<Reward>` 字段，可以声明这个字段来自 `QuestReward`：

```toml
[[structs]]
name = "Reward"

[[structs.fields]]
name = "item_id"
type = "ref<Item.id>"

[[structs.fields]]
name = "count"
type = "i32"

[[tables]]
id = "quest"
name = "Quest"
mode = "map"
key = "id"

[[tables.fields]]
name = "id"
type = "i32"

[[tables.fields]]
name = "name"
type = "string"

[[tables.fields]]
name = "rewards"
type = "list<struct<Reward>>"
from = { table = "QuestReward", parent_key = "id", child_key = "quest_id", order_by = "sort_order" }

[[tables]]
id = "questreward"
name = "QuestReward"
mode = "list"

[[tables.fields]]
name = "quest_id"
type = "ref<Quest.id>"

[[tables.fields]]
name = "sort_order"
type = "i32"

[[tables.fields]]
name = "item_id"
type = "ref<Item.id>"

[[tables.fields]]
name = "count"
type = "i32"
```

含义是：

- `from.table = "QuestReward"`：从 `QuestReward` 子表读取匹配行。
- `from.parent_key = "id"`：父行用自己的 `Quest.id` 值参与匹配。
- `from.child_key = "quest_id"`：子行的 `QuestReward.quest_id` 等于父 key 时被选中。
- `from.order_by = "sort_order"`：匹配到多行时，按子表里的 `sort_order` 字段升序排序。

用上面的示例数据，`Quest.id = 1001` 会得到两行奖励，顺序是 `2001`，然后 `2002`。

导出后的父行就像直接拥有了 `rewards` 字段：

```json
{
  "id": 1001,
  "name": "First Quest",
  "rewards": [
    {"item_id": 2001, "count": 10},
    {"item_id": 2002, "count": 1}
  ]
}
```

字段类型决定允许匹配多少行：

| 字段类型 | 匹配行数 | 没有匹配行时 |
| --- | --- | --- |
| `list<T>` | 0 到多行 | 空列表 |
| `optional<T>` | 0 或 1 行 | `null` |
| `T` | 必须正好 1 行 | 校验错误 |
| `map<K,V>` | 0 到多行，key 必须唯一 | 空 Map |

如果 `T` 或 `optional<T>` 匹配到多行，Sora 会报错。

## 派生 Map

派生 Map 仍使用和派生 list 相同的父子表关联。增加 `from.key` 指定哪个子表字段成为 Map key，再用 `from.field` 指定 Map value：

```scon
tables {
  Item {
    mode = "map"
    key = "id"
    fields {
      id = "i32"
      drop_rates {
        type = "map<string,i32>"
        from {
          table = "ItemDropRate"
          parent_key = "id"
          child_key = "item_id"
          key = "rarity"
          field = "rate"
        }
      }
    }
  }

  ItemDropRate {
    mode = "list"
    fields {
      item_id = "ref<Item.id>"
      rarity = "string"
      rate = "i32"
    }
  }
}
```

可编辑数据仍然是普通子表行，不需要人为添加子行 id：

| item_id | rarity | rate |
| ---: | --- | ---: |
| 1001 | Common | 80 |
| 1001 | Epic | 20 |

对于 `Item.id = 1001`，Sora 会物化出：

```json
"drop_rates": [["Common", 80], ["Epic", 20]]
```

Sora 使用 pair array 表示 Map，因此非 string key 也不会产生歧义；生成运行时会暴露目标语言正常的 Map 或 Dictionary 类型。

规则保持刻意收敛：

- 类型正好是 `map<K,V>` 的派生字段必须声明 `from.key`；非 Map 字段不能声明 `from.key`。
- 子表 key 字段必须存在，并且类型必须和 `K` 严格匹配。比较前会把 `ref<Table.field>` 展开为目标字段类型，但不会进行任何隐式转换。
- 存在 `from.field` 时，对应子字段类型必须和 `V` 兼容。
- 不写 `from.field` 时，`V` 必须是由子表同名字段组装的 struct。
- 同一个父行匹配出来的 Map key 不能重复；enum alias 在重复检查时按规范枚举值处理。
- 没有匹配子行时得到空 Map。
- `order_by` 可以控制 pair 的确定顺序，但 Map 查找语义不依赖顺序。

Studio 会从现有 `from` 自动建立反向使用关系，因此源表可以显示 `Item.drop_rates (item_id -> id, key=rarity, field=rate)`，不需要在 Schema 中再声明一份 owner。一张源表仍然可以供多个父字段或父表使用。

## 复制子表的单个字段

不写 `from.field` 时，Sora 会从子表的同名字段组装 struct。

如果父字段只需要接收子行中的某一个字段，设置 `from.field`：

```toml
[[unions]]
name = "EventCondition"
tag = "type"

[[unions.variants]]
name = "QuestCompleted"

[[unions.variants.fields]]
name = "quest_id"
type = "ref<Quest.id>"

[[unions.variants]]
name = "HasItem"

[[unions.variants.fields]]
name = "item_id"
type = "ref<Item.id>"

[[unions.variants.fields]]
name = "count"
type = "i32"

[[tables.fields]]
name = "condition"
type = "union<EventCondition>"
from = { table = "EventConditionEntry", parent_key = "id", child_key = "event_id", field = "value" }

[[tables]]
id = "eventconditionentry"
name = "EventConditionEntry"
mode = "list"

[[tables.fields]]
name = "event_id"
type = "ref<Event.id>"

[[tables.fields]]
name = "value"
type = "union<EventCondition>"
parser = { kind = "tagged_columns", prefix = "" }
```

含义是：`Event.condition` 接收 `EventConditionEntry.value`，前提是该子行的 `event_id` 等于 `Event.id`。子表里仍然可以有 `id`、`event_id`、备注、排序字段等辅助列；只有 `from.field` 指向的 `value` 会被复制到父表字段。

`EventConditionEntry` 在 Excel 中可以这样写：

| A | B | C | D | E |
| --- | --- | --- | --- | --- |
| `event_id` | `type` | `quest_id` | `item_id` | `count` |
| `1` | `QuestCompleted` | `5002` |  |  |
| `2` | `HasItem` |  | `1001` | `2` |

## From 配置

`from` 对象有这些配置：

| 选项 | 必填 | 含义 |
| --- | --- | --- |
| `table` | 是 | 子表名。Sora 会从这张表扫描匹配行。 |
| `parent_key` | 是 | 父表上的字段名。每个父行用这个字段值参与匹配。 |
| `child_key` | 是 | 子表上的字段名。子行的这个字段值等于父 key 时，就会被选中。 |
| `key` | `map<K,V>` 必填 | 子表上的字段名，其值成为每个 Map entry 的 key；非 Map 派生字段不能使用。 |
| `field` | 否 | 子表上的字段名。存在时，Sora 复制这个字段的值，而不是从整行组装 struct。 |
| `order_by` | 否 | 子表上的字段名。存在时，匹配到的子行按这个字段升序排序。 |

`order_by` 是字段名，不是表达式。没有 `desc`、多字段排序、过滤条件或自定义排序语法。省略 `order_by` 时，匹配行保持源表读取顺序。

`order_by` 指向的字段必须存在于子表中。它通常会是 `i32` 这类排序字段，例如 `sort_order`、`seq`、`rank`。排序是升序。

不写 `from.field` 时，派生值类型必须是 struct，也就是 `list<struct<...>>`、`struct<...>`、`optional<struct<...>>` 或 `map<K,struct<...>>`。结构体字段会从子表同名字段复制：

```toml
[[structs]]
name = "Reward"

[[structs.fields]]
name = "item_id"
type = "ref<Item.id>"

[[structs.fields]]
name = "count"
type = "i32"
```

这里 `Reward.item_id` 和 `Reward.count` 都必须在 `QuestReward` 上存在兼容字段。

写了 `from.field` 时，派生值类型必须和该子表字段兼容。例如 `type = "union<EventCondition>"` 可以从同样是 `union<EventCondition>` 的子表字段 `value` 派生。

派生字段不能同时声明 `default`。它的值来自匹配到的子行。

## 多个派生字段读取同一张子表

多张父表可以从同一张子表派生字段。这个过程不会消耗或移动子行，只是读取子表，并把匹配值复制到每个父字段。

例如 `Quest` 和 `QuestPreview` 都可以从 `QuestReward` 获取奖励：

```toml
[[tables]]
id = "quest"
name = "Quest"
mode = "map"
key = "id"

[[tables.fields]]
name = "rewards"
type = "list<struct<Reward>>"
from = { table = "QuestReward", parent_key = "id", child_key = "quest_id", order_by = "sort_order" }

[[tables]]
id = "questpreview"
name = "QuestPreview"
mode = "map"
key = "id"

[[tables.fields]]
name = "rewards"
type = "list<struct<Reward>>"
from = { table = "QuestReward", parent_key = "id", child_key = "quest_id", order_by = "sort_order" }
```

如果 `Quest.id = 1001` 和 `QuestPreview.id = 1001` 都存在，两张父表都会收到来自 `QuestReward.quest_id = 1001` 的奖励列表。Sora 不会把子行标记为已被 `Quest` 使用，也不会从 `QuestReward` 删除这行。
