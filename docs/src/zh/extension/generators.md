# 生成器

generator 把归一化 IR 转成某个语言目标的文件。

## Registration

生成器注册时包含：

- canonical target id；
- alias；
- display metadata；
- 支持的 runtime format；
- 可选 formatter 集成；
- `CodeGenerator` 实现。

这样内置生成器和下游生成器都能使用同一条管线。

## Implementation Shape

```rust
pub trait CodeGenerator: Send + Sync {
    fn generate(&self, context: CodegenContext<'_>, out_dir: &Path) -> Result<()>;
}
```

generator 接收：

- 归一化 IR；
- 解析后的 target options；
- 输出目录；
- runtime format 选择。

它不应该修改 IR，也不应该依赖 IR 中存在语言相关字段。

## Target Options

语言相关选项放在 `[codegen.<target>]` 下：

```toml
[codegen.rust]
runtime_format = "sora"
map_type = "btree"
string_storage = "owned"
```

这些选项的解释权属于对应 generator。
