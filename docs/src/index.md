# Sora

Sora is a schema-first game configuration compiler.

It reads schema modules and table data, validates them into a normalized model, and emits runtime-ready data bundles plus strongly typed code for game and tooling languages.

The important part is that the schema is the contract. Excel, CSV, TOML, generated code, and exported runtime bundles are all projections of that contract. A designer can edit rows in a workbook, while game code consumes strongly typed generated APIs.

## What Sora Does

```text
schema modules -> Excel/CSV/TOML data -> validation
                                      |-> runtime bundle
                                      |-> generated code
```

Sora currently focuses on these stages:

- describe tables, records, enums, unions, references, indexes, and validation rules in TOML schema files;
- generate Excel templates from the schema so spreadsheet headers stay consistent;
- load table data from TOML, CSV, or Excel `.xlsx`;
- validate data against the normalized schema and cross-table references;
- export data as Sora binary, debug JSON, JSON bundle, CBOR bundle, or Sora Protobuf bundle;
- generate language runtimes that load those exported bundles.

## When This Fits

Sora is intended for game configuration and similar data-heavy applications where:

- designers or tools edit tabular data;
- runtime code wants typed access instead of loose dictionaries;
- schema changes should be reviewed in source control;
- generated language support should be extendable by downstream users.

The project is still early, so the public API can change. The design goal is to keep the core schema and IR independent from individual language backends, so downstream users can add generators or exporters without patching the core pipeline.
