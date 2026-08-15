# 枚举、结构体和联合

这些定义让 schema 可以表达超过扁平表格的数据结构。

## Enums

```scon
enums {
  Rarity = ["Common", "Uncommon", "Rare", "Epic", "Legendary"]
}
```

枚举让源数据保持可读，同时让生成代码获得受约束的类型。

需要为枚举及枚举项添加说明时，可以使用详细形式。Sora 会把注释保留在 schema lock 中，并在生成代码中输出目标语言对应的文档注释：

```scon
enums {
  Rarity {
    comment = "物品稀有度"
    values = [
      { id = 0, name = "Common", comment = "常见物品" },
      { id = 1, name = "Rare", comment = "稀有物品" },
    ]
  }
}
```

alias 可以保留导入数据或旧数据里的名称：

```scon
enums {
  Rarity {
    values = ["Common", "Uncommon", "Rare", "Epic", "Legendary"]
    aliases { Purple = "Epic" }
  }
}
```

## Structs

```scon
structs {
  ResourceCost {
    fields {
      kind = "enum<ResourceKind>"
      id = "i32"
      count { type = "i32" range = [1, 999999] }
    }
  }
}
```

结构体适合多处复用的嵌套值。字段可以通过 `type = "struct<ResourceCost>"` 引用结构体。

Struct field 使用和 table field 相同的字段属性，包括 `type`、`default`、`comment`、`range`、`length`、`parser` 和 `groups`；字段名来自 key。`key`、`from` 这类表专用属性对普通 struct field 没有意义。完整字段参考见[类型](types.md#field-rules)。

在 Excel、CSV 这类单元格输入中，struct 字段默认可以写成 JSON object 文本：

```json
{"kind":"Gold","id":0,"count":100}
```

如果希望单元格更紧凑，可以在引用 struct 的字段上声明 `parser = { kind = "tuple" }`。Tuple 值按 struct 字段顺序书写：

```text
Gold,0,100
```

## Unions

当一个字段可能是几种不同形状之一时，使用 union。例如事件条件可能是“完成任务”，也可能是“拥有道具”：

```json
{"type":"QuestCompleted","quest_id":5002}
```

```json
{"type":"HasItem","item_id":1001,"count":2}
```

`type` 的值决定当前是哪一个 variant。剩余字段取决于这个 variant。

```scon
unions {
  RewardAction {
    tag = "type"
    variants {
      AddItem { fields { item_id = "ref<Item.id>" count = "i32" } }
      UnlockStage { fields { stage_id = "ref<Stage.id>" } }
    }
  }
}
```

当一个字段可能是多个 tagged shape 之一时，使用 union。常见例子包括条件、奖励、触发器和脚本动作。

如果省略，union 的 `tag` 默认是 `type`。源数据必须包含这个 tag，值是 variant 名称。其余字段必须匹配被选中的 variant；未知字段和缺少非 optional 的 variant field 都会校验失败。

最直接的 Excel 或 CSV 写法是把单个 union 值写成 JSON object 文本：

| 字段类型 | 单元格值 |
| --- | --- |
| `union<RewardAction>` | `{"type":"AddItem","item_id":1001,"count":2}` |

union 列表建议声明 `parser = { kind = "json" }`，然后在单元格中写 JSON array：

```scon
fields {
  actions { type = "list<union<RewardAction>>" parser = "json" }
}
```

```json
[
  {"type":"AddItem","item_id":1001,"count":2},
  {"type":"UnlockStage","stage_id":9002}
]
```

如果不想在 Excel/CSV 单元格里写 JSON，可以把一个 `union<T>` 字段展开成多列。下面的 `action` 字段是单个 union 值：

```scon
fields {
  action { type = "union<RewardAction>" parser = "tagged_columns" }
}
```

Excel 中会出现这些列：

| A | B | C | D | E | F |
| --- | --- | --- | --- | --- | --- |
| `id` | `name` | `action.type` | `action.item_id` | `action.count` | `action.stage_id` |
| `1` | `Give Sword` | `AddItem` | `1001` | `2` |  |
| `2` | `Open Stage` | `UnlockStage` |  |  | `9002` |

`action.type` 填 variant 名称。当前行如果是 `AddItem`，只填写 `item_id` 和 `count`；如果是 `UnlockStage`，只填写 `stage_id`。其它 variant 的列保持为空。

`tagged_columns` 只能用于类型正好是 `union<T>` 的字段，不能直接用于 `list<union<T>>`。如果一个父表字段需要多个 union 值，通常把每个 union 值拆成子表的一行，再由父表聚合回来：

```scon
tables {
  EventRule {
    fields {
      actions {
        type = "list<union<RewardAction>>"
        from = { table = "EventActionEntry", parent_key = "id", child_key = "event_id", field = "value", order_by = "seq" }
      }
    }
  }
  EventActionEntry {
    mode = "list"
    fields {
      event_id = "ref<EventRule.id>"
      seq = "i32"
      value {
        type = "union<RewardAction>"
        parser = { kind = "tagged_columns", prefix = "" }
      }
    }
  }
}
```

父表 `EventRule` 只保留普通字段：

| A | B |
| --- | --- |
| `id` | `name` |
| `1` | `First Event` |

子表 `EventActionEntry` 每一行填写一个 action：

| A | B | C | D | E | F |
| --- | --- | --- | --- | --- | --- |
| `event_id` | `seq` | `type` | `item_id` | `count` | `stage_id` |
| `1` | `1` | `AddItem` | `1001` | `2` |  |
| `1` | `2` | `UnlockStage` |  |  | `9002` |

导出时，`EventRule.actions` 会得到两个 union 值，顺序由 `seq` 决定。这里的 `prefix = ""` 让子表列名直接叫 `type`、`item_id`、`count`、`stage_id`；如果这些列名会和同一张表的其它字段冲突，就不要使用空 prefix。

完整展开规则见[单元格 Parser](parsers.md#tagged_columns)。

TOML 数据文件里可以直接用普通嵌套 table 写 union：

```toml
[[rows]]
id = 1
condition = { type = "QuestCompleted", quest_id = 5002 }
actions = [
  { type = "AddItem", item_id = 1001, count = 2 },
  { type = "UnlockStage", stage_id = 9002 },
]
```
