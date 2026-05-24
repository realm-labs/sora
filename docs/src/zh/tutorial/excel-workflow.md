# Excel 工作流

Excel 支持围绕生成模板设计。schema 拥有表结构，Excel 是这个 schema 的可编辑投影。

## 生成模板

有两种生成 Excel 模板的方式。

第一种是直接运行命令：

```bash
sora excel-template --project project.toml --out generated/excel
```

这条命令的意思是：读取 `project.toml` 里的 schema，然后把 Excel 模板写到 `generated/excel` 目录。这个目录应该只放模板产物，可以删除后重新生成，不应该放手工编辑的源数据。

第二种是把模板输出目录写进 `project.toml`，之后统一运行 `sora build`：

```toml
[build]
excel_templates = "generated/excel"
```

```bash
sora build --project project.toml
```

这两种方式生成的是同一类文件。区别只是：第一种只生成 Excel 模板；第二种会和 schema lock、codegen、export 等 build 输出一起执行。

## 模板目录和数据目录

`excel_templates` 不是输入数据目录，而是模板输出目录。真正的数据输入目录通常是 `[build].data_root` 或命令里的 `--data-root`。

推荐把两个目录分开：

| 路径 | 作用 | 是否可重新生成 |
| --- | --- | --- |
| `generated/excel` | 带 schema 表头的生成 workbook 模板。 | 是 |
| `data` | export 和 build 读取的、已经填写行数据的文件。 | 否 |

不要把 `excel-template --out` 或 `[build].excel_templates` 指向已经有手工编辑数据 workbook 的目录，除非你明确想替换这些文件。生成模板用于新 workbook；已经有真实数据的 workbook 应该使用 `excel-sync`。

## 同步已有 Workbook

真实项目里已有数据通常很多，这时不要把数据行复制到新模板，而应该使用 `excel-sync`。它会根据当前 schema 更新 workbook 表头，同时保留数据行：

```bash
sora excel-sync --project project.toml --data-root data
```

不带 `--write` 时，命令只预览将要发生的变化。确认后再写入文件：

```bash
sora excel-sync --project project.toml --data-root data --write
```

写入已有 workbook 前，Sora 会先把旧文件复制到 `data/.sora-backup/<timestamp>/` 下。

同步时按 `#field` 行匹配字段，而不是按列位置匹配：

- 仍然存在于 schema 中的字段会保留原数据；
- schema 新增字段会插入为空列；
- 类型、parser、scope、range、length、注释和表 metadata 变化会刷新生成表头；
- 从 schema 中删除的字段不会从 Excel 中删除，而是保留为 Sora 忽略的 legacy 列，由策划在合适的时候手动删除；
- 同一个 workbook 中不属于 schema 的 sheet 会作为 value-only sheet 保留下来。

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

schema 变更后，先运行 `sora excel-sync --project project.toml --data-root data` 预览表头变化，确认后再加 `--write` 写回。这样既保留了电子表格编辑体验，也避免 Excel 变成第二套 schema 语言。

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
