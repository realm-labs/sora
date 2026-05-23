# Data Export

Sora separates data export from language code generation.

The exporter receives validated data and writes a runtime bundle. Generated code then reads one of those bundle formats. This lets the same schema and data feed several languages or runtime storage choices.

The short version:

```text
source data -> export format -> generated code runtime_format
```

For example, if generated Rust code uses `runtime_format = "sora"`, the build must also write a `binary` export. Code generation decides how to read; export writes the file that will be read.

## Built-in Exports

| Format | Purpose |
| --- | --- |
| `binary` | Native sectioned Sora binary bundle. |
| `json-debug` | Human-readable debug output for inspection. |
| `json` | Runtime JSON bundle. |
| `cbor` | Runtime CBOR bundle. |
| `sora-protobuf` | Runtime Protobuf bundle using Sora's value model. |
| `proto` | Typed Protobuf bundle using a generated game-specific schema. |

The `binary` export is selected by `runtime_format = "sora"` in codegen options.

## Command Example

```bash
sora export \
  --format binary \
  --default-source-format xlsx \
  --project project.toml \
  --data-root data \
  --out generated/config.sora
```

## Build Manifest Example

Build manifests can declare multiple exports:

```toml
[[build.exports]]
format = "binary"
out = "generated/config.sora"

[[build.exports]]
format = "json-debug"
out = "generated/debug-json"
```

When `sora build` runs, it checks that configured codegen targets have a matching export for their selected runtime format.
