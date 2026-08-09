# 项目配置

项目清单既可以只是 schema root，也可以是完整的构建描述。它可以写成 TOML、YAML、JSON 或 Lua；本页示例使用 TOML。

```toml
includes = ["schema/items.toml"]

[project]
id = "game_config"
views = ["views/client.toml", "views/server.toml"]

[groups.common]
default = true

[groups.server]

[parsers]
scripts = ["tools/parsers.lua"]

[type_mappings]
scripts = ["tools/type_mappings.lua"]

[source_loaders]
scripts = ["tools/source_loaders.lua"]

[build]
default_source_format = "xlsx"
data_root = "data"
view = "client"
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

`project.id` 是 canonical schema 的稳定身份，不会再被拿来推导任何语言的
package 或 namespace。每张表也必须声明稳定 `id`；重命名表不会改变这个身份。

Group 在项目中集中声明。Schema 实体和字段通过 `groups = [...]` 归组；省略时
继承所有 `default = true` 的 group。

View 是具名的外部契约。它选择 group 和表，可以按稳定表 ID 修改导出表名，并
持有各目标语言自己的 binding：

```toml
# views/client.toml
name = "client"
contract = "game/client-v1"
groups = ["common"]

[tables]
include = ["item", "settings"]

[names.tables]
item = "ClientItem"

[bindings.kotlin]
package = "com.example.game.config"

[bindings.csharp]
namespace = "Example.Game.Config"

[bindings.go]
package = "gameconfig"

[bindings.c]
prefix = "game_config"

[bindings.cpp]
namespace = "game::config"

[bindings.proto-schema]
package = "game.config"
```

表选择和别名始终使用稳定表 ID，而不是显示名称。不同 manifest 可以引用不同
view 文件，让每个消费方拥有独立版本的契约，同时共享同一份 canonical schema。

运行所有配置好的输出：

```bash
sora build --project project.toml
```

`data_root` 和 `excel_templates` 的用途不同。`data_root` 是 export 和 build 读取的输入目录，里面放已经填写过行数据的文件。`excel_templates` 是生成 workbook 模板的输出目录，schema 变更后可以删除并重新生成。不要把 `excel_templates` 指向已经编辑过的数据目录，除非你明确想替换那些 workbook。

`[parsers].scripts` 列出 CLI 读取该 project 时使用的自定义 Lua 单元格 parser 脚本。路径相对 project 文件所在目录。脚本 API 见[单元格 Parser](schema/parsers.md#自定义-lua-parser)。

`[type_mappings].scripts` 列出用于自定义生成语言类型的 Lua 脚本。路径相对 project 文件所在目录。类型映射只影响 codegen：schema 仍然使用 `struct<Vec3>` 这类语言无关的 Sora 类型，映射脚本可以把这个命名类型映射到目标语言自己的类型。

`[source_loaders].scripts` 列出项目级、只读的 Lua 表数据源 Loader。路径相对 project 文件所在目录。运行时会把这些 Loader 与内置格式注册到同一个 Registry，因此 build export、直接 export、查询、校验、Studio 和 MCP 使用同一组 Loader。API 与安全模型见 [Lua Source Loader](extension/source-loaders.md)。

多语言通过 project root 的 `[localization]` 声明。它的 sources 独立于普通 `[[tables]]`；见[多语言](localization.md)。

只运行一个配置好的 codegen target：

```bash
sora build --project project.toml --target rust
```

## Target Options

语言相关的运行时选项放在 `[codegen.<target>]` 下。Package、namespace、
module 和符号前缀归各个 view 的 binding 所有：

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

类型映射脚本返回带 `type_mappings` 的 table。每条映射对应一个目标语言和一个命名 schema 类型：

```lua
return {
  type_mappings = {
    {
      target = "csharp",
      schema_type = "Vec3",
      type_name = "Vector3",
      nullable_type_name = "Vector3?",
      decode = "GameMappings.ToVector3({value})",
      value_decode = "GameMappings.ToVector3({value})",
      imports = { "UnityEngine" },
    },
  },
}
```

`nullable_type_name` 是可选字段。当 `optional<schema_type>` 需要不同于后端默认 nullable wrapper 的目标语言类型表达式时使用它。

`decode` 包裹默认的 binary runtime decode 表达式，`value_decode` 包裹 JSON/CBOR/protobuf 风格的 value decode 表达式。`{value}` 会替换成生成器默认生成的表达式。

C target 使用写入目标指针的 decode 函数，所以 C 映射应使用 `decode_into`，而不是 `decode`。`{target}` 会替换成输出指针表达式。C 映射也可以提供 `free`，其中 `{target}` 会替换成需要释放的指针：

```lua
{
  target = "c",
  schema_type = "Vec3",
  type_name = "game_vector3",
  decode_into = "game_vector3_decode(reader, {target})",
  free = "game_vector3_free({target});",
  imports = { "#include \"vector3.h\"" },
}
```

`imports` 是目标语言相关的，只由需要它的语言生成器输出。C#、Java、Kotlin、Scala 期望不带关键字的 namespace/path；Go 期望类似 `"example.com/game/vector"` 的 import spec；Python、TypeScript、JavaScript、Dart、Godot、C、C++、Rust 期望完整 import/include/use/preload 行。

`runtime_format` 可以是 `sora`、`json`、`cbor` 或 `sora-protobuf`，但不是每个 target 都支持所有 runtime format。支持矩阵见[运行时格式](codegen/runtime-formats.md)。

## 内置 Target Options

| Target | Options |
| --- | --- |
| `rust` | `runtime_format` 默认 `sora`；`map_type = "std"` 或 `"fx_hash_map"`，默认 `std`；`string_storage = "owned"` 或 `"arc"`，默认 `owned`。 |
| `kotlin` | `runtime_format` 默认 `sora`；view binding 必须提供 `package`。 |
| `csharp` | `runtime_format` 默认 `sora`；view binding 必须提供 `namespace`。 |
| `java` | `runtime_format` 默认 `sora`；view binding 必须提供 `package`；`nullable_annotation` 默认 `SoraNullable`，也可以设置成 `org.jetbrains.annotations.Nullable` 这类 annotation class，或设置为 `""` 禁用 annotation。 |
| `scala` | `runtime_format` 默认 `sora`；view binding 必须提供 `package`；`scala_version = "2.12"`、`"2.13"` 或 `"3"`，默认 `3`。 |
| `go` | `runtime_format` 默认 `sora`；view binding 必须提供 `package`。 |
| `dart` | `runtime_format = "json"`、`"cbor"` 或 `"sora-protobuf"`。建议显式设置；Dart 不支持 `sora`。 |
| `godot` | `runtime_format = "json"`，这是 Godot 唯一支持的 runtime format；`godot_version` 默认 `"4.3"`。Godot 4.4+ 输出还会使用强类型 Dictionary。 |
| `c` | `runtime_format = "sora"`；`c_standard = "c99"`、`"c11"`、`"c17"` 或 `"c23"`，默认 `c11`；view binding 必须提供 `prefix`。 |
| `cpp` | `runtime_format = "sora"`；`cpp_standard = "c++11"`、`"c++14"`、`"c++17"`、`"c++20"` 或 `"c++23"`，默认 `c++17`；view binding 必须提供 `namespace`。 |
| `typescript` | `runtime_format` 默认 `sora`；`enum_repr = "string"` 或 `"integer"`，默认 `string`。 |
| `javascript` | `runtime_format` 默认 `sora`；`enum_repr = "string"` 或 `"integer"`，默认 `string`；`emit_dts` 是 boolean，默认 `true`。 |
| `erlang` | `runtime_format` 默认 `sora`；`enum_repr = "atom"` 或 `"integer"`，默认 `atom`。 |
| `lua` | `runtime_format` 默认 `sora`；`module` 是可选 require/import 前缀；`lua_version = "5.1"`、`"5.2"`、`"5.3"`、`"5.4"` 或 `"luajit"`，默认 `5.4`；`enum_repr = "string"` 或 `"integer"`，默认 `string`。 |
| `python` | `runtime_format` 默认 `sora`。 |
| `proto-schema` | View binding 必须提供 `package`。它生成 `.proto` schema 文件，不生成 runtime loader。 |

包含多种语言选项的示例：

```toml
[codegen.rust]
runtime_format = "sora"
map_type = "fx_hash_map"
string_storage = "arc"

[codegen.cpp]
runtime_format = "sora"
cpp_standard = "c++20"

[codegen.javascript]
runtime_format = "json"
enum_repr = "integer"
emit_dts = true

[codegen.godot]
runtime_format = "json"
godot_version = "4.3"
```
