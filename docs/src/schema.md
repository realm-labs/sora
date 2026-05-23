# Schema

A schema module is a TOML file included by a project manifest.

```toml
package = "game_config"
includes = ["schema/items.toml", "schema/skills.toml"]
```

## Enums

```toml
[[enums]]
name = "ItemType"
values = ["Weapon", "Armor", "Material"]
```

## Structs

```toml
[[structs]]
name = "Cost"

[[structs.fields]]
name = "gold"
type = "i32"
required = true
```

## Unions

```toml
[[unions]]
name = "Reward"
tag = "kind"

[[unions.variants]]
name = "Item"
type = "struct<ItemReward>"
```

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

Common field types include primitives, enums, structs, unions, references, lists, fixed arrays, maps, and optionals:

```text
i32
string
enum<ItemType>
struct<Cost>
union<Reward>
ref<Item.id>
list<i32>
array<i32,3>
optional<string>
```
