# CLI Reference

Use `sora --help` for the installed binary's exact help text, and `sora <command> --help` for command-specific options. This page summarizes the common workflow commands, aliases, and short flags.

## Global Options

Global options can be placed before or after the subcommand.

| Option | Description |
| --- | --- |
| `-j, --jobs <N>` | Maximum worker threads. Must be greater than zero. |
| `--serial` | Disable parallel execution. |
| `--parser-script <PATH>` | Load a custom Lua cell parser script. Can be repeated. Project-level parser scripts can also be configured in `[parsers].scripts` in `project.toml`. |
| `-h, --help` | Print help. |
| `-V, --version` | Print the CLI version. |

## Command Aliases

| Command | Aliases |
| --- | --- |
| `build` | `b` |
| `check` | `c` |
| `init` | `i` |
| `gen` | `g` |
| `export` | `e` |
| `diff` | `d` |
| `excel-template` | `template`, `et` |
| `excel-sync` | `sync`, `es` |
| `schema-lock` | `lock`, `sl` |
| `studio` | `st` |

## Common Short Flags

| Short | Long | Used by |
| --- | --- | --- |
| `-p` | `--project` | Project-reading commands. |
| `-o` | `--out` | `init`, `gen`, `export`, `diff`, `excel-template`, `schema-lock`. |
| `-s` | `--scope` | `build`, `gen`, `export`, `diff`, `excel-template`, `excel-sync`, `schema-lock`. |
| `-t` | `--target` | `build`, `gen`. |
| `-f` | `--format` | `export`. |
| `-d` | `--data-root` | `build`, `export`, `excel-sync`. |
| `-l` | `--lock`, `--left-root` | `check`, `diff`. |
| `-r` | `--right-root` | `diff`. |
| `-c` | `--clean` | `build`. |
| `-w` | `--write` | `excel-sync`. |

## Commands

### `init`

Create a new project scaffold.

```bash
sora init --out my-config --schema-format toml
sora i -o my-config --schema-format yaml
```

| Option | Description |
| --- | --- |
| `-o, --out <DIR>` | Output directory for the scaffold. |
| `--schema-format <toml|yaml|json|lua>` | Schema file format. Defaults to `toml`. |
| `--force` | Allow writing into an existing scaffold path. |

### `check`

Validate a project schema, optionally against a schema lock.

```bash
sora check --project project.toml
sora c -p project.toml -l generated/schema.lock
```

| Option | Description |
| --- | --- |
| `-p, --project <PATH>` | Project manifest path. |
| `-l, --lock <PATH>` | Existing schema lock to verify against. |

### `build`

Run outputs declared in `[build]` in `project.toml`, such as schema locks, Excel templates, codegen, and exports.

```bash
sora build --project project.toml
sora b -p project.toml -t rust -c
```

| Option | Description |
| --- | --- |
| `-p, --project <PATH>` | Project manifest path. |
| `--default-source-format <csv|json|toml|xlsx|yaml>` | Source format used when a table source omits `format`. |
| `-d, --data-root <DIR>` | Data input root. Overrides `[build].data_root`. |
| `-s, --scope <NAME>` | Build only schema items included in a scope. |
| `-t, --target <NAME>` | Codegen target to run. Can be repeated. |
| `-c, --clean` | Delete selected generated outputs before rebuilding. |

### `gen`

Generate code for one target directly, without using `[build.codegen]`.

```bash
sora gen --target rust --project project.toml --out generated/rust
sora g -t typescript -p project.toml -o generated/typescript
```

| Option | Description |
| --- | --- |
| `-t, --target <NAME>` | Codegen target, such as `rust`, `typescript`, or `python`. |
| `-p, --project <PATH>` | Project manifest path. |
| `-o, --out <DIR>` | Output directory. |
| `--format-code <never|auto|required>` | Run formatter after codegen. Defaults to `never`. |
| `-s, --scope <NAME>` | Generate only schema items included in a scope. |

### `export`

Load table data and export runtime data.

```bash
sora export --project project.toml --data-root data --format json --out generated/config.json
sora e -p project.toml -d data -f binary -o generated/config.sora
```

| Option | Description |
| --- | --- |
| `-f, --format <NAME>` | Export format, such as `binary`, `json`, `debug-json`, `cbor`, `sora-protobuf`, or `typed-protobuf`. |
| `--default-source-format <csv|json|toml|xlsx|yaml>` | Source format used when a table source omits `format`. |
| `-p, --project <PATH>` | Project manifest path. |
| `-d, --data-root <DIR>` | Data input root. |
| `-o, --out <PATH>` | Output file or directory, depending on export format. |
| `-s, --scope <NAME>` | Export only schema items included in a scope. |
| `--compression <none|zstd>` | Export compression. `zstd` is only supported by binary exports. |
| `--compression-level <N>` | Compression level for compressed exports. |

### `diff`

Compare two data roots using the same project schema.

```bash
sora diff --project project.toml --left-root old-data --right-root data --out generated/diff.json
sora d -p project.toml -l old-data -r data -o generated/diff.json
```

| Option | Description |
| --- | --- |
| `--default-source-format <csv|json|toml|xlsx|yaml>` | Source format used when a table source omits `format`. |
| `-p, --project <PATH>` | Project manifest path. |
| `-l, --left-root <DIR>` | Baseline data root. |
| `-r, --right-root <DIR>` | Changed data root. |
| `-o, --out <PATH>` | Diff output path. |
| `-s, --scope <NAME>` | Diff only schema items included in a scope. |

### `excel-template`

Generate empty Excel workbooks from the schema. Use this for new workbooks, not for existing data files.

```bash
sora excel-template --project project.toml --out generated/excel
sora et -p project.toml -o generated/excel
```

| Option | Description |
| --- | --- |
| `-p, --project <PATH>` | Project manifest path. |
| `-o, --out <DIR>` | Output directory for generated workbooks. |
| `-s, --scope <NAME>` | Generate templates only for schema items included in a scope. |

### `excel-sync`

Preview or apply schema header updates to existing Excel data workbooks while preserving data rows. Removed schema fields stay as ignored legacy columns.

```bash
sora excel-sync --project project.toml --data-root data
sora es -p project.toml -d data -w
```

| Option | Description |
| --- | --- |
| `-p, --project <PATH>` | Project manifest path. |
| `-d, --data-root <DIR>` | Data workbook root. |
| `-s, --scope <NAME>` | Sync only schema items included in a scope. |
| `-w, --write` | Write workbook changes. Without this flag, the command previews changes only. |

### `schema-lock`

Write a schema lock for the current normalized schema.

```bash
sora schema-lock --project project.toml --out generated/schema.lock
sora sl -p project.toml -o generated/schema.lock
```

| Option | Description |
| --- | --- |
| `-p, --project <PATH>` | Project manifest path. |
| `-o, --out <PATH>` | Schema lock output path. |
| `-s, --scope <NAME>` | Lock only schema items included in a scope. |

### `studio`

Start the embedded Sora Studio schema editor.

```bash
sora studio --project project.toml
sora st -p project.toml --port 5180
```

| Option | Description |
| --- | --- |
| `-p, --project <PATH>` | Project manifest path. |
| `--host <IP>` | Bind address. Defaults to `127.0.0.1`. |
| `--port <PORT>` | Port. Defaults to `5174`. |
