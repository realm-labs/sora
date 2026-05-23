# 快速开始

这一页会创建一个最小的道具表，生成 Excel 模板，导出运行时数据包，并生成可以读取它的 Rust 代码。

开发阶段可以从源码安装 CLI：

```bash
cargo install --path crates/sora-cli
```

## 1. 创建项目

创建 `project.toml`：

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

创建 `schema/items.toml`：

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

## 2. 生成 Excel 模板

工作簿表头由 schema 生成：

```bash
sora excel-template --project project.toml --out generated/excel
```

这会生成 `generated/excel/Item.xlsx`。复制到 `data/Item.xlsx`，然后在生成表头下面填写行数据：

| id | name | item_type | max_stack |
| --- | --- | --- | --- |
| 1001 | Iron Sword | Weapon | 1 |
| 2001 | Health Potion | Consumable | 99 |

## 3. 检查、导出和生成代码

校验 schema：

```bash
sora check --project project.toml
```

运行 `[build]` 中声明的全部输出。这个过程会在写出导出文件前加载并校验源数据：

```bash
sora build --project project.toml
```

也可以分开执行：

```bash
sora gen --target rust --project project.toml --out generated/rust

sora export \
  --format binary \
  --default-source-format xlsx \
  --project project.toml \
  --data-root data \
  --out generated/config.sora
```

## 4. 下一步

阅读[第一份配置](tutorial/first-config.md)了解完整闭环，或者查看 `examples/showcase/project.toml` 作为多语言项目参考。
