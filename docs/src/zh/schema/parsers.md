# 单元格 Parser

Parser 只用于 Excel、CSV 这类单元格输入。大多数 parser 告诉 Sora 如何把一个 cell 转成类型化值；`columns`、`tagged_columns` 这类投影 parser 告诉 Sora 一个字段如何映射到多列输入。字符串形式的 `default` 会走单 cell parser 的同一套路径。TOML 行数据通常可以直接使用 TOML array/table，不需要单元格 parser。

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

Parser option 都是字符串。未知 parser、当前 parser 不支持的 option、空 option value 都会在 schema normalization 阶段报错。例外是 `columns.prefix`、`tagged_columns.prefix` 这类投影前缀，`""` 有明确含义。

## 自定义 Lua Parser

项目可以在 `project.toml` 里加载项目内的 Lua parser 脚本：

```toml
[parsers]
scripts = ["tools/parsers.lua"]
```

脚本路径按 project 文件所在目录解析。之后所有读取该 project 的命令都能使用这些自定义 parser，不需要反复写命令行参数：

```bash
sora build --project project.toml
sora export --project project.toml --data-root data --format json --out generated/config.json
```

CLI 命令也可以通过全局 `--parser-script` 临时追加 parser 脚本：

```bash
sora --parser-script tools/parsers.lua build --project project.toml
sora --parser-script tools/parsers.lua export --project project.toml --data-root data --format json --out generated/config.json
```

这个参数可以重复传，并追加在 project 配置的脚本之后。自定义 parser 属于项目可信代码。Sora 会用受限 Lua 标准库加载脚本，不暴露 `io`、`os`、`package` 或 `debug`。

Parser 脚本返回一个带 `parsers` 的 table。每个 parser 必须定义 `parse(cell, ctx)`。`options` 是支持的 parser option 列表。`validate(field)` 可选，会在 schema normalization 阶段执行。

```lua
return {
  parsers = {
    slug = {
      options = { "prefix" },
      validate = function(field)
        if field.type ~= "string" then
          error("slug parser requires string")
        end
      end,
      parse = function(cell, ctx)
        local text = string.lower(string.gsub(cell.text, "%s+", "-"))
        if ctx.options.prefix ~= nil then
          return ctx.options.prefix .. text
        end
        return text
      end,
    },
  },
}
```

Schema 字段按名字使用自定义 parser：

```toml
[[tables.fields]]
name = "tag"
type = "string"
parser = { kind = "slug", prefix = "item-" }
```

`cell` 包含 `kind`、`text`，以及适用时的 `value`。`ctx` 包含 `field`、`type`、`options`、`path`，以及 `row`、`column`、worksheet 的 `sheet` 等位置信息。Lua 返回值会映射成 Sora 数据值：`nil`、bool、integer、float、string、array-like table 和 string-keyed table。

自定义 Lua parser 是单 cell parser。它不会替代 `columns`、`tagged_columns` 这类投影 parser，不能读取相邻 cell，也不会改变 schema、source loading 或生成 runtime 的行为。

## 默认解析

字段没有声明 `parser` 时，Sora 按字段类型做默认解析：

| 类型 | Cell 写法 |
| --- | --- |
| `bool` | 布尔 cell、`true`、`false`，或数字 cell：0 为 false，非 0 为 true。 |
| `i32`、`i64`、`ref<Table.key>` | 整数 cell、整数字符串，或无小数部分的 float cell。 |
| `duration` | 带 `d`、`h`、`m`、`s` 或 `ms` 单位的时长文本，例如 `500ms`、`30s` 或 `1h 30m`。单位必须按从大到小排列。 |
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
| `columns` | `struct<T>` 或 `optional<struct<T>>` | 多列 |
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

## columns

`columns` 适合把一个 struct 展开成普通 Excel/CSV 列来编辑，而不是写 JSON 或一个紧凑 tuple cell。它只能用于 table field 上的 `struct<T>` 或 `optional<struct<T>>`。

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
parser = { kind = "columns", prefix = "price_" }
```

CSV header 和行：

```csv
id,name,price_kind,price_id,price_count
1,Iron Sword,Gold,0,100
```

解析出的 `price`：

```json
{"kind":"Gold","id":0,"count":100}
```

默认 prefix 会使用字段名，例如字段 `price` 会投影出 `price.kind`、`price.id`、`price.count`。只有当 struct 字段名应该直接位于当前表顶层时，才使用 `prefix = ""`。Sora 会拒绝投影列名冲突。

`columns` 不会递归展开嵌套 struct 或 union。如果被展开出来的 struct 字段本身仍然是复杂类型，可以给这个子字段声明 `tuple`、`split`、`map`、`json` 这类单 cell parser；如果嵌套数据很大或需要重复出现，应该拆成独立表，再用 `ref` 或派生字段连接。这样可以避免表变得很宽，也让复杂记录更容易复用。

生成 XLSX 模板时，同一个 `columns` 字段投影出来的列会使用同一组表头颜色。

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

`tagged_columns` 也不会递归展开 variant 字段里的嵌套 struct 或嵌套 union。Variant 字段仍然可以使用 `tuple`、`split`、`map`、`json` 这类单 cell parser。如果某个 variant 需要很大的嵌套对象或重复嵌套对象，应该把那部分数据建成独立表，再通过引用或派生字段组合，而不是继续把 union 行横向展开。

生成 XLSX 模板时，同一个 `tagged_columns` 字段投影出来的列会使用同一组表头颜色，tag 列会在同色组里更醒目。

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
| 一个 struct 展开成多列 | `columns` |
| 一组紧凑 struct | `tuple_list` |
| 简单 key/value pair | `map` |
| 一个 union 展开成多列 | `tagged_columns` |
| 嵌套值、容器里的 union、需要转义、或想直接写 JSON | `json` |
