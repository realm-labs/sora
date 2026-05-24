# CLI 参考

已安装二进制的精确帮助文本以 `sora --help` 为准；单个命令的参数可以用 `sora <command> --help` 查看。本页集中整理常用工作流命令、alias 和短参数。

## 全局参数

全局参数可以放在子命令前，也可以放在子命令后。

| 参数 | 说明 |
| --- | --- |
| `-j, --jobs <N>` | 最大工作线程数。必须大于 0。 |
| `--serial` | 禁用并行执行。 |
| `--parser-script <PATH>` | 加载自定义 Lua 单元格 parser 脚本。可以重复传。项目级 parser 脚本也可以配置在 `project.toml` 的 `[parsers].scripts` 中。 |
| `-h, --help` | 打印帮助。 |
| `-V, --version` | 打印 CLI 版本。 |

## 命令 Alias

| 命令 | Alias |
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

## 常用短参数

| 短参数 | 长参数 | 使用命令 |
| --- | --- | --- |
| `-p` | `--project` | 读取 project 的命令。 |
| `-o` | `--out` | `init`、`gen`、`export`、`diff`、`excel-template`、`schema-lock`。 |
| `-s` | `--scope` | `build`、`gen`、`export`、`diff`、`excel-template`、`excel-sync`、`schema-lock`。 |
| `-t` | `--target` | `build`、`gen`。 |
| `-f` | `--format` | `export`。 |
| `-d` | `--data-root` | `build`、`export`、`excel-sync`。 |
| `-l` | `--lock`、`--left-root` | `check`、`diff`。 |
| `-r` | `--right-root` | `diff`。 |
| `-c` | `--clean` | `build`。 |
| `-w` | `--write` | `excel-sync`。 |

## 命令

### `init`

创建新的项目脚手架。

```bash
sora init --out my-config --schema-format toml
sora i -o my-config --schema-format yaml
```

| 参数 | 说明 |
| --- | --- |
| `-o, --out <DIR>` | 脚手架输出目录。 |
| `--schema-format <toml|yaml|json|lua>` | Schema 文件格式。默认 `toml`。 |
| `--force` | 允许写入已有脚手架路径。 |

### `check`

校验项目 schema，也可以和已有 schema lock 对比。

```bash
sora check --project project.toml
sora c -p project.toml -l generated/schema.lock
```

| 参数 | 说明 |
| --- | --- |
| `-p, --project <PATH>` | 项目清单路径。 |
| `-l, --lock <PATH>` | 用于校验的已有 schema lock。 |

### `build`

运行 `project.toml` 中 `[build]` 声明的输出，例如 schema lock、Excel 模板、codegen 和 export。

```bash
sora build --project project.toml
sora b -p project.toml -t rust -c
```

| 参数 | 说明 |
| --- | --- |
| `-p, --project <PATH>` | 项目清单路径。 |
| `--default-source-format <csv|toml|xlsx>` | 表 source 未声明 `format` 时使用的 source 格式。 |
| `-d, --data-root <DIR>` | 数据输入根目录。覆盖 `[build].data_root`。 |
| `-s, --scope <NAME>` | 只构建包含在某个 scope 中的 schema item。 |
| `-t, --target <NAME>` | 要运行的 codegen target。可以重复传。 |
| `-c, --clean` | 重建前删除选中的生成输出。 |

### `gen`

直接为某个 target 生成代码，不依赖 `[build.codegen]`。

```bash
sora gen --target rust --project project.toml --out generated/rust
sora g -t typescript -p project.toml -o generated/typescript
```

| 参数 | 说明 |
| --- | --- |
| `-t, --target <NAME>` | Codegen target，例如 `rust`、`typescript` 或 `python`。 |
| `-p, --project <PATH>` | 项目清单路径。 |
| `-o, --out <DIR>` | 输出目录。 |
| `--format-code <never|auto|required>` | 代码生成后运行 formatter。默认 `never`。 |
| `-s, --scope <NAME>` | 只生成包含在某个 scope 中的 schema item。 |

### `export`

读取表数据并导出运行时数据。

```bash
sora export --project project.toml --data-root data --format json --out generated/config.json
sora e -p project.toml -d data -f binary -o generated/config.sora
```

| 参数 | 说明 |
| --- | --- |
| `-f, --format <NAME>` | 导出格式，例如 `binary`、`json`、`debug-json`、`cbor`、`sora-protobuf` 或 `typed-protobuf`。 |
| `--default-source-format <csv|toml|xlsx>` | 表 source 未声明 `format` 时使用的 source 格式。 |
| `-p, --project <PATH>` | 项目清单路径。 |
| `-d, --data-root <DIR>` | 数据输入根目录。 |
| `-o, --out <PATH>` | 输出文件或目录，取决于导出格式。 |
| `-s, --scope <NAME>` | 只导出包含在某个 scope 中的 schema item。 |
| `--compression <none|zstd>` | 导出压缩。`zstd` 只支持 binary export。 |
| `--compression-level <N>` | 压缩导出的压缩等级。 |

### `diff`

使用同一份项目 schema 比较两个数据根目录。

```bash
sora diff --project project.toml --left-root old-data --right-root data --out generated/diff.json
sora d -p project.toml -l old-data -r data -o generated/diff.json
```

| 参数 | 说明 |
| --- | --- |
| `--default-source-format <csv|toml|xlsx>` | 表 source 未声明 `format` 时使用的 source 格式。 |
| `-p, --project <PATH>` | 项目清单路径。 |
| `-l, --left-root <DIR>` | 基准数据根目录。 |
| `-r, --right-root <DIR>` | 变更后的数据根目录。 |
| `-o, --out <PATH>` | Diff 输出路径。 |
| `-s, --scope <NAME>` | 只比较包含在某个 scope 中的 schema item。 |

### `excel-template`

根据 schema 生成空 Excel workbook。它适合新建 workbook，不适合覆盖已有数据文件。

```bash
sora excel-template --project project.toml --out generated/excel
sora et -p project.toml -o generated/excel
```

| 参数 | 说明 |
| --- | --- |
| `-p, --project <PATH>` | 项目清单路径。 |
| `-o, --out <DIR>` | 生成 workbook 的输出目录。 |
| `-s, --scope <NAME>` | 只为包含在某个 scope 中的 schema item 生成模板。 |

### `excel-sync`

预览或应用已有 Excel 数据 workbook 的 schema 表头更新，同时保留数据行。从 schema 中删除的字段会保留为被忽略的 legacy 列。

```bash
sora excel-sync --project project.toml --data-root data
sora es -p project.toml -d data -w
```

| 参数 | 说明 |
| --- | --- |
| `-p, --project <PATH>` | 项目清单路径。 |
| `-d, --data-root <DIR>` | 数据 workbook 根目录。 |
| `-s, --scope <NAME>` | 只同步包含在某个 scope 中的 schema item。 |
| `-w, --write` | 写入 workbook 变更。不带这个参数时只预览变化。 |

### `schema-lock`

为当前归一化 schema 写出 schema lock。

```bash
sora schema-lock --project project.toml --out generated/schema.lock
sora sl -p project.toml -o generated/schema.lock
```

| 参数 | 说明 |
| --- | --- |
| `-p, --project <PATH>` | 项目清单路径。 |
| `-o, --out <PATH>` | Schema lock 输出路径。 |
| `-s, --scope <NAME>` | 只锁定包含在某个 scope 中的 schema item。 |

### `studio`

启动内置的 Sora Studio schema 编辑器。

```bash
sora studio --project project.toml
sora st -p project.toml --port 5180
```

| 参数 | 说明 |
| --- | --- |
| `-p, --project <PATH>` | 项目清单路径。 |
| `--host <IP>` | 绑定地址。默认 `127.0.0.1`。 |
| `--port <PORT>` | 端口。默认 `5174`。 |
