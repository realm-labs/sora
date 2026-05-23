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
required = true
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

Indexes generate lookup helpers in targets that support them.

```toml
[[tables.indexes]]
name = "by_name"
fields = ["name"]
unique = true
```

A unique index expects at most one row per key. A non-unique index groups matching rows.

## Validation

Sora validates table rows after loading source data:

- required fields must be present unless a default exists;
- key fields must be unique for map tables;
- enum values must be valid;
- references must point to existing rows;
- numeric ranges and length ranges must pass;
- parser output must match the declared field type.
