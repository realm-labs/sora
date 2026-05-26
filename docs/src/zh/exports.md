# 数据导出

Sora 将数据导出和语言代码生成分离。

Exporter 接收已经校验的数据，并写出运行时数据包。生成代码随后读取这些数据包格式。这允许同一份 schema 和数据服务多个语言或不同运行时存储选择。

简化来看：

```text
source data -> export format -> generated code runtime_format
```

例如生成的 Rust 代码使用 `runtime_format = "sora"` 时，构建配置里也必须写出一个 `binary` export。代码生成决定“怎么读”，数据导出负责“写出要读的文件”。

## 内置导出

| Format | Purpose |
| --- | --- |
| `binary` | 原生 sectioned Sora binary bundle。 |
| `json-debug` | 便于检查的人类可读 debug 输出。 |
| `json` | Runtime JSON bundle。 |
| `cbor` | Runtime CBOR bundle。 |
| `sora-protobuf` | 使用 Sora value model 的 runtime Protobuf bundle。 |
| `proto` | 使用生成出的业务 schema 的 typed Protobuf bundle。 |
| `i18n-binary` | 单个 locale 的二进制语言包。 |
| `i18n-json` | 单个 locale 的 JSON 语言包。 |

codegen 中的 `runtime_format = "sora"` 对应 `binary` 导出。

## 命令示例

```bash
sora export \
  --format binary \
  --default-source-format xlsx \
  --project project.toml \
  --data-root data \
  --out generated/config.sora
```

## Build Manifest 示例

构建清单可以声明多个导出：

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

`sora build` 运行时会检查配置的 codegen target 是否有匹配的 runtime format 导出。

语言包是独立运行时资源，由生成的 i18n runtime 挂载。见[多语言](localization.md)。
