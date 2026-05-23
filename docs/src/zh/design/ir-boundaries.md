# IR 边界

归一化 IR 描述 schema 语义，不应该编码语言相关的 codegen 选择。

## 属于 IR 的内容

- package 和 include 的 schema module；
- enum、struct、union、table、field 和 index；
- table mode 和 key；
- source metadata；
- field type、default、parser、range、length 和 comment；
- reference 和派生子表字段 metadata；
- scope。

## 不属于 IR 的内容

- Rust map 实现选择；
- TypeScript enum 表示方式；
- Lua module 名称；
- runtime decoder 依赖选择；
- formatter 设置；
- target-specific 文件布局。

这些设置应该放在 `[codegen.<target>]` 或 generator registration metadata 中。

## 扩展边界

```text
schema input -> normalized IR -> validation
                              |-> exporter registry
                              |-> codegen registry
```

新的语言生成器应该消费 IR 和自己的 target options。新的运行时数据格式应该作为 exporter 添加。除非实际数据语义发生变化，否则二者都不应该要求修改 schema model。
