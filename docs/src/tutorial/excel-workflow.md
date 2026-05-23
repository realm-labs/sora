# Excel Workflow

Excel support is designed around generated templates. The schema owns the table shape; Excel is an editable projection of that schema.

## Generate Templates

```bash
sora excel-template --project project.toml --out generated/excel
```

Or let `sora build` generate templates when `[build].excel_templates` is configured:

```toml
[build]
excel_templates = "generated/excel"
```

Sora groups sheets by `tables.source.file`. Multiple tables can live in the same workbook when their sources point at the same `.xlsx` file.

## Header Rows

Generated sheets include several header rows:

| Row | Purpose |
| --- | --- |
| `@table` metadata | Table name, mode, key, scope, and schema hash. |
| `#name` | Display name row for the spreadsheet. |
| `#field` | Stable schema field names read by Sora. |
| `#type` | Type hints such as `i32`, `enum<ItemType>`, or `struct<Cost>(kind: enum<ResourceKind>, id: i32, count: i32)`. |
| `#scope` | Scope information for each field. |
| `#rule` | Key, required, optional, parser, and range hints. |
| `#desc` | Field comments for designers and reviewers. |

Data rows start after the generated header.

## What Users Should Edit

Users should edit data rows. They should not hand-maintain field names, types, key metadata, or validation rules in Excel. Those rows are regenerated from schema changes.

When the schema changes, regenerate the template, then migrate or paste the data rows into the new sheet. This keeps spreadsheet editing convenient without making Excel a second schema language.

## Common Field Shapes

Simple fields map directly to cells:

| id | name | max_stack |
| --- | --- | --- |
| 1001 | Iron Sword | 1 |

Structured values use parsers when a cell needs a compact representation:

```toml
[[tables.fields]]
name = "price"
type = "struct<ResourceCost>"
parser = { kind = "tuple" }
comment = "Tuple: kind,id,count"
```

Example cell:

```text
Item,1001,3
```

Collections can use JSON or map-style parsers:

```toml
[[tables.fields]]
name = "tags"
type = "set<string>"
parser = { kind = "json" }
default = "[\"misc\"]"

[[tables.fields]]
name = "attributes"
type = "map<string,i32>"
parser = { kind = "map" }
comment = "Map pairs: key,value|key,value"
```

Example cells:

```text
["starter","melee"]
attack,12|speed,2
```
