# Schema

A project root and each included schema module may use SCON, TOML, YAML, JSON, or Lua. SCON is recommended for new files.

```scon
project { id = "game_config" }
includes = ["schema/items.scon", "schema/skills.scon"]
```

Every file is first parsed into a format-specific keyed representation and then lowered to a format-independent schema module. The root must declare `project`; included modules must not declare root-only `project`, `groups`, `views`, `codegen`, or `localization`. Includes may cross formats and are loaded recursively in declaration order, with cycle, duplicate-module, and duplicate-declaration checks.

Declaration names are keys and are not repeated inside bodies. Unknown properties are rejected.

## Enums

```scon
enums {
  ItemType = ["Weapon", "Armor", "Material"]
}
```

Enums keep their value order and are stored symbolically in editable data.

## Structs

```scon
structs {
  Cost {
    fields {
      gold = "i32"
    }
  }
}
```

Struct fields preserve author order and model reusable nested values.

## Unions

```scon
unions {
  RewardAction {
    tag = "type"
    variants {
      AddItem {
        fields {
          item_id = "ref<Item.id>"
        }
      }
    }
  }
}
```

Union variants preserve author order. `tag` is the discriminator used in source and runtime values.

## Tables

```scon
tables {
  Item {
    mode = "map"
    key = "id"
    source {
      format = "xlsx"
      file = "Item.xlsx"
      sheet = "Item"
    }
    fields {
      id = "i32"
    }
  }
}
```

See [Schema Formats](schema/formats.md) for equivalent natural forms in all five frontends, [Tables](schema/tables.md) for sources and indexes, [Types](schema/types.md) for field types, and [Cell Parsers](schema/parsers.md) for compact cell encodings.
