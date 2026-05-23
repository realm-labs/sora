# Quick Start

Install from source during development:

```bash
cargo install --path crates/sora-cli
```

Create a project manifest:

```toml
package = "game_config"
includes = ["schema/items.toml"]
```

Define a schema module:

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

[[tables.fields]]
name = "name"
type = "string"
required = true

[[tables.fields]]
name = "item_type"
type = "enum<ItemType>"
required = true
```

Generate code:

```bash
sora gen rust --project project.toml --out generated/rust
```

Export data:

```bash
sora export \
  --format binary \
  --data-format xlsx \
  --project project.toml \
  --data-root data \
  --out generated/config.sora
```

For a complete multi-language project, start from `examples/showcase/project.toml`.
