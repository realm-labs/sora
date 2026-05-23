# IR Boundaries

The normalized IR describes schema semantics. It should not encode language-specific codegen choices.

## Belongs in IR

- packages and included schema modules;
- enums, structs, unions, tables, fields, and indexes;
- table modes and keys;
- source metadata;
- field types, defaults, parsers, ranges, lengths, and comments;
- references and aggregation metadata;
- scopes.

## Does Not Belong in IR

- Rust map implementation choices;
- TypeScript enum representation choices;
- Lua module names;
- runtime decoder dependency choices;
- formatter settings;
- target-specific file layout.

Those settings belong in `[codegen.<target>]` or in generator registration metadata.

## Extension Boundary

```text
schema input -> normalized IR -> validation
                              |-> exporter registry
                              |-> codegen registry
```

A new language generator should consume the IR and its own target options. A new runtime data format should be added as an exporter. Neither should require changing the schema model unless the actual data semantics need to change.
