# Schema

project root 和每个 include module 都可以使用 SCON、TOML、YAML、JSON 或 Lua；新文件推荐 SCON。

```scon
project { id = "game_config" }
includes = ["schema/items.scon", "schema/skills.scon"]
```

每个文件先解析成对应格式的 keyed 表示，再 lower 为格式无关的 schema module。root 必须声明 `project`；include module 不能声明 root-only 的 `project`、`groups`、`views`、`codegen` 或 `localization`。include 可以跨格式，Sora 按声明顺序递归加载，并检测循环、重复 module 和跨 module 重名。

声明名只存在于 key，body 内不再重复 `name`；未知属性会被拒绝。

```scon
enums {
  ItemType = ["Weapon", "Armor", "Material"]
}

structs {
  Cost {
    fields {
      gold = "i32"
    }
  }
}

unions {
  RewardAction {
    variants {
      AddItem {
        fields {
          item_id = "ref<Item.id>"
        }
      }
    }
  }
}

tables {
  Item {
    mode = "map"
    key = "id"
    source {
      file = "Item.xlsx"
      sheet = "Item"
    }
    fields {
      id = "i32"
    }
  }
}
```

enum values、struct fields、union variants 和 table fields 都保留作者顺序。等价的五格式自然写法见 [Schema 格式](schema/formats.md)，表与索引见 [Tables](schema/tables.md)，字段类型见 [Types](schema/types.md)，单元格编码见 [Cell Parsers](schema/parsers.md)。
