# References and Aggregation

References let one table point to another table's key field.

## References

```toml
[[tables.fields]]
name = "required_item"
type = "ref<Item.id>"
required = true
```

Sora validates that every value points to an existing row in the referenced table.

References are still stored as values in source data. The generated runtime can expose them as key values or target-specific wrapper types depending on the language backend.

## Aggregation

Aggregation fields collect child rows onto a parent row. They are useful when editable data should stay normalized across tables, while runtime models should expose convenient nested values.

For example, quest rewards can be stored as two tables:

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

At runtime, `Quest` may want a direct `rewards: list<Reward>` field. Declare a list aggregation field on the `Quest` table:

```toml
[[tables.fields]]
name = "rewards"
type = "list<struct<Reward>>"
source_table = "QuestReward"
parent_key = "id"
child_key = "quest_id"
order_by = "sort_order"
```

This means:

- `source_table = "QuestReward"`: collect rows from the `QuestReward` child table.
- `parent_key = "id"`: use the parent table's `Quest.id` value for matching.
- `child_key = "quest_id"`: match child rows where `QuestReward.quest_id` equals the parent key.
- `order_by = "sort_order"`: when several child rows match, sort them by the child table's `sort_order` field in ascending order.

With the example data above, `Quest.id = 1001` collects two reward rows, ordered as `2001`, then `2002`.

For a one-row child value, use a non-list field type. `T` requires exactly one matching child row, while `optional<T>` allows zero or one matching child row. If more than one child row matches, Sora reports an error.

When the parent should receive one field from the child row instead of a struct assembled from the whole row, declare `value_field`:

```toml
[[tables.fields]]
name = "condition"
type = "union<EventCondition>"
source_table = "EventConditionEntry"
parent_key = "id"
child_key = "event_id"
value_field = "value"
```

This means `Event.condition` receives `EventConditionEntry.value` for the child row whose `event_id` matches `Event.id`. The child table may still contain helper columns such as `id`, `event_id`, notes, or sort fields; only `value_field` is copied into the parent field.

## Aggregation Options

Aggregation metadata has these options:

| Option | Required | Meaning |
| --- | --- | --- |
| `source_table` | yes | Child table name. Sora scans this table for matching rows. |
| `parent_key` | yes | Field name on the parent table. Each parent row uses this field value for matching. |
| `child_key` | yes | Field name on the child table. A child row is collected when this value equals the parent key. |
| `value_field` | no | Field name on the child table. When present, aggregation copies this field's value instead of assembling a struct from the child row. |
| `order_by` | no | Field name on the child table. When present, matched child rows are sorted by this field in ascending order. |

`source_table`, `parent_key`, and `child_key` must appear together. `value_field` and `order_by` are only valid as part of an aggregation field. Supplying only some aggregation metadata is an incomplete aggregation configuration.

`order_by` is currently just a field name, not an expression. There is no `desc`, multi-field ordering, filtering, or custom sort syntax. If `order_by` is omitted, matched rows keep the source table read order.

The `order_by` field must exist on the child table. It is usually an `i32` ordering field such as `sort_order`, `seq`, or `rank`. Sorting is ascending.

Without `value_field`, the aggregation value type must be a struct, either `list<struct<...>>`, `struct<...>`, or `optional<struct<...>>`. Struct fields are copied from child table fields with the same names:

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

With `value_field`, the aggregation value type must be compatible with that child field. For example, `type = "union<EventCondition>"` can aggregate from a child field `value` whose type is also `union<EventCondition>`.

An aggregation field cannot also declare `default`. Its value comes from matched child rows. If no rows match, `list<T>` becomes an empty list, `optional<T>` becomes `null`, and `T` reports an error.

## Multiple Aggregations from One Child Table

Several parent tables can aggregate from the same child table. Aggregation does not consume or move child rows. It reads the child table and copies matching values into each parent field.

For example, both `Quest` and `QuestPreview` can aggregate rewards from `QuestReward`:

```toml
[[tables]]
name = "Quest"
mode = "map"
key = "id"

[[tables.fields]]
name = "rewards"
type = "list<struct<Reward>>"
source_table = "QuestReward"
parent_key = "id"
child_key = "quest_id"
order_by = "sort_order"

[[tables]]
name = "QuestPreview"
mode = "map"
key = "id"

[[tables.fields]]
name = "rewards"
type = "list<struct<Reward>>"
source_table = "QuestReward"
parent_key = "id"
child_key = "quest_id"
order_by = "sort_order"
```

If both `Quest.id = 1001` and `QuestPreview.id = 1001` exist, both parent rows receive the reward list from `QuestReward.quest_id = 1001`. Sora does not mark the child row as already used by `Quest`, and it does not remove the row from `QuestReward`.

The child table remains a normal table and is still present in exported data unless you exclude it through another mechanism such as scope.

Aggregation keeps editable tables normalized while generated runtime models can expose convenient nested data.
