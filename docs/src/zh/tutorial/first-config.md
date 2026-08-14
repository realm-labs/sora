# 第一份配置

这个教程会创建一个小型道具配置表。实际项目中的大型配置也是同样模式：定义 schema、生成可编辑工作簿、填写行数据、导出运行时数据包、生成代码。

## 项目结构

```text
project.scon
schema/items.scon
data/Item.xlsx
generated/
```

## 项目清单

```scon
project { id = "game_config" }
groups { common { default = true } }
views {
  default {
    contract = "game_config/default"
    groups = ["common"]
  }
}
includes = ["schema/items.scon"]

codegen {
  rust {
    crate { name = "game-config" }
  }
}

build {
  default_source_format = "xlsx"
  data_root = "data"
  view = "default"
  schema_lock = "generated/schema.lock"
  excel_templates = "generated/excel"
  codegen = [{ target = "rust", out = "generated/rust", format = "auto" }]
  exports = [{ format = "binary", out = "generated/config.sora" }]
}
```

`schema_lock` 保存归一化 schema，`excel_templates` 写出带生成表头的工作簿，`build.codegen` 声明语言输出，`build.exports` 声明运行时数据输出。

## Schema

```scon
enums { ItemType = ["Weapon", "Armor", "Material", "Consumable"] }

tables {
  Item {
    mode = "map"
    key = "id"
    source { format = "xlsx" file = "Item.xlsx" sheet = "Item" }
    fields {
      id { type = "i32" comment = "Item id" }
      name { type = "string" comment = "Display name" }
      item_type { type = "enum<ItemType>" comment = "Item category" }
      max_stack { type = "i32" default = "1" range = [1, 9999] comment = "Stack limit" }
    }
  }
}
```

这个表使用 `mode = "map"`，因此生成运行时会提供按 `id` 查找的接口。

## Excel 模板

生成工作簿：

```bash
sora excel-template --project project.scon --out generated/excel
```

生成出的 sheet 在可编辑数据区上方有多行元数据：

| #field | id | name | item_type | max_stack |
| --- | --- | --- | --- | --- |
| #type | i32 | string | `enum<ItemType>` | i32 |
| #input | key |  |  | range=1..9999 |
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
sora build --project project.scon
```

预期产物：

- `generated/schema.lock`
- `generated/excel/Item.xlsx`
- `generated/rust/Cargo.toml`
- `generated/rust/src/lib.rs`
- `generated/config.sora`

如果只想校验 schema，可以运行 `sora check --project project.scon`。
