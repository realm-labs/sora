# Excel Workflow

Excel support is designed around generated templates. The schema owns the table shape; Excel is an editable projection of that schema.

## Generate Templates

There are two ways to generate Excel templates.

The direct command only writes templates:

```bash
sora excel-template --project project.scon --out generated/excel
```

This reads the schema from `project.scon` and writes generated workbooks under `generated/excel`. The directory is safe to delete and regenerate because it should contain template artifacts, not hand-edited source data.

The build workflow can do the same thing when `excel_templates` is configured:

```scon
build { excel_templates = "generated/excel" }
```

```bash
sora build --project project.scon
```

Both paths generate the same kind of template files. The direct command only writes Excel templates. `sora build` runs the template output together with the other configured build outputs such as schema locks, code generation, and exports.

## Template Directory vs Data Directory

`excel_templates` is an output directory for templates. It is not the runtime data input directory. Data input normally comes from `[build].data_root` or the `--data-root` command option.

The usual layout keeps these paths separate:

| Path | Role | Can be regenerated |
| --- | --- | --- |
| `generated/excel` | Generated workbook templates with schema headers. | Yes |
| `data` | Edited table rows used by export and build. | No |

Do not point `excel-template --out` or `[build].excel_templates` at a directory that already contains edited data workbooks unless replacing those files is intentional. Use generated templates for new workbooks; use `excel-sync` for workbooks that already contain real data.

## Sync Existing Workbooks

For real projects with existing data, use `excel-sync` instead of copying rows into a fresh template. It updates workbook headers from the current schema while preserving data rows:

```bash
sora excel-sync --project project.scon --data-root data
```

Without `--write`, the command only previews what would change. To write the updated workbook files:

```bash
sora excel-sync --project project.scon --data-root data --write
```

When writing an existing workbook, Sora first copies the old file under `data/.sora-backup/<unique-batch>/`. The backup directory ignores itself from Git and retains only the 20 most recent batches.

Sync matches columns by the `#field` row, not by column position:

- existing schema fields keep their data;
- new schema fields are added as empty columns;
- changed type, parser, groups, range, length, comments, and table metadata refresh the generated header rows;
- fields removed from schema are not deleted from Excel. They are kept as legacy columns ignored by Sora, so designers can delete them manually when they are ready;
- non-schema sheets in the same workbook are preserved as value-only sheets.

The XLSX file is the ownership boundary for one table definition. Two table definitions cannot reference the same workbook. Multiple worksheets in one workbook are supported only as row partitions of that one table, for example monthly activity data:

```scon
tables {
  Activity {
    mode = "map"
    source { format = "xlsx" file = "Activity.xlsx" sheets = ["2026-01", "2026-02"] }
  }
}
```

Both sheets use the `Activity` schema and their rows are combined during loading. To bind another table, give it another file:

```scon
tables {
  Skill { mode = "map" source { format = "xlsx" file = "Skill.xlsx" } }
}
```

If `sheet` and `sheets` are omitted, a new template derives its default Sheet name from the workbook filename. For example, table `presentation.SurfaceResourceVisualProfile` with `file = "VisualProfile.xlsx"` uses the Sheet `VisualProfile`; the namespace never enters the physical Sheet name. The default is sanitized and truncated to Excel's 31-character limit. The complete canonical table name remains in each data Sheet's `@table` metadata. Explicit `sheet` and `sheets` values are preserved.

## Header Rows

Generated sheets include several header rows:

| Row | Purpose |
| --- | --- |
| `@table` metadata | Canonical table name, mode, key, groups, and schema hash. |
| `#name` | Display name row for the spreadsheet. |
| `#field` | Stable schema field names read by Sora. |
| `#type` | Type hints such as `i32`, `enum<ItemType>`, or `struct<Cost>(kind: enum<ResourceKind>, id: i32, count: i32)`. |
| `#groups` | Group membership for each field. |
| `#input` | Input hints such as key, parser, range, length, or derived-field source. |
| `#desc` | Field comments for designers and reviewers. |

Data rows start after the generated header.

## What Users Should Edit

Users should edit data rows. They should not hand-maintain field names, types, key metadata, input hints, or validation rules in Excel. Those rows are regenerated from schema changes.

If a column's `#input` cell starts with `from=`, that field is derived from another table. Leave the generated placeholder in that column and edit the child table rows instead.

When the schema changes, run `sora excel-sync --project project.scon --data-root data` to preview header changes, then rerun with `--write` after reviewing them. This keeps spreadsheet editing convenient without making Excel a second schema language.

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
