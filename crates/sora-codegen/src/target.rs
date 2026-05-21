#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodegenTarget {
    Rust,
    Kotlin,
    CSharp,
    Java,
    Scala,
    Go,
    Dart,
    Godot,
    C,
    Cpp,
    TypeScript,
    JavaScript,
    Erlang,
    Lua,
    ProtoSchema,
    Python,
}
