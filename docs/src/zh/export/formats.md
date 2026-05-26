# 导出格式

导出格式是运行时数据包格式。它和 Excel、CSV、TOML、JSON、YAML 这类 source format 是两回事。

| Export | Codegen Runtime Format | Output Shape | Use When |
| --- | --- | --- | --- |
| `binary` | `sora` | 原生 sectioned binary bundle。 | 需要紧凑、自包含的 Sora runtime。 |
| `json` | `json` | Runtime JSON bundle。 | 需要易检查、易接入平台工具。 |
| `cbor` | `cbor` | Runtime CBOR bundle。 | 需要通用紧凑二进制 value format。 |
| `sora-protobuf` | `sora-protobuf` | 用 Protobuf 编码的 Sora value model。 | 想使用 Protobuf transport，但不想为每个游戏维护 `.proto` model。 |
| `proto` | none | 使用生成出的业务 schema 的 typed Protobuf bundle。 | 需要面向外部工具的业务 `.proto` 契约。 |
| `json-debug` | none | 按表输出的 debug JSON。 | 用于检查、review 或测试。 |
| `i18n-binary` | none | 单个 locale 的原生二进制语言包。 | 需要和配置包分开挂载的生产多语言资源。 |
| `i18n-json` | none | 单个 locale 的 debug JSON 语言包。 | 需要可 review 的文本，用于外包翻译交付或测试。 |

构建输出示例：

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

[[build.exports]]
format = "i18n-binary"
out = "generated/i18n/zh_cn.sora-i18n"
locale = "zh_cn"
```

生成运行时只加载它支持的 runtime format。`json-debug` 面向人和工具，不用于 generated runtime loading。

多语言导出需要 `[localization]`，并在 build manifest 中为 export 指定 `locale`。见[多语言](../localization.md)。
