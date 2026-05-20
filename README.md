# Sora

Sora is a Rust-first game configuration compiler that turns schemas and table data into strongly typed code and runtime-ready data artifacts.

## Status

Sora is in its first milestone. It currently supports TOML schemas, TOML, CSV, and Excel `.xlsx` table data, normalized IR, recursive data validation, Rust/Kotlin model generation, generated Rust/Kotlin binary runtime readers, a pluggable exporter registry, a native sectioned binary exporter, a debug JSON exporter, and generated Excel `.xlsx` template projections.

## Design Principles

- Schema is the source of truth.
- Excel is the editing surface.
- Generated Excel headers are schema projections, not a second schema.
- Data exporters are pluggable backends, not hardcoded pipeline stages.
- Debug JSON is useful for inspection, but it is not special in the core architecture.

## Example Commands

```bash
cargo run -p sora-cli -- check \
  --project examples/simple/project.toml

cargo run -p sora-cli -- gen rust \
  --project examples/simple/project.toml \
  --out generated/rust

cargo run -p sora-cli -- gen kotlin \
  --project examples/simple/project.toml \
  --out generated/kotlin

cargo run -p sora-cli -- excel-template \
  --project examples/simple/project.toml \
  --out generated/excel

cargo run -p sora-cli -- export \
  --format binary \
  --data-format xlsx \
  --project examples/simple/project.toml \
  --data-root generated/excel \
  --out generated/config.sora

cargo run -p sora-cli -- export \
  --format json-debug \
  --data-format xlsx \
  --project examples/simple/project.toml \
  --data-root generated/excel \
  --out generated/debug-json

cargo run -p sora-cli -- export \
  --format json-debug \
  --data-format csv \
  --project examples/simple/project.toml \
  --data-root generated/csv \
  --out generated/debug-json
```

## Workspace Architecture

- `sora-cli`: command-line interface.
- `sora-core`: pipeline orchestration.
- `sora-input`: input adapter traits and loaded in-memory input.
- `sora-input-toml`: TOML schema and TOML data input adapter.
- `sora-input-xlsx`: Excel `.xlsx` data input adapter.
- `sora-schema`: format-neutral schema model.
- `sora-ir`: normalized schema IR and type parsing.
- `sora-data`: data IR and validation.
- `sora-codegen`: Rust and Kotlin code generation.
- `sora-export`: exporter trait, registry, and built-in exporters.
- `sora-excel`: Excel `.xlsx` template projection.
- `sora-diagnostics`: shared typed errors.
- `sora-templates`: built-in template location helpers.

## Schema Format

TOML projects use a root manifest plus included schema modules. The root manifest declares the package and module list:

```toml
package = "game_config"
includes = ["schema/items.toml", "schema/skills.toml"]
```

Included modules define enums, structs, tables, fields, keys, comments, source files, and future aggregation metadata. Field type strings are normalized into IR types such as `i32`, `string`, `enum<ItemType>`, `list<i32>`, `array<i32,3>`, `ref<Item.id>`, and `optional<string>`.

Table data sources are structured:

```toml
[tables.source]
format = "xlsx"
file = "Item.xlsx"
sheet = "Item"
```

Multiple tables may point at different sheets in the same workbook by reusing the same `file` with different `sheet` values.

## Input Architecture

Sora core consumes input through `SchemaInput` and `DataInput` traits. Concrete source formats live in separate adapter crates. TOML is implemented by `sora-input-toml`, CSV by `sora-input-csv`, and Excel by `sora-input-xlsx`, not by `sora-core` or `sora-input`. Future adapters, such as RON, JSON, or Luban compatibility importers, should translate their source format into `SchemaFile` and `ConfigData` before entering the normal IR, validation, codegen, and exporter pipeline.

## Data Format

The primary data source is generated Excel `.xlsx`. Each table declares its workbook and sheet in schema:

```toml
[tables.source]
format = "xlsx"
file = "Item.xlsx"
sheet = "Item"
```

The CLI can still read TOML row data through `--data-format toml` for tests and simple automation, and CSV row data through `--data-format csv` when each file has a header row matching schema field names. Validation checks required fields, unknown fields, primitive compatibility, enum values, ranges, struct fields, references, map keys, and singleton row counts.

Inline object fields can use tuple parsing when JSON is too verbose for table editing. Define a struct, then set `parser = "tuple"` and a `separator` on a `struct<T>` field:

```toml
[[structs]]
name = "ResourceCost"

[[structs.fields]]
name = "kind"
type = "enum<ResourceType>"

[[structs.fields]]
name = "id"
type = "i32"

[[structs.fields]]
name = "count"
type = "i32"

[[tables.fields]]
name = "cost"
type = "struct<ResourceCost>"
parser = "tuple"
separator = ","
```

The table cell is filled in struct field order:

```text
Item,2003,4
```

Generated Excel templates expand tuple struct fields in the `#type` row, for example `struct<ResourceCost>(kind: enum<ResourceType>, id: i32, count: i32)`, and include the same shape in the cell note.

## Exporter Architecture

Exporters implement a common `DataExporter` trait and are selected by format name through `ExporterRegistry`. Built-in formats are:

- `binary`: writes a production-oriented `.sora` bundle file.
- `json-debug`: writes deterministic per-table JSON files for inspection.

The binary bundle uses a language-neutral sectioned layout: a fixed header, a section directory, a schema section, and one raw table section per table. Compression is currently `none` at the section level, leaving room for future LZ4/Zstd without changing the table row encoding.

## Codegen Architecture

Codegen uses MiniJinja templates, but type mapping is computed in Rust before rendering. Rust generation includes models plus a small `runtime.rs` reader for `.sora` binary bundles. Kotlin currently generates models only. Future targets may include TypeScript, Java, C#, Go, Lua, and Python.

## Excel Template Projection

Sora generates `.xlsx` templates from schema IR. Header rows include the table name, mode, key, schema hash, field names, field types, rules, and descriptions. These headers are projections for human editing and future verification; they are not authoritative schema.

## Roadmap

Phase 2:

- Stable binary reader API refinements.

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
