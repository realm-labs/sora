# 单元格 Parser

Parser 只用于 Excel、CSV 这类单元格输入。它告诉 Sora 如何把一个 cell 转成类型化值。字符串形式的 `default` 也会走同一套 parser。TOML 行数据通常可以直接使用 TOML array/table，不需要单元格 parser。

当默认 cell 写法太冗长或容易歧义时，再声明 parser：

```toml
[[tables.fields]]
name = "tags"
type = "list<string>"
parser = { kind = "split", separator = "|" }
```

对应 cell：

```text
starter|melee|weapon
```

Parser option 都是字符串。未知 parser、当前 parser 不支持的 option、空 option value 都会在 schema normalization 阶段报错。例外是 `tagged_columns.prefix`，`""` 有明确含义。

## 默认解析

字段没有声明 `parser` 时，Sora 按字段类型做默认解析：

| 类型 | Cell 写法 |
| --- | --- |
| `bool` | 布尔 cell、`true`、`false`，或数字 cell：0 为 false，非 0 为 true。 |
| `i32`、`i64`、`ref<Table.key>` | 整数 cell、整数字符串，或无小数部分的 float cell。 |
| `f32`、`f64` | 数字 cell 或数字字符串。 |
| `string`、`enum<Name>` | cell 展示文本。 |
| `struct<Name>`、`union<Name>` | JSON object 文本。 |
| `list<T>`、`set<T>`、`array<T,N>` | 逗号分隔文本。JSON array 请使用 `json` parser。 |
| `map<K,V>` | JSON pair array，例如 `[["atk",10],["hp",20]]`。 |
| `optional<T>` | 空 cell 解析成 `null`；非空时按内部 `T` 解析。 |

默认集合解析刻意保持简单。Primitive item 会按类型解析。struct 和 union 集合 item 必须写成 JSON object 文本。嵌套集合不能靠单个分隔符可靠表达，应该使用 `parser = { kind = "json" }`。

## Parser 速查

| Parser | 适用类型 | Cell 形状 |
| --- | --- | --- |
| `split` | `list<T>`、`set<T>`、`array<T,N>`，或包在这些类型外的 `optional` | `a,b,c` |
| `tuple` | `struct<T>` 或 `optional<struct<T>>` | `Gold,0,100` |
| `tuple_list` | `list<struct<T>>`、`set<struct<T>>`、`array<struct<T>,N>`，或包在这些类型外的 `optional` | `Gold,0,100\|Gem,0,5` |
| `map` | `map<K,V>` 或 `optional<map<K,V>>` | `atk,10\|hp,20` |
| `tagged_columns` | 只能用于 `union<T>` | 多列 |
| `json` | 任意类型 | 匹配字段类型的 JSON value |

`array<T,N>` 会检查 item 数量。`tuple` 会检查 value 数量是否等于被引用 struct 的字段数。

## split

`split` 适合扁平集合，例如 primitive、enum、ref，或其它可以稳定用分隔符切开的简单值。

```toml
[[tables.fields]]
name = "starter_items"
type = "list<ref<Item.id>>"
parser = { kind = "split" }
```

Cell：

```text
1001,1002,1003
```

解析结果：

```json
[1001,1002,1003]
```

当逗号不适合作为分隔符时，声明 `separator`：

```toml
[[tables.fields]]
name = "tags"
type = "set<string>"
parser = { kind = "split", separator = "|" }
```

Cell：

```text
starter|melee|weapon
```

## tuple

`tuple` 适合把一个很小的 struct 写在一个 cell 里。值的顺序等于该 struct 的字段声明顺序。

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

[[tables.fields]]
name = "price"
type = "struct<ResourceCost>"
parser = { kind = "tuple" }
```

Cell：

```text
Gold,0,100
```

解析结果：

```json
{"kind":"Gold","id":0,"count":100}
```

如果 struct 字段值里经常出现逗号，可以换分隔符：

```toml
parser = { kind = "tuple", separator = "|" }
```

Cell：

```text
Gold|0|100
```

## tuple_list

`tuple_list` 适合一组小 struct。`separator` 用来切一个 struct 内部的字段，`item_separator` 用来切 list item。

```toml
[[tables.fields]]
name = "materials"
type = "list<struct<ResourceCost>>"
parser = { kind = "tuple_list" }
```

Cell：

```text
Item,2003,4|Gold,0,1000
```

解析结果：

```json
[
  {"kind":"Item","id":2003,"count":4},
  {"kind":"Gold","id":0,"count":1000}
]
```

自定义分隔符：

```toml
parser = { kind = "tuple_list", separator = ":", item_separator = ";" }
```

Cell：

```text
Item:2003:4;Gold:0:1000
```

## map

`map` 适合简单 key/value pair。`separator` 用来切 key 和 value，`item_separator` 用来切 map entry。

```toml
[[tables.fields]]
name = "attributes"
type = "map<string,i32>"
parser = { kind = "map" }
```

Cell：

```text
atk,10|hp,20
```

解析结果：

```json
[["atk",10],["hp",20]]
```

Sora 导出 map 时使用 pair array，这样非 string key 也不会有歧义。如果你更想在 cell 里写 JSON，也可以使用 `parser = { kind = "json" }`，然后写同样的 pair-array 形状：

```json
[["atk",10],["hp",20]]
```

## tagged_columns

`tagged_columns` 用来把一个 `union<T>` 值展开到多列 Excel/CSV 中编辑。它只能用于类型正好是 `union<T>` 的 table field。它不能用于 `optional<union<T>>`、`list<union<T>>`、`set<union<T>>` 或其它容器。

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
name = "value"
type = "union<EventCondition>"
parser = { kind = "tagged_columns", prefix = "" }
```

CSV header 和行：

```csv
id,type,quest_id,item_id,count
1,QuestCompleted,5002,,
2,HasItem,,1001,2
```

tag 列填写 union variant 名称。只有被选中的 variant 的字段列可以填写值。默认 prefix 会使用字段名，例如字段 `condition` 会投影出 `condition.type`、`condition.quest_id`、`condition.item_id`。只有当这些列应该直接位于当前表顶层时，才使用 `prefix = ""`。

Sora 会拒绝投影列名冲突。例如表里已经有普通字段 `type`，同时又对 tag 也是 `type` 的 union 使用 `prefix = ""`。

## json

`json` 适合嵌套值、容器里的 union、嵌套集合，以及任何需要明确转义的复杂形状。

```toml
[[tables.fields]]
name = "actions"
type = "list<union<RewardAction>>"
parser = { kind = "json" }
```

Cell：

```json
[
  {"type":"AddItem","item_id":1007,"count":3},
  {"type":"UnlockStage","stage_id":9002}
]
```

单个 union 值：

```toml
[[tables.fields]]
name = "condition"
type = "union<EventCondition>"
parser = { kind = "json" }
```

Cell：

```json
{"type":"QuestCompleted","quest_id":5002}
```

`map<K,V>` 的 JSON 写法是 pair array，不是 JSON object：

```json
[["atk",10],["hp",20]]
```

## 怎么选

| 需求 | 推荐 |
| --- | --- |
| 扁平 primitive 列表 | `split` |
| 一个紧凑 struct | `tuple` |
| 一组紧凑 struct | `tuple_list` |
| 简单 key/value pair | `map` |
| 一个 union 展开成多列 | `tagged_columns` |
| 嵌套值、容器里的 union、需要转义、或想直接写 JSON | `json` |
