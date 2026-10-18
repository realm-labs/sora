use std::{fs, path::Path};

use minijinja::{Environment, Value, syntax::SyntaxConfig};
use sora_diagnostics::{Result, SoraError};
use sora_templates::template_source;

pub(crate) fn render_template(
    target: &str,
    file_name: &str,
    ctx: impl Into<Value>,
) -> Result<String> {
    let template_name = format!("{target}/{file_name}");
    let source = template_source(target, file_name).ok_or_else(|| SoraError::RenderTemplate {
        template: template_name.clone(),
        message: "embedded template not found".to_owned(),
    })?;
    let mut env = Environment::new();
    env.set_syntax(
        SyntaxConfig::builder()
            .keep_trailing_newline(true)
            .build()
            .map_err(|source| SoraError::RenderTemplate {
                template: template_name.clone(),
                message: source.to_string(),
            })?,
    );
    env.add_template(file_name, source)
        .map_err(|source| SoraError::RenderTemplate {
            template: template_name.clone(),
            message: source.to_string(),
        })?;
    let template = env
        .get_template(file_name)
        .map_err(|source| SoraError::RenderTemplate {
            template: template_name.clone(),
            message: source.to_string(),
        })?;
    let rendered = template
        .render(ctx)
        .map_err(|source| SoraError::RenderTemplate {
            template: template_name,
            message: source.to_string(),
        })?;
    Ok(normalize_line_endings(rendered))
}

fn normalize_line_endings(content: String) -> String {
    if content.contains("\r\n") {
        content.replace("\r\n", "\n")
    } else {
        content
    }
}

pub(crate) fn ensure_dir(path: &Path) -> Result<()> {
    fs::create_dir_all(path).map_err(|source| SoraError::CreateDir {
        path: path.to_path_buf(),
        source,
    })
}

pub(crate) fn write_file(path: &Path, content: String) -> Result<()> {
    if let Some(parent) = path.parent() {
        ensure_dir(parent)?;
    }
    fs::write(path, content).map_err(|source| SoraError::WriteFile {
        path: path.to_path_buf(),
        source,
    })
}

#[cfg(test)]
mod tests {
    use minijinja::{context, value::Serde};
    use serde::{Serialize, Serializer, ser::Error as SerializeError};
    use sora_diagnostics::SoraError;

    use crate::model::BaseEnumValue;

    use super::{normalize_line_endings, render_template};

    #[test]
    fn renders_borrowed_serde_contexts_with_nested_sequences() {
        #[derive(Serialize)]
        struct Enum {
            name: &'static str,
            comment: Option<&'static str>,
            values: Vec<BaseEnumValue>,
        }

        #[derive(Serialize)]
        struct EnumContext {
            package: String,
            r#enum: Enum,
        }

        let ctx = EnumContext {
            package: "game_config".to_owned(),
            r#enum: Enum {
                name: "ItemType",
                comment: Some("类别 <&> \"quoted\""),
                values: vec![
                    BaseEnumValue {
                        id: 7,
                        name: "Weapon".to_owned(),
                        comment: Some("Weapon item".to_owned()),
                    },
                    BaseEnumValue {
                        id: 42,
                        name: "Armor".to_owned(),
                        comment: None,
                    },
                ],
            },
        };

        let rendered = render_template(
            "go",
            "enum.go.j2",
            context! { package => &ctx.package, enum => Serde(&ctx.r#enum) },
        )
        .unwrap();
        let serialized = render_template("go", "enum.go.j2", Serde(&ctx)).unwrap();

        assert_eq!(rendered, serialized);
        assert!(rendered.contains("package game_config\n"));
        assert!(rendered.contains("// ItemType: 类别 <&> \"quoted\"\n"));
        assert!(rendered.contains(
            "    // ItemTypeWeapon: Weapon item\n    ItemTypeWeapon ItemType = 7\n    ItemTypeArmor ItemType = 42\n"
        ));
        assert!(rendered.ends_with("}\n"));
        assert!(!rendered.contains('\r'));
    }

    #[test]
    fn empty_context_preserves_template_bytes_and_trailing_newline() {
        let rendered = render_template("javascript", "package.json.j2", context! {}).unwrap();

        assert_eq!(rendered, "{\n  \"type\": \"module\"\n}\n");
    }

    #[test]
    fn rendered_serde_values_use_lf_line_endings() {
        #[derive(Serialize)]
        struct Manifest {
            name: &'static str,
            version: &'static str,
            edition: &'static str,
            publish: bool,
        }

        let manifest = Manifest {
            name: "game-config",
            version: "1.2.3\r\n",
            edition: "2024",
            publish: false,
        };
        let rendered = render_template(
            "rust",
            "Cargo.toml.j2",
            context! { package => Serde(&manifest) },
        )
        .unwrap();

        assert!(rendered.contains("version = \"1.2.3\n\"\n"));
        assert!(rendered.contains("publish = false\n"));
        assert!(!rendered.contains('\r'));
    }

    #[test]
    fn missing_templates_keep_the_qualified_template_name() {
        let error = render_template("rust", "missing.rs.j2", context! {}).unwrap_err();

        match error {
            SoraError::RenderTemplate { template, message } => {
                assert_eq!(template, "rust/missing.rs.j2");
                assert_eq!(message, "embedded template not found");
            }
            other => panic!("expected a template error, got {other:?}"),
        }
    }

    #[test]
    fn serde_conversion_errors_keep_the_qualified_template_name() {
        struct InvalidEnum;

        impl Serialize for InvalidEnum {
            fn serialize<S: Serializer>(&self, _serializer: S) -> Result<S::Ok, S::Error> {
                Err(SerializeError::custom("enum serialization failed"))
            }
        }

        let error = render_template(
            "go",
            "enum.go.j2",
            context! { package => "game_config", enum => Serde(&InvalidEnum) },
        )
        .unwrap_err();

        match error {
            SoraError::RenderTemplate { template, message } => {
                assert_eq!(template, "go/enum.go.j2");
                assert!(message.contains("enum serialization failed"), "{message}");
            }
            other => panic!("expected a template error, got {other:?}"),
        }
    }

    #[test]
    fn rendered_templates_use_lf_line_endings() {
        assert_eq!(
            normalize_line_endings("first\r\nsecond\r\n".to_owned()),
            "first\nsecond\n"
        );
        assert_eq!(
            normalize_line_endings("first\nsecond\n".to_owned()),
            "first\nsecond\n"
        );
    }
}
