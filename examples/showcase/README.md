# Sora Showcase Example

This example is intentionally larger than `examples/simple`.

It keeps a real Excel workbook in `data/GameConfig.xlsx` and generated outputs in
`generated/` so the whole pipeline is easy to inspect:

- schema: `project.toml` and `schema/game.toml`
- Excel data: `data/GameConfig.xlsx`
- generated Rust: `generated/rust`
- generated Kotlin: `generated/kotlin`
- binary bundle: `generated/config.sora`
- debug JSON: `generated/debug-json`
- schema lock: `generated/schema.lock`

Regenerate everything:

```powershell
cargo run -p sora-showcase-builder
```

Run the Rust smoke example:

```powershell
cargo run -p sora-showcase-rust-smoke
```

Run the Kotlin smoke example when a Kotlin-capable Gradle setup is available:

```powershell
gradle -p examples/showcase/kotlin-smoke run
```
