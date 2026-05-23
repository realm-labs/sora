# Schema

schema module 是被项目清单 include 的 TOML 或 YAML 文件。

```toml
package = "game_config"
includes = ["schema/items.toml", "schema/skills.toml"]
```

Schema 是 Sora 的事实来源。它描述稳定的数据契约；Excel 工作簿等源文件只包含需要按契约校验的行数据。

支持的文件格式以及等价的 TOML/YAML 写法见 [Schema 格式](schema/formats.md)。

## Enums

```toml
[[enums]]
name = "ItemType"
values = ["Weapon", "Armor", "Material"]
```

枚举在可编辑数据中用符号值表示，在支持的语言中会生成原生或接近原生的枚举结构。

## Structs

```toml
[[structs]]
name = "Cost"

[[structs.fields]]
name = "gold"
type = "i32"
required = true
```

结构体适合复用的对象形状，比如消耗、奖励、坐标、属性修正等嵌套值。

## Unions

```toml
[[unions]]
name = "RewardAction"
tag = "type"

[[unions.variants]]
name = "AddItem"

[[unions.variants.fields]]
name = "item_id"
type = "ref<Item.id>"
required = true
```

联合用于 tagged variant。`tag` 是源数据和运行时值里的判别字段名。

## Tables

```toml
[[tables]]
name = "Item"
mode = "map"
key = "id"

[tables.source]
format = "xlsx"
file = "Item.xlsx"
sheet = "Item"
```

表定义带 source 的行集合。模式、key、source、索引和聚合见[表](schema/tables.md)。

## Field Types

常见字段类型包括 primitive、enum、struct、union、reference、list、set、fixed array、map 和 optional：

```text
i32
string
enum<ItemType>
struct<Cost>
union<Reward>
ref<Item.id>
list<i32>
set<string>
array<i32,3>
map<string,i32>
optional<string>
```

完整说明见[类型](schema/types.md)。

`split`、`tuple`、`tuple_list`、`map`、`json` 等 Excel/CSV 紧凑单元格写法见[单元格 Parser](schema/parsers.md)。
