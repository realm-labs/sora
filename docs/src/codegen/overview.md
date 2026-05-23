# Code Generation

Code generation is driven by a registry of language generators.

Each generator declares:

- a target id and aliases;
- display metadata;
- supported runtime formats;
- optional formatter integration;
- a `CodeGenerator` implementation.

This lets built-in languages and downstream generators use the same pipeline shape.

```text
schema files -> schema model -> normalized IR -> generator registry -> target generator
```

Generate a target directly:

```bash
sora gen typescript --project project.toml --out generated/typescript
```

Or declare it in the build manifest:

```toml
[[build.codegen]]
target = "typescript"
out = "typescript/generated"
format = "auto"
```

`format` can be `never`, `auto`, or `required`. `auto` runs a known formatter when it is available. `required` fails if the formatter is missing or returns an error.
