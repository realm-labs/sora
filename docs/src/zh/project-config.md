# 项目配置

项目清单既可以只是 schema root，也可以是完整的构建描述。它可以写成 TOML、YAML、JSON 或 Lua；本页示例使用 TOML。

```toml
package = "game_config"
includes = ["schema/items.toml"]

[parsers]
scripts = ["tools/parsers.lua"]

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

`data_root` 和 `excel_templates` 的用途不同。`data_root` 是 export 和 build 读取的输入目录，里面放已经填写过行数据的文件。`excel_templates` 是生成 workbook 模板的输出目录，schema 变更后可以删除并重新生成。不要把 `excel_templates` 指向已经编辑过的数据目录，除非你明确想替换那些 workbook。

`[parsers].scripts` 列出 CLI 读取该 project 时使用的自定义 Lua 单元格 parser 脚本。路径相对 project 文件所在目录。脚本 API 见[单元格 Parser](schema/parsers.md#自定义-lua-parser)。

多语言通过 project root 的 `[localization]` 声明。它的 sources 独立于普通 `[[tables]]`；见[多语言](localization.md)。

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

`runtime_format` 可以是 `sora`、`json`、`cbor` 或 `sora-protobuf`，但不是每个 target 都支持所有 runtime format。支持矩阵见[运行时格式](codegen/runtime-formats.md)。

## 内置 Target Options

| Target | Options |
| --- | --- |
| `rust` | `runtime_format` 默认 `sora`；`map_type = "std"` 或 `"fx_hash_map"`，默认 `std`；`string_storage = "owned"` 或 `"arc"`，默认 `owned`。 |
| `kotlin` | `runtime_format` 默认 `sora`。 |
| `csharp` | `runtime_format` 默认 `sora`。 |
| `java` | `runtime_format` 默认 `sora`。 |
| `scala` | `runtime_format` 默认 `sora`；`scala_version = "2.12"`、`"2.13"` 或 `"3"`，默认 `3`。 |
| `go` | `runtime_format` 默认 `sora`。 |
| `dart` | `runtime_format = "json"`、`"cbor"` 或 `"sora-protobuf"`。建议显式设置；Dart 不支持 `sora`。 |
| `godot` | `runtime_format = "json"`。建议显式设置；这是 Godot 唯一支持的 runtime format。 |
| `c` | `runtime_format = "sora"`；`c_standard = "c99"`、`"c11"`、`"c17"` 或 `"c23"`，默认 `c11`；`prefix` 是可选 symbol prefix。 |
| `cpp` | `runtime_format = "sora"`；`cpp_standard = "c++11"`、`"c++14"`、`"c++17"`、`"c++20"` 或 `"c++23"`，默认 `c++17`；`namespace` 是可选 C++ namespace。 |
| `typescript` | `runtime_format` 默认 `sora`；`enum_repr = "string"` 或 `"integer"`，默认 `string`。 |
| `javascript` | `runtime_format` 默认 `sora`；`enum_repr = "string"` 或 `"integer"`，默认 `string`；`emit_dts` 是 boolean，默认 `true`。 |
| `erlang` | `runtime_format` 默认 `sora`；`enum_repr = "atom"` 或 `"integer"`，默认 `atom`。 |
| `lua` | `runtime_format` 默认 `sora`；`module` 是可选 require/import 前缀；`lua_version = "5.1"`、`"5.2"`、`"5.3"`、`"5.4"` 或 `"luajit"`，默认 `5.4`；`enum_repr = "string"` 或 `"integer"`，默认 `string`。 |
| `python` | `runtime_format` 默认 `sora`。 |
| `proto-schema` | 没有 target options。它生成 `.proto` schema 文件，不生成 runtime loader。 |

包含多种语言选项的示例：

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
