# Enums, Structs, and Unions

These definitions let schemas model more than flat tables.

## Enums

```toml
[[enums]]
name = "Rarity"
values = ["Common", "Uncommon", "Rare", "Epic", "Legendary"]
```

Enums keep source data readable while generated code receives a constrained type.

Aliases can keep imported or legacy names readable:

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

[[structs.fields]]
name = "id"
type = "i32"

[[structs.fields]]
name = "count"
type = "i32"
range = [1, 999999]
```

Use structs for nested values that appear in many places. A field can reference a struct with `type = "struct<ResourceCost>"`.

Struct fields use the same field properties as table fields, including `name`, `type`, `default`, `comment`, `range`, `length`, `parser`, and `scope`. Table-specific properties such as `key` and `from` are not meaningful for normal struct fields. See [Types](types.md#field-rules) for the full field reference.

In cell-based inputs, a struct field can be written as JSON object text by default:

```json
{"kind":"Gold","id":0,"count":100}
```

For compact cells, declare `parser = { kind = "tuple" }` on the field that references the struct. Tuple values follow the struct field order:

```text
Gold,0,100
```

## Unions

Use a union when one field can contain different shapes. For example, an event condition might be either "quest completed" or "player has item":

```json
{"type":"QuestCompleted","quest_id":5002}
```

```json
{"type":"HasItem","item_id":1001,"count":2}
```

The `type` value selects which variant is present. The rest of the fields depend on that variant.

```toml
[[unions]]
name = "RewardAction"
tag = "type"

[[unions.variants]]
name = "AddItem"

[[unions.variants.fields]]
name = "item_id"
type = "ref<Item.id>"

[[unions.variants.fields]]
name = "count"
type = "i32"

[[unions.variants]]
name = "UnlockStage"

[[unions.variants.fields]]
name = "stage_id"
type = "ref<Stage.id>"
```

Use unions when a field can contain one of several tagged shapes. Examples include conditions, rewards, triggers, and scripted actions.

The union `tag` defaults to `type` if omitted. Source data must include that tag with the variant name. The remaining fields must match the selected variant; unknown fields and missing non-optional variant fields are validation errors.

The most direct Excel or CSV form is JSON object text in one cell:

| Field type | Cell value |
| --- | --- |
| `union<RewardAction>` | `{"type":"AddItem","item_id":1001,"count":2}` |

For a list of union values, declare `parser = { kind = "json" }` and write a JSON array:

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

If you do not want JSON in Excel or CSV cells, a single `union<T>` field can be expanded into several columns. This `action` field is one union value:

```toml
[[tables.fields]]
name = "action"
type = "union<RewardAction>"
parser = { kind = "tagged_columns" }
```

The Excel sheet then has columns like this:

| A | B | C | D | E | F |
| --- | --- | --- | --- | --- | --- |
| `id` | `name` | `action.type` | `action.item_id` | `action.count` | `action.stage_id` |
| `1` | `Give Sword` | `AddItem` | `1001` | `2` |  |
| `2` | `Open Stage` | `UnlockStage` |  |  | `9002` |

`action.type` contains the variant name. An `AddItem` row fills only `item_id` and `count`; an `UnlockStage` row fills only `stage_id`. Columns for other variants stay empty.

`tagged_columns` is only valid on a field whose type is exactly `union<T>`; it cannot be applied directly to `list<union<T>>`. When a parent field needs several union values, put each union value in a child row and derive the parent list from that child table:

```toml
[[tables.fields]]
name = "actions"
type = "list<union<RewardAction>>"
from = { table = "EventActionEntry", parent_key = "id", child_key = "event_id", field = "value", order_by = "seq" }

[[tables]]
name = "EventActionEntry"
mode = "list"

[[tables.fields]]
name = "event_id"
type = "ref<EventRule.id>"

[[tables.fields]]
name = "seq"
type = "i32"

[[tables.fields]]
name = "value"
type = "union<RewardAction>"
parser = { kind = "tagged_columns", prefix = "" }
```

The parent `EventRule` sheet keeps ordinary columns:

| A | B |
| --- | --- |
| `id` | `name` |
| `1` | `First Event` |

The child `EventActionEntry` sheet stores one action per row:

| A | B | C | D | E | F |
| --- | --- | --- | --- | --- | --- |
| `event_id` | `seq` | `type` | `item_id` | `count` | `stage_id` |
| `1` | `1` | `AddItem` | `1001` | `2` |  |
| `1` | `2` | `UnlockStage` |  |  | `9002` |

On export, `EventRule.actions` receives two union values ordered by `seq`. The `prefix = ""` option makes the child table columns use plain names such as `type`, `item_id`, `count`, and `stage_id`; do not use an empty prefix if those names conflict with other fields on the same table.

See [Cell Parsers](parsers.md#tagged_columns) for the exact column rules.

In TOML data files, unions can be written as normal nested tables:

```toml
[[rows]]
id = 1
condition = { type = "QuestCompleted", quest_id = 5002 }
actions = [
  { type = "AddItem", item_id = 1001, count = 2 },
  { type = "UnlockStage", stage_id = 9002 },
]
```
