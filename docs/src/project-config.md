# Project Config

The project manifest can be used as a simple schema root or as a full build description. It can be written as TOML, YAML, JSON, or Lua; examples on this page use TOML.

```toml
package = "game_config"
includes = ["schema/items.toml"]

[parsers]
scripts = ["tools/parsers.lua"]

[type_mappings]
scripts = ["tools/type_mappings.lua"]

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

Run every configured output:

```bash
sora build --project project.toml
```

`data_root` and `excel_templates` serve different purposes. `data_root` is the input directory used by export and build, so it contains edited table rows. `excel_templates` is an output directory for generated workbook templates, so it can be deleted and regenerated after schema changes. Do not point `excel_templates` at your edited data directory unless replacing those workbooks is intentional.

`[parsers].scripts` lists custom Lua cell parser scripts used by CLI commands that read the project. Paths are relative to the project file. See [Cell Parsers](schema/parsers.md#custom-lua-parsers) for the script API.

`[type_mappings].scripts` lists Lua scripts that customize generated language types. Paths are relative to the project file. Type mappings are codegen-only: the schema still uses language-neutral Sora types such as `struct<Vec3>`, while the mapping script can map that named type to a target-specific type.

Localization is declared at the project root with `[localization]`. Its sources are independent from normal `[[tables]]`; see [Localization](localization.md).

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

Type mapping scripts return a table with `type_mappings`. Each mapping targets one language and one named schema type:

```lua
return {
  type_mappings = {
    {
      target = "csharp",
      schema_type = "Vec3",
      type_name = "Vector3",
      decode = "GameMappings.ToVector3({value})",
      value_decode = "GameMappings.ToVector3({value})",
      imports = { "UnityEngine" },
    },
  },
}
```

`decode` wraps the normal binary runtime decode expression, and `value_decode` wraps JSON/CBOR/protobuf-style value decode. The `{value}` placeholder is replaced with the generated default expression.

`imports` is target-specific and is only emitted by language generators that need it. C#, Java, Kotlin, and Scala expect an import namespace/path without the leading keyword. Go expects an import spec such as `"example.com/game/vector"`. Python, TypeScript, JavaScript, Dart, and Godot expect a complete import/preload line.

`runtime_format` can be `sora`, `json`, `cbor`, or `sora-protobuf`, but not every target supports every runtime format. See [Runtime Formats](codegen/runtime-formats.md) for the support matrix.

## Built-In Target Options

| Target | Options |
| --- | --- |
| `rust` | `runtime_format` default `sora`; `map_type = "std"` or `"fx_hash_map"` default `std`; `string_storage = "owned"` or `"arc"` default `owned`. |
| `kotlin` | `runtime_format` default `sora`. |
| `csharp` | `runtime_format` default `sora`. |
| `java` | `runtime_format` default `sora`. |
| `scala` | `runtime_format` default `sora`; `scala_version = "2.12"`, `"2.13"`, or `"3"` default `3`. |
| `go` | `runtime_format` default `sora`. |
| `dart` | `runtime_format = "json"`, `"cbor"`, or `"sora-protobuf"`. Set this explicitly; `sora` is not supported for Dart. |
| `godot` | `runtime_format = "json"`. Set this explicitly; it is the only supported Godot runtime format. |
| `c` | `runtime_format = "sora"`; `c_standard = "c99"`, `"c11"`, `"c17"`, or `"c23"` default `c11`; `prefix` optional symbol prefix. |
| `cpp` | `runtime_format = "sora"`; `cpp_standard = "c++11"`, `"c++14"`, `"c++17"`, `"c++20"`, or `"c++23"` default `c++17`; `namespace` optional C++ namespace. |
| `typescript` | `runtime_format` default `sora`; `enum_repr = "string"` or `"integer"` default `string`. |
| `javascript` | `runtime_format` default `sora`; `enum_repr = "string"` or `"integer"` default `string`; `emit_dts` boolean default `true`. |
| `erlang` | `runtime_format` default `sora`; `enum_repr = "atom"` or `"integer"` default `atom`. |
| `lua` | `runtime_format` default `sora`; `module` optional require/import prefix; `lua_version = "5.1"`, `"5.2"`, `"5.3"`, `"5.4"`, or `"luajit"` default `5.4`; `enum_repr = "string"` or `"integer"` default `string`. |
| `python` | `runtime_format` default `sora`. |
| `proto-schema` | No target options. Generates `.proto` schema files instead of a runtime loader. |

Example with several language-specific options:

```toml
[codegen.rust]
runtime_format = "sora"
map_type = "fx_hash_map"
string_storage = "arc"

[codegen.cpp]
runtime_format = "sora"
cpp_standard = "c++20"
namespace = "game::config"

[codegen.javascript]
runtime_format = "json"
enum_repr = "integer"
emit_dts = true
```
