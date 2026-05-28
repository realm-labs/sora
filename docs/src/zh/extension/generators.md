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
- 已注册的类型映射 providers；
- 输出目录；
- runtime format 选择。

它不应该修改 IR，也不应该依赖 IR 中存在语言相关字段。

## 类型映射

语言生成器可以先查询 `context.type_mappings`，再回退到内置类型映射。provider 按 target 加命名 schema 类型匹配，例如 `struct<Vec3>`，并返回生成类型名和可选 decode 包裹表达式。容器类型应该递归走同一个 mapper，因此 `list<struct<Vec3>>` 会自动变成目标语言中映射后的列表类型。

schema 保持语言无关。项目自己的映射规则应该放在库注册代码或 CLI Lua 类型映射脚本里，不放在字段定义里。

## Target Options

语言相关选项放在 `[codegen.<target>]` 下：

```toml
[codegen.rust]
runtime_format = "sora"
map_type = "btree"
string_storage = "owned"
```

这些选项的解释权属于对应 generator。
