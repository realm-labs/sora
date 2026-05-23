# 扩展 Sora

Sora 设计上可以作为库使用，便于项目方增加自己的语言或数据格式支持。

扩展边界被刻意拆开：

```text
input adapter -> schema model -> normalized IR -> data validation
                                      |-> exporter
                                      |-> code generator
```

## 添加 Code Generator

实现 generator trait：

```rust
pub trait CodeGenerator: Send + Sync {
    fn generate(&self, context: CodegenContext<'_>, out_dir: &Path) -> Result<()>;
}
```

注册 target id、alias、runtime capability 和可选 formatter 配置。

更完整的说明见[生成器](extension/generators.md)。

## 保持 IR 中立

语言相关配置应该放在 target options 和 generator 代码里。归一化 IR 只描述 schema 语义：package、table、field、type、key、index、union 和 validation metadata。

## 添加 Exporter

Exporter 和 generator 是分离的。如果需要新的运行时数据包格式，就添加 data exporter。如果需要新的语言目标，就添加 code generator。

导出器边界见[导出器](extension/exporters.md)。
