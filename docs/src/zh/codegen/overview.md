# 代码生成

代码生成会把归一化 schema IR 转成目标语言的 row type、table container 和 config loader。

它由语言生成器 registry 驱动。

每个生成器声明：

- target id 和 alias；
- 展示元数据；
- 支持的 runtime format；
- 可选 formatter 集成；
- `CodeGenerator` 实现。

因此内置语言和下游生成器可以使用同一种管线形状。

```text
schema files -> schema model -> normalized IR -> generator registry -> target generator -> files
```

直接生成一个目标：

```bash
sora gen --target typescript --project project.toml --out generated/typescript
```

也可以在构建清单中声明：

```toml
[[build.codegen]]
target = "typescript"
out = "typescript/generated"
format = "auto"
```

`format` 可以是 `never`、`auto` 或 `required`。`auto` 会在 formatter 可用时运行；`required` 会在 formatter 缺失或失败时报错。

## Runtime Format

每个目标可以选择 runtime format：

```toml
[codegen.typescript]
runtime_format = "json"
```

runtime format 只控制该目标生成的 loader 代码，不改变 schema 或源数据。

## 生成代码形状

生成代码通常包含：

- schema enum 对应的枚举；
- struct、union variant 和 table row 对应的 record type；
- `map`、`list`、`singleton` table container；
- key 和 index lookup helper；
- 选定 runtime format 的顶层 config loader。
