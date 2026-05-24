# Cell Parsers

Parsers are only for cell-based inputs such as Excel and CSV. Most parsers tell Sora how to turn one cell into a typed value; projection parsers such as `columns` and `tagged_columns` tell Sora how one field maps to several input columns. String `default` values use the same parser path for single-cell parsers. TOML row data can usually use native TOML arrays and tables instead.

Use a parser when the default cell format is too verbose or ambiguous:

```toml
[[tables.fields]]
name = "tags"
type = "list<string>"
parser = { kind = "split", separator = "|" }
```

With that schema, the cell value is:

```text
starter|melee|weapon
```

Parser options are string values. Unknown parser kinds, unsupported options, and empty option values fail during schema normalization. The exception is projection prefixes such as `columns.prefix` and `tagged_columns.prefix`, where `""` is meaningful.

## Default Parsing

If a field has no `parser`, Sora uses type-aware default parsing:

| Type | Cell format |
| --- | --- |
| `bool` | Boolean cells, `true`, `false`, or numeric cells where zero is false and non-zero is true. |
| `i32`, `i64`, `ref<Table.key>` | Integer cells, integer text, or whole-number float cells. |
| `f32`, `f64` | Numeric cells or numeric text. |
| `string`, `enum<Name>` | Cell display text. |
| `struct<Name>`, `union<Name>` | JSON object text. |
| `list<T>`, `set<T>`, `array<T,N>` | Comma-separated text. Use `json` for JSON arrays. |
| `map<K,V>` | JSON array of two-item pairs, for example `[["atk",10],["hp",20]]`. |
| `optional<T>` | Empty cell becomes `null`; otherwise the inner `T` is parsed. |

Default collection parsing is intentionally simple. Primitive items are parsed by type. Struct and union collection items must be JSON object text. Nested collections cannot be represented safely with one separator; use `parser = { kind = "json" }`.

## Parser Summary

| Parser | Valid target types | Cell shape |
| --- | --- | --- |
| `split` | `list<T>`, `set<T>`, `array<T,N>`, or `optional` around those types | `a,b,c` |
| `tuple` | `struct<T>` or `optional<struct<T>>` | `Gold,0,100` |
| `columns` | `struct<T>` or `optional<struct<T>>` | Multiple columns |
| `tuple_list` | `list<struct<T>>`, `set<struct<T>>`, `array<struct<T>,N>`, or `optional` around those types | `Gold,0,100\|Gem,0,5` |
| `map` | `map<K,V>` or `optional<map<K,V>>` | `atk,10\|hp,20` |
| `tagged_columns` | `union<T>` only | Multiple columns |
| `json` | Any type | JSON value matching the field type |

`array<T,N>` checks the parsed item count. `tuple` checks the value count against the referenced struct's field count.

## split

Use `split` for a flat collection of primitive values, enums, refs, or simple values that can be separated reliably.

```toml
[[tables.fields]]
name = "starter_items"
type = "list<ref<Item.id>>"
parser = { kind = "split" }
```

Cell:

```text
1001,1002,1003
```

Parsed value:

```json
[1001,1002,1003]
```

Use `separator` when comma is not a good separator:

```toml
[[tables.fields]]
name = "tags"
type = "set<string>"
parser = { kind = "split", separator = "|" }
```

Cell:

```text
starter|melee|weapon
```

## tuple

Use `tuple` when a single struct is small enough to fit naturally in one cell. Values follow the referenced struct's field declaration order.

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

Cell:

```text
Gold,0,100
```

Parsed value:

```json
{"kind":"Gold","id":0,"count":100}
```

Use `separator` if struct values themselves commonly contain commas:

```toml
parser = { kind = "tuple", separator = "|" }
```

Cell:

```text
Gold|0|100
```

## columns

Use `columns` when one struct should be edited as normal Excel or CSV columns instead of as JSON or one compact tuple cell. It is valid on `struct<T>` and `optional<struct<T>>` table fields.

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

CSV headers and row:

```csv
id,name,price_kind,price_id,price_count
1,Iron Sword,Gold,0,100
```

Parsed `price` value:

```json
{"kind":"Gold","id":0,"count":100}
```

With the default prefix, a field named `price` projects columns such as `price.kind`, `price.id`, and `price.count`. Use `prefix = ""` only when the struct field names should live at the table's top level. Sora rejects projected column name conflicts.

`columns` does not recursively project nested structs or unions. If a projected struct field is itself complex, either give that child field a single-cell parser such as `tuple`, `split`, `map`, or `json`, or move the nested data into a dedicated table and connect it with `ref` or a derived field. This keeps the spreadsheet narrow and keeps complex records reusable.

For generated XLSX templates, columns projected from the same `columns` field share the same header color.

## tuple_list

Use `tuple_list` for a list of small structs. `separator` splits fields inside one struct item. `item_separator` splits items in the list.

```toml
[[tables.fields]]
name = "materials"
type = "list<struct<ResourceCost>>"
parser = { kind = "tuple_list" }
```

Cell:

```text
Item,2003,4|Gold,0,1000
```

Parsed value:

```json
[
  {"kind":"Item","id":2003,"count":4},
  {"kind":"Gold","id":0,"count":1000}
]
```

Custom separators:

```toml
parser = { kind = "tuple_list", separator = ":", item_separator = ";" }
```

Cell:

```text
Item:2003:4;Gold:0:1000
```

## map

Use `map` when a map is simple enough to write as repeated key/value pairs. `separator` splits key from value. `item_separator` splits map entries.

```toml
[[tables.fields]]
name = "attributes"
type = "map<string,i32>"
parser = { kind = "map" }
```

Cell:

```text
atk,10|hp,20
```

Parsed value:

```json
[["atk",10],["hp",20]]
```

Sora exports maps as pair arrays so non-string keys remain unambiguous. If you prefer JSON cell syntax, use `parser = { kind = "json" }` and write the same pair-array shape:

```json
[["atk",10],["hp",20]]
```

## tagged_columns

Use `tagged_columns` when one `union<T>` value should be edited across multiple Excel or CSV columns. It is only valid on a table field whose type is exactly `union<T>`. It is intentionally not valid for `optional<union<T>>`, `list<union<T>>`, `set<union<T>>`, or other containers.

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

CSV headers and rows:

```csv
id,type,quest_id,item_id,count
1,QuestCompleted,5002,,
2,HasItem,,1001,2
```

The tag column contains the union variant name. Only fields for the selected variant may contain values. With the default prefix, a field named `condition` projects columns such as `condition.type`, `condition.quest_id`, and `condition.item_id`. Use `prefix = ""` only when the projected columns should live at the table's top level.

Sora rejects projected column name conflicts, for example a normal table field named `type` plus `prefix = ""` for a union whose tag is also `type`.

`tagged_columns` also does not recursively project nested structs or nested unions inside variant fields. Variant fields can still use single-cell parsers such as `tuple`, `split`, `map`, or `json`. If a variant needs a large nested object or repeated nested objects, model that data as a dedicated table and reference or derive it instead of widening the union row.

For generated XLSX templates, columns projected from the same `tagged_columns` field share the same header color. The tag column uses the same color group with stronger emphasis.

## json

Use `json` for nested values, unions inside containers, nested collections, and any shape that needs explicit escaping.

```toml
[[tables.fields]]
name = "actions"
type = "list<union<RewardAction>>"
parser = { kind = "json" }
```

Cell:

```json
[
  {"type":"AddItem","item_id":1007,"count":3},
  {"type":"UnlockStage","stage_id":9002}
]
```

For one union value:

```toml
[[tables.fields]]
name = "condition"
type = "union<EventCondition>"
parser = { kind = "json" }
```

Cell:

```json
{"type":"QuestCompleted","quest_id":5002}
```

For `map<K,V>`, JSON uses an array of pairs, not a JSON object:

```json
[["atk",10],["hp",20]]
```

## Choosing a Parser

| Need | Prefer |
| --- | --- |
| Flat list of primitive values | `split` |
| One compact struct | `tuple` |
| One struct spread across columns | `columns` |
| Repeated compact structs | `tuple_list` |
| Simple key/value pairs | `map` |
| One union spread across columns | `tagged_columns` |
| Nested values, unions in containers, escaping, or JSON-shaped cells | `json` |
