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
        assert_eq!(EMBEDDED_TEMPLATES.len(), 31);
        for template in EMBEDDED_TEMPLATES {
            let source = template_source(template.target, template.file_name)
                .expect("embedded template should be registered");
            assert!(!source.trim().is_empty());
        }
    }
}
