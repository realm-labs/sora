# Schema as Source of Truth

Sora is schema-first. The TOML schema is the contract for configuration data; source files and generated outputs are projections of that contract.

```text
schema modules
  -> normalized IR
  -> Excel headers
  -> validation
  -> runtime exports
  -> generated language code
```

This design avoids the common problem where a spreadsheet, a hand-written parser, and runtime code all define slightly different versions of the same data shape.

## Consequences

- Field names, types, keys, defaults, references, and validation rules live in schema.
- Excel and CSV files provide values, not a second schema.
- Runtime export formats do not change the data model.
- Language options belong to codegen targets, not to the IR.
- Downstream users can add generators or exporters without changing schema semantics.

The schema can still include editing hints such as `comment`, parser hints, ranges, and length limits. Those hints are part of the data contract because they affect validation or generated projections.
