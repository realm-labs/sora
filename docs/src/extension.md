# Extending Sora

Sora is designed to be used as a library by projects that need their own language or data format support.

## Add a Code Generator

Implement the generator trait:

```rust
pub trait CodeGenerator: Send + Sync {
    fn generate(&self, context: CodegenContext<'_>, out_dir: &Path) -> Result<()>;
}
```

Register it with an id, aliases, runtime capabilities, and optional formatter configuration.

## Keep the IR Neutral

Language-specific settings belong in target options and generator code. The normalized IR should describe schema semantics only: packages, tables, fields, types, keys, indexes, unions, and validation metadata.

## Add an Exporter

Exporters are separate from generators. Add a data exporter when you need a new runtime bundle format. Add a code generator when you need a new language target.

## Suggested Extension Boundary

```text
input adapter -> schema model -> normalized IR -> data validation
                                      |-> exporter
                                      |-> code generator
```

This boundary keeps custom language support from changing schema parsing or validation.
