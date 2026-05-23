# Code Generation

Code generation turns the normalized schema IR into target-language row types, table containers, and config loaders.

It is driven by a registry of language generators.

Each generator declares:

- a target id and aliases;
- display metadata;
- supported runtime formats;
- optional formatter integration;
- a `CodeGenerator` implementation.

This lets built-in languages and downstream generators use the same pipeline shape.

```text
schema files -> schema model -> normalized IR -> generator registry -> target generator -> files
```

Generate a target directly:

```bash
sora gen --target typescript --project project.toml --out generated/typescript
```

Or declare it in the build manifest:

```toml
[[build.codegen]]
target = "typescript"
out = "typescript/generated"
format = "auto"
```

`format` can be `never`, `auto`, or `required`. `auto` runs a known formatter when it is available. `required` fails if the formatter is missing or returns an error.

## Runtime Format

Each target can choose a runtime format:

```toml
[codegen.typescript]
runtime_format = "json"
```

The selected runtime format controls the loader code emitted for that target. It does not change the schema or the source data.

## Generated Shape

Generated code generally contains:

- enums for schema enums;
- record types for structs, union variants, and table rows;
- table containers for `map`, `list`, and `singleton` tables;
- lookup helpers for keys and indexes where supported;
- a top-level config loader for the selected runtime format.
