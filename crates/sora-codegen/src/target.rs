#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodegenTarget {
    Rust,
    Kotlin,
    CSharp,
    Java,
    Go,
    TypeScript,
    JavaScript,
    Erlang,
    Lua,
    Proto,
    Python,
}
