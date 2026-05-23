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

Aggregation fields collect child rows onto a parent row.

```toml
[[tables.fields]]
name = "rewards"
type = "list<struct<Reward>>"
source_table = "QuestReward"
parent_key = "id"
child_key = "quest_id"
order_by = "sort_order"
```

This says: for each parent row, collect rows from `QuestReward` where `Quest.id == QuestReward.quest_id`, optionally sorted by `sort_order`.

Aggregation keeps editable tables normalized while generated runtime models can expose convenient nested data.
