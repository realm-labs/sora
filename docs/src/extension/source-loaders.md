# Lua Source Loaders

Lua source loaders let a project translate a custom, possibly multi-file format into ordinary Sora rows while keeping the standard build, export, query, validation, Studio, and MCP workflows. They are format-neutral project extensions, not dynamic native plugins.

## Configuration

Declare one or more scripts in the project manifest. Script paths must be project-relative and cannot contain `..`:

```toml
[source_loaders]
scripts = ["tools/source_loaders.lua"]
```

Then select the registered format on a table. A directory has no useful extension, so it normally uses an explicit format:

```toml
[tables.source]
format = "custom_format"
file = "catalog/items"
```

A loader may declare extensions. Sora can infer the format when `source.format` is omitted and the source file has one of those extensions.

## Script API

A script returns a `source_loaders` table. Sora calls `load` once per table source, not once per cell:

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

`source` contains the relative configured `path`, the table `name`, and the selected `format`. The loader must return `{ rows = { ... } }`, where every row is a string-keyed object.

Returned boolean, integer, finite float, UTF-8 string, array, and string-keyed object values recursively become `sora_data::Value`. Use `ctx.null` for an explicit null value. `ctx.json_decode` also represents every JSON `null` as this sentinel, so null object fields and null array elements are preserved. Lua `nil` still means an absent table entry and therefore cannot represent a stored null. Non-empty sequential Lua tables are arrays; empty `{}` is an object. Use `ctx.array({})` when an empty array is required. Objects are materialized in key order. After loading, Sora still applies its normal schema, required, enum, ref, range, table-mode, view, localization-catalog, and export validation.

The host context provides:

| API | Result |
| --- | --- |
| `ctx.read_text(relative_path)` | Read one UTF-8 file. |
| `ctx.read_bytes(relative_path)` | Read one file as a Lua byte string. |
| `ctx.list(relative_directory)` | Return direct entries sorted by normalized relative path. Each entry has `path`, `name`, and `kind` (`file` or `directory`). |
| `ctx.json_decode(text)` | Decode JSON into Lua values, preserving empty arrays. |
| `ctx.null` | Stable sentinel for a stored null value; do not mutate it. |
| `ctx.array(table)` | Mark a Lua table, including an empty one, as an array. |
| `ctx.error(diagnostic)` | Stop with a structured diagnostic. |

A diagnostic has `message`, optional relative `path`, optional `line`, `column`, and `field`:

```lua
ctx.error({
  path = "parts/header.custom",
  line = 4,
  column = 9,
  field = "id",
  message = "invalid record id",
})
```

Use dot calls (`ctx.read_text("part.json")`), not colon calls.

## Security, trust, and determinism

Lua does not receive `io`, `os`, `package`, `debug`, `require`, `dofile`, `loadfile`, shell, network, system-time, printing, or random-number capabilities. It cannot load native libraries or start processes. The standard table, string, deterministic math, and UTF-8 libraries remain available.

All filesystem operations go through the Rust host. Paths are relative to the current table source root; absolute paths and `..` are rejected. Canonicalization rejects symlinks that escape that root. A file source permits only `.`; a directory source can enumerate and read descendants. Enumeration order is stable.

The runtime enforces per-load limits: 100,000 rows, value depth 64, 4,096 file reads, 64 MiB read bytes, 100,000 listed entries, 128 MiB Lua memory, and 10,000,000 Lua instructions. `ExecutionContext` cancellation is checked before and during Lua execution. These limits are currently fixed rather than configurable.

Sora records the script path and SHA-256 plus every file actually read by the host. Source-loader scripts and data-root contents participate in the project/data revision, so script or input changes invalidate revision-bound work. MCP project discovery treats source-loader scripts like parser and type-mapping scripts: trust is required for the exact inspected path and digest before the project is opened.

Built-in formats cannot be overridden. Repeating a format or extension across loaders fails project loading, including extensions owned by built-in formats. This keeps extension inference unambiguous; omit `extensions` and use explicit `source.format` when a project format must read a conventionally named file.

## Limitations

Lua source loaders are read-only in the first version. Data Mutation and Studio write-back reject a custom source format as not mutable; Sora never guesses a built-in writer. Localization sources are not supported by this table-loader API. There is no runtime installation, dynamic library ABI, external process execution, or network plugin mechanism.
