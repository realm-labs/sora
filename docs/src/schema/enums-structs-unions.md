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

Use structs for nested values that appear in many places. A field can reference a struct with `type = "struct<ResourceCost>"`.

Struct fields use the same field properties as table fields, including `name`, `type`, `required`, `default`, `comment`, `range`, `length`, `parser`, and `scope`. Table-specific properties such as `key` and aggregation metadata are not meaningful for normal struct fields. See [Types](types.md#field-rules) for the full field reference.

In cell-based inputs, a struct field can be written as JSON object text by default:

```json
{"kind":"Gold","id":0,"count":100}
```

For compact cells, declare `parser = { kind = "tuple" }` on the field that references the struct. Tuple values follow the struct field order:

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

Use unions when a field can contain one of several tagged shapes. Examples include conditions, rewards, triggers, and scripted actions.

The union `tag` defaults to `type` if omitted. Source data must include that tag with the variant name. The remaining fields must match the selected variant; unknown fields and missing required variant fields are validation errors.

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
