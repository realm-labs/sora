# References and Derived Fields

References let one table point to another table's primary key. The target table must be `mode = "map"`, and the referenced field must be that table's `key`.

## References

```toml
[[tables.fields]]
name = "required_item"
type = "ref<Item.id>"
required = true
```

Sora validates that every value points to an existing row in the referenced table.

References are still stored as values in source data. The generated runtime can expose them as key values or target-specific wrapper types depending on the language backend.

References can be nested in containers such as `list<ref<Item.id>>`, `set<ref<Item.id>>`, or `optional<ref<Item.id>>`. The same primary-key rule applies to the inner `ref`.

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
[[tables.fields]]
name = "rewards"
type = "list<struct<Reward>>"
from = { table = "QuestReward", parent_key = "id", child_key = "quest_id", order_by = "sort_order" }
```

This means:

- `from.table = "QuestReward"`: read matching rows from the `QuestReward` child table.
- `from.parent_key = "id"`: use the parent row's `Quest.id` value for matching.
- `from.child_key = "quest_id"`: match child rows where `QuestReward.quest_id` equals the parent key.
- `from.order_by = "sort_order"`: when several child rows match, sort them by the child table's `sort_order` field in ascending order.

With the example data above, `Quest.id = 1001` receives two reward rows, ordered as `2001`, then `2002`.

The field type controls how many child rows may match:

| Field type | Match count | Result when no row matches |
| --- | --- | --- |
| `list<T>` | zero or more | empty list |
| `optional<T>` | zero or one | `null` |
| `T` | exactly one | validation error |

If `T` or `optional<T>` matches more than one child row, Sora reports an error.

## Copying One Child Field

Without `from.field`, Sora assembles a struct from child table fields with the same names as the struct fields.

When the parent should receive one field from the child row instead, set `from.field`:

```toml
[[tables.fields]]
name = "condition"
type = "union<EventCondition>"
from = { table = "EventConditionEntry", parent_key = "id", child_key = "event_id", field = "value" }
```

This means `Event.condition` receives `EventConditionEntry.value` for the child row whose `event_id` matches `Event.id`. The child table may still contain helper columns such as `id`, `event_id`, notes, or sort fields; only `from.field` is copied into the parent field.

## From Options

The `from` object has these options:

| Option | Required | Meaning |
| --- | --- | --- |
| `table` | yes | Child table name. Sora scans this table for matching rows. |
| `parent_key` | yes | Field name on the parent table. Each parent row uses this field value for matching. |
| `child_key` | yes | Field name on the child table. A child row is selected when this value equals the parent key. |
| `field` | no | Field name on the child table. When present, Sora copies this field's value instead of assembling a struct from the child row. |
| `order_by` | no | Field name on the child table. When present, matched child rows are sorted by this field in ascending order. |

`order_by` is a field name, not an expression. There is no `desc`, multi-field ordering, filtering, or custom sort syntax. If `order_by` is omitted, matched rows keep the source table read order.

The `order_by` field must exist on the child table. It is usually an `i32` ordering field such as `sort_order`, `seq`, or `rank`. Sorting is ascending.

Without `from.field`, the derived value type must be a struct, either `list<struct<...>>`, `struct<...>`, or `optional<struct<...>>`. Struct fields are copied from child table fields with the same names:

```toml
[[structs]]
name = "Reward"

[[structs.fields]]
name = "item_id"
type = "ref<Item.id>"
required = true

[[structs.fields]]
name = "count"
type = "i32"
required = true
```

Here `Reward.item_id` and `Reward.count` must both exist as compatible fields on `QuestReward`.

With `from.field`, the derived value type must be compatible with that child field. For example, `type = "union<EventCondition>"` can derive from a child field `value` whose type is also `union<EventCondition>`.

A derived field cannot also declare `default`. Its value comes from matched child rows.

## Multiple Derived Fields from One Child Table

Several parent tables can derive fields from the same child table. This does not consume or move child rows. It reads the child table and copies matching values into each parent field.

For example, both `Quest` and `QuestPreview` can receive rewards from `QuestReward`:

```toml
[[tables]]
name = "Quest"
mode = "map"
key = "id"

[[tables.fields]]
name = "rewards"
type = "list<struct<Reward>>"
from = { table = "QuestReward", parent_key = "id", child_key = "quest_id", order_by = "sort_order" }

[[tables]]
name = "QuestPreview"
mode = "map"
key = "id"

[[tables.fields]]
name = "rewards"
type = "list<struct<Reward>>"
from = { table = "QuestReward", parent_key = "id", child_key = "quest_id", order_by = "sort_order" }
```

If both `Quest.id = 1001` and `QuestPreview.id = 1001` exist, both parent rows receive the reward list from `QuestReward.quest_id = 1001`. Sora does not mark the child row as already used by `Quest`, and it does not remove the row from `QuestReward`.
