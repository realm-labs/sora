# Sora

Sora is a schema-first game configuration compiler.

It reads schema modules and table data, validates them into a normalized model, and emits runtime-ready data bundles plus strongly typed code for game and tooling languages.

Sora currently focuses on a small set of clear pipeline stages:

- describe tables, records, enums, and unions in TOML schema files;
- load table data from TOML, CSV, or Excel `.xlsx`;
- validate data against the normalized schema;
- export data as Sora binary, debug JSON, JSON bundle, CBOR bundle, or Sora Protobuf bundle;
- generate language runtimes that load those exported bundles.

The project is still early, so the public API can change. The design goal is to keep the core schema and IR independent from individual language backends, so downstream users can add generators or exporters without patching the core pipeline.
