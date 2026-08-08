# Runtime Formats

Select a runtime format per codegen target:

```toml
[codegen.rust]
runtime_format = "sora"
```

Runtime formats are the formats generated code can load. They correspond to export formats:

| Codegen `runtime_format` | Required Export |
| --- | --- |
| `sora` | `binary` |
| `json` | `json` |
| `cbor` | `cbor` |
| `sora-protobuf` | `sora-protobuf` |

This setting does not change Excel, CSV, TOML, JSON, YAML, or schema files. It only changes the loader generated for the target language. The selected runtime format must have a matching export in the project build.

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

## Choosing a Format

Use `sora` when you want the native Sora binary bundle and the target supports it.

Use `json` when inspectability, tooling, or platform simplicity matters more than compactness.

Use `cbor` when you want a compact general-purpose binary value format and your runtime already has a CBOR dependency.

Use `sora-protobuf` when your environment prefers Protobuf transport but you still want Sora's schema-driven value model.

The CI runtime matrix generates every supported combination in this table and syntax-checks languages where the check is lightweight.

## Godot JSON and Generated Types

Godot codegen targets Godot 4.3 by default. Set the target version when the generated project uses a newer Godot release:

```toml
[codegen.godot]
runtime_format = "json"
godot_version = "4.4"
```

Generated GDScript uses concrete table key and index parameter types, typed row arrays, and one class per discriminated-union variant. Godot 4.4 and newer additionally use typed dictionaries; Godot 4.3 keeps dictionaries untyped because that syntax is unavailable there. Nested typed collections fall back to `Array` or `Dictionary` where GDScript does not allow a typed collection as another typed collection's element type.

The generated JSON loader preserves `i64`, duration, and datetime integer values outside JSON's interoperable safe-integer range. It quotes those number tokens internally before calling Godot's JSON parser, then decodes their decimal strings back to exact 64-bit integers. The exported JSON format itself does not change.

Optional primitive values remain `Variant` because GDScript has no nullable primitive type. Optional generated records, unions, and text keys retain their concrete object type and use `null` as the empty value.
