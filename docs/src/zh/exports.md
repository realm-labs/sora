# 数据导出

Sora 将数据导出和语言代码生成分离。

Exporter 接收已经校验的数据，并写出运行时数据包。生成代码随后读取这些数据包格式。这允许同一份 schema 和数据服务多个语言或不同运行时存储选择。

## 内置导出

| Format | Purpose |
| --- | --- |
| `binary` | 原生 sectioned Sora binary bundle。 |
| `json-debug` | 便于检查的人类可读 debug 输出。 |
| `json` | Runtime JSON bundle。 |
| `cbor` | Runtime CBOR bundle。 |
| `sora-protobuf` | 使用 Sora value model 的 runtime Protobuf bundle。 |
| `proto` | 使用生成出的业务 schema 的 typed Protobuf bundle。 |

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
```

`sora build` 运行时会检查配置的 codegen target 是否有匹配的 runtime format 导出。
