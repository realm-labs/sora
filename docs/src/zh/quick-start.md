# 快速开始

这一页会创建一个最小的道具表，生成 Excel 模板，导出运行时数据包，并生成可以读取它的 Rust 代码。

可以从 [GitHub Releases](https://github.com/realm-labs/sora/releases) 下载对应平台的压缩包，解压后把 `sora` 放到 `PATH` 中。

如果本机已有 Rust 工具链，也可以从 crates.io 安装已发布的 CLI：

```bash
cargo install sora-cli
```

本地开发时可以从源码安装：

```bash
cargo install --path crates/sora-cli
```

## 1. 创建项目

最快的方式是直接生成同一个最小项目：

```bash
sora init --out my-config --schema-format toml
cd my-config
```

`--schema-format` 支持 `toml`、`yaml`、`json` 和 `lua`。脚手架会生成这个目录结构：

| 路径 | 谁编辑 | 作用 |
| --- | --- | --- |
| `project.toml` | 你 | 项目入口、构建输出、默认数据目录。 |
| `schema/items.toml` | 你 | `Item` 表的 schema。 |
| `data/Item.xlsx` | 策划或工具 | 可编辑行数据。 |
| `generated/` | Sora | schema lock、Excel 模板、生成代码、导出数据。 |

本节后面的内容展示生成出来的文件，方便理解项目结构。`project.toml` 内容如下：

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

这里 `default_source_format = "xlsx"` 表示表数据默认来自 Excel。`data_root = "data"` 表示导出和 build 时会从 `data/Item.xlsx` 读取 `Item.xlsx`。`binary` export 会写出 Rust 代码要加载的运行时数据包，因为 Rust 默认使用 `runtime_format = "sora"`。

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
comment = "Item id"

[[tables.fields]]
name = "name"
type = "string"
comment = "Display name"

[[tables.fields]]
name = "item_type"
type = "enum<ItemType>"
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

只校验 schema：

```bash
sora check --project project.toml
```

运行 `[build]` 中声明的全部输出。这个过程会在写出导出文件前加载并校验源数据：

```bash
sora build --project project.toml
```

也可以用 CLI 内置的 Sora Studio 打开这个项目：

```bash
sora studio --project project.toml
```

命令会打印一个本地地址。用浏览器打开后，可以可视化 schema 关系、编辑 schema module、预览将要写入的变更，并保存回项目文件。

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

如果想用可视化方式编辑 schema，继续读 [Sora Studio](studio.md)。也可以阅读[第一份配置](tutorial/first-config.md)了解完整闭环，或者查看 `examples/showcase/project.toml` 作为多语言项目参考。
