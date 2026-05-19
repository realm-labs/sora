# Sora

Sora is a Rust-first game configuration compiler that turns schemas and table data into strongly typed code and runtime-ready data artifacts.

## Status

Sora is in its first milestone. It currently supports TOML schemas, simple TOML table data, normalized IR, basic validation, Rust/Kotlin model generation, a pluggable exporter registry, a binary bundle exporter, a debug JSON exporter, and generated Excel `.xlsx` template projections.

## Design Principles

- Schema is the source of truth.
- Excel is the editing surface.
- Generated Excel headers are schema projections, not a second schema.
- Data exporters are pluggable backends, not hardcoded pipeline stages.
- Debug JSON is useful for inspection, but it is not special in the core architecture.

## Example Commands

```bash
cargo run -p sora-cli -- check \
  --schema examples/simple/schema.toml

cargo run -p sora-cli -- gen rust \
  --schema examples/simple/schema.toml \
  --out generated/rust

cargo run -p sora-cli -- gen kotlin \
  --schema examples/simple/schema.toml \
  --out generated/kotlin

cargo run -p sora-cli -- export \
  --format binary \
  --schema examples/simple/schema.toml \
  --data-root examples/simple/data \
  --out generated/config.sora

cargo run -p sora-cli -- export \
  --format json-debug \
  --schema examples/simple/schema.toml \
  --data-root examples/simple/data \
  --out generated/debug-json

cargo run -p sora-cli -- excel-template \
  --schema examples/simple/schema.toml \
  --out generated/excel
```

## Workspace Architecture

- `sora-cli`: command-line interface.
- `sora-core`: pipeline orchestration.
- `sora-input`: input adapter traits and loaded in-memory input.
- `sora-input-toml`: TOML schema and TOML data input adapter.
- `sora-schema`: TOML schema loading.
- `sora-ir`: normalized schema IR and type parsing.
- `sora-data`: data IR, TOML data loading, and validation.
- `sora-codegen`: Rust and Kotlin code generation.
- `sora-export`: exporter trait, registry, and built-in exporters.
- `sora-excel`: Excel `.xlsx` template projection.
- `sora-diagnostics`: shared typed errors.
- `sora-templates`: built-in template location helpers.

## Schema Format

Schemas are TOML files that define packages, enums, structs, tables, fields, keys, comments, source files, and future aggregation metadata. Field type strings are normalized into IR types such as `i32`, `string`, `enum<ItemType>`, `list<i32>`, `array<i32,3>`, `ref<Item.id>`, and `optional<string>`.

## Input Architecture

Sora core consumes input through `SchemaInput` and `DataInput` traits. Concrete source formats live in separate adapter crates. TOML is implemented by `sora-input-toml`, not by `sora-core` or `sora-input`. Future adapters, such as RON, JSON, Excel, or Luban compatibility importers, should translate their source format into `SchemaFile` and `ConfigData` before entering the normal IR, validation, codegen, and exporter pipeline.

## Data Format

The current data source is one TOML file per table. Each file uses `[[rows]]`:

```toml
[[rows]]
id = 1001
name = "Iron Sword"
item_type = "Weapon"
max_stack = 1
```

Validation currently checks required fields, unknown fields, primitive compatibility, enum values, and duplicate map keys.

## Exporter Architecture

Exporters implement a common `DataExporter` trait and are selected by format name through `ExporterRegistry`. Built-in formats are:

- `binary`: writes a production-oriented `.sora` bundle file.
- `json-debug`: writes deterministic per-table JSON files for inspection.

The binary bundle currently stores deterministic JSON payloads inside a simple binary container. That payload format is intentionally replaceable.

## Codegen Architecture

Codegen uses MiniJinja templates, but type mapping is computed in Rust before rendering. The first targets are Rust and Kotlin, with room for future TypeScript, Java, C#, Go, Lua, and Python generators.

## Excel Template Projection

Sora generates `.xlsx` templates from schema IR. Header rows include the table name, mode, key, schema hash, field names, field types, rules, and descriptions. These headers are projections for human editing and future verification; they are not authoritative schema.

## Roadmap

Phase 2:

- CSV data source.
- Real Excel `.xlsx` reader.
- Schema hash verification for Excel headers.
- Ref validation.
- Range validation.
- Generated Rust runtime loader.

Phase 3:

- Child table aggregation.
- Nested object loading.
- Custom parser system.
- Reward, condition expression, and formula parsers.

Phase 4:

- Polymorphic union types.
- Kotlin sealed class generation.
- Secondary indexes.
- Client/server field tags.
- `schema.lock` generation.
- Configuration diff.

Phase 5:

- Stable compact binary format.
- Hot reload friendly bundles.
- Compatibility checking.
- External exporter plugin mechanism.
- VSCode extension or LSP.
- Excel comments and dropdown generation.
- CI report output.
