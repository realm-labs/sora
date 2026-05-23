# Runtime Formats

Select a runtime format per codegen target:

```toml
[codegen.rust]
runtime_format = "sora"
```

## Support Matrix

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

Dependency meanings:

| Kind | Meaning |
| --- | --- |
| self-contained | Generated runtime includes the decoder. |
| standard library | Generated runtime uses the language standard library. |
| managed dependency | Generated runtime expects normal package dependencies for that ecosystem. |
| user adapter | Generated runtime exposes an adapter hook and the application supplies the concrete decoder. |

The CI runtime matrix generates every supported combination in this table and syntax-checks languages where the check is lightweight.
