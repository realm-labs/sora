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

聚合字段会把子表行收集到父表行上。它适合这种情况：编辑时希望数据保持成多张规范化表，运行时又希望父对象直接带着子列表。

例如任务奖励可以拆成两张表：

`Quest`：

| id | name |
| --- | --- |
| 1001 | First Quest |
| 1002 | Second Quest |

`QuestReward`：

| quest_id | sort_order | item_id | count |
| --- | --- | --- | --- |
| 1001 | 1 | 2001 | 10 |
| 1001 | 2 | 2002 | 1 |
| 1002 | 1 | 2003 | 5 |

运行时可能希望 `Quest` 里直接有 `rewards: list<Reward>`。这时在 `Quest` 表上声明一个聚合字段：

```toml
[[tables.fields]]
name = "rewards"
type = "list<struct<Reward>>"
source_table = "QuestReward"
parent_key = "id"
child_key = "quest_id"
order_by = "sort_order"
```

含义是：

- `source_table = "QuestReward"`：从 `QuestReward` 这张子表收集行。
- `parent_key = "id"`：父表 `Quest` 用自己的 `id` 作为匹配值。
- `child_key = "quest_id"`：子表 `QuestReward` 用 `quest_id` 和父表匹配。
- `order_by = "sort_order"`：匹配到多行时，按子表里的 `sort_order` 字段升序排序。

对上面的数据，`Quest.id = 1001` 会收集两条奖励行，并按 `sort_order` 排成 `2001`、`2002`。

## 聚合选项

聚合相关字段只有这几个：

| 选项 | 必填 | 含义 |
| --- | --- | --- |
| `source_table` | 是 | 子表名。Sora 会从这张表扫描匹配行。 |
| `parent_key` | 是 | 父表上的字段名。每个父行用这个字段值参与匹配。 |
| `child_key` | 是 | 子表上的字段名。子行的这个字段值等于父行 `parent_key` 时，就会被收集。 |
| `order_by` | 否 | 子表上的字段名。存在时，匹配到的子行按这个字段升序排序。 |

`source_table`、`parent_key`、`child_key` 必须一起出现。只写其中一部分是不完整的聚合配置。

`order_by` 目前只是一个字段名，不是表达式。当前没有 `desc`、多字段排序、过滤条件或自定义排序语法。省略 `order_by` 时，匹配行保持源表读取顺序。

`order_by` 指向的字段必须存在于子表中。它通常会是 `i32` 这类排序字段，例如 `sort_order`、`seq`、`rank`。排序是升序。

聚合字段本身必须是 `list<struct<...>>`。结构体字段会从子表同名字段复制：

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

这里的 `Reward.item_id` 和 `Reward.count` 都必须能在 `QuestReward` 表中找到同名字段，并且类型兼容。

聚合字段不能同时声明 `default`。它的值来自匹配到的子表行；没有匹配行时就是空列表。

## 同一子表的多处聚合

多张父表可以聚合同一张子表。聚合不会消费或移动子表行；它只是读取子表，然后把匹配结果复制成父表上的嵌套列表。

例如 `Quest` 和 `QuestPreview` 都可以从 `QuestReward` 聚合奖励：

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

如果 `Quest.id = 1001` 和 `QuestPreview.id = 1001` 都存在，它们都会得到 `QuestReward.quest_id = 1001` 的奖励列表。Sora 不会认为这条子行已经被 `Quest` 使用过，也不会从 `QuestReward` 中删除它。

子表本身仍然是一张普通表，也会继续出现在导出的数据中，除非你通过 scope 等其他机制把它排除。

聚合让可编辑表保持规范化，同时生成运行时模型可以暴露更方便的嵌套数据。
