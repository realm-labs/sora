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

For a single union value in Excel or CSV, write JSON object text:

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

If you do not want JSON in Excel or CSV, the new part is the entry table's `union<T>` field: it can use `parser = { kind = "tagged_columns", prefix = "" }` so the workbook has normal columns such as `type`, `quest_id`, `item_id`, and `count`. Any parent table reference to that entry row still uses the existing `ref<Table.key>` or `list<ref<Table.key>>` semantics. See [Cell Parsers](parsers.md#tagged-union-columns) for the exact rules.

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
