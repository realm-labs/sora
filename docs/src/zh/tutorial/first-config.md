# 第一份配置

这个教程会创建一个小型道具配置表。实际项目中的大型配置也是同样模式：定义 schema、生成可编辑工作簿、填写行数据、导出运行时数据包、生成代码。

## 项目结构

```text
project.toml
schema/items.toml
data/Item.xlsx
generated/
```

## 项目清单

```toml
package = "game_config"
includes = ["schema/items.toml"]

[build]
default_source_format = "xlsx"
data_root = "data"
schema_lock = "generated/schema.lock"
excel_templates = "generated/excel"

[[build.codegen]]
target = "rust"
out = "generated/rust"
format = "auto"

[[build.exports]]
format = "binary"
out = "generated/config.sora"
```

`schema_lock` 保存归一化 schema，`excel_templates` 写出带生成表头的工作簿，`build.codegen` 声明语言输出，`build.exports` 声明运行时数据输出。

## Schema

```toml
[[enums]]
name = "ItemType"
values = ["Weapon", "Armor", "Material", "Consumable"]

[[tables]]
name = "Item"
mode = "map"
key = "id"

[tables.source]
format = "xlsx"
file = "Item.xlsx"
sheet = "Item"

[[tables.fields]]
name = "id"
type = "i32"
key = true
required = true
comment = "Item id"

[[tables.fields]]
name = "name"
type = "string"
required = true
comment = "Display name"

[[tables.fields]]
name = "item_type"
type = "enum<ItemType>"
required = true
comment = "Item category"

[[tables.fields]]
name = "max_stack"
type = "i32"
default = "1"
range = [1, 9999]
comment = "Stack limit"
```

这个表使用 `mode = "map"`，因此生成运行时会提供按 `id` 查找的接口。

## Excel 模板

生成工作簿：

```bash
sora excel-template --project project.toml --out generated/excel
```

生成出的 sheet 在可编辑数据区上方有多行元数据：

| #field | id | name | item_type | max_stack |
| --- | --- | --- | --- | --- |
| #type | i32 | string | `enum<ItemType>` | i32 |
| #rule | key | required | required | range=1..9999 |
| #desc | Item id | Display name | Item category | Stack limit |

数据行从生成表头之后开始：

| id | name | item_type | max_stack |
| --- | --- | --- | --- |
| 1001 | Iron Sword | Weapon | 1 |
| 2001 | Health Potion | Consumable | 99 |

生成后可以把工作簿复制到 `data/Item.xlsx`，或者在实验阶段直接让 source 指向生成位置。

## 构建

运行配置好的所有输出：

```bash
sora build --project project.toml
```

预期产物：

- `generated/schema.lock`
- `generated/excel/Item.xlsx`
- `generated/rust`
- `generated/config.sora`

如果只想校验 schema，可以运行 `sora check --project project.toml`。
