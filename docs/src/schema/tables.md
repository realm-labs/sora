# Tables

Tables are source-backed row collections. A table schema declares the table mode, source location, fields, and optional indexes.

## Modes

| Mode | Shape | Typical Use |
| --- | --- | --- |
| `map` | Rows keyed by one field. | Items, quests, levels, buffs. |
| `list` | Ordered rows without keyed lookup. | Drop entries, weighted pools, ordered steps. |
| `singleton` | One row. | Global settings, tuning constants. |

```toml
[[tables]]
name = "Item"
mode = "map"
key = "id"

[[tables.fields]]
name = "id"
type = "i32"
key = true
```

For map tables, `key` names the field used by generated lookup APIs.

## Source

```toml
[tables.source]
format = "xlsx"
file = "Core.xlsx"
sheet = "Item"
```

`format` can be omitted when the project or command provides a default source format. `file` is resolved under the command's `--data-root` during export and validation.

## Indexes

Indexes are extra lookup paths on a table. They are different from the `key` of a `mode = "map"` table:

| Concept | Purpose |
| --- | --- |
| table `key` | The primary key. A map table uses it to keep rows unique and to generate the main `get(id)` lookup. |
| `[[tables.indexes]]` | Additional lookup paths, such as lookup by name, grouping by type, or finding drops by stage. |

For example, an `Item` table can use `id` as its primary key:

```toml
[[tables]]
name = "Item"
mode = "map"
key = "id"

[[tables.fields]]
name = "id"
type = "i32"
key = true

[[tables.fields]]
name = "name"
type = "string"

[[tables.fields]]
name = "item_type"
type = "enum<ItemType>"
```

Add a unique index when another field should also identify at most one row:

```toml
[[tables.indexes]]
name = "by_name"
fields = ["name"]
unique = true
```

Example data:

| id | name | item_type |
| --- | --- | --- |
| 1001 | Iron Sword | Weapon |
| 1002 | Wood Shield | Armor |

`unique = true` means `name` cannot repeat. Generated code for targets that support the index can expose a helper similar to `get_by_name("Iron Sword")`, returning one row or no row.

Use a non-unique index when a key can match many rows:

```toml
[[tables.indexes]]
name = "by_item_type"
fields = ["item_type"]
unique = false
```

Example data:

| id | name | item_type |
| --- | --- | --- |
| 1001 | Iron Sword | Weapon |
| 1002 | Bronze Axe | Weapon |
| 2001 | Wood Shield | Armor |

`unique = false` means one key can match several rows. Generated code for targets that support the index can expose a helper similar to `get_by_item_type(ItemType::Weapon)`, returning the matching rows.

`fields` is a list, so a unique index can also express combined uniqueness:

```toml
[[tables.indexes]]
name = "by_world_stage"
fields = ["world", "stage"]
unique = true
```

This requires each `(world, stage)` pair to be unique. For example, `(1, 1)` can appear once, while `(1, 2)` is a different key. Current generated lookup helpers mainly support single-field indexes on non-singleton tables; combined indexes are most useful for validation today.

## Validation

Sora validates table rows after loading source data:

- non-optional fields must be present unless a default exists;
- key fields must be unique for map tables;
- enum values must be valid;
- references must point to existing rows;
- numeric ranges and length ranges must pass;
- parser output must match the declared field type.
