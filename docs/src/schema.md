# Schema

A schema module is a TOML, YAML, or JSON file included by a project manifest.

```toml
package = "game_config"
includes = ["schema/items.toml", "schema/skills.toml"]
```

Schema modules are the source of truth for Sora. They describe the stable data contract; source files such as Excel workbooks contain row values that are checked against that contract.

See [Schema Formats](schema/formats.md) for the supported file formats and equivalent TOML/YAML shapes.

## Enums

```toml
[[enums]]
name = "ItemType"
values = ["Weapon", "Armor", "Material"]
```

Enums are stored by symbolic value in editable data and generated as native enum-like constructs when the target language supports them.

## Structs

```toml
[[structs]]
name = "Cost"

[[structs.fields]]
name = "gold"
type = "i32"
required = true
```

Structs model repeated object shapes. They are useful for costs, rewards, coordinates, stat modifiers, and other nested values.

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

Unions model tagged variants. The `tag` field is the discriminator name used in source data and runtime values.

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

Tables define source-backed row collections. See [Tables](schema/tables.md) for modes, keys, sources, indexes, and aggregation.

## Field Types

Common field types include primitives, enums, structs, unions, references, lists, sets, fixed arrays, maps, and optionals:

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

See [Types](schema/types.md) for the full list and examples.

See [Cell Parsers](schema/parsers.md) for compact Excel/CSV cell formats such as `split`, `tuple`, `tuple_list`, `map`, and `json`.
