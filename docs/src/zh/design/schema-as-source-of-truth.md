# Schema 是事实来源

Sora 是 schema-first 的。TOML schema 是配置数据的契约；源文件和生成产物都是这个契约的投影。

```text
schema modules
  -> normalized IR
  -> Excel headers
  -> validation
  -> runtime exports
  -> generated language code
```

这个设计避免常见问题：电子表格、手写 parser 和运行时代码各自定义了一套略有差异的数据形状。

## 结果

- 字段名、类型、key、default、reference 和 validation rule 都在 schema 中定义。
- Excel 和 CSV 文件提供值，而不是第二套 schema。
- runtime export format 不改变数据模型。
- 语言选项属于 codegen target，不属于 IR。
- 下游用户可以添加 generator 或 exporter，而不改变 schema 语义。

schema 仍然可以包含编辑提示，例如 `comment`、parser hint、range 和 length limit。这些提示属于数据契约的一部分，因为它们会影响校验或生成投影。
