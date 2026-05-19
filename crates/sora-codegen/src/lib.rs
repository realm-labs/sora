use std::{fs, path::Path};

use heck::{ToLowerCamelCase, ToPascalCase, ToSnakeCase};
use minijinja::{Environment, context};
use serde::Serialize;
use sora_diagnostics::{Result, SoraError};
use sora_ir::{ConfigIr, FieldIr, TableIr, TypeIr};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodegenTarget {
    Rust,
    Kotlin,
}

pub trait CodeGenerator {
    fn generate(&self, ir: &ConfigIr, out_dir: &Path) -> Result<()>;
}

pub struct RustCodeGenerator;
pub struct KotlinCodeGenerator;

#[derive(Debug, Clone, Serialize)]
pub struct CodegenModel {
    pub package: String,
    pub enums: Vec<CodegenEnum>,
    pub records: Vec<CodegenRecord>,
    pub modules: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CodegenEnum {
    pub name: String,
    pub values: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CodegenRecord {
    pub name: String,
    pub pascal_name: String,
    pub snake_name: String,
    pub camel_name: String,
    pub fields: Vec<CodegenField>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CodegenField {
    pub raw_name: String,
    pub rust_name: String,
    pub rust_type: String,
    pub kotlin_name: String,
    pub kotlin_type: String,
    pub comment: Option<String>,
}

impl CodeGenerator for RustCodeGenerator {
    fn generate(&self, ir: &ConfigIr, out_dir: &Path) -> Result<()> {
        ensure_dir(out_dir)?;
        let model = build_model(ir)?;

        for item in &model.enums {
            let rendered = render_template("rust", "enum.rs.j2", context! { enum => item })?;
            write_file(
                &out_dir.join(format!("{}.rs", item.name.to_snake_case())),
                rendered,
            )?;
        }

        for record in &model.records {
            let rendered = render_template("rust", "struct.rs.j2", context! { record => record })?;
            write_file(&out_dir.join(format!("{}.rs", record.snake_name)), rendered)?;
        }

        let rendered = render_template("rust", "mod.rs.j2", context! { model => &model })?;
        write_file(&out_dir.join("mod.rs"), rendered)
    }
}

impl CodeGenerator for KotlinCodeGenerator {
    fn generate(&self, ir: &ConfigIr, out_dir: &Path) -> Result<()> {
        ensure_dir(out_dir)?;
        let model = build_model(ir)?;

        for item in &model.enums {
            let rendered = render_template(
                "kotlin",
                "enum.kt.j2",
                context! { package => &model.package, enum => item },
            )?;
            write_file(&out_dir.join(format!("{}.kt", item.name)), rendered)?;
        }

        for record in &model.records {
            let rendered = render_template(
                "kotlin",
                "data_class.kt.j2",
                context! { package => &model.package, record => record },
            )?;
            write_file(
                &out_dir.join(format!("{}.kt", record.pascal_name)),
                rendered,
            )?;
        }

        let rendered = render_template("kotlin", "package.kt.j2", context! { model => &model })?;
        write_file(&out_dir.join("Package.kt"), rendered)
    }
}

pub fn generator_for_target(target: CodegenTarget) -> Box<dyn CodeGenerator> {
    match target {
        CodegenTarget::Rust => Box::new(RustCodeGenerator),
        CodegenTarget::Kotlin => Box::new(KotlinCodeGenerator),
    }
}

pub fn build_model(ir: &ConfigIr) -> Result<CodegenModel> {
    let enums = ir
        .enums
        .iter()
        .map(|item| CodegenEnum {
            name: item.name.clone(),
            values: item.values.clone(),
        })
        .collect::<Vec<_>>();

    let records = ir
        .structs
        .iter()
        .map(|item| {
            build_record(
                ir,
                &TableLike {
                    name: &item.name,
                    fields: &item.fields,
                },
            )
        })
        .chain(ir.tables.iter().map(|item| build_record(ir, &item.into())))
        .collect::<Result<Vec<_>>>()?;

    let modules = enums
        .iter()
        .map(|item| item.name.to_snake_case())
        .chain(records.iter().map(|item| item.snake_name.clone()))
        .collect();

    Ok(CodegenModel {
        package: ir.package.clone(),
        enums,
        records,
        modules,
    })
}

pub fn rust_type_name(ir: &ConfigIr, ty: &TypeIr) -> String {
    rust_type_name_inner(ir, ty)
}

pub fn kotlin_type_name(ir: &ConfigIr, ty: &TypeIr) -> String {
    kotlin_type_name_inner(ir, ty)
}

fn build_record(ir: &ConfigIr, item: &TableLike<'_>) -> Result<CodegenRecord> {
    let fields = item
        .fields
        .iter()
        .map(|field| {
            Ok(CodegenField {
                raw_name: field.name.clone(),
                rust_name: field.name.to_snake_case(),
                rust_type: rust_type_name(ir, &field.ty),
                kotlin_name: field.name.to_lower_camel_case(),
                kotlin_type: kotlin_type_name(ir, &field.ty),
                comment: field.comment.clone(),
            })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(CodegenRecord {
        name: item.name.to_owned(),
        pascal_name: item.name.to_pascal_case(),
        snake_name: item.name.to_snake_case(),
        camel_name: item.name.to_lower_camel_case(),
        fields,
    })
}

fn rust_type_name_inner(ir: &ConfigIr, ty: &TypeIr) -> String {
    match ty {
        TypeIr::Bool => "bool".to_owned(),
        TypeIr::I32 => "i32".to_owned(),
        TypeIr::I64 => "i64".to_owned(),
        TypeIr::F32 => "f32".to_owned(),
        TypeIr::F64 => "f64".to_owned(),
        TypeIr::String => "String".to_owned(),
        TypeIr::Enum(name) | TypeIr::Struct(name) => name.clone(),
        TypeIr::List(element) => format!("Vec<{}>", rust_type_name_inner(ir, element)),
        TypeIr::Array { element, len } => {
            format!("[{}; {len}]", rust_type_name_inner(ir, element))
        }
        TypeIr::Ref { table, field } => ref_type(ir, table, field, rust_type_name_inner, "i32"),
        TypeIr::Optional(element) => format!("Option<{}>", rust_type_name_inner(ir, element)),
    }
}

fn kotlin_type_name_inner(ir: &ConfigIr, ty: &TypeIr) -> String {
    match ty {
        TypeIr::Bool => "Boolean".to_owned(),
        TypeIr::I32 => "Int".to_owned(),
        TypeIr::I64 => "Long".to_owned(),
        TypeIr::F32 => "Float".to_owned(),
        TypeIr::F64 => "Double".to_owned(),
        TypeIr::String => "String".to_owned(),
        TypeIr::Enum(name) | TypeIr::Struct(name) => name.clone(),
        TypeIr::List(element) | TypeIr::Array { element, .. } => {
            format!("List<{}>", kotlin_type_name_inner(ir, element))
        }
        TypeIr::Ref { table, field } => ref_type(ir, table, field, kotlin_type_name_inner, "Int"),
        TypeIr::Optional(element) => format!("{}?", kotlin_type_name_inner(ir, element)),
    }
}

fn ref_type(
    ir: &ConfigIr,
    table_name: &str,
    field_name: &str,
    mapper: fn(&ConfigIr, &TypeIr) -> String,
    fallback: &str,
) -> String {
    ir.tables
        .iter()
        .find(|table| table.name == table_name)
        .and_then(|table| table.fields.iter().find(|field| field.name == field_name))
        .map(|field| mapper(ir, &field.ty))
        .unwrap_or_else(|| fallback.to_owned())
}

fn render_template(target: &str, file_name: &str, ctx: impl Serialize) -> Result<String> {
    let path = sora_templates::target_templates_dir(target).join(file_name);
    let source = fs::read_to_string(&path).map_err(|source| SoraError::ReadFile {
        path: path.clone(),
        source,
    })?;
    let mut env = Environment::new();
    env.add_template(file_name, &source)
        .map_err(|source| SoraError::RenderTemplate {
            template: file_name.to_owned(),
            message: source.to_string(),
        })?;
    let template = env
        .get_template(file_name)
        .map_err(|source| SoraError::RenderTemplate {
            template: file_name.to_owned(),
            message: source.to_string(),
        })?;
    template
        .render(ctx)
        .map_err(|source| SoraError::RenderTemplate {
            template: file_name.to_owned(),
            message: source.to_string(),
        })
}

fn ensure_dir(path: &Path) -> Result<()> {
    fs::create_dir_all(path).map_err(|source| SoraError::CreateDir {
        path: path.to_path_buf(),
        source,
    })
}

fn write_file(path: &Path, content: String) -> Result<()> {
    if let Some(parent) = path.parent() {
        ensure_dir(parent)?;
    }
    fs::write(path, content).map_err(|source| SoraError::WriteFile {
        path: path.to_path_buf(),
        source,
    })
}

struct TableLike<'a> {
    name: &'a str,
    fields: &'a [FieldIr],
}

impl<'a> From<&'a TableIr> for TableLike<'a> {
    fn from(table: &'a TableIr) -> Self {
        Self {
            name: &table.name,
            fields: &table.fields,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sora_ir::{normalize_schema, parse_type};
    use sora_schema::SchemaFile;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn maps_rust_types() {
        let ir = example_ir();
        let cases = [
            ("bool", "bool"),
            ("i32", "i32"),
            ("i64", "i64"),
            ("f32", "f32"),
            ("f64", "f64"),
            ("string", "String"),
            ("enum<ItemType>", "ItemType"),
            ("struct<Reward>", "Reward"),
            ("list<i32>", "Vec<i32>"),
            ("array<i32,3>", "[i32; 3]"),
            ("optional<string>", "Option<String>"),
            ("ref<Item.id>", "i32"),
        ];

        for (source, expected) in cases {
            assert_eq!(rust_type_name(&ir, &parse_type(source).unwrap()), expected);
        }
    }

    #[test]
    fn maps_kotlin_types() {
        let ir = example_ir();
        let cases = [
            ("bool", "Boolean"),
            ("i32", "Int"),
            ("i64", "Long"),
            ("f32", "Float"),
            ("f64", "Double"),
            ("string", "String"),
            ("enum<ItemType>", "ItemType"),
            ("struct<Reward>", "Reward"),
            ("list<i32>", "List<Int>"),
            ("array<i32,3>", "List<Int>"),
            ("optional<string>", "String?"),
            ("ref<Item.id>", "Int"),
        ];

        for (source, expected) in cases {
            assert_eq!(
                kotlin_type_name(&ir, &parse_type(source).unwrap()),
                expected
            );
        }
    }

    #[test]
    fn generates_rust_and_kotlin_files() {
        let ir = example_ir();
        let base = temp_dir();
        let rust_out = base.join("rust");
        let kotlin_out = base.join("kotlin");

        RustCodeGenerator.generate(&ir, &rust_out).unwrap();
        KotlinCodeGenerator.generate(&ir, &kotlin_out).unwrap();

        let rust_item = fs::read_to_string(rust_out.join("item.rs")).unwrap();
        let kotlin_item = fs::read_to_string(kotlin_out.join("Item.kt")).unwrap();

        assert!(rust_item.contains("pub struct Item"));
        assert!(rust_item.contains("pub item_type: ItemType"));
        assert!(kotlin_item.contains("data class Item"));
        assert!(kotlin_item.contains("val itemType: ItemType"));

        let _ = fs::remove_dir_all(base);
    }

    fn example_ir() -> ConfigIr {
        let schema: SchemaFile = toml::from_str(
            r#"
package = "game_config"

[[enums]]
name = "ItemType"
values = ["Weapon", "Armor", "Material", "Consumable"]

[[tables]]
name = "Item"
mode = "map"
key = "id"

[[tables.fields]]
name = "id"
type = "i32"
key = true
required = true
comment = "Item id"

[[tables.fields]]
name = "item_type"
type = "enum<ItemType>"
required = true
comment = "Item type"
"#,
        )
        .unwrap();

        normalize_schema(schema).unwrap()
    }

    fn temp_dir() -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("sora-codegen-test-{unique}"))
    }
}
