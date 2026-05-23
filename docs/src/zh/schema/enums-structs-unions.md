# 枚举、结构体和联合

这些定义让 schema 可以表达超过扁平表格的数据结构。

## Enums

```toml
[[enums]]
name = "Rarity"
values = ["Common", "Uncommon", "Rare", "Epic", "Legendary"]
```

枚举让源数据保持可读，同时让生成代码获得受约束的类型。

alias 可以保留导入数据或旧数据里的名称：

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

结构体适合多处复用的嵌套值。字段可以通过 `type = "struct<ResourceCost>"` 引用结构体。

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

当一个字段可能是多个 tagged shape 之一时，使用 union。常见例子包括条件、奖励、触发器和脚本动作。
