# 引用和聚合

引用让一个表指向另一个表的 key 字段。

## References

```toml
[[tables.fields]]
name = "required_item"
type = "ref<Item.id>"
required = true
```

Sora 会校验每个值都指向被引用表中的已有行。

引用在源数据中仍然是值。生成运行时可以根据语言后端设计，把它暴露为 key value 或目标语言特定的 wrapper type。

## Aggregation

聚合字段会把子表行收集到父表行上。

```toml
[[tables.fields]]
name = "rewards"
type = "list<struct<Reward>>"
source_table = "QuestReward"
parent_key = "id"
child_key = "quest_id"
order_by = "sort_order"
```

含义是：对每个父行，从 `QuestReward` 中收集满足 `Quest.id == QuestReward.quest_id` 的行，并可选按 `sort_order` 排序。

聚合让可编辑表保持规范化，同时生成运行时模型可以暴露更方便的嵌套数据。
