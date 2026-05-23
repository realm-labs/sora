# Cell Parsers

Parsers control how cell-based inputs such as Excel and CSV turn one cell into a typed value. They also apply to string defaults. TOML row data can usually use native TOML arrays and tables instead of cell parsers.

```toml
[[tables.fields]]
name = "tags"
type = "list<string>"
parser = { kind = "split", separator = "|" }
```

Parser options are string values. Unknown parser kinds, unsupported options, and empty option values fail during schema normalization.

## Default Cell Parsing

If a field has no `parser`, Sora uses type-aware default parsing:

| Type | Cell format |
| --- | --- |
| `bool` | Boolean cells, `true`, `false`, or numeric cells where zero is false and non-zero is true. |
| `i32`, `i64`, `ref<Table.field>` | Integer cells, integer text, or whole-number float cells. |
| `f32`, `f64` | Numeric cells or numeric text. |
| `string`, `enum<Name>` | Cell display text. |
| `struct<Name>`, `union<Name>` | JSON object text. |
| `list<T>`, `set<T>`, `array<T,N>` | Comma-separated text. A surrounding `[...]` pair is allowed. |
| `map<K,V>` | JSON array of two-item pairs, for example `[["atk",10],["hp",20]]`. |
| `optional<T>` | Empty cell becomes `null`; otherwise the inner `T` is parsed. |

For comma-separated collections, primitive items are parsed by type. Struct and union collection items must be JSON object text. Nested collections cannot be represented with a single separator; use `json`.

## Built-In Parsers

| Parser | Valid target types | Options | Default format |
| --- | --- | --- | --- |
| `split` | `list<T>`, `set<T>`, `array<T,N>`, or `optional` around those types | `separator`, default `,` | `a,b,c` or `[a,b,c]` |
| `tuple` | `struct<T>` or `optional<struct<T>>` | `separator`, default `,` | Values in the struct field declaration order, for example `Gold,0,100` |
| `tuple_list` | `list<struct<T>>`, `set<struct<T>>`, `array<struct<T>,N>`, or `optional` around those types | `separator`, default `,`; `item_separator`, default `|` | `Gold,0,100|Gem,0,5` |
| `map` | `map<K,V>` or `optional<map<K,V>>` | `separator`, default `,`; `item_separator`, default `|` | `atk,10|hp,20` |
| `json` | Any type | none | JSON value matching the field type |

`array<T,N>` checks the parsed item count. `tuple` checks the value count against the referenced struct's field count.

## JSON Shapes

`json` is the safest parser for nested values, unions, nested collections, and maps that need unambiguous escaping.

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

Example cells:

```json
{"type":"QuestCompleted","quest_id":5002}
```

```json
[
  {"type":"AddItem","item_id":1007,"count":3},
  {"type":"UnlockStage","stage_id":9002}
]
```

For `map<K,V>`, JSON uses an array of pairs, not a JSON object:

```json
[["atk",10],["hp",20]]
```

## Tuple Field Order

`tuple` and `tuple_list` use the referenced struct's schema field order. For this struct:

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

`parser = { kind = "tuple" }` expects:

```text
Gold,0,100
```

If a nested struct field has its own parser, that nested parser is used while parsing the tuple item.
