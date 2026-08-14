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

For a standalone generated crate, configure Rust codegen like this:

```scon
build {
  codegen = [{ target = "rust", out = "generated/game-config" }]
}

codegen {
  rust {
    runtime_format = "sora"
    crate { name = "game-config" }
  }
}
```

Add it to the consuming application's manifest:

```toml
[dependencies]
game-config = { path = "../generated/game-config" }
```

```rust
use game_config::{SoraConfig, runtime::SoraBundle};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bytes = std::fs::read("generated/config.sora")?;
    let bundle = SoraBundle::parse(&bytes)?;
    let config = SoraConfig::from_source(&bundle)?;

    if let Some(item) = config.item().get(&1001) {
        println!("{} stacks to {}", item.name, item.max_stack);
    }

    Ok(())
}
```

Exact names are derived from schema names and target language conventions. For example, a table named `Item` generally becomes an item row type plus an item table accessor.

Omit `codegen.rust.crate` to keep generating a module directory for inclusion with
`mod generated;` inside an existing crate.

## Adapter Targets

Some targets expose adapter hooks for formats where the ecosystem dependency should be supplied by the application. For example, Lua, Erlang, and Dart can accept `decode_cbor` or `decode_sora_protobuf` functions instead of embedding a specific third-party decoder.

See [Runtime Adapters](../codegen/adapters.md) for examples.
