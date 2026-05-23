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
required = true
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

索引用于在支持的目标语言中生成 lookup helper。

```toml
[[tables.indexes]]
name = "by_name"
fields = ["name"]
unique = true
```

unique index 要求每个 key 最多对应一行。非 unique index 会把匹配行分组。

## Validation

加载 source 数据后，Sora 会校验表行：

- required 字段必须存在，除非有 default；
- map 表的 key 字段必须唯一；
- enum value 必须有效；
- reference 必须指向已有行；
- numeric range 和 length range 必须通过；
- parser 输出必须匹配声明的字段类型。
