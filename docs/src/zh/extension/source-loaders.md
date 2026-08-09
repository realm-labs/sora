# Lua Source Loader

Lua Source Loader 让项目把自定义的、可能由多个文件组成的格式转换成普通 Sora 行，同时继续使用标准 build、export、query、validation、Studio 和 MCP 工作流。它是格式无关的项目扩展，不是动态原生插件。

## 配置

在项目 manifest 中声明一个或多个脚本。脚本路径必须相对项目，不能包含 `..`：

```toml
[source_loaders]
scripts = ["tools/source_loaders.lua"]
```

然后在表上选择已注册的格式。目录没有可用于推断的扩展名，因此通常显式写格式：

```toml
[tables.source]
format = "custom_format"
file = "catalog/items"
```

Loader 可以声明扩展名。当 `source.format` 省略且 source 文件带有相应扩展名时，Sora 可以推断格式。

## 脚本 API

脚本返回 `source_loaders` 表。Sora 对每张表的数据源调用一次 `load`，不是逐 Cell 调用：

```lua
return {
  source_loaders = {
    custom_format = {
      extensions = { "custom" },
      load = function(source, ctx)
        local rows = {}
        for _, entry in ipairs(ctx.list(".")) do
          if entry.kind == "file" then
            local document = ctx.json_decode(ctx.read_text(entry.path))
            table.insert(rows, {
              id = document.id,
              name = document.name,
              tags = document.tags,
            })
          end
        end
        return { rows = rows }
      end,
    },
  },
}
```

`source` 包含配置中的相对 `path`、表 `name` 和选中的 `format`。Loader 必须返回 `{ rows = { ... } }`，每行必须是 string-keyed object。

返回的 bool、integer、有限 float、UTF-8 string、array 和 string-keyed object 会递归转换成 `sora_data::Value`。需要显式空值时使用 `ctx.null`。`ctx.json_decode` 也会把每个 JSON `null` 表示为这个哨兵，因此 object 中的 null 字段和 array 中的 null 元素都不会丢失。Lua `nil` 仍表示 table 中不存在该项，不能用来保存空值。非空连续 Lua table 是 array；空 `{}` 是 object。需要空 array 时使用 `ctx.array({})`。Object 按 key 排序存储。加载后，Sora 仍统一执行 schema、required、enum、ref、range、表模式、View、localization catalog 和 export 校验。

Host context 提供：

| API | 结果 |
| --- | --- |
| `ctx.read_text(relative_path)` | 读取一个 UTF-8 文件。 |
| `ctx.read_bytes(relative_path)` | 把文件读取为 Lua byte string。 |
| `ctx.list(relative_directory)` | 按规范化相对路径稳定排序，返回直接子项；每项包含 `path`、`name` 和 `kind`（`file` 或 `directory`）。 |
| `ctx.json_decode(text)` | 把 JSON 解码为 Lua value，并保留空 array。 |
| `ctx.null` | 表示已存储空值的稳定哨兵；不要修改它。 |
| `ctx.array(table)` | 把 Lua table（包括空 table）标记为 array。 |
| `ctx.error(diagnostic)` | 以结构化诊断停止。 |

诊断包含 `message`，以及可选的相对 `path`、`line`、`column`、`field`：

```lua
ctx.error({
  path = "parts/header.custom",
  line = 4,
  column = 9,
  field = "id",
  message = "invalid record id",
})
```

请使用点调用（`ctx.read_text("part.json")`），不要使用冒号调用。

## 安全、Trust 与确定性

Lua 不会获得 `io`、`os`、`package`、`debug`、`require`、`dofile`、`loadfile`、Shell、网络、系统时间、打印或随机数能力，也不能加载原生库或启动进程。只保留 table、string、确定性 math 和 UTF-8 标准库。

所有文件操作都由 Rust Host 提供。路径相对当前表的 source root；绝对路径和 `..` 会被拒绝。canonicalization 会拒绝逃逸 source root 的 symlink。文件型 source 只允许 `.`；目录型 source 可以枚举和读取后代。枚举顺序稳定。

每次 load 固定限制为：100,000 行、value 深度 64、4,096 次文件读取、64 MiB 读取字节、100,000 个枚举项、128 MiB Lua 内存、10,000,000 条 Lua 指令。执行前和执行中都会检查 `ExecutionContext` cancellation。这些限制目前不可配置。

Sora 自动记录脚本路径与 SHA-256，以及 Host 实际读取的每个文件。Source Loader 脚本和 data root 内容会进入 project/data revision，因此脚本或输入变化会使 revision-bound 操作失效。MCP 项目发现把 Source Loader 脚本与 parser、type mapping 脚本同等对待：打开项目之前，必须信任精确的已检查路径和摘要。

Loader 不能覆盖内置格式。不同 Loader 重复注册格式或扩展名都会导致项目加载失败，内置格式已占用的扩展名也不能重复注册。这样可以保证扩展名推断没有歧义；如果项目格式必须读取使用常规扩展名的文件，不要声明该扩展名，并显式指定 `source.format`。

## 限制

第一版 Lua Source Loader 只读。Data Mutation 和 Studio 写回遇到自定义格式时会明确拒绝为不可变；Sora 不会猜测某个内置 writer。该表 Loader API 不支持 localization source。它不提供运行时安装、动态库 ABI、外部进程执行或网络插件机制。
