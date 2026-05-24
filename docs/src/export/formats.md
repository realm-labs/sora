# Export Formats

Export formats are runtime bundle formats. They are independent from source formats such as Excel, CSV, TOML, JSON, or YAML.

| Export | Codegen Runtime Format | Output Shape | Use When |
| --- | --- | --- | --- |
| `binary` | `sora` | Native sectioned binary bundle. | You want a compact self-contained Sora runtime. |
| `json` | `json` | Runtime JSON bundle. | You want easy inspection or simple platform integration. |
| `cbor` | `cbor` | Runtime CBOR bundle. | You want a compact general-purpose binary value format. |
| `sora-protobuf` | `sora-protobuf` | Sora value model encoded with Protobuf. | You want Protobuf-based transport without per-game `.proto` models. |
| `proto` | none | Typed Protobuf bundle using the generated game-specific schema. | You want a business `.proto` contract for external tooling. |
| `json-debug` | none | Per-table debug JSON. | You want reviewable output for inspection and tests. |

Example build outputs:

```toml
[[build.exports]]
format = "binary"
out = "generated/config.sora"

[[build.exports]]
format = "json"
out = "generated/config.json"

[[build.exports]]
format = "json-debug"
out = "generated/debug-json"
```

Generated runtimes only load runtime formats they support. `json-debug` is for humans and tools, not generated runtime loading.
