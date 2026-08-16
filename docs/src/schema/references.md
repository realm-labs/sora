# References and Derived Fields

References let one table point to another table's primary key. Derived fields copy or assemble data from matching rows in another table.

| Feature | What source data stores | What runtime model gets |
| --- | --- | --- |
| `ref<Item.id>` | The target row id, such as `1001`. | The id value or a target-specific wrapper. |
| `from = { ... }` | Rows stay in a child table. | The parent row receives a copied/nested value. |

Use `ref` when the relationship itself should remain an id. Use `from` when exported data should contain a convenient nested field.

The target of a `ref` must be a `mode = "map"` table, and the referenced field must be that table's `key`.

## References

```toml
[[tables.fields]]
name = "required_item"
type = "ref<Item.id>"
```

Sora validates that every value points to an existing row in the referenced table.

References are still stored as values in source data. The generated runtime can expose them as key values or target-specific wrapper types depending on the language backend.

References can be nested in containers such as `list<ref<Item.id>>`, `set<ref<Item.id>>`, or `optional<ref<Item.id>>`. The same primary-key rule applies to the inner `ref`.

### Rust strong table keys

Rust code generation gives every primitive `map` table key a table-owned nominal newtype. A table `Item` keyed by `id: u32` generates `ItemId`; both `Item.id` and every `ref<Item.id>` use that exact type, including references nested inside containers, structs, and unions. Another `u32`-keyed table receives a different Rust type, so its id cannot be passed accidentally.

The wrapper is `#[repr(transparent)]` and uses transparent Serde, so bundles, JSON, schema locks, and other data formats still store the original scalar. It exposes explicit `from_raw` and `raw`/`as_str` accessors without exposing the tuple field or implementing `Deref` to the primitive. String and text key tables also provide an explicit `get_str(&str)` lookup helper.

Enum primary keys use the generated enum directly. If a table key is itself `ref<Other.id>`, it reuses `Other`'s final key type rather than adding another wrapper. Qualified table names keep the key type in the owning table module; Sora does not create a root re-export.

This is the standard Rust representation and has no compatibility switch. Other language generators currently retain their existing key representation.

## Derived Fields

A derived field is not read from the current table's cell. It is built from matching rows in another table.

This keeps editable data normalized while generated runtime models can expose convenient nested values. For example, quest rewards can be stored as two tables:

`Quest`:

| id | name |
| --- | --- |
| 1001 | First Quest |
| 1002 | Second Quest |

`QuestReward`:

| quest_id | sort_order | item_id | count |
| --- | --- | --- | --- |
| 1001 | 1 | 2001 | 10 |
| 1001 | 2 | 2002 | 1 |
| 1002 | 1 | 2003 | 5 |

At runtime, `Quest` may want a direct `rewards: list<Reward>` field. Declare that the field comes from `QuestReward`:

```toml
[[structs]]
name = "Reward"

[[structs.fields]]
name = "item_id"
type = "ref<Item.id>"

[[structs.fields]]
name = "count"
type = "i32"

[[tables]]
id = "quest"
name = "Quest"
mode = "map"
key = "id"

[[tables.fields]]
name = "id"
type = "i32"

[[tables.fields]]
name = "name"
type = "string"

[[tables.fields]]
name = "rewards"
type = "list<struct<Reward>>"
from = { table = "QuestReward", parent_key = "id", child_key = "quest_id", order_by = "sort_order" }

[[tables]]
id = "questreward"
name = "QuestReward"
mode = "list"

[[tables.fields]]
name = "quest_id"
type = "ref<Quest.id>"

[[tables.fields]]
name = "sort_order"
type = "i32"

[[tables.fields]]
name = "item_id"
type = "ref<Item.id>"

[[tables.fields]]
name = "count"
type = "i32"
```

This means:

- `from.table = "QuestReward"`: read matching rows from the `QuestReward` child table.
- `from.parent_key = "id"`: use the parent row's `Quest.id` value for matching.
- `from.child_key = "quest_id"`: match child rows where `QuestReward.quest_id` equals the parent key.
- `from.order_by = "sort_order"`: when several child rows match, sort them by the child table's `sort_order` field in ascending order.

With the example data above, `Quest.id = 1001` receives two reward rows, ordered as `2001`, then `2002`.

The exported parent row is shaped as if `rewards` had been written directly on `Quest`:

```json
{
  "id": 1001,
  "name": "First Quest",
  "rewards": [
    {"item_id": 2001, "count": 10},
    {"item_id": 2002, "count": 1}
  ]
}
```

The field type controls how many child rows may match:

| Field type | Match count | Result when no row matches |
| --- | --- | --- |
| `list<T>` | zero or more | empty list |
| `optional<T>` | zero or one | `null` |
| `T` | exactly one | validation error |
| `map<K,V>` | zero or more, with unique keys | empty map |

If `T` or `optional<T>` matches more than one child row, Sora reports an error.

## Deriving a Map

A derived map uses the same parent/child join as a derived list. Add `from.key` to select the child field that becomes each map entry's key, and use `from.field` to select the value:

```scon
tables {
  Item {
    mode = "map"
    key = "id"
    fields {
      id = "i32"
      drop_rates {
        type = "map<string,i32>"
        from {
          table = "ItemDropRate"
          parent_key = "id"
          child_key = "item_id"
          key = "rarity"
          field = "rate"
        }
      }
    }
  }

  ItemDropRate {
    mode = "list"
    fields {
      item_id = "ref<Item.id>"
      rarity = "string"
      rate = "i32"
    }
  }
}
```

Editable rows stay simple and need no synthetic child-row id:

| item_id | rarity | rate |
| ---: | --- | ---: |
| 1001 | Common | 80 |
| 1001 | Epic | 20 |

For `Item.id = 1001`, Sora materializes:

```json
"drop_rates": [["Common", 80], ["Epic", 20]]
```

Sora uses pair arrays for maps so non-string key types remain unambiguous. Generated runtimes expose the target language's normal map or dictionary type.

The rules are deliberately narrow:

- An exact `map<K,V>` derived field must declare `from.key`; `from.key` is invalid on non-map fields.
- The child key field must exist and its type must exactly match `K`. Sora resolves `ref<Table.field>` to the referenced field type before comparing; it performs no implicit conversion.
- `from.field`, when present, must be type-compatible with `V`.
- Without `from.field`, `V` must be a struct assembled from same-named child fields.
- Duplicate map keys among rows matched to the same parent are an error. Enum aliases count as their canonical enum value for duplicate detection.
- No matching child rows produce an empty map.
- `order_by` may control deterministic pair order, although map lookup semantics do not depend on order.

Studio derives reverse usages from the existing `from` relationships. A source table can therefore show entries such as `Item.drop_rates (item_id -> id, key=rarity, field=rate)` without adding a second ownership declaration to the schema. One source table may still feed several parent fields or tables.

## Copying One Child Field

Without `from.field`, Sora assembles a struct from child table fields with the same names as the struct fields.

When the parent should receive one field from the child row instead, set `from.field`:

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
name = "condition"
type = "union<EventCondition>"
from = { table = "EventConditionEntry", parent_key = "id", child_key = "event_id", field = "value" }

[[tables]]
id = "eventconditionentry"
name = "EventConditionEntry"
mode = "list"

[[tables.fields]]
name = "event_id"
type = "ref<Event.id>"

[[tables.fields]]
name = "value"
type = "union<EventCondition>"
parser = { kind = "tagged_columns", prefix = "" }
```

This means `Event.condition` receives `EventConditionEntry.value` for the child row whose `event_id` matches `Event.id`. The child table may still contain helper columns such as `id`, `event_id`, notes, or sort fields; only the `value` field named by `from.field` is copied into the parent field.

In Excel, `EventConditionEntry` can look like this:

| A | B | C | D | E |
| --- | --- | --- | --- | --- |
| `event_id` | `type` | `quest_id` | `item_id` | `count` |
| `1` | `QuestCompleted` | `5002` |  |  |
| `2` | `HasItem` |  | `1001` | `2` |

## From Options

The `from` object has these options:

| Option | Required | Meaning |
| --- | --- | --- |
| `table` | yes | Child table name. Sora scans this table for matching rows. |
| `parent_key` | yes | Field name on the parent table. Each parent row uses this field value for matching. |
| `child_key` | yes | Field name on the child table. A child row is selected when this value equals the parent key. |
| `key` | for `map<K,V>` | Child field copied into each map entry's key. Invalid for non-map derived fields. |
| `field` | no | Field name on the child table. When present, Sora copies this field's value instead of assembling a struct from the child row. |
| `order_by` | no | Field name on the child table. When present, matched child rows are sorted by this field in ascending order. |

`order_by` is a field name, not an expression. There is no `desc`, multi-field ordering, filtering, or custom sort syntax. If `order_by` is omitted, matched rows keep the source table read order.

The `order_by` field must exist on the child table. It is usually an `i32` ordering field such as `sort_order`, `seq`, or `rank`. Sorting is ascending.

Without `from.field`, the derived value type must be a struct: `list<struct<...>>`, `struct<...>`, `optional<struct<...>>`, or `map<K,struct<...>>`. Struct fields are copied from child table fields with the same names:

```toml
[[structs]]
name = "Reward"

[[structs.fields]]
name = "item_id"
type = "ref<Item.id>"

[[structs.fields]]
name = "count"
type = "i32"
```

Here `Reward.item_id` and `Reward.count` must both exist as compatible fields on `QuestReward`.

With `from.field`, the derived value type must be compatible with that child field. For example, `type = "union<EventCondition>"` can derive from a child field `value` whose type is also `union<EventCondition>`.

A derived field cannot also declare `default`. Its value comes from matched child rows.

## Multiple Derived Fields from One Child Table

Several parent tables can derive fields from the same child table. This does not consume or move child rows. It reads the child table and copies matching values into each parent field.

For example, both `Quest` and `QuestPreview` can receive rewards from `QuestReward`:

```toml
[[tables]]
id = "quest"
name = "Quest"
mode = "map"
key = "id"

[[tables.fields]]
name = "rewards"
type = "list<struct<Reward>>"
from = { table = "QuestReward", parent_key = "id", child_key = "quest_id", order_by = "sort_order" }

[[tables]]
id = "questpreview"
name = "QuestPreview"
mode = "map"
key = "id"

[[tables.fields]]
name = "rewards"
type = "list<struct<Reward>>"
from = { table = "QuestReward", parent_key = "id", child_key = "quest_id", order_by = "sort_order" }
```

If both `Quest.id = 1001` and `QuestPreview.id = 1001` exist, both parent rows receive the reward list from `QuestReward.quest_id = 1001`. Sora does not mark the child row as already used by `Quest`, and it does not remove the row from `QuestReward`.
