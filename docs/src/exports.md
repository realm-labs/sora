# Data Export

Sora separates data export from language code generation.

The exporter receives validated data and writes a runtime bundle. Generated code then reads one of those bundle formats.

## Built-in Exports

| Format | Purpose |
| --- | --- |
| `binary` | Native sectioned Sora binary bundle. |
| `json-debug` | Human-readable debug output for inspection. |
| `json` | Runtime JSON bundle. |
| `cbor` | Runtime CBOR bundle. |
| `sora-protobuf` | Runtime Protobuf bundle using Sora's value model. |

Example:

```bash
sora export \
  --format binary \
  --data-format xlsx \
  --project project.toml \
  --data-root data \
  --out generated/config.sora
```

Build manifests can declare multiple exports:

```toml
[[build.exports]]
format = "binary"
out = "generated/config.sora"

[[build.exports]]
format = "json-debug"
out = "generated/debug-json"
```
