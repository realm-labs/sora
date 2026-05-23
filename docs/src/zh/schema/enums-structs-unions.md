# 枚举、结构体和联合

这些定义让 schema 可以表达超过扁平表格的数据结构。

## Enums

```toml
[[enums]]
name = "Rarity"
values = ["Common", "Uncommon", "Rare", "Epic", "Legendary"]
```

枚举让源数据保持可读，同时让生成代码获得受约束的类型。

alias 可以保留导入数据或旧数据里的名称：

```toml
[[enums.aliases]]
name = "Purple"
alias = "Epic"
```

## Structs

```toml
[[structs]]
name = "ResourceCost"

[[structs.fields]]
name = "kind"
type = "enum<ResourceKind>"
required = true

[[structs.fields]]
name = "id"
type = "i32"
required = true

[[structs.fields]]
name = "count"
type = "i32"
required = true
range = [1, 999999]
```

结构体适合多处复用的嵌套值。字段可以通过 `type = "struct<ResourceCost>"` 引用结构体。

Struct field 使用和 table field 相同的字段属性，包括 `name`、`type`、`required`、`default`、`comment`、`range`、`length`、`parser` 和 `scope`。`key`、`from` 这类表专用属性对普通 struct field 没有意义。完整字段参考见[类型](types.md#field-rules)。

在 Excel、CSV 这类单元格输入中，struct 字段默认可以写成 JSON object 文本：

```json
{"kind":"Gold","id":0,"count":100}
```

如果希望单元格更紧凑，可以在引用 struct 的字段上声明 `parser = { kind = "tuple" }`。Tuple 值按 struct 字段顺序书写：

```text
Gold,0,100
```

## Unions

```toml
[[unions]]
name = "RewardAction"
tag = "type"

[[unions.variants]]
name = "AddItem"

[[unions.variants.fields]]
name = "item_id"
type = "ref<Item.id>"
required = true

[[unions.variants.fields]]
name = "count"
type = "i32"
required = true

[[unions.variants]]
name = "UnlockStage"

[[unions.variants.fields]]
name = "stage_id"
type = "ref<Stage.id>"
required = true
```

当一个字段可能是多个 tagged shape 之一时，使用 union。常见例子包括条件、奖励、触发器和脚本动作。

如果省略，union 的 `tag` 默认是 `type`。源数据必须包含这个 tag，值是 variant 名称。其余字段必须匹配被选中的 variant；未知字段和缺少 required variant field 都会校验失败。

Excel 或 CSV 中的单个 union 值写成 JSON object 文本：

| 字段类型 | 单元格值 |
| --- | --- |
| `union<RewardAction>` | `{"type":"AddItem","item_id":1001,"count":2}` |

union 列表建议声明 `parser = { kind = "json" }`，然后在单元格中写 JSON array：

```toml
[[tables.fields]]
name = "actions"
type = "list<union<RewardAction>>"
parser = { kind = "json" }
```

```json
[
  {"type":"AddItem","item_id":1001,"count":2},
  {"type":"UnlockStage","stage_id":9002}
]
```

如果不想在 Excel/CSV 里写 JSON，这次新增的是条目表里的 `union<T>` 字段写法：它可以使用 `parser = { kind = "tagged_columns", prefix = "" }`，这样工作簿里就是 `type`、`quest_id`、`item_id`、`count` 这类普通列。父表引用这些条目行时仍然使用已有的 `ref<Table.key>` 或 `list<ref<Table.key>>` 语义。完整规则见[单元格 Parser](parsers.md#tagged-union-columns)。

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
