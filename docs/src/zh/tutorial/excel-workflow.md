# Excel 工作流

Excel 支持围绕生成模板设计。schema 拥有表结构，Excel 是这个 schema 的可编辑投影。

## 生成模板

有两种生成 Excel 模板的方式。

第一种是直接运行命令：

```bash
sora excel-template --project project.toml --out generated/excel
```

这条命令的意思是：读取 `project.toml` 里的 schema，然后把 Excel 模板写到 `generated/excel` 目录。

第二种是把模板输出目录写进 `project.toml`，之后统一运行 `sora build`：

```toml
[build]
excel_templates = "generated/excel"
```

```bash
sora build --project project.toml
```

这两种方式生成的是同一类文件。区别只是：第一种只生成 Excel 模板；第二种会和 schema lock、codegen、export 等 build 输出一起执行。

`excel_templates` 不是输入数据目录，而是模板输出目录。真正的数据输入目录通常是 `[build].data_root` 或命令里的 `--data-root`。

每个表最终生成到哪个 workbook 和 sheet，由表自己的 source 决定：

```toml
[[tables]]
name = "Item"

[tables.source]
format = "xlsx"
file = "Core.xlsx"
sheet = "Item"

[[tables]]
name = "Quest"

[tables.source]
format = "xlsx"
file = "Core.xlsx"
sheet = "Quest"
```

上面的配置会在 `generated/excel/Core.xlsx` 中生成两个 sheet：`Item` 和 `Quest`。

如果另一个表写成：

```toml
[tables.source]
format = "xlsx"
file = "Battle.xlsx"
sheet = "Skill"
```

它就会生成到另一个文件：`generated/excel/Battle.xlsx` 的 `Skill` sheet。

## 表头行

生成的 sheet 包含多行表头：

| Row | Purpose |
| --- | --- |
| `@table` metadata | 表名、mode、key、scope 和 schema hash。 |
| `#name` | 面向表格编辑的显示名行。 |
| `#field` | Sora 读取的稳定 schema 字段名。 |
| `#type` | 类型提示，例如 `i32`、`enum<ItemType>` 或 `struct<Cost>(kind: enum<ResourceKind>, id: i32, count: i32)`。 |
| `#scope` | 每个字段的 scope 信息。 |
| `#input` | key、parser、range、length 或派生字段来源等输入提示。 |
| `#desc` | 给编辑者和 reviewer 看的字段注释。 |

数据行从生成表头之后开始。

## 用户应该编辑什么

用户应该编辑数据行，不应该手工维护 Excel 里的字段名、类型、key 元数据、输入提示或校验规则。这些行会从 schema 重新生成。

如果某列的 `#input` 以 `from=` 开头，这个字段是从另一张表派生出来的。保留该列里的生成占位内容，去编辑对应的子表行。

schema 变更后，重新生成模板，然后迁移或粘贴已有数据行。这样既保留了电子表格编辑体验，也避免 Excel 变成第二套 schema 语言。

## 常见字段形状

简单字段直接映射到单元格：

| id | name | max_stack |
| --- | --- | --- |
| 1001 | Iron Sword | 1 |

结构化值可以用 parser 在单元格里写成紧凑形式：

```toml
[[tables.fields]]
name = "price"
type = "struct<ResourceCost>"
parser = { kind = "tuple" }
comment = "Tuple: kind,id,count"
```

示例单元格：

```text
Item,1001,3
```

集合可以使用 JSON 或 map 风格 parser：

```toml
[[tables.fields]]
name = "tags"
type = "set<string>"
parser = { kind = "json" }
default = "[\"misc\"]"

[[tables.fields]]
name = "attributes"
type = "map<string,i32>"
parser = { kind = "map" }
comment = "Map pairs: key,value|key,value"
```

示例单元格：

```text
["starter","melee"]
attack,12|speed,2
```
