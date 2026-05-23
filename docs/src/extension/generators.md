# Generators

A generator turns the normalized IR into files for one language target.

## Registration

Generators are registered with:

- a canonical target id;
- aliases;
- display metadata;
- supported runtime formats;
- optional formatter integration;
- a `CodeGenerator` implementation.

This lets built-in generators and downstream generators use the same pipeline.

## Implementation Shape

```rust
pub trait CodeGenerator: Send + Sync {
    fn generate(&self, context: CodegenContext<'_>, out_dir: &Path) -> Result<()>;
}
```

The generator receives:

- the normalized IR;
- parsed target options;
- the output directory;
- runtime format selection.

It should not mutate the IR or rely on language-specific fields being present in the IR.

## Target Options

Language-specific options live under `[codegen.<target>]`:

```toml
[codegen.rust]
runtime_format = "sora"
map_type = "btree"
string_storage = "owned"
```

The generator owns the interpretation of these options.
