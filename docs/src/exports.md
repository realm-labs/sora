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
| `i18n-binary` | Binary locale pack for one locale. |
| `i18n-json` | JSON locale pack for one locale. |

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

[[build.exports]]
format = "i18n-binary"
out = "generated/i18n/zh_cn.sora-i18n"
locale = "zh_cn"
```

When `sora build` runs, it checks that configured codegen targets have a matching export for their selected runtime format.

Localization packs are separate runtime assets and are mounted by the generated i18n runtime. See [Localization](localization.md).
