# Load Generated Code

Generated code contains strongly typed row models, table containers, and a config loader for the selected runtime format.

## Choose a Runtime Format

```toml
[codegen.rust]
runtime_format = "sora"
```

The runtime format selected by code generation must match an exported bundle:

```toml
[[build.exports]]
format = "binary"
out = "generated/config.sora"
```

`runtime_format = "sora"` corresponds to the `binary` export. `json`, `cbor`, and `sora-protobuf` correspond to their matching export formats.

## Rust Example

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

Exact names are derived from schema names and target language conventions. For example, a table named `Item` generally becomes an item row type plus an item table accessor.

## Adapter Targets

Some targets expose adapter hooks for formats where the ecosystem dependency should be supplied by the application. For example, Lua, Erlang, and Dart can accept `decode_cbor` or `decode_sora_protobuf` functions instead of embedding a specific third-party decoder.

See [Runtime Adapters](../codegen/adapters.md) for examples.
