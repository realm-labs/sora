#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodegenTarget {
    Rust,
    Kotlin,
    CSharp,
    Java,
    Go,
    Dart,
    C,
    Cpp,
    TypeScript,
    JavaScript,
    Erlang,
    Lua,
    Proto,
    Python,
}
