# 单元格 Parser

Parser 控制 Excel、CSV 这类单元格输入如何把一个 cell 转成类型化值。字符串形式的 `default` 也会走 parser。TOML 行数据通常可以直接使用 TOML array/table，不需要单元格 parser。

```toml
[[tables.fields]]
name = "tags"
type = "list<string>"
parser = { kind = "split", separator = "|" }
```

Parser option 都是字符串。未知 parser、当前 parser 不支持的 option、空 option value 都会在 schema normalization 阶段报错。例外是 `tagged_columns.prefix`，`""` 有明确含义。

## 默认 Cell 解析

字段没有声明 `parser` 时，Sora 按字段类型做默认解析：

| 类型 | Cell 写法 |
| --- | --- |
| `bool` | 布尔 cell、`true`、`false`，或数字 cell：0 为 false，非 0 为 true。 |
| `i32`、`i64`、`ref<Table.key>` | 整数 cell、整数字符串，或无小数部分的 float cell。 |
| `f32`、`f64` | 数字 cell 或数字字符串。 |
| `string`、`enum<Name>` | cell 展示文本。 |
| `struct<Name>`、`union<Name>` | JSON object 文本。 |
| `list<T>`、`set<T>`、`array<T,N>` | 逗号分隔文本。允许外层带 `[...]`。 |
| `map<K,V>` | JSON pair array，例如 `[["atk",10],["hp",20]]`。 |
| `optional<T>` | 空 cell 解析成 `null`；非空时按内部 `T` 解析。 |

逗号分隔集合里的 primitive item 会按类型解析。struct 和 union 集合 item 必须写成 JSON object 文本。嵌套集合不能靠单个分隔符表达，应该使用 `json` parser。

## 内置 Parser

| Parser | 适用类型 | Options | 默认写法 |
| --- | --- | --- | --- |
| `split` | `list<T>`、`set<T>`、`array<T,N>`，或包在这些类型外的 `optional` | `separator`，默认 `,` | `a,b,c` 或 `[a,b,c]` |
| `tuple` | `struct<T>` 或 `optional<struct<T>>` | `separator`，默认 `,` | 按 struct 字段声明顺序写值，例如 `Gold,0,100` |
| `tuple_list` | `list<struct<T>>`、`set<struct<T>>`、`array<struct<T>,N>`，或包在这些类型外的 `optional` | `separator`，默认 `,`；`item_separator`，默认 `|` | `Gold,0,100|Gem,0,5` |
| `map` | `map<K,V>` 或 `optional<map<K,V>>` | `separator`，默认 `,`；`item_separator`，默认 `|` | `atk,10|hp,20` |
| `tagged_columns` | 只能用于 `union<T>` | `prefix`，默认 `<field>.` | 多列：一个 tag 列加 union variant fields |
| `json` | 任意类型 | 无 | 匹配字段类型的 JSON value |

`array<T,N>` 会检查 item 数量。`tuple` 会检查 value 数量是否等于被引用 struct 的字段数。

## Tagged Union Columns

`tagged_columns` 用来把一个 `union<T>` 值展开到多列 Excel/CSV 中编辑。它只能用于类型正好是 `union<T>` 的 table field。它不能用于 `optional<union<T>>`、`list<union<T>>`、`set<union<T>>` 或其它容器。`ref<EventConditionEntry.id>` 这类引用仍然是已有的主键引用语义；这个 parser 只改变被引用的 union 条目表如何填写自己的 `union<T>` 值。

默认 prefix 会使用字段名。例如字段 `condition` 的类型是 `union<EventCondition>`，会投影出 `condition.type`、`condition.quest_id`、`condition.item_id` 这类列。如果这个表本身就是 union 词表，可以设置 `prefix = ""` 让列展开到顶层：

```toml
[[tables.fields]]
name = "value"
type = "union<EventCondition>"
required = true
parser = { kind = "tagged_columns", prefix = "" }
```

CSV 示例：

```csv
id,type,quest_id,item_id,count
1,QuestCompleted,5002,,
2,HasItem,,1001,2
```

tag 列必须填写 union variant 名称。只有被选中的 variant 的字段列可以填写值。Sora 会拒绝投影列名冲突，例如表里已经有普通字段 `type`，同时又对 tag 也是 `type` 的 union 使用 `prefix = ""`。

## JSON 形状

对于嵌套值、union、嵌套集合，以及需要明确转义的 map，`json` 是最稳妥的 parser。

```toml
[[tables.fields]]
name = "condition"
type = "union<EventCondition>"
parser = { kind = "json" }

[[tables.fields]]
name = "actions"
type = "list<union<RewardAction>>"
parser = { kind = "json" }
```

对应单元格示例：

```json
{"type":"QuestCompleted","quest_id":5002}
```

```json
[
  {"type":"AddItem","item_id":1007,"count":3},
  {"type":"UnlockStage","stage_id":9002}
]
```

`map<K,V>` 的 JSON 写法是 pair array，不是 JSON object：

```json
[["atk",10],["hp",20]]
```

## Tuple 字段顺序

`tuple` 和 `tuple_list` 使用被引用 struct 的 schema 字段声明顺序。对于这个 struct：

```toml
[[structs]]
name = "ResourceCost"

[[structs.fields]]
name = "kind"
type = "enum<ResourceKind>"

[[structs.fields]]
name = "id"
type = "i32"

[[structs.fields]]
name = "count"
type = "i32"
```

`parser = { kind = "tuple" }` 期望：

```text
Gold,0,100
```

如果嵌套 struct 字段自己声明了 parser，解析 tuple item 时也会使用这个嵌套 parser。
