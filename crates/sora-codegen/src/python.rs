use std::{collections::HashSet, path::Path};

use heck::{ToPascalCase, ToShoutySnakeCase, ToSnakeCase};
use minijinja::context;
use serde::Serialize;
use sora_diagnostics::{Result, SoraError};
use sora_ir::model::{ConfigIr, TypeIr};

use crate::{
    generator::{CodeGenerator, CodegenContext, runtime_format_name},
    model::{
        BaseField, BaseImport, BaseIndex, BaseModel, BaseRecord, BaseTable, BaseUnion,
        BaseUnionVariant, build_base_model,
    },
    options::LanguageCodegenOptions,
    render::{ensure_dir, render_template, write_file},
};

pub struct PythonCodeGenerator;
crate::impl_test_codegen_generate!(PythonCodeGenerator, "python");

impl CodeGenerator for PythonCodeGenerator {
    fn generate(&self, context: CodegenContext<'_>, out_dir: &Path) -> Result<()> {
        let ir = context.ir;
        let options = context.options::<LanguageCodegenOptions>()?;
        ensure_dir(out_dir)?;

        let model = PythonModel::from_base_model(ir, build_base_model(ir)?);
        let runtime_format = runtime_format_name(options.runtime_format);
        validate_python_model(&model)?;

        for item in &model.enums {
            let rendered = render_template(
                "python",
                "enum.py.j2",
                context! { enum => item, runtime_format => runtime_format },
            )?;
            write_file(&out_dir.join(format!("{}.py", item.snake_name)), rendered)?;
        }

        for record in &model.records {
            let rendered = render_template(
                "python",
                "record.py.j2",
                context! { record => record, runtime_format => runtime_format },
            )?;
            write_file(&out_dir.join(format!("{}.py", record.snake_name)), rendered)?;
        }

        for union in &model.unions {
            let rendered = render_template(
                "python",
                "union.py.j2",
                context! { union => union, runtime_format => runtime_format },
            )?;
            write_file(&out_dir.join(format!("{}.py", union.snake_name)), rendered)?;
        }

        let rendered = render_template(
            "python",
            "runtime.py.j2",
            context! { runtime_format => runtime_format },
        )?;
        write_file(&out_dir.join("sora_runtime.py"), rendered)?;

        let rendered = render_template(
            "python",
            "config.py.j2",
            context! { model => &model, runtime_format => runtime_format },
        )?;
        write_file(&out_dir.join("sora_config.py"), rendered)?;

        let rendered = render_template("python", "__init__.py.j2", context! { model => &model })?;
        write_file(&out_dir.join("__init__.py"), rendered)
    }
}

#[derive(Debug, Clone, Serialize)]
struct PythonModel {
    package: String,
    schema_fingerprint: String,
    enums: Vec<PythonEnum>,
    unions: Vec<PythonUnion>,
    records: Vec<PythonRecord>,
    tables: Vec<PythonTable>,
    modules: Vec<String>,
    has_localization: bool,
    locales: Vec<String>,
    default_locale: String,
}

#[derive(Debug, Clone, Serialize)]
struct PythonEnum {
    name: String,
    snake_name: String,
    value_names: Vec<String>,
    values: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct PythonUnion {
    pascal_name: String,
    snake_name: String,
    tag: String,
    variants: Vec<PythonUnionVariant>,
    imports: Vec<PythonImport>,
}

#[derive(Debug, Clone, Serialize)]
struct PythonUnionVariant {
    raw_name: String,
    name: String,
    fields: Vec<PythonField>,
    has_text_keys: bool,
}

#[derive(Debug, Clone, Serialize)]
struct PythonRecord {
    pascal_name: String,
    snake_name: String,
    imports: Vec<PythonImport>,
    fields: Vec<PythonField>,
    uses_text_key: bool,
    has_text_keys: bool,
    table: Option<PythonTable>,
}

#[derive(Debug, Clone, Serialize)]
struct PythonImport {
    module: String,
    name: String,
}

#[derive(Debug, Clone, Serialize)]
struct PythonTable {
    name: String,
    pascal_name: String,
    snake_name: String,
    mode: String,
    row_type: String,
    key_name: Option<String>,
    key_field_name: Option<String>,
    key_type: Option<String>,
    unique_indexes: Vec<PythonIndex>,
    non_unique_indexes: Vec<PythonIndex>,
}

#[derive(Debug, Clone, Serialize)]
struct PythonIndex {
    name: String,
    field_name: String,
    param_name: String,
    param_type: String,
    key_type: String,
}

#[derive(Debug, Clone, Serialize)]
struct PythonField {
    raw_name: String,
    name: String,
    type_name: String,
    decode: String,
    value_decode: String,
    collect_text_keys: String,
    comment: Option<String>,
}

impl PythonModel {
    fn from_base_model(ir: &ConfigIr, model: BaseModel) -> Self {
        let enums = model
            .enums
            .into_iter()
            .map(|item| PythonEnum {
                name: python_type_identifier(&item.pascal_name),
                snake_name: python_module_name(&item.snake_name),
                value_names: item
                    .values
                    .iter()
                    .map(|value| python_enum_value_name(value))
                    .collect(),
                values: item.values,
            })
            .collect::<Vec<_>>();

        let tables = model
            .tables
            .into_iter()
            .map(|item| python_table(ir, item))
            .collect::<Vec<_>>();
        let records = model
            .records
            .into_iter()
            .map(|item| {
                let row_type = python_type_identifier(&item.pascal_name);
                let table = tables
                    .iter()
                    .find(|table| table.row_type == row_type)
                    .cloned();
                python_record(ir, item, table)
            })
            .collect::<Vec<_>>();
        let unions = model
            .unions
            .into_iter()
            .map(|item| python_union(ir, item))
            .collect::<Vec<_>>();
        let modules = enums
            .iter()
            .map(|item| item.snake_name.clone())
            .chain(records.iter().map(|item| item.snake_name.clone()))
            .chain(unions.iter().map(|item| item.snake_name.clone()))
            .collect();

        Self {
            package: model.package,
            schema_fingerprint: model.schema_fingerprint,
            enums,
            unions,
            records,
            tables,
            modules,
            has_localization: ir.localization.is_some(),
            locales: ir
                .localization
                .as_ref()
                .map(|item| item.locales.clone())
                .unwrap_or_default(),
            default_locale: ir
                .localization
                .as_ref()
                .map(|item| item.default_locale.clone())
                .unwrap_or_default(),
        }
    }
}

fn python_record(ir: &ConfigIr, record: BaseRecord, table: Option<PythonTable>) -> PythonRecord {
    let fields = record
        .fields
        .into_iter()
        .map(|field| python_field(ir, field))
        .collect::<Vec<_>>();
    let uses_text_key = fields
        .iter()
        .any(|field| field.type_name.contains("TextKey"));
    let has_text_keys = fields
        .iter()
        .any(|field| !field.collect_text_keys.is_empty());
    PythonRecord {
        pascal_name: python_type_identifier(&record.pascal_name),
        snake_name: python_module_name(&record.snake_name),
        imports: record.imports.into_iter().map(python_import).collect(),
        fields,
        uses_text_key,
        has_text_keys,
        table,
    }
}

fn python_union(ir: &ConfigIr, union: BaseUnion) -> PythonUnion {
    PythonUnion {
        pascal_name: python_type_identifier(&union.pascal_name),
        snake_name: python_module_name(&union.snake_name),
        tag: python_field_identifier(&union.tag),
        variants: union
            .variants
            .into_iter()
            .map(|variant| python_variant(ir, variant))
            .collect(),
        imports: union.imports.into_iter().map(python_import).collect(),
    }
}

fn python_variant(ir: &ConfigIr, variant: BaseUnionVariant) -> PythonUnionVariant {
    let fields = variant
        .fields
        .into_iter()
        .map(|field| python_field(ir, field))
        .collect::<Vec<_>>();
    let has_text_keys = fields
        .iter()
        .any(|field| !field.collect_text_keys.is_empty());
    PythonUnionVariant {
        raw_name: variant.name,
        name: python_type_identifier(&variant.pascal_name),
        fields,
        has_text_keys,
    }
}

fn python_table(ir: &ConfigIr, table: BaseTable) -> PythonTable {
    let key_type = table
        .key_field
        .as_ref()
        .map(|field| python_type_name(ir, &field.ty));
    PythonTable {
        name: table.name,
        pascal_name: python_type_identifier(&table.pascal_name),
        snake_name: python_module_name(&table.snake_name),
        mode: table.mode_name,
        row_type: python_type_identifier(&table.pascal_name),
        key_name: table.key_name,
        key_field_name: table
            .key_field
            .as_ref()
            .map(|field| python_field_identifier(&field.snake_name)),
        key_type,
        unique_indexes: table
            .unique_indexes
            .into_iter()
            .map(|index| python_index(ir, index))
            .collect(),
        non_unique_indexes: table
            .non_unique_indexes
            .into_iter()
            .map(|index| python_index(ir, index))
            .collect(),
    }
}

fn python_index(ir: &ConfigIr, index: BaseIndex) -> PythonIndex {
    let key_type = python_type_name(ir, &index.field.ty);
    PythonIndex {
        name: python_field_identifier(&index.snake_name),
        field_name: python_field_identifier(&index.field.snake_name),
        param_name: python_field_identifier(&index.field.snake_name),
        param_type: key_type.clone(),
        key_type,
    }
}

fn python_field(ir: &ConfigIr, field: BaseField) -> PythonField {
    let collect_text_keys =
        python_collect_text_keys(ir, &field.ty, &format!("self.{}", field.snake_name), 8);
    PythonField {
        raw_name: field.raw_name,
        name: python_field_identifier(&field.snake_name),
        type_name: python_type_name(ir, &field.ty),
        decode: python_decode_expr(ir, &field.ty),
        value_decode: python_value_decode_expr(ir, &field.ty, "__VALUE__"),
        collect_text_keys,
        comment: field.comment,
    }
}

fn python_import(import: BaseImport) -> PythonImport {
    PythonImport {
        module: python_module_name(&import.module),
        name: python_type_identifier(&import.name),
    }
}

fn validate_python_model(model: &PythonModel) -> Result<()> {
    reject_duplicates(
        "Python module",
        model.modules.iter().map(String::as_str),
        "module names collide after Python identifier normalization",
    )?;

    for item in &model.enums {
        reject_duplicates(
            "Python enum value",
            item.value_names.iter().map(String::as_str),
            &format!(
                "enum `{}` has values that collide after Python identifier normalization",
                item.name
            ),
        )?;
    }

    for record in &model.records {
        reject_duplicates(
            "Python field",
            record.fields.iter().map(|field| field.name.as_str()),
            &format!(
                "record `{}` has fields that collide after Python identifier normalization",
                record.pascal_name
            ),
        )?;
    }

    for union in &model.unions {
        reject_duplicates(
            "Python union variant",
            union.variants.iter().map(|variant| variant.name.as_str()),
            &format!(
                "union `{}` has variants that collide after Python identifier normalization",
                union.pascal_name
            ),
        )?;
        for variant in &union.variants {
            reject_duplicates(
                "Python union field",
                variant.fields.iter().map(|field| field.name.as_str()),
                &format!(
                    "union variant `{}.{}` has fields that collide after Python identifier normalization",
                    union.pascal_name, variant.name
                ),
            )?;
        }
    }

    Ok(())
}

fn reject_duplicates<'a>(
    kind: &'static str,
    values: impl Iterator<Item = &'a str>,
    message: &str,
) -> Result<()> {
    let mut seen = HashSet::new();
    for value in values {
        if !seen.insert(value) {
            return Err(SoraError::InvalidSchema(format!(
                "{message}: duplicate {kind} `{value}`"
            )));
        }
    }
    Ok(())
}

fn python_type_name(ir: &ConfigIr, ty: &TypeIr) -> String {
    match ty {
        TypeIr::Bool => "bool".to_owned(),
        TypeIr::I8
        | TypeIr::U8
        | TypeIr::I16
        | TypeIr::U16
        | TypeIr::I32
        | TypeIr::U32
        | TypeIr::I64 => "int".to_owned(),
        TypeIr::F32 | TypeIr::F64 => "float".to_owned(),
        TypeIr::String => "str".to_owned(),
        TypeIr::Text => "TextKey".to_owned(),
        TypeIr::Enum(name) | TypeIr::Struct(name) | TypeIr::Union(name) => {
            python_type_identifier(name)
        }
        TypeIr::List(element) | TypeIr::Set(element) | TypeIr::Array { element, .. } => {
            format!("list[{}]", python_type_name(ir, element))
        }
        TypeIr::Map { key, value } => {
            format!(
                "dict[{}, {}]",
                python_type_name(ir, key),
                python_type_name(ir, value)
            )
        }
        TypeIr::Ref { table, field } => ref_target_type(ir, table, field)
            .map(|ty| python_type_name(ir, ty))
            .unwrap_or_else(|| "int".to_owned()),
        TypeIr::Optional(element) => {
            format!("{} | None", python_type_name(ir, element))
        }
    }
}

fn python_decode_expr(ir: &ConfigIr, ty: &TypeIr) -> String {
    match ty {
        TypeIr::Bool => "reader.read_bool()".to_owned(),
        TypeIr::I8 | TypeIr::I16 | TypeIr::I32 => "reader.read_i32()".to_owned(),
        TypeIr::U8 | TypeIr::U16 | TypeIr::U32 => "reader.read_u32()".to_owned(),
        TypeIr::I64 => "reader.read_i64()".to_owned(),
        TypeIr::F32 => "reader.read_f32()".to_owned(),
        TypeIr::F64 => "reader.read_f64()".to_owned(),
        TypeIr::String => "reader.read_string()".to_owned(),
        TypeIr::Text => "TextKey(reader.read_string())".to_owned(),
        TypeIr::Enum(name) | TypeIr::Struct(name) | TypeIr::Union(name) => {
            format!("{}.decode(reader)", python_type_identifier(name))
        }
        TypeIr::List(element) | TypeIr::Set(element) | TypeIr::Array { element, .. } => {
            format!(
                "reader.read_list(lambda: {})",
                python_decode_expr(ir, element)
            )
        }
        TypeIr::Map { key, value } => format!(
            "reader.read_map(lambda: {}, lambda: {})",
            python_decode_expr(ir, key),
            python_decode_expr(ir, value)
        ),
        TypeIr::Ref { table, field } => ref_target_type(ir, table, field)
            .map(|ty| python_decode_expr(ir, ty))
            .unwrap_or_else(|| "reader.read_i32()".to_owned()),
        TypeIr::Optional(element) => {
            format!(
                "reader.read_optional(lambda: {})",
                python_decode_expr(ir, element)
            )
        }
    }
}

fn python_value_decode_expr(ir: &ConfigIr, ty: &TypeIr, value: &str) -> String {
    match ty {
        TypeIr::Bool => format!("{value}.as_bool()"),
        TypeIr::I8
        | TypeIr::U8
        | TypeIr::I16
        | TypeIr::U16
        | TypeIr::I32
        | TypeIr::U32
        | TypeIr::I64 => format!("{value}.as_int()"),
        TypeIr::F32 | TypeIr::F64 => format!("{value}.as_float()"),
        TypeIr::String => format!("{value}.as_string()"),
        TypeIr::Text => format!("TextKey({value}.as_string())"),
        TypeIr::Enum(name) | TypeIr::Struct(name) | TypeIr::Union(name) => {
            format!("{}.decode_value({value})", python_type_identifier(name))
        }
        TypeIr::List(element) | TypeIr::Set(element) | TypeIr::Array { element, .. } => {
            let item_decode = python_value_decode_expr(ir, element, "item");
            format!("{value}.as_list(lambda item: {item_decode})")
        }
        TypeIr::Map {
            key,
            value: element,
        } => {
            let key_decode = python_value_decode_expr(ir, key, "item");
            let value_decode = python_value_decode_expr(ir, element, "item");
            format!("{value}.as_map(lambda item: {key_decode}, lambda item: {value_decode})")
        }
        TypeIr::Ref { table, field } => ref_target_type(ir, table, field)
            .map(|ty| python_value_decode_expr(ir, ty, value))
            .unwrap_or_else(|| format!("{value}.as_int()")),
        TypeIr::Optional(element) => {
            let item_decode = python_value_decode_expr(ir, element, value);
            format!("None if {value}.is_null() else {item_decode}")
        }
    }
}

fn python_collect_text_keys(ir: &ConfigIr, ty: &TypeIr, value: &str, indent: usize) -> String {
    let pad = " ".repeat(indent);
    let nested_pad = " ".repeat(indent + 4);
    match ty {
        TypeIr::Text => format!("{pad}out.append({value})"),
        TypeIr::Optional(element) => {
            let inner = python_collect_text_keys(ir, element, "item", indent + 4);
            if inner.is_empty() {
                String::new()
            } else {
                format!("{pad}if {value} is not None:\n{nested_pad}item = {value}\n{inner}")
            }
        }
        TypeIr::List(element) | TypeIr::Set(element) | TypeIr::Array { element, .. } => {
            let inner = python_collect_text_keys(ir, element, "item", indent + 4);
            if inner.is_empty() {
                String::new()
            } else {
                format!("{pad}for item in {value}:\n{inner}")
            }
        }
        TypeIr::Map {
            key,
            value: element,
        } => {
            let key_inner = python_collect_text_keys(ir, key, "key", indent + 4);
            let value_inner = python_collect_text_keys(ir, element, "item", indent + 4);
            if key_inner.is_empty() && value_inner.is_empty() {
                String::new()
            } else {
                format!("{pad}for key, item in {value}.items():\n{key_inner}\n{value_inner}")
            }
        }
        TypeIr::Struct(_) | TypeIr::Union(_) => format!("{pad}{value}.collect_text_keys(out)"),
        TypeIr::Ref { table, field } => ref_target_type(ir, table, field)
            .map(|ty| python_collect_text_keys(ir, ty, value, indent))
            .unwrap_or_default(),
        TypeIr::Bool
        | TypeIr::I8
        | TypeIr::U8
        | TypeIr::I16
        | TypeIr::U16
        | TypeIr::I32
        | TypeIr::U32
        | TypeIr::I64
        | TypeIr::F32
        | TypeIr::F64
        | TypeIr::String
        | TypeIr::Enum(_) => String::new(),
    }
}

fn ref_target_type<'a>(ir: &'a ConfigIr, table: &str, field: &str) -> Option<&'a TypeIr> {
    ir.tables
        .iter()
        .find(|candidate| candidate.name == table)
        .and_then(|table| {
            table
                .fields
                .iter()
                .find(|candidate| candidate.name == *field)
        })
        .map(|field| &field.ty)
}

fn python_identifier(value: String) -> String {
    let mut output = String::new();
    for (index, ch) in value.chars().enumerate() {
        if ch == '_' || ch.is_ascii_alphanumeric() {
            if index == 0 && ch.is_ascii_digit() {
                output.push('_');
            }
            output.push(ch);
        } else if !output.ends_with('_') {
            output.push('_');
        }
    }

    if output.is_empty() {
        output.push('_');
    }
    if is_python_keyword(&output) {
        output.push('_');
    }
    output
}

fn python_type_identifier(raw_name: &str) -> String {
    python_identifier(raw_name.to_pascal_case())
}

fn python_module_name(raw_name: &str) -> String {
    python_identifier(raw_name.to_snake_case())
}

fn python_field_identifier(raw_name: &str) -> String {
    python_identifier(raw_name.to_snake_case())
}

fn python_enum_value_name(raw_name: &str) -> String {
    python_identifier(raw_name.to_shouty_snake_case())
}

fn is_python_keyword(value: &str) -> bool {
    matches!(
        value,
        "False"
            | "None"
            | "True"
            | "and"
            | "as"
            | "assert"
            | "async"
            | "await"
            | "break"
            | "class"
            | "continue"
            | "def"
            | "del"
            | "elif"
            | "else"
            | "except"
            | "finally"
            | "for"
            | "from"
            | "global"
            | "if"
            | "import"
            | "in"
            | "is"
            | "lambda"
            | "nonlocal"
            | "not"
            | "or"
            | "pass"
            | "raise"
            | "return"
            | "try"
            | "while"
            | "with"
            | "yield"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::options::{LanguageCodegenOptions, RuntimeFormat};
    use sora_ir::normalize::normalize_schema;
    use sora_schema::model::SchemaFile;
    use std::{
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };

    static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn generates_python_files() {
        let ir = example_ir();
        let base = temp_dir();

        PythonCodeGenerator.generate(&ir, &base).unwrap();

        let item = std::fs::read_to_string(base.join("item.py")).unwrap();
        let item_type = std::fs::read_to_string(base.join("item_type.py")).unwrap();
        let action = std::fs::read_to_string(base.join("action.py")).unwrap();
        let runtime = std::fs::read_to_string(base.join("sora_runtime.py")).unwrap();
        let config = std::fs::read_to_string(base.join("sora_config.py")).unwrap();
        let init = std::fs::read_to_string(base.join("__init__.py")).unwrap();

        assert!(item.contains("@dataclass(frozen=True, slots=True)"));
        assert!(item.contains("class Item:"));
        assert!(item.contains("def decode(reader: SoraReader) -> Item:"));
        assert!(item.contains("item_type = ItemType.decode(reader)"));
        assert!(item.contains("large_id = reader.read_i64()"));
        assert!(item_type.contains("class ItemType(Enum):"));
        assert!(action.contains("class Action:"));
        assert!(action.contains("def decode(reader: SoraReader) -> Action:"));
        assert!(action.contains("class ActionAddItem(Action):"));
        assert!(!action.contains("Action.decode = staticmethod"));
        assert!(runtime.contains("class SoraReader:"));
        assert!(runtime.contains("class SoraConfigTable:"));
        assert!(runtime.contains("def read_u32_at(data: bytes, offset: int) -> int:"));
        assert!(runtime.contains("def read_i64(self) -> int:"));
        assert!(runtime.contains("duplicate map key"));
        assert!(item.contains("class ItemTable"));
        assert!(!config.contains("class ItemTable"));
        assert!(item.contains("def get(self, key: int) -> Item | None:"));
        assert!(item.contains(") -> Item | None:"));
        assert!(item.contains(") -> list[Item]:"));
        assert!(config.contains("class SoraConfig:"));
        assert!(config.contains("def from_bytes(bytes_data: bytes) -> SoraConfig:"));
        assert!(init.contains("from .sora_config import SoraConfig"));

        let _ = std::fs::remove_dir_all(base);
    }

    #[test]
    fn python_supports_export_runtime_formats() {
        for (runtime_format, parse_function) in [
            (RuntimeFormat::Json, "parse_json"),
            (RuntimeFormat::Cbor, "parse_cbor"),
            (RuntimeFormat::SoraProtobuf, "parse_protobuf"),
        ] {
            let ir = example_ir();
            let base = temp_dir();

            PythonCodeGenerator
                .generate_with_options(&ir, LanguageCodegenOptions { runtime_format }, &base)
                .unwrap();

            let item = std::fs::read_to_string(base.join("item.py")).unwrap();
            let item_type = std::fs::read_to_string(base.join("item_type.py")).unwrap();
            let action = std::fs::read_to_string(base.join("action.py")).unwrap();
            let runtime = std::fs::read_to_string(base.join("sora_runtime.py")).unwrap();
            let config = std::fs::read_to_string(base.join("sora_config.py")).unwrap();

            assert!(runtime.contains("class SoraValueBundle"));
            assert!(runtime.contains(parse_function));
            assert!(config.contains(&format!("SoraValueBundle.{parse_function}(bytes_data)")));
            assert!(item.contains("def decode_value(value: SoraValue) -> Item:"));
            assert!(item.contains("obj.get(\"id\").as_int()"));
            assert!(item_type.contains("def decode_value(value: SoraValue) -> ItemType:"));
            assert!(action.contains("def decode_value(value: SoraValue) -> Action:"));

            let _ = std::fs::remove_dir_all(base);
        }
    }

    fn example_ir() -> ConfigIr {
        let schema: SchemaFile = toml::from_str(
            r#"
package = "game_config"

[[enums]]
name = "ItemType"
values = ["Weapon", "Armor"]

[[unions]]
name = "Action"

[[unions.variants]]
name = "AddItem"

[[unions.variants.fields]]
name = "item_id"
type = "i32"

[[tables]]
name = "Item"
mode = "map"
key = "id"

[[tables.fields]]
name = "id"
type = "i32"

[[tables.fields]]
name = "name"
type = "string"

[[tables.fields]]
name = "item_type"
type = "enum<ItemType>"

[[tables.fields]]
name = "large_id"
type = "i64"

[[tables.fields]]
name = "action"
type = "union<Action>"

[[tables.indexes]]
name = "by_name"
fields = ["name"]
unique = true

[[tables.indexes]]
name = "by_item_type"
fields = ["item_type"]
"#,
        )
        .unwrap();

        normalize_schema(schema).unwrap()
    }

    fn temp_dir() -> PathBuf {
        let unique = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("sora-python-codegen-test-{unique}"))
    }
}
