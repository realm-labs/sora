# 运行时格式

按 codegen target 选择 runtime format：

```toml
[codegen.rust]
runtime_format = "sora"
```

Runtime format 是生成代码能加载的数据格式。它们对应导出格式：

| Codegen `runtime_format` | Required Export |
| --- | --- |
| `sora` | `binary` |
| `json` | `json` |
| `cbor` | `cbor` |
| `sora-protobuf` | `sora-protobuf` |

这个设置不会改变 Excel、CSV、TOML 或 schema 文件。它只决定目标语言生成什么 loader。选定的 runtime format 必须在项目 build 中有匹配的 export。

## 支持矩阵

| Target | `sora` | `json` | `cbor` | `sora-protobuf` |
| --- | --- | --- | --- | --- |
| Rust | self-contained | managed dependency | managed dependency | managed dependency |
| Kotlin | self-contained | managed dependency | managed dependency | managed dependency |
| C# | self-contained | managed dependency | managed dependency | managed dependency |
| Java | self-contained | managed dependency | managed dependency | managed dependency |
| Scala | self-contained | managed dependency | managed dependency | managed dependency |
| Go | self-contained | managed dependency | managed dependency | managed dependency |
| TypeScript | self-contained | managed dependency | managed dependency | managed dependency |
| JavaScript | self-contained | managed dependency | managed dependency | managed dependency |
| Python | self-contained | managed dependency | managed dependency | managed dependency |
| Dart | not supported | standard library | user adapter | user adapter |
| Godot | not supported | standard library | not supported | not supported |
| C | self-contained | not supported | not supported | not supported |
| C++ | self-contained | not supported | not supported | not supported |
| Erlang | self-contained | user adapter | user adapter | user adapter |
| Lua | self-contained | user adapter | user adapter | user adapter |

依赖类型含义：

| Kind | Meaning |
| --- | --- |
| self-contained | 生成 runtime 内置 decoder。 |
| standard library | 生成 runtime 使用语言标准库。 |
| managed dependency | 生成 runtime 预期使用该生态的常规 package dependency。 |
| user adapter | 生成 runtime 暴露 adapter hook，由应用提供具体 decoder。 |

## 如何选择

目标支持时，优先用 `sora` 获得原生 Sora binary bundle。

需要更强可检查性、工具友好性或平台接入简单性时，用 `json`。

已有 CBOR 依赖，并希望使用通用紧凑二进制 value format 时，用 `cbor`。

运行环境偏好 Protobuf transport，但仍想保留 Sora 的 schema-driven value model 时，用 `sora-protobuf`。

CI runtime matrix 会生成此表中每个支持组合，并对轻量可检查的语言做语法检查。
