# Sora

Sora is a configuration table compiler for games and data-heavy tools. It turns a small schema plus editable table data into strongly typed code and runtime-ready data bundles.

The goal is practical, restrained configuration tooling. A project should be able to keep data in Excel or another familiar source, generate the code and data files needed by runtime systems, and still leave the workbook understandable to someone who has not read a large manual first.

[Documentation](https://realm-labs.github.io/sora/) · [中文文档](https://realm-labs.github.io/sora/zh/)

## Purpose

Sora exists to make configuration tables simple to edit, validate, generate, and ship:

- describe the data contract once in schema files;
- generate spreadsheet templates from that schema instead of hand-maintaining headers;
- load data from Excel `.xlsx`, CSV, or TOML row files;
- validate required fields, types, references, indexes, defaults, and nested values;
- export compact runtime bundles and inspection-friendly JSON;
- generate typed access code for the languages used by the project.

The project deliberately avoids turning table editing into a second programming language. When a feature would make a normal spreadsheet hard to read, Sora prefers explicit schema, generated templates, references to child tables, and small parser conventions over large cell-local DSLs. Advanced cases should be possible, but the common path should remain obvious from the sheet.

## Status

Sora is in an early but runnable milestone. The public schema and CLI can still change while the core model settles.

Current support includes:

- schema files in TOML, YAML, JSON, or Lua;
- table data from Excel `.xlsx`, CSV, or TOML;
- generated Excel `.xlsx` templates;
- normalized IR, recursive validation, defaults, references, derived child-table fields, polymorphic unions, and secondary unique indexes;
- schema locks and config diffs;
- runtime exports as Sora binary, JSON, debug JSON, CBOR, Sora Protobuf, and typed Protobuf;
- generated Rust, Kotlin, C#, Java, Scala, Go, C, C++, TypeScript, JavaScript, Erlang, Lua, Python, and Proto code.

## Design Principles

- Schema is the source of truth.
- Excel and other table sources are editing surfaces, not hidden schemas.
- Generated Excel headers are schema projections, not a second schema.
- Common table cells should stay readable without project-specific DSL knowledge.
- Complex data should move into schema, references, or child tables before it makes a workbook unreadable.
- New concepts need to earn their place; useful beats clever.
- Data exporters are pluggable backends, not hardcoded pipeline stages.
- Debug JSON is useful for inspection, but it is not special in the core architecture.

## Installation

Download the archive for your platform from the GitHub release page, unpack it, and place the `sora` binary on your `PATH`.

Release asset names follow this pattern:

- `sora-vX.Y.Z-windows-x64.zip`
- `sora-vX.Y.Z-linux-x64.tar.gz`
- `sora-vX.Y.Z-macos-x64.tar.gz`
- `sora-vX.Y.Z-macos-arm64.tar.gz`

Each release also publishes a `.sha256` checksum file next to every archive.

For local development from a checkout:

```bash
cargo run -p sora-cli -- --version
cargo install --path crates/sora-cli
```

## Example Commands

The preferred workflow is to declare build outputs in `project.toml` and run one command:

```bash
sora build --project examples/showcase/project.toml
```

For one-off or CI workflows, each stage is still available as a separate command:

```bash
sora check \
  --project examples/simple/project.toml

sora gen --target rust \
  --project examples/simple/project.toml \
  --out generated/rust

sora gen --target kotlin \
  --project examples/simple/project.toml \
  --out generated/kotlin

sora gen --target scala \
  --project examples/simple/project.toml \
  --out generated/scala

sora gen --target c \
  --project examples/simple/project.toml \
  --out generated/c

sora gen --target cpp \
  --project examples/simple/project.toml \
  --out generated/cpp

sora gen --target typescript \
  --project examples/simple/project.toml \
  --out generated/typescript

sora gen --target javascript \
  --project examples/simple/project.toml \
  --out generated/javascript

sora gen --target erlang \
  --project examples/simple/project.toml \
  --out generated/erlang

sora excel-template \
  --project examples/simple/project.toml \
  --out generated/excel

sora export \
  --format binary \
  --default-source-format xlsx \
  --project examples/simple/project.toml \
  --data-root generated/excel \
  --out generated/config.sora

sora export \
  --format json-debug \
  --default-source-format xlsx \
  --project examples/simple/project.toml \
  --data-root generated/excel \
  --out generated/debug-json

sora export \
  --format json-debug \
  --default-source-format csv \
  --project examples/simple/project.toml \
  --data-root generated/csv \
  --out generated/debug-json
```

## Workspace Architecture

- `sora-cli`: command-line interface.
- `sora-config-format`: shared TOML/YAML/JSON/Lua document loading.
- `sora-core`: pipeline orchestration.
- `sora-input`: input adapter traits and loaded in-memory input.
- `sora-input-csv`: CSV data input adapter.
- `sora-input-schema`: schema project file input.
- `sora-input-toml`: TOML data input adapter.
- `sora-input-xlsx`: Excel `.xlsx` data input adapter.
- `sora-schema`: format-neutral schema model.
- `sora-ir`: normalized schema IR and type parsing.
- `sora-data`: data IR and validation.
- `sora-codegen`: Rust, Kotlin, C#, Java, Scala, Go, C, C++, TypeScript, JavaScript, Erlang, Lua, Python, and Proto code generation.
- `sora-export`: exporter trait, registry, and built-in exporters.
- `sora-excel`: Excel `.xlsx` template projection.
- `sora-diagnostics`: shared typed errors.
- `sora-templates`: built-in templates embedded into the CLI binary.

## Schema Format

Projects use a root manifest plus included schema modules. Schema and project files can be written as TOML, YAML, JSON, or Lua; examples here use TOML. The root manifest declares the package and module list:

```toml
package = "game_config"
includes = ["schema/items.toml", "schema/skills.toml"]
```

The same root manifest can declare build outputs. Relative paths are resolved from the project file directory:

```toml
[build]
default_source_format = "xlsx"
data_root = "data"
schema_lock = "generated/schema.lock"
excel_templates = "generated/excel"

[[build.codegen]]
target = "rust"
out = "rust/src/generated"
format = "auto"

[[build.codegen]]
target = "kotlin"
out = "kotlin/src/generated/kotlin"

[[build.codegen]]
target = "csharp"
out = "csharp/src/generated/csharp"

[[build.codegen]]
target = "java"
out = "java/src/generated/java"

[[build.codegen]]
target = "scala"
out = "scala/src/generated/scala"

[[build.codegen]]
target = "go"
out = "go/internal/showcase"
format = "auto"

[[build.codegen]]
target = "c"
out = "c/generated"
format = "auto"

[[build.codegen]]
target = "cpp"
out = "cpp/generated"
format = "auto"

[[build.codegen]]
target = "typescript"
out = "typescript/generated"

[[build.codegen]]
target = "javascript"
out = "javascript/generated"

[[build.codegen]]
target = "erlang"
out = "erlang/generated"
format = "auto"

[[build.codegen]]
target = "lua"
out = "lua/generated"

[[build.exports]]
format = "binary"
out = "generated/config.sora"

[[build.exports]]
format = "json-debug"
out = "generated/debug-json"
```

`sora build --project project.toml --target rust --clean` rebuilds only the configured Rust codegen target, while schema lock, Excel templates, and exports still follow the project build config.

Codegen targets can opt into post-generation formatting with `format = "never"`, `format = "auto"`, or `format = "required"`. `auto` runs a supported formatter when the command exists in `PATH`; `required` fails the build if the formatter is missing or exits with an error. Built-in formatter hooks cover Rust (`rustfmt`), Go (`gofmt`), Erlang (`erlfmt`), Python (`black`), C (`clang-format`), C++ (`clang-format`), and Scala (`scalafmt`).

Included modules define enums, structs, unions, tables, fields, keys, indexes, comments, source files, and derived field metadata. Field type strings are normalized into IR types such as `i32`, `string`, `enum<ItemType>`, `struct<ResourceCost>`, `union<RewardAction>`, `list<i32>`, `array<i32,3>`, `ref<Item.id>`, and `optional<string>`.

Table data sources are structured:

```toml
[tables.source]
format = "xlsx"
file = "Item.xlsx"
sheet = "Item"
```

Multiple tables may point at different sheets in the same workbook by reusing the same `file` with different `sheet` values.

## Input Architecture

Sora core consumes input through `SchemaInput` and `DataInput` traits. Shared TOML/YAML/JSON/Lua document parsing lives in `sora-config-format`; schema project loading lives in `sora-input-schema`; concrete table data formats live in separate adapter crates. TOML row data is implemented by `sora-input-toml`, CSV by `sora-input-csv`, and Excel by `sora-input-xlsx`, not by `sora-core` or `sora-input`. When a caller needs a full project input, `sora-input::project::SplitProjectInput` composes one schema input with one data input without making either adapter own the other side.

Cell parser behavior is registry-driven. `sora-ir::parser::ParserRegistry` validates parser metadata during schema normalization, and `sora-input::parser::ParserRegistry` executes cell parsing at input time. The default registries include `split`, `tuple`, `tuple_list`, and `json`; library users can register additional Rust parser implementations and call the `_with_parsers` APIs when they need project-specific DSLs.

Build execution is routed through `sora-execution::ExecutionContext`. The default context enables parallel work through Rayon, while library callers can construct a serial context or a fixed-size thread pool and pass it through the `_with_context` pipeline/input/export APIs.

## Data Format

The primary data source is generated Excel `.xlsx`. Each table declares its workbook and sheet in schema:

```toml
[tables.source]
format = "xlsx"
file = "Item.xlsx"
sheet = "Item"
```

The CLI can still read TOML row data through `--default-source-format toml` for tests and simple automation, and CSV row data through `--default-source-format csv` when each file has a header row matching schema field names. Validation checks required fields, unknown fields, primitive compatibility, enum values, ranges, struct fields, references, map keys, and singleton row counts.

Inline object fields can use tuple parsing when JSON is too verbose for table editing. Define a struct, then set `parser = { kind = "tuple" }` on a `struct<T>` field:

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
parser = { kind = "tuple" }
```

The table cell is filled in struct field order:

```text
Item,2003,4
```

Lists of small inline structs can use `tuple_list`. The default item separator is `|`, and each item still uses comma-separated struct fields:

```toml
[[tables.fields]]
name = "materials"
type = "list<struct<ResourceCost>>"
parser = { kind = "tuple_list" }
```

```text
Item,2003,4|Gold,0,1000
```

Use `item_separator` when `;` is preferred:

```toml
parser = { kind = "tuple_list", item_separator = ";" }
```

```text
Item,2003,4;Gold,0,1000
```

Generated Excel templates expand tuple struct fields in the `#type` row, for example `struct<ResourceCost>(kind: enum<ResourceType>, id: i32, count: i32)` or `list<struct<ResourceCost>>(kind: enum<ResourceType>, id: i32, count: i32)`, and include the same shape in the cell note.

## Exporter Architecture

Exporters implement a common `DataExporter` trait and are selected by format name through `ExporterRegistry`. Built-in formats are:

- `binary`: writes a production-oriented `.sora` bundle file.
- `json-debug`: writes deterministic per-table JSON files for inspection.
- `json`: writes a runtime JSON bundle.
- `cbor`: writes a runtime CBOR bundle.
- `sora-protobuf`: writes a Sora value-model Protobuf bundle.
- `proto`: writes a typed Protobuf bundle.

The binary bundle uses a language-neutral sectioned layout: a fixed header, a section directory, a schema section, and one raw table section per table. Compression is `none` at the section level.

## Codegen Architecture

Codegen uses MiniJinja templates embedded into the CLI binary, but type mapping is computed in Rust before rendering. Rust, Kotlin, C#, Java, Scala, Go, C, C++, TypeScript, JavaScript, Erlang, Lua, and Python generation include models plus small binary runtime readers for `.sora` bundles. Scala generation supports `scala_version = "2.12" | "2.13" | "3"` with Scala 3 as the default; Scala 3 emits native `enum`, while Scala 2 emits `sealed trait` plus `case object` enums. C generation emits `.h/.c` files with explicit decode/free lifecycles and supports `c_standard = "c99" | "c11" | "c17" | "c23"` with `c11` as the default. C++ generation emits header-only C++ and supports `cpp_standard = "c++11" | "c++14" | "c++17" | "c++20" | "c++23"` with `c++17` as the default. TypeScript and JavaScript generation target modern ESM and map `i64` to `bigint`; JavaScript generation also emits `.d.ts` files by default. Erlang generation emits plain `.erl` modules, maps rows to maps, uses UTF-8 binaries for strings, and supports `enum_repr = "atom" | "integer"`. Lua generation emits EmmyLua annotations for editor type hints and supports `lua_version = "5.1"`, `"5.2"`, `"5.3"`, `"5.4"`, or `"luajit"`; Lua 5.3/5.4 use `string.unpack`, while older runtimes get a generated compatibility decoder. Lua, TypeScript, and JavaScript support `enum_repr = "string" | "integer"`.

## Excel Template Projection

Sora generates `.xlsx` templates from schema IR. Header rows include the table name, mode, key, schema hash, field names, field types, rules, and descriptions. These headers are projections for human editing; they are not authoritative schema.
