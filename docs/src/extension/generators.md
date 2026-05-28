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
- the registered type mapping providers;
- the output directory;
- runtime format selection.

It should not mutate the IR or rely on language-specific fields being present in the IR.

## Type Mappings

Language generators can consult `context.type_mappings` before falling back to their built-in type mapping. A provider maps a target plus a named schema type, such as `struct<Vec3>`, to a generated type name and optional decode wrappers. Container types should recurse through the same mapper so `list<struct<Vec3>>` automatically becomes a list of the mapped target type.

The schema remains language-neutral. Project-specific mappings belong in library registration code or CLI Lua type mapping scripts, not in field definitions.

## Target Options

Language-specific options live under `[codegen.<target>]`:

```toml
[codegen.rust]
runtime_format = "sora"
map_type = "btree"
string_storage = "owned"
```

The generator owns the interpretation of these options.
