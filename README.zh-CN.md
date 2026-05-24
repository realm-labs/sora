# Sora

[English](README.md) · [中文](README.zh-CN.md)

Sora 是面向游戏和数据密集型工具的配置表编译器。它把一份小型 schema 和可编辑的表格数据转换为强类型代码和可直接用于运行时的数据包。

Sora 的目标是实用、克制的配置工具链。项目可以继续把数据放在 Excel 或其他熟悉的数据源里，同时生成运行时需要的代码和数据文件，并让没有读过大段手册的人也能看懂工作簿。

[文档](https://realm-labs.github.io/sora/zh/) · [English Documentation](https://realm-labs.github.io/sora/)

## 目标

Sora 用来让配置表的编辑、校验、生成和交付更简单：

- 在 schema 文件中一次性描述数据契约；
- 从 schema 生成电子表格模板，避免手工维护表头；
- 从 Excel `.xlsx`、CSV、TOML、JSON 或 YAML 行数据加载数据；
- 校验必填字段、类型、引用、索引、默认值和嵌套值；
- 导出紧凑的运行时数据包和便于检查的 JSON；
- 为项目使用的语言生成强类型访问代码。

项目有意避免把表格编辑变成另一门编程语言。如果某个功能会让普通电子表格变得难读，Sora 会优先选择显式 schema、生成模板、对子表的引用以及少量 parser 约定，而不是在单元格里塞入大型 DSL。高级场景应该可以表达，但常见路径应该从表格本身就能看清楚。

## 状态

Sora 还处在早期但可运行的里程碑阶段。核心模型稳定之前，公开 schema 和 CLI 仍可能变化。

需要稳定生成输出的项目应该固定 `sora` CLI 版本。只有真实的生成 runtime 不兼容时，Sora 才会手动升级 runtime/export format version；当前不会用 edition flag 保留旧 schema 语义。见[版本与兼容性](https://realm-labs.github.io/sora/zh/versioning.html)。

当前支持：

- TOML、YAML、JSON 或 Lua schema 文件；
- 来自 Excel `.xlsx`、CSV、TOML、JSON 或 YAML 的表格数据；
- 在 `project.toml` 中配置或由 CLI 加载的自定义 Lua 单元格 parser；
- Sora Studio，内置在 CLI 中的浏览器可视化 schema 编辑器；
- 生成 Excel `.xlsx` 模板；
- 归一化 IR、递归校验、默认值、引用、派生子表字段、多态联合和 secondary unique index；
- schema lock 和配置 diff；
- 导出 Sora binary、JSON、debug JSON、CBOR、Sora Protobuf 和 typed Protobuf；
- 生成 Rust、Kotlin、C#、Java、Scala、Go、C、C++、TypeScript、JavaScript、Erlang、Lua、Python 和 Proto 代码。

## 设计原则

- Schema 是事实来源。
- Excel 和其他表格来源是编辑界面，不是隐藏 schema。
- 生成的 Excel 表头是 schema 投影，不是第二份 schema。
- 常见表格单元格应该不依赖项目专属 DSL 也能读懂。
- 复杂数据应该先进入 schema、引用或子表，而不是让工作簿变得难读。
- 新概念必须证明自己的价值；有用优先于聪明。
- 数据导出器是可插拔后端，不是硬编码流水线阶段。
- Debug JSON 适合检查，但它在核心架构里没有特殊地位。

## 安装

从 [GitHub Releases](https://github.com/realm-labs/sora/releases) 下载对应平台的压缩包，解压后把 `sora` 二进制放到 `PATH` 中。

Release 资源命名遵循这个模式：

- `sora-vX.Y.Z-windows-x64.zip`
- `sora-vX.Y.Z-linux-x64.tar.gz`
- `sora-vX.Y.Z-macos-arm64.tar.gz`

每个 release 也会为每个压缩包发布对应的 `.sha256` 校验文件。

如果本机已有 Rust 工具链，也可以从 crates.io 安装已发布的 CLI：

```bash
cargo install sora-cli
```

本地开发时可以从源码安装：

```bash
cargo run -p sora-cli -- --version
cargo install --path crates/sora-cli
```

## 示例命令

推荐的工作流是在 `project.toml` 中声明构建输出，然后运行一个命令：

```bash
sora build --project examples/showcase/project.toml
```

如果要用 Sora Studio 查看和编辑同一份 schema：

```bash
sora studio --project examples/showcase/project.toml
```

schema 变更后，如果要更新已有 Excel 数据 workbook 的表头：

```bash
sora excel-sync --project project.toml --data-root data
sora excel-sync --project project.toml --data-root data --write
```

对于一次性任务或 CI 流程，每个阶段也可以单独执行：

```bash
sora check \
  --project examples/simple/project.toml

sora gen --target rust \
  --project examples/simple/project.toml \
  --out generated/rust

sora gen --target kotlin \
  --project examples/simple/project.toml \
  --out generated/kotlin

sora gen --target scala \
  --project examples/simple/project.toml \
  --out generated/scala

sora gen --target c \
  --project examples/simple/project.toml \
  --out generated/c

sora gen --target cpp \
  --project examples/simple/project.toml \
  --out generated/cpp

sora gen --target typescript \
  --project examples/simple/project.toml \
  --out generated/typescript

sora gen --target javascript \
  --project examples/simple/project.toml \
  --out generated/javascript

sora gen --target erlang \
  --project examples/simple/project.toml \
  --out generated/erlang

sora excel-template \
  --project examples/simple/project.toml \
  --out generated/excel

sora export \
  --format binary \
  --default-source-format xlsx \
  --project examples/simple/project.toml \
  --data-root generated/excel \
  --out generated/config.sora

sora export \
  --format json-debug \
  --default-source-format xlsx \
  --project examples/simple/project.toml \
  --data-root generated/excel \
  --out generated/debug-json

sora export \
  --format json-debug \
  --default-source-format csv \
  --project examples/simple/project.toml \
  --data-root generated/csv \
  --out generated/debug-json
```

## Workspace 架构

- `sora-cli`: 命令行接口。
- `sora-config-format`: 共享的 TOML/YAML/JSON/Lua 文档加载。
- `sora-core`: 流水线编排。
- `sora-input`: 输入 adapter trait 和加载后的内存输入。
- `sora-input-csv`: CSV 数据输入 adapter。
- `sora-input-schema`: schema project 文件输入。
- `sora-input-structured`: JSON 和 YAML 数据输入 adapter。
- `sora-input-toml`: TOML 数据输入 adapter。
- `sora-input-xlsx`: Excel `.xlsx` 数据输入 adapter。
- `sora-schema`: 与格式无关的 schema model。
- `sora-ir`: 归一化 schema IR 和类型解析。
- `sora-data`: 数据 IR 和校验。
- `sora-codegen`: Rust、Kotlin、C#、Java、Scala、Go、C、C++、TypeScript、JavaScript、Erlang、Lua、Python 和 Proto 代码生成。
- `sora-export`: exporter trait、注册表和内置 exporter。
- `sora-excel`: Excel `.xlsx` 模板投影。
- `sora-studio`: 由 CLI 提供的内置 schema 可视化和编辑 UI。
- `sora-diagnostics`: 共享的类型化错误。
- `sora-templates`: 嵌入 CLI 二进制的内置模板。

## Schema 格式

项目使用一个根 manifest 加上若干被 include 的 schema module。Schema 和项目文件可以写成 TOML、YAML、JSON 或 Lua；这里的示例使用 TOML。根 manifest 声明 package 和 module 列表：

```toml
package = "game_config"
includes = ["schema/items.toml", "schema/skills.toml"]
```

同一个根 manifest 也可以声明构建输出。相对路径从项目文件所在目录解析：

```toml
[build]
default_source_format = "xlsx"
data_root = "data"
schema_lock = "generated/schema.lock"
excel_templates = "generated/excel"

[[build.codegen]]
target = "rust"
out = "rust/src/generated"
format = "auto"

[[build.codegen]]
target = "kotlin"
out = "kotlin/src/generated/kotlin"

[[build.codegen]]
target = "csharp"
out = "csharp/src/generated/csharp"

[[build.codegen]]
target = "java"
out = "java/src/generated/java"

[[build.codegen]]
target = "scala"
out = "scala/src/generated/scala"

[[build.codegen]]
target = "go"
out = "go/internal/showcase"
format = "auto"

[[build.codegen]]
target = "c"
out = "c/generated"
format = "auto"

[[build.codegen]]
target = "cpp"
out = "cpp/generated"
format = "auto"

[[build.codegen]]
target = "typescript"
out = "typescript/generated"

[[build.codegen]]
target = "javascript"
out = "javascript/generated"

[[build.codegen]]
target = "erlang"
out = "erlang/generated"
format = "auto"

[[build.codegen]]
target = "lua"
out = "lua/generated"

[[build.exports]]
format = "binary"
out = "generated/config.sora"

[[build.exports]]
format = "json-debug"
out = "generated/debug-json"
```

`data_root` 是 export 和 build 读取的源数据目录。`excel_templates` 只是生成 workbook 模板的输出目录。建议始终把两者分开：schema 变更后先重新生成模板到 `generated/excel`，再把行数据复制或迁移到 `data`。

`sora build --project project.toml --target rust --clean` 只会重建配置好的 Rust codegen target；schema lock、Excel 模板和 export 仍按项目 build 配置执行。

Codegen target 可以通过 `format = "never"`、`format = "auto"` 或 `format = "required"` 启用生成后的格式化。`auto` 会在 `PATH` 中存在对应命令时运行支持的 formatter；`required` 会在 formatter 缺失或执行失败时报错。内置 formatter hook 覆盖 Rust (`rustfmt`)、Go (`gofmt`)、Erlang (`erlfmt`)、Python (`black`)、C (`clang-format`)、C++ (`clang-format`) 和 Scala (`scalafmt`)。

Included module 定义枚举、结构体、联合、表、字段、key、index、注释、source 文件和派生字段元数据。字段类型字符串会被归一化为 IR 类型，例如 `i32`、`string`、`enum<ItemType>`、`struct<ResourceCost>`、`union<RewardAction>`、`list<i32>`、`array<i32,3>`、`ref<Item.id>` 和 `optional<string>`。

表数据源是结构化的：

```toml
[tables.source]
format = "xlsx"
file = "Item.xlsx"
sheet = "Item"
```

多个表可以通过复用同一个 `file`、设置不同 `sheet` 指向同一个工作簿中的不同 sheet。

## 输入架构

Sora core 通过 `SchemaInput` 和 `DataInput` trait 消费输入。共享的 TOML/YAML/JSON/Lua 文档解析位于 `sora-config-format`；schema project 加载位于 `sora-input-schema`；具体表格数据格式位于独立 adapter crate。TOML 行数据由 `sora-input-toml` 实现，CSV 由 `sora-input-csv` 实现，JSON/YAML 由 `sora-input-structured` 实现，Excel 由 `sora-input-xlsx` 实现，而不是放在 `sora-core` 或 `sora-input` 中。当调用方需要完整项目输入时，`sora-input::project::SplitProjectInput` 会组合一个 schema input 和一个 data input，而不让任一 adapter 拥有另一侧。

单元格 parser 行为由 registry 驱动。`sora-ir::parser::ParserRegistry` 在 schema 归一化时校验 parser 元数据，`sora-input::parser::ParserRegistry` 在输入阶段执行单元格解析。默认 registry 包含 `split`、`tuple`、`tuple_list` 和 `json`；库用户需要项目专属 DSL 时，可以注册额外的 Rust parser 实现，并调用 `_with_parsers` API。

构建执行通过 `sora-execution::ExecutionContext` 路由。默认 context 通过 Rayon 启用并行工作；库调用方也可以构造串行 context 或固定大小线程池，并传给 `_with_context` pipeline/input/export API。

## 数据格式

主要数据源是生成的 Excel `.xlsx`。每张表会在 schema 中声明自己的工作簿和 sheet：

```toml
[tables.source]
format = "xlsx"
file = "Item.xlsx"
sheet = "Item"
```

CLI 也可以通过 `--default-source-format` 或每张表的 `source.format` 读取 TOML、JSON、YAML 和 CSV 行数据。JSON/YAML 表文件是 row object 数组；当 `source.file` 指向目录时，每个匹配的 JSON/YAML 文件会作为一条 row object 读入。CSV 文件使用和 schema 字段名匹配的 header row。校验会检查必填字段、未知字段、primitive 兼容性、枚举值、范围、结构体字段、引用、map key 和 singleton 行数。

当 JSON 对表格编辑来说过于冗长时，内联 object 字段可以使用 tuple 解析。先定义一个 struct，然后在 `struct<T>` 字段上设置 `parser = { kind = "tuple" }`：

```toml
[[structs]]
name = "ResourceCost"

[[structs.fields]]
name = "kind"
type = "enum<ResourceType>"

[[structs.fields]]
name = "id"
type = "i32"

[[structs.fields]]
name = "count"
type = "i32"

[[tables.fields]]
name = "cost"
type = "struct<ResourceCost>"
parser = { kind = "tuple" }
```

表格单元格按 struct 字段顺序填写：

```text
Item,2003,4
```

小型内联 struct 的 list 可以使用 `tuple_list`。默认 item separator 是 `|`，每个 item 内部仍使用逗号分隔 struct 字段：

```toml
[[tables.fields]]
name = "materials"
type = "list<struct<ResourceCost>>"
parser = { kind = "tuple_list" }
```

```text
Item,2003,4|Gold,0,1000
```

如果更偏好 `;`，可以使用 `item_separator`：

```toml
parser = { kind = "tuple_list", item_separator = ";" }
```

```text
Item,2003,4;Gold,0,1000
```

生成的 Excel 模板会在 `#type` 行展开 tuple struct 字段，例如 `struct<ResourceCost>(kind: enum<ResourceType>, id: i32, count: i32)` 或 `list<struct<ResourceCost>>(kind: enum<ResourceType>, id: i32, count: i32)`，并在单元格 note 中包含同样的结构。

## Exporter 架构

Exporter 实现统一的 `DataExporter` trait，并通过 `ExporterRegistry` 按 format 名称选择。内置格式包括：

- `binary`: 写出面向生产的 `.sora` bundle 文件。
- `json-debug`: 写出确定性的逐表 JSON 文件，便于检查。
- `json`: 写出运行时 JSON bundle。
- `cbor`: 写出运行时 CBOR bundle。
- `sora-protobuf`: 写出 Sora value-model Protobuf bundle。
- `proto`: 写出 typed Protobuf bundle。

Binary bundle 使用语言无关的 sectioned 布局：固定 header、section directory、schema section 和每张表一个 raw table section。Section 层 compression 为 `none`。

## Codegen 架构

Codegen 使用嵌入 CLI 二进制的 MiniJinja 模板，但类型映射会先在 Rust 中计算再渲染。Rust、Kotlin、C#、Java、Scala、Go、C、C++、TypeScript、JavaScript、Erlang、Lua 和 Python 生成包含 model 以及用于读取 `.sora` bundle 的小型 binary runtime reader。Scala 生成支持 `scala_version = "2.12" | "2.13" | "3"`，默认 Scala 3；Scala 3 生成原生 `enum`，Scala 2 生成 `sealed trait` 加 `case object` enum。C 生成 `.h/.c` 文件，带显式 decode/free 生命周期，并支持 `c_standard = "c99" | "c11" | "c17" | "c23"`，默认 `c11`。C++ 生成 header-only C++，并支持 `cpp_standard = "c++11" | "c++14" | "c++17" | "c++20" | "c++23"`，默认 `c++17`。TypeScript 和 JavaScript 面向现代 ESM，并把 `i64` 映射为 `bigint`；JavaScript 默认也会生成 `.d.ts` 文件。Erlang 生成普通 `.erl` module，把行映射为 map，字符串使用 UTF-8 binary，并支持 `enum_repr = "atom" | "integer"`。Lua 生成 EmmyLua annotation 供编辑器类型提示使用，并支持 `lua_version = "5.1"`、`"5.2"`、`"5.3"`、`"5.4"` 或 `"luajit"`；Lua 5.3/5.4 使用 `string.unpack`，更旧的 runtime 使用生成的兼容 decoder。Lua、TypeScript 和 JavaScript 支持 `enum_repr = "string" | "integer"`。

## Excel 模板投影

Sora 从 schema IR 生成 `.xlsx` 模板。Header row 包含表名、mode、key、schema hash、字段名、字段类型、规则和描述。这些 header 是供人编辑的投影，不是权威 schema。
