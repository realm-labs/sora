use std::path::{Path, PathBuf};

struct EmbeddedTemplate {
    target: &'static str,
    file_name: &'static str,
    source: &'static str,
}

const EMBEDDED_TEMPLATES: &[EmbeddedTemplate] = &[
    EmbeddedTemplate {
        target: "csharp",
        file_name: "config.cs.j2",
        source: include_str!("../../../templates/csharp/config.cs.j2"),
    },
    EmbeddedTemplate {
        target: "csharp",
        file_name: "enum.cs.j2",
        source: include_str!("../../../templates/csharp/enum.cs.j2"),
    },
    EmbeddedTemplate {
        target: "csharp",
        file_name: "record.cs.j2",
        source: include_str!("../../../templates/csharp/record.cs.j2"),
    },
    EmbeddedTemplate {
        target: "csharp",
        file_name: "runtime.cs.j2",
        source: include_str!("../../../templates/csharp/runtime.cs.j2"),
    },
    EmbeddedTemplate {
        target: "csharp",
        file_name: "union.cs.j2",
        source: include_str!("../../../templates/csharp/union.cs.j2"),
    },
    EmbeddedTemplate {
        target: "dart",
        file_name: "config.dart.j2",
        source: include_str!("../../../templates/dart/config.dart.j2"),
    },
    EmbeddedTemplate {
        target: "dart",
        file_name: "enum.dart.j2",
        source: include_str!("../../../templates/dart/enum.dart.j2"),
    },
    EmbeddedTemplate {
        target: "dart",
        file_name: "library.dart.j2",
        source: include_str!("../../../templates/dart/library.dart.j2"),
    },
    EmbeddedTemplate {
        target: "dart",
        file_name: "record.dart.j2",
        source: include_str!("../../../templates/dart/record.dart.j2"),
    },
    EmbeddedTemplate {
        target: "dart",
        file_name: "runtime.dart.j2",
        source: include_str!("../../../templates/dart/runtime.dart.j2"),
    },
    EmbeddedTemplate {
        target: "dart",
        file_name: "union.dart.j2",
        source: include_str!("../../../templates/dart/union.dart.j2"),
    },
    EmbeddedTemplate {
        target: "c",
        file_name: "config.c.j2",
        source: include_str!("../../../templates/c/config.c.j2"),
    },
    EmbeddedTemplate {
        target: "c",
        file_name: "config.h.j2",
        source: include_str!("../../../templates/c/config.h.j2"),
    },
    EmbeddedTemplate {
        target: "c",
        file_name: "enum.c.j2",
        source: include_str!("../../../templates/c/enum.c.j2"),
    },
    EmbeddedTemplate {
        target: "c",
        file_name: "enum.h.j2",
        source: include_str!("../../../templates/c/enum.h.j2"),
    },
    EmbeddedTemplate {
        target: "c",
        file_name: "record.c.j2",
        source: include_str!("../../../templates/c/record.c.j2"),
    },
    EmbeddedTemplate {
        target: "c",
        file_name: "record.h.j2",
        source: include_str!("../../../templates/c/record.h.j2"),
    },
    EmbeddedTemplate {
        target: "c",
        file_name: "runtime.c.j2",
        source: include_str!("../../../templates/c/runtime.c.j2"),
    },
    EmbeddedTemplate {
        target: "c",
        file_name: "runtime.h.j2",
        source: include_str!("../../../templates/c/runtime.h.j2"),
    },
    EmbeddedTemplate {
        target: "c",
        file_name: "types.c.j2",
        source: include_str!("../../../templates/c/types.c.j2"),
    },
    EmbeddedTemplate {
        target: "c",
        file_name: "types.h.j2",
        source: include_str!("../../../templates/c/types.h.j2"),
    },
    EmbeddedTemplate {
        target: "c",
        file_name: "union.c.j2",
        source: include_str!("../../../templates/c/union.c.j2"),
    },
    EmbeddedTemplate {
        target: "c",
        file_name: "union.h.j2",
        source: include_str!("../../../templates/c/union.h.j2"),
    },
    EmbeddedTemplate {
        target: "cpp",
        file_name: "config.hpp.j2",
        source: include_str!("../../../templates/cpp/config.hpp.j2"),
    },
    EmbeddedTemplate {
        target: "cpp",
        file_name: "enum.hpp.j2",
        source: include_str!("../../../templates/cpp/enum.hpp.j2"),
    },
    EmbeddedTemplate {
        target: "cpp",
        file_name: "record.hpp.j2",
        source: include_str!("../../../templates/cpp/record.hpp.j2"),
    },
    EmbeddedTemplate {
        target: "cpp",
        file_name: "runtime.hpp.j2",
        source: include_str!("../../../templates/cpp/runtime.hpp.j2"),
    },
    EmbeddedTemplate {
        target: "cpp",
        file_name: "union.hpp.j2",
        source: include_str!("../../../templates/cpp/union.hpp.j2"),
    },
    EmbeddedTemplate {
        target: "go",
        file_name: "config.go.j2",
        source: include_str!("../../../templates/go/config.go.j2"),
    },
    EmbeddedTemplate {
        target: "go",
        file_name: "enum.go.j2",
        source: include_str!("../../../templates/go/enum.go.j2"),
    },
    EmbeddedTemplate {
        target: "go",
        file_name: "record.go.j2",
        source: include_str!("../../../templates/go/record.go.j2"),
    },
    EmbeddedTemplate {
        target: "go",
        file_name: "runtime.go.j2",
        source: include_str!("../../../templates/go/runtime.go.j2"),
    },
    EmbeddedTemplate {
        target: "go",
        file_name: "union.go.j2",
        source: include_str!("../../../templates/go/union.go.j2"),
    },
    EmbeddedTemplate {
        target: "java",
        file_name: "config.java.j2",
        source: include_str!("../../../templates/java/config.java.j2"),
    },
    EmbeddedTemplate {
        target: "java",
        file_name: "enum.java.j2",
        source: include_str!("../../../templates/java/enum.java.j2"),
    },
    EmbeddedTemplate {
        target: "java",
        file_name: "record.java.j2",
        source: include_str!("../../../templates/java/record.java.j2"),
    },
    EmbeddedTemplate {
        target: "java",
        file_name: "runtime.java.j2",
        source: include_str!("../../../templates/java/runtime.java.j2"),
    },
    EmbeddedTemplate {
        target: "java",
        file_name: "union.java.j2",
        source: include_str!("../../../templates/java/union.java.j2"),
    },
    EmbeddedTemplate {
        target: "kotlin",
        file_name: "config.kt.j2",
        source: include_str!("../../../templates/kotlin/config.kt.j2"),
    },
    EmbeddedTemplate {
        target: "kotlin",
        file_name: "data_class.kt.j2",
        source: include_str!("../../../templates/kotlin/data_class.kt.j2"),
    },
    EmbeddedTemplate {
        target: "kotlin",
        file_name: "enum.kt.j2",
        source: include_str!("../../../templates/kotlin/enum.kt.j2"),
    },
    EmbeddedTemplate {
        target: "kotlin",
        file_name: "package.kt.j2",
        source: include_str!("../../../templates/kotlin/package.kt.j2"),
    },
    EmbeddedTemplate {
        target: "kotlin",
        file_name: "runtime.kt.j2",
        source: include_str!("../../../templates/kotlin/runtime.kt.j2"),
    },
    EmbeddedTemplate {
        target: "kotlin",
        file_name: "union.kt.j2",
        source: include_str!("../../../templates/kotlin/union.kt.j2"),
    },
    EmbeddedTemplate {
        target: "typescript",
        file_name: "config.ts.j2",
        source: include_str!("../../../templates/typescript/config.ts.j2"),
    },
    EmbeddedTemplate {
        target: "typescript",
        file_name: "enum.ts.j2",
        source: include_str!("../../../templates/typescript/enum.ts.j2"),
    },
    EmbeddedTemplate {
        target: "typescript",
        file_name: "index.ts.j2",
        source: include_str!("../../../templates/typescript/index.ts.j2"),
    },
    EmbeddedTemplate {
        target: "typescript",
        file_name: "record.ts.j2",
        source: include_str!("../../../templates/typescript/record.ts.j2"),
    },
    EmbeddedTemplate {
        target: "typescript",
        file_name: "runtime.ts.j2",
        source: include_str!("../../../templates/typescript/runtime.ts.j2"),
    },
    EmbeddedTemplate {
        target: "typescript",
        file_name: "union.ts.j2",
        source: include_str!("../../../templates/typescript/union.ts.j2"),
    },
    EmbeddedTemplate {
        target: "javascript",
        file_name: "config.d.ts.j2",
        source: include_str!("../../../templates/javascript/config.d.ts.j2"),
    },
    EmbeddedTemplate {
        target: "javascript",
        file_name: "config.js.j2",
        source: include_str!("../../../templates/javascript/config.js.j2"),
    },
    EmbeddedTemplate {
        target: "javascript",
        file_name: "enum.d.ts.j2",
        source: include_str!("../../../templates/javascript/enum.d.ts.j2"),
    },
    EmbeddedTemplate {
        target: "javascript",
        file_name: "enum.js.j2",
        source: include_str!("../../../templates/javascript/enum.js.j2"),
    },
    EmbeddedTemplate {
        target: "javascript",
        file_name: "index.d.ts.j2",
        source: include_str!("../../../templates/javascript/index.d.ts.j2"),
    },
    EmbeddedTemplate {
        target: "javascript",
        file_name: "index.js.j2",
        source: include_str!("../../../templates/javascript/index.js.j2"),
    },
    EmbeddedTemplate {
        target: "javascript",
        file_name: "package.json.j2",
        source: include_str!("../../../templates/javascript/package.json.j2"),
    },
    EmbeddedTemplate {
        target: "javascript",
        file_name: "record.d.ts.j2",
        source: include_str!("../../../templates/javascript/record.d.ts.j2"),
    },
    EmbeddedTemplate {
        target: "javascript",
        file_name: "record.js.j2",
        source: include_str!("../../../templates/javascript/record.js.j2"),
    },
    EmbeddedTemplate {
        target: "javascript",
        file_name: "runtime.d.ts.j2",
        source: include_str!("../../../templates/javascript/runtime.d.ts.j2"),
    },
    EmbeddedTemplate {
        target: "javascript",
        file_name: "runtime.js.j2",
        source: include_str!("../../../templates/javascript/runtime.js.j2"),
    },
    EmbeddedTemplate {
        target: "javascript",
        file_name: "union.d.ts.j2",
        source: include_str!("../../../templates/javascript/union.d.ts.j2"),
    },
    EmbeddedTemplate {
        target: "javascript",
        file_name: "union.js.j2",
        source: include_str!("../../../templates/javascript/union.js.j2"),
    },
    EmbeddedTemplate {
        target: "erlang",
        file_name: "config.erl.j2",
        source: include_str!("../../../templates/erlang/config.erl.j2"),
    },
    EmbeddedTemplate {
        target: "erlang",
        file_name: "enum.erl.j2",
        source: include_str!("../../../templates/erlang/enum.erl.j2"),
    },
    EmbeddedTemplate {
        target: "erlang",
        file_name: "record.erl.j2",
        source: include_str!("../../../templates/erlang/record.erl.j2"),
    },
    EmbeddedTemplate {
        target: "erlang",
        file_name: "runtime.erl.j2",
        source: include_str!("../../../templates/erlang/runtime.erl.j2"),
    },
    EmbeddedTemplate {
        target: "erlang",
        file_name: "union.erl.j2",
        source: include_str!("../../../templates/erlang/union.erl.j2"),
    },
    EmbeddedTemplate {
        target: "lua",
        file_name: "config.lua.j2",
        source: include_str!("../../../templates/lua/config.lua.j2"),
    },
    EmbeddedTemplate {
        target: "lua",
        file_name: "enum.lua.j2",
        source: include_str!("../../../templates/lua/enum.lua.j2"),
    },
    EmbeddedTemplate {
        target: "lua",
        file_name: "record.lua.j2",
        source: include_str!("../../../templates/lua/record.lua.j2"),
    },
    EmbeddedTemplate {
        target: "lua",
        file_name: "runtime.lua.j2",
        source: include_str!("../../../templates/lua/runtime.lua.j2"),
    },
    EmbeddedTemplate {
        target: "lua",
        file_name: "union.lua.j2",
        source: include_str!("../../../templates/lua/union.lua.j2"),
    },
    EmbeddedTemplate {
        target: "rust",
        file_name: "enum.rs.j2",
        source: include_str!("../../../templates/rust/enum.rs.j2"),
    },
    EmbeddedTemplate {
        target: "rust",
        file_name: "mod.rs.j2",
        source: include_str!("../../../templates/rust/mod.rs.j2"),
    },
    EmbeddedTemplate {
        target: "rust",
        file_name: "runtime.rs.j2",
        source: include_str!("../../../templates/rust/runtime.rs.j2"),
    },
    EmbeddedTemplate {
        target: "rust",
        file_name: "struct.rs.j2",
        source: include_str!("../../../templates/rust/struct.rs.j2"),
    },
    EmbeddedTemplate {
        target: "rust",
        file_name: "union.rs.j2",
        source: include_str!("../../../templates/rust/union.rs.j2"),
    },
    EmbeddedTemplate {
        target: "python",
        file_name: "__init__.py.j2",
        source: include_str!("../../../templates/python/__init__.py.j2"),
    },
    EmbeddedTemplate {
        target: "python",
        file_name: "config.py.j2",
        source: include_str!("../../../templates/python/config.py.j2"),
    },
    EmbeddedTemplate {
        target: "python",
        file_name: "enum.py.j2",
        source: include_str!("../../../templates/python/enum.py.j2"),
    },
    EmbeddedTemplate {
        target: "python",
        file_name: "record.py.j2",
        source: include_str!("../../../templates/python/record.py.j2"),
    },
    EmbeddedTemplate {
        target: "python",
        file_name: "runtime.py.j2",
        source: include_str!("../../../templates/python/runtime.py.j2"),
    },
    EmbeddedTemplate {
        target: "python",
        file_name: "union.py.j2",
        source: include_str!("../../../templates/python/union.py.j2"),
    },
];

pub fn template_source(target: &str, file_name: &str) -> Option<&'static str> {
    EMBEDDED_TEMPLATES
        .iter()
        .find(|template| template.target == target && template.file_name == file_name)
        .map(|template| template.source)
}

pub fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("crate should live under workspace/crates")
        .to_path_buf()
}

pub fn templates_dir() -> PathBuf {
    workspace_root().join("templates")
}

pub fn target_templates_dir(target: &str) -> PathBuf {
    templates_dir().join(target)
}

#[cfg(test)]
mod tests {
    use super::{EMBEDDED_TEMPLATES, template_source};

    #[test]
    fn embeds_all_templates() {
        assert_eq!(EMBEDDED_TEMPLATES.len(), 84);
        for template in EMBEDDED_TEMPLATES {
            let source = template_source(template.target, template.file_name)
                .expect("embedded template should be registered");
            assert!(!source.trim().is_empty());
        }
    }
}
