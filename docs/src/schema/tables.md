# Tables

Tables are source-backed row collections. A table schema declares the table mode, source location, fields, and optional indexes.

## Modes

| Mode | Shape | Typical Use |
| --- | --- | --- |
| `map` | Rows keyed by one field. | Items, quests, levels, buffs. |
| `list` | Ordered rows without keyed lookup. | Drop entries, weighted pools, ordered steps. |
| `singleton` | One row. | Global settings, tuning constants. |

```scon
tables {
  Item {
    mode = "map"
    key = "id"
    fields { id = "i32" }
  }
}
```

For map tables, `key` names the table's primary key field. Sora uses it for row uniqueness, generated lookup APIs, Excel template hints, and `ref<Table.key>` validation.

## Source

```scon
source {
  format = "xlsx"
  file = "Core.xlsx"
  sheet = "Item"
}
```

`format` can be omitted when the project or command provides a default source format. `file` is resolved under the command's `--data-root` during export and validation.

Built-in source formats are `xlsx`, `csv`, `toml`, `json`, and `yaml`. JSON and YAML table files are arrays of row objects:

```json
[
  { "id": 1001, "name": "Iron Sword" },
  { "id": 1002, "name": "Health Potion" }
]
```

For JSON and YAML, `file` can also point to a directory. In that case Sora recursively reads every matching `.json`, `.yaml`, or `.yml` file as one row object, sorted by path.

### Split one XLSX table across sheets

An XLSX-backed table can combine several worksheets into one logical table. This is useful for data that is naturally partitioned for editing, such as one activity sheet per month:

An XLSX workbook belongs to exactly one table definition. Multiple tables cannot share the same `source.file`; use separate workbook files for separate schemas. Multiple sheets in that file are partitions of its one table, not separate table definitions.

```scon
source {
  format = "xlsx"
  file = "Activity.xlsx"
  sheets = ["2026-01", "2026-02", "2026-03"]
}
```

Every selected worksheet uses the same columns and parsers declared by the table schema. Sora concatenates their rows, then validates and exports the combined result as one table. Primary-key, unique-index, reference, and `singleton` checks therefore apply across all selected sheets, not separately per sheet.

Selectors may contain `*` (any sequence) and `?` (one character):

```scon
source {
  format = "xlsx"
  file = "Activity.xlsx"
  sheets = ["2026-*", "special"]
}
```

Explicit sheet names keep declaration order. Matches from each wildcard selector are sorted by sheet name. Overlapping selectors include a worksheet only once. Loading fails when an explicit worksheet is missing or a wildcard matches nothing.

`sheets` is supported only for XLSX sources and cannot be combined with `sheet`. New template generation requires explicit sheet names because a wildcard cannot invent worksheets; wildcard selectors can be used with an existing workbook. Excel template generation and sync create missing explicitly named worksheets, and sync preserves rows in each selected worksheet. Row-level Studio data mutation currently rejects multi-sheet tables because merged rows do not retain worksheet ownership; schema editing, loading, validation, and export remain supported.

## Indexes

Indexes are extra lookup paths on a table. They are different from the `key` of a `mode = "map"` table:

| Concept | Purpose |
| --- | --- |
| table `key` | The primary key. A map table uses it to keep rows unique and to generate the main `get(id)` lookup. |
| table `indexes` | Additional keyed lookup paths, such as lookup by name, grouping by type, or finding drops by stage. |

For example, an `Item` table can use `id` as its primary key:

```scon
tables {
  Item {
    mode = "map"
    key = "id"
    fields {
      id = "i32"
      name = "string"
      item_type = "enum<ItemType>"
    }
  }
}
```

Add a unique index when another field should also identify at most one row:

```scon
indexes {
  by_name {
    fields = ["name"]
    unique = true
  }
}
```

Example data:

| id | name | item_type |
| --- | --- | --- |
| 1001 | Iron Sword | Weapon |
| 1002 | Wood Shield | Armor |

`unique = true` means `name` cannot repeat. Generated code for targets that support the index can expose a helper similar to `get_by_name("Iron Sword")`, returning one row or no row.

Use a non-unique index when a key can match many rows:

```scon
indexes {
  by_item_type = ["item_type"]
}
```

Example data:

| id | name | item_type |
| --- | --- | --- |
| 1001 | Iron Sword | Weapon |
| 1002 | Bronze Axe | Weapon |
| 2001 | Wood Shield | Armor |

`unique = false` means one key can match several rows. Generated code for targets that support the index can expose a helper similar to `get_by_item_type(ItemType::Weapon)`, returning the matching rows.

`fields` is a list, so a unique index can also express combined uniqueness:

```scon
indexes {
  by_world_stage {
    fields = ["world", "stage"]
    unique = true
  }
}
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
