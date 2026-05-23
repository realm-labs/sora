# 单元格 Parser

Parser 控制 Excel、CSV 这类单元格输入如何把一个 cell 转成类型化值。字符串形式的 `default` 也会走 parser。TOML 行数据通常可以直接使用 TOML array/table，不需要单元格 parser。

```toml
[[tables.fields]]
name = "tags"
type = "list<string>"
parser = { kind = "split", separator = "|" }
```

Parser option 都是字符串。未知 parser、当前 parser 不支持的 option、空 option value 都会在 schema normalization 阶段报错。

## 默认 Cell 解析

字段没有声明 `parser` 时，Sora 按字段类型做默认解析：

| 类型 | Cell 写法 |
| --- | --- |
| `bool` | 布尔 cell、`true`、`false`，或数字 cell：0 为 false，非 0 为 true。 |
| `i32`、`i64`、`ref<Table.field>` | 整数 cell、整数字符串，或无小数部分的 float cell。 |
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
| `json` | 任意类型 | 无 | 匹配字段类型的 JSON value |

`array<T,N>` 会检查 item 数量。`tuple` 会检查 value 数量是否等于被引用 struct 的字段数。

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
