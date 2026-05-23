# Exporters

An exporter writes validated configuration data into a runtime bundle.

Exporters are separate from code generators because the same exported data can be consumed by many languages.

## When to Add an Exporter

Add an exporter when you need:

- a new runtime wire format;
- a platform-specific asset package;
- a different compression or section layout;
- an inspection format for tooling.

Do not add an exporter just to support a new programming language. Add a code generator for that.

## Expected Boundary

An exporter should consume:

- the normalized schema IR;
- validated config data;
- exporter options;
- an output target.

It should not depend on a specific language generator.
