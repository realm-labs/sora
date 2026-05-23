# 项目配置

项目清单既可以只是 schema root，也可以是完整的构建描述。

```toml
package = "game_config"
includes = ["schema/items.toml"]

[build]
default_source_format = "xlsx"
data_root = "data"
schema_lock = "generated/schema.lock"
excel_templates = "generated/excel"

[[build.codegen]]
target = "rust"
out = "rust/src/generated"
format = "auto"

[[build.exports]]
format = "binary"
out = "generated/config.sora"
```

运行所有配置好的输出：

```bash
sora build --project project.toml
```

只运行一个配置好的 codegen target：

```bash
sora build --project project.toml --target rust
```

## Target Options

语言相关选项放在 `[codegen.<target>]` 下：

```toml
[codegen.rust]
runtime_format = "sora"

[codegen.typescript]
runtime_format = "json"
enum_repr = "string"

[codegen.lua]
runtime_format = "cbor"
lua_version = "5.4"
```

这些选项由对应生成器消费。归一化 IR 保持语言无关。
