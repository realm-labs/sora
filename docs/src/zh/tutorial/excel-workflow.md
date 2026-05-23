# Excel 工作流

Excel 支持围绕生成模板设计。schema 拥有表结构，Excel 是这个 schema 的可编辑投影。

## 生成模板

```bash
sora excel-template --project project.toml --out generated/excel
```

也可以在 `[build].excel_templates` 中配置，让 `sora build` 生成模板：

```toml
[build]
excel_templates = "generated/excel"
```

Sora 会按 `tables.source.file` 将 sheet 分组。多个表可以通过指向同一个 `.xlsx` 文件，放到同一个工作簿的不同 sheet 中。

## 表头行

生成的 sheet 包含多行表头：

| Row | Purpose |
| --- | --- |
| `@table` metadata | 表名、mode、key、scope 和 schema hash。 |
| `#name` | 面向表格编辑的显示名行。 |
| `#field` | Sora 读取的稳定 schema 字段名。 |
| `#type` | 类型提示，例如 `i32`、`enum<ItemType>` 或 `struct<Cost>(kind: enum<ResourceKind>, id: i32, count: i32)`。 |
| `#scope` | 每个字段的 scope 信息。 |
| `#rule` | key、required、optional、parser 和 range 提示。 |
| `#desc` | 给编辑者和 reviewer 看的字段注释。 |

数据行从生成表头之后开始。

## 用户应该编辑什么

用户应该编辑数据行，不应该手工维护 Excel 里的字段名、类型、key 元数据或校验规则。这些行会从 schema 重新生成。

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
