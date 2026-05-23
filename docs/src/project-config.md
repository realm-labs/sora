# Project Config

The project manifest can be used as a simple schema root or as a full build description.

```toml
package = "game_config"
includes = ["schema/items.toml"]

[build]
data_format = "xlsx"
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

Run every configured output:

```bash
sora build --project project.toml
```

Run one configured codegen target:

```bash
sora build --project project.toml --target rust
```

## Target Options

Language-specific options live under `[codegen.<target>]`:

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

These options are consumed by the selected generator. The normalized IR stays language-neutral.
