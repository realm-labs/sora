# 加载生成代码

生成代码包含强类型 row model、table container，以及所选 runtime format 的 config loader。

## 选择 Runtime Format

```toml
[codegen.rust]
runtime_format = "sora"
```

代码生成选择的 runtime format 必须有匹配的导出数据包：

```toml
[[build.exports]]
format = "binary"
out = "generated/config.sora"
```

`runtime_format = "sora"` 对应 `binary` 导出。`json`、`cbor` 和 `sora-protobuf` 分别对应同名导出格式。

## Rust 示例

```rust
mod generated;

use generated::SoraConfig;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bytes = std::fs::read("generated/config.sora")?;
    let config = SoraConfig::from_sora_bytes(&bytes)?;

    if let Some(item) = config.items.get(&1001) {
        println!("{} stacks to {}", item.name, item.max_stack);
    }

    Ok(())
}
```

具体名称会由 schema 名称和目标语言命名习惯决定。例如 `Item` 表通常会生成 item row type 和 item table accessor。

## Adapter Targets

有些目标语言会为部分格式暴露 adapter hook，因为具体生态依赖应由应用自己提供。例如 Lua、Erlang 和 Dart 可以传入 `decode_cbor` 或 `decode_sora_protobuf` 函数，而不是让生成代码绑定某个第三方解码库。

示例见[运行时适配器](../codegen/adapters.md)。
