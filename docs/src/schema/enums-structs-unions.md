# Enums, Structs, and Unions

These definitions let schemas model more than flat tables.

## Enums

```toml
[[enums]]
name = "Rarity"
values = ["Common", "Uncommon", "Rare", "Epic", "Legendary"]
```

Enums keep source data readable while generated code receives a constrained type.

Aliases can keep imported or legacy names readable:

```toml
[[enums.aliases]]
name = "Purple"
alias = "Epic"
```

## Structs

```toml
[[structs]]
name = "ResourceCost"

[[structs.fields]]
name = "kind"
type = "enum<ResourceKind>"
required = true

[[structs.fields]]
name = "id"
type = "i32"
required = true

[[structs.fields]]
name = "count"
type = "i32"
required = true
range = [1, 999999]
```

Use structs for nested values that appear in many places. A field can reference a struct with `type = "struct<ResourceCost>"`.

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

[[unions.variants.fields]]
name = "count"
type = "i32"
required = true

[[unions.variants]]
name = "UnlockStage"

[[unions.variants.fields]]
name = "stage_id"
type = "ref<Stage.id>"
required = true
```

Use unions when a field can contain one of several tagged shapes. Examples include conditions, rewards, triggers, and scripted actions.
