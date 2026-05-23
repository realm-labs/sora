# 表

表是带 source 的行集合。表 schema 声明表模式、source 位置、字段和可选索引。

## Modes

| Mode | Shape | Typical Use |
| --- | --- | --- |
| `map` | 通过一个字段作为 key 的行集合。 | 道具、任务、等级、buff。 |
| `list` | 没有 keyed lookup 的有序行集合。 | 掉落项、权重池、有序步骤。 |
| `singleton` | 单行。 | 全局配置、调参常量。 |

```toml
[[tables]]
name = "Item"
mode = "map"
key = "id"

[[tables.fields]]
name = "id"
type = "i32"
key = true
```

对于 map 表，`key` 指定生成 lookup API 使用的字段。

## Source

```toml
[tables.source]
format = "xlsx"
file = "Core.xlsx"
sheet = "Item"
```

当项目或命令提供默认 source format 时，`format` 可以省略。导出和校验时，`file` 会基于命令的 `--data-root` 解析。

## Indexes

索引是表上的额外查询入口。它和 `mode = "map"` 的 `key` 不一样：

| 概念 | 用途 |
| --- | --- |
| table `key` | 表的主键。map 表必须靠它保证每行唯一，并生成主要的 `get(id)` 查询。 |
| `[[tables.indexes]]` | 额外查询方式。比如按名字查、按类型分组、按关卡查掉落。 |

例如 `Item` 表的主键是 `id`，运行时代码通常会按 `id` 取一个道具：

```toml
[[tables]]
name = "Item"
mode = "map"
key = "id"

[[tables.fields]]
name = "id"
type = "i32"
key = true

[[tables.fields]]
name = "name"
type = "string"

[[tables.fields]]
name = "item_type"
type = "enum<ItemType>"
```

如果还希望按 `name` 查道具，可以加一个 unique index：

```toml
[[tables.indexes]]
name = "by_name"
fields = ["name"]
unique = true
```

对应数据可以长这样：

| id | name | item_type |
| --- | --- | --- |
| 1001 | Iron Sword | Weapon |
| 1002 | Wood Shield | Armor |

`unique = true` 表示 `name` 不能重复。生成代码在支持该目标时会提供类似 `get_by_name("Iron Sword")` 的 helper，返回单行或空值。

如果希望按分类拿到多行，就用非 unique index：

```toml
[[tables.indexes]]
name = "by_item_type"
fields = ["item_type"]
unique = false
```

对应数据：

| id | name | item_type |
| --- | --- | --- |
| 1001 | Iron Sword | Weapon |
| 1002 | Bronze Axe | Weapon |
| 2001 | Wood Shield | Armor |

`unique = false` 表示同一个 key 可以匹配多行。生成代码在支持该目标时会提供类似 `get_by_item_type(ItemType::Weapon)` 的 helper，返回匹配行列表。

`fields` 是列表，因此 unique index 也可以表达组合唯一性：

```toml
[[tables.indexes]]
name = "by_world_stage"
fields = ["world", "stage"]
unique = true
```

这会要求 `(world, stage)` 组合不能重复。例如 `(1, 1)` 只能出现一次，`(1, 2)` 可以再出现一次。当前生成 lookup helper 主要支持非 singleton 表上的单字段 index；组合 index 更适合先用于数据校验。

## Validation

加载 source 数据后，Sora 会校验表行：

- 非 optional 字段必须存在，除非有 default；
- map 表的 key 字段必须唯一；
- enum value 必须有效；
- reference 必须指向已有行；
- numeric range 和 length range 必须通过；
- parser 输出必须匹配声明的字段类型。
