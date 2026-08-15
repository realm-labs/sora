# 表

表是带 source 的行集合。表 schema 声明表模式、source 位置、字段和可选索引。

## Modes

| Mode | Shape | Typical Use |
| --- | --- | --- |
| `map` | 通过一个字段作为 key 的行集合。 | 道具、任务、等级、buff。 |
| `list` | 没有 keyed lookup 的有序行集合。 | 掉落项、权重池、有序步骤。 |
| `singleton` | 单行。 | 全局配置、调参常量。 |

```scon
tables {
  Item {
    mode = "map"
    key = "id"
    fields { id = "i32" }
  }
}
```

对于 map 表，`key` 指定表的主键字段。Sora 会用它做行唯一性校验、生成 lookup API、生成 Excel 模板提示，并校验 `ref<Table.key>`。

## Source

```scon
source {
  format = "xlsx"
  file = "Core.xlsx"
  sheet = "Item"
}
```

当项目或命令提供默认 source format 时，`format` 可以省略。导出和校验时，`file` 会基于命令的 `--data-root` 解析。

内置 source format 包括 `xlsx`、`csv`、`toml`、`json` 和 `yaml`。JSON 和 YAML 表文件是 row object 数组：

```json
[
  { "id": 1001, "name": "Iron Sword" },
  { "id": 1002, "name": "Health Potion" }
]
```

对 JSON 和 YAML，`file` 也可以指向目录。这种情况下，Sora 会递归读取每个匹配的 `.json`、`.yaml` 或 `.yml` 文件，每个文件作为一条 row object，并按路径排序。

### 将一张 XLSX 表拆到多个 Sheet

一个使用 XLSX source 的逻辑表可以合并多个工作表。这适合按编辑维度自然分区的数据，例如活动配置按月份拆分：

一个 XLSX workbook 只归属于一个表定义。多个表不能共享同一个 `source.file`；不同 Schema 必须使用不同 workbook 文件。同一文件里的多个 Sheet 是同一个表的数据分片，不是不同表定义。

```scon
source {
  format = "xlsx"
  file = "Activity.xlsx"
  sheets = ["2026-01", "2026-02", "2026-03"]
}
```

每个选中的 Sheet 都使用该表 schema 声明的同一套列和 parser。Sora 会依次合并所有行，再把结果作为一张表统一校验和导出。因此主键、unique index、reference 和 `singleton` 校验都跨 Sheet 生效，而不是每个 Sheet 单独校验。

选择器可以包含 `*`（任意长度字符）和 `?`（单个字符）：

```scon
source {
  format = "xlsx"
  file = "Activity.xlsx"
  sheets = ["2026-*", "special"]
}
```

显式 Sheet 名按声明顺序合并；每个通配选择器匹配到的 Sheet 按名称排序。多个选择器重复命中同一个 Sheet 时只会合并一次。读取数据时，显式 Sheet 不存在，或通配选择器一个 Sheet 都没有匹配到，都会报错。

`sheets` 只支持 XLSX source，不能与 `sheet` 同时使用。创建全新模板时必须写明 Sheet 名，因为通配符无法凭空决定要创建哪些 Sheet；已有工作簿可以使用通配选择器。Excel 模板生成和同步会创建缺失的显式命名 Sheet，同步时会分别保留每个 Sheet 中的行。Studio 的行级数据修改目前会拒绝多 Sheet 表，因为合并后的行尚未保留来源 Sheet 信息；schema 编辑、读取、校验和导出不受影响。

## Indexes

索引是表上的额外查询入口。它和 `mode = "map"` 的 `key` 不一样：

| 概念 | 用途 |
| --- | --- |
| table `key` | 表的主键。map 表必须靠它保证每行唯一，并生成主要的 `get(id)` 查询。 |
| table `indexes` | keyed 的额外查询方式。比如按名字查、按类型分组、按关卡查掉落。 |

例如 `Item` 表的主键是 `id`，运行时代码通常会按 `id` 取一个道具：

```scon
tables {
  Item {
    mode = "map"
    key = "id"
    fields {
      id = "i32"
      name = "string"
      item_type = "enum<ItemType>"
    }
  }
}
```

如果还希望按 `name` 查道具，可以加一个 unique index：

```scon
indexes {
  by_name {
    fields = ["name"]
    unique = true
  }
}
```

对应数据可以长这样：

| id | name | item_type |
| --- | --- | --- |
| 1001 | Iron Sword | Weapon |
| 1002 | Wood Shield | Armor |

`unique = true` 表示 `name` 不能重复。生成代码在支持该目标时会提供类似 `get_by_name("Iron Sword")` 的 helper，返回单行或空值。

如果希望按分类拿到多行，就用非 unique index：

```scon
indexes {
  by_item_type = ["item_type"]
}
```

对应数据：

| id | name | item_type |
| --- | --- | --- |
| 1001 | Iron Sword | Weapon |
| 1002 | Bronze Axe | Weapon |
| 2001 | Wood Shield | Armor |

`unique = false` 表示同一个 key 可以匹配多行。生成代码在支持该目标时会提供类似 `get_by_item_type(ItemType::Weapon)` 的 helper，返回匹配行列表。

`fields` 是列表，因此 unique index 也可以表达组合唯一性：

```scon
indexes {
  by_world_stage {
    fields = ["world", "stage"]
    unique = true
  }
}
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
