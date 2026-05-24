# Sora

Sora helps you keep game configuration data understandable while still giving runtime code typed access.

You write a schema that describes table shapes, fill the table rows in Excel, CSV, or TOML, and let Sora validate the data. After validation, Sora writes a runtime data bundle and generates code that knows how to load that bundle.

The schema is the contract. Excel, CSV, TOML, generated code, and exported runtime bundles are all projections of that contract. A designer can edit rows in a workbook, while game code consumes strongly typed generated APIs.

For a small project, the file flow looks like this:

```text
project.toml
  -> schema/items.toml
  -> data/Item.xlsx
  -> generated/config.sora
  -> generated/rust
```

You normally hand-write `project.toml` and schema files. Designers or tools edit files under `data/`. Files under `generated/` are Sora outputs.

## What Sora Does

```text
schema modules -> Excel/CSV/TOML data -> validation
                                      |-> runtime bundle
                                      |-> generated code
```

Sora currently focuses on these stages:

- describe tables, records, enums, unions, references, indexes, and validation rules in schema files;
- inspect and edit schema modules in the embedded Sora Studio UI;
- generate Excel templates from the schema so spreadsheet headers stay consistent;
- load table data from TOML, CSV, or Excel `.xlsx`;
- validate data against the normalized schema and cross-table references;
- export data as Sora binary, debug JSON, JSON bundle, CBOR bundle, or Sora Protobuf bundle;
- generate language runtimes that load those exported bundles.

## Common Terms

Sora uses the word `format` in a few different places:

| Term | Meaning | Example |
| --- | --- | --- |
| Schema format | The file format used to write schema/project files. | TOML, YAML, JSON, Lua |
| Source format | The editable table data format. | Excel `.xlsx`, CSV, TOML |
| Export format | The data bundle written after validation. | `binary`, `json`, `cbor` |
| Runtime format | The bundle format generated code expects to load. | `sora`, `json`, `cbor` |

For example, Rust codegen with `runtime_format = "sora"` needs a matching `binary` export. The source data can still come from Excel.

## When This Fits

Sora is intended for game configuration and similar data-heavy applications where:

- designers or tools edit tabular data;
- runtime code wants typed access instead of loose dictionaries;
- schema changes should be reviewed in source control;
- generated language support should be extendable by downstream users.

The project is still early, so the public API can change. The design goal is to keep the core schema and IR independent from individual language backends, so downstream users can add generators or exporters without patching the core pipeline.

Projects that need stable output should pin the `sora` CLI version. Runtime/export format versions are bumped only for actual generated-runtime incompatibility; Sora does not currently maintain old schema semantics behind edition flags. See [Versioning and Compatibility](versioning.md).

## Suggested Reading Order

Start with [Quick Start](quick-start.md), then read [Sora Studio](studio.md), [First Config](tutorial/first-config.md), and [Excel Workflow](tutorial/excel-workflow.md). After that, the most useful reference pages are [Types](schema/types.md), [Tables](schema/tables.md), [Cell Parsers](schema/parsers.md), [References and Derived Fields](schema/references.md), and [Versioning and Compatibility](versioning.md).

Design notes and extension pages are meant for readers who already understand the basic build flow.
