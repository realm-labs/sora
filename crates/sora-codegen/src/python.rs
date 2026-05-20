use std::{collections::HashSet, path::Path};

use heck::{ToPascalCase, ToShoutySnakeCase, ToSnakeCase};
use minijinja::context;
use serde::Serialize;
use sora_diagnostics::{Result, SoraError};
use sora_ir::model::{ConfigIr, TableModeIr, TypeIr};

use crate::{
    generator::{CodeGenerator, ensure_sora_runtime_format},
    model::{
        CodegenField, CodegenImport, CodegenIndex, CodegenModel, CodegenRecord, CodegenTable,
        CodegenUnion, CodegenUnionVariant, LanguageBackend, TableNameParts, build_model,
    },
    render::{ensure_dir, render_template, write_file},
};

pub struct PythonCodeGenerator;

impl CodeGenerator for PythonCodeGenerator {
    fn generate(&self, ir: &ConfigIr, out_dir: &Path) -> Result<()> {
        ensure_sora_runtime_format("python", ir.codegen.python.runtime_format)?;
        ensure_dir(out_dir)?;

        let backend = PythonBackend;
        let model = PythonModel::from_codegen_model(build_model(ir, &backend)?);
        validate_python_model(&model)?;

        for item in &model.enums {
            let rendered = render_template("python", "enum.py.j2", context! { enum => item })?;
            write_file(&out_dir.join(format!("{}.py", item.snake_name)), rendered)?;
        }

        for record in &model.records {
            let rendered =
                render_template("python", "record.py.j2", context! { record => record })?;
            write_file(&out_dir.join(format!("{}.py", record.snake_name)), rendered)?;
        }

        for union in &model.unions {
            let rendered = render_template("python", "union.py.j2", context! { union => union })?;
            write_file(&out_dir.join(format!("{}.py", union.snake_name)), rendered)?;
        }

        let rendered = render_template("python", "runtime.py.j2", context! {})?;
        write_file(&out_dir.join("sora_runtime.py"), rendered)?;

        let rendered = render_template("python", "config.py.j2", context! { model => &model })?;
        write_file(&out_dir.join("sora_config.py"), rendered)?;

        let rendered = render_template("python", "__init__.py.j2", context! { model => &model })?;
        write_file(&out_dir.join("__init__.py"), rendered)
    }
}

#[derive(Debug, Clone, Serialize)]
struct PythonModel {
    package: String,
    enums: Vec<PythonEnum>,
    unions: Vec<PythonUnion>,
    records: Vec<PythonRecord>,
    tables: Vec<PythonTable>,
    modules: Vec<String>,
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
}

#[derive(Debug, Clone, Serialize)]
struct PythonRecord {
    pascal_name: String,
    snake_name: String,
    imports: Vec<PythonImport>,
    fields: Vec<PythonField>,
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
    comment: Option<String>,
}

impl PythonModel {
    fn from_codegen_model(model: CodegenModel) -> Self {
        let enums = model
            .enums
            .into_iter()
            .map(|item| PythonEnum {
                name: python_type_identifier(&item.name),
                snake_name: python_module_name(&item.snake_name),
                value_names: item
                    .values
                    .iter()
                    .map(|value| python_enum_value_name(value))
                    .collect(),
                values: item.values,
            })
            .collect::<Vec<_>>();

        let records = model
            .records
            .into_iter()
            .map(python_record)
            .collect::<Vec<_>>();
        let unions = model
            .unions
            .into_iter()
            .map(python_union)
            .collect::<Vec<_>>();
        let tables = model
            .tables
            .into_iter()
            .map(python_table)
            .collect::<Vec<_>>();
        let modules = enums
            .iter()
            .map(|item| item.snake_name.clone())
            .chain(records.iter().map(|item| item.snake_name.clone()))
            .chain(unions.iter().map(|item| item.snake_name.clone()))
            .collect();

        Self {
            package: model.package,
            enums,
            unions,
            records,
            tables,
            modules,
        }
    }
}

fn python_record(record: CodegenRecord) -> PythonRecord {
    PythonRecord {
        pascal_name: python_type_identifier(&record.pascal_name),
        snake_name: python_module_name(&record.snake_name),
        imports: record.imports.into_iter().map(python_import).collect(),
        fields: record.fields.into_iter().map(python_field).collect(),
    }
}

fn python_union(union: CodegenUnion) -> PythonUnion {
    PythonUnion {
        pascal_name: python_type_identifier(&union.pascal_name),
        snake_name: python_module_name(&union.snake_name),
        tag: python_field_identifier(&union.tag),
        variants: union.variants.into_iter().map(python_variant).collect(),
        imports: union.imports.into_iter().map(python_import).collect(),
    }
}

fn python_variant(variant: CodegenUnionVariant) -> PythonUnionVariant {
    PythonUnionVariant {
        raw_name: variant.raw_name,
        name: python_type_identifier(&variant.name),
        fields: variant.fields.into_iter().map(python_field).collect(),
    }
}

fn python_table(table: CodegenTable) -> PythonTable {
    PythonTable {
        name: table.name,
        pascal_name: python_type_identifier(&table.pascal_name),
        snake_name: python_module_name(&table.snake_name),
        mode: table.mode,
        row_type: python_type_identifier(&table.row_type),
        key_name: table.key_name,
        key_field_name: table
            .key_field_name
            .map(|name| python_field_identifier(&name)),
        key_type: table.key_type,
        unique_indexes: table.unique_indexes.into_iter().map(python_index).collect(),
        non_unique_indexes: table
            .non_unique_indexes
            .into_iter()
            .map(python_index)
            .collect(),
    }
}

fn python_index(index: CodegenIndex) -> PythonIndex {
    PythonIndex {
        name: python_field_identifier(&index.name),
        field_name: python_field_identifier(&index.field_name),
        param_name: python_field_identifier(&index.param_name),
        param_type: index.param_type,
        key_type: index.key_type,
    }
}

fn python_field(field: CodegenField) -> PythonField {
    PythonField {
        raw_name: field.raw_name,
        name: python_field_identifier(&field.name),
        type_name: field.type_name,
        decode: field.decode,
        comment: field.comment,
    }
}

fn python_import(import: CodegenImport) -> PythonImport {
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

pub struct PythonBackend;

impl LanguageBackend for PythonBackend {
    fn field_name(&self, raw_name: &str) -> String {
        raw_name.to_snake_case()
    }

    fn type_name(&self, ir: &ConfigIr, ty: &TypeIr) -> String {
        self.python_type_name(ir, ty)
    }

    fn decode_expr(&self, ir: &ConfigIr, ty: &TypeIr) -> String {
        self.python_decode_expr(ir, ty)
    }

    fn row_type(&self, table: &TableNameParts<'_>) -> String {
        table.pascal_name.to_owned()
    }

    fn container_type(
        &self,
        _table: &TableNameParts<'_>,
        mode: TableModeIr,
        row_type: &str,
        key_type: Option<&str>,
    ) -> String {
        match mode {
            TableModeIr::List => format!("list[{}]", row_type),
            TableModeIr::Map => format!("dict[{}, {}]", key_type.unwrap_or("int"), row_type),
            TableModeIr::Singleton => row_type.to_owned(),
        }
    }
}

impl PythonBackend {
    fn python_type_name(&self, ir: &ConfigIr, ty: &TypeIr) -> String {
        match ty {
            TypeIr::Bool => "bool".to_owned(),
            TypeIr::I32 | TypeIr::I64 => "int".to_owned(),
            TypeIr::F32 | TypeIr::F64 => "float".to_owned(),
            TypeIr::String => "str".to_owned(),
            TypeIr::Enum(name) | TypeIr::Struct(name) | TypeIr::Union(name) => {
                python_type_identifier(name)
            }
            TypeIr::List(element) | TypeIr::Array { element, .. } => {
                format!("list[{}]", self.python_type_name(ir, element))
            }
            TypeIr::Ref { table, field } => self
                .ref_target_type(ir, table, field)
                .map(|ty| self.python_type_name(ir, ty))
                .unwrap_or_else(|| "int".to_owned()),
            TypeIr::Optional(element) => {
                format!("{} | None", self.python_type_name(ir, element))
            }
        }
    }

    fn python_decode_expr(&self, ir: &ConfigIr, ty: &TypeIr) -> String {
        match ty {
            TypeIr::Bool => "reader.read_bool()".to_owned(),
            TypeIr::I32 => "reader.read_i32()".to_owned(),
            TypeIr::I64 => "reader.read_i64()".to_owned(),
            TypeIr::F32 => "reader.read_f32()".to_owned(),
            TypeIr::F64 => "reader.read_f64()".to_owned(),
            TypeIr::String => "reader.read_string()".to_owned(),
            TypeIr::Enum(name) | TypeIr::Struct(name) | TypeIr::Union(name) => {
                format!("{}.decode(reader)", python_type_identifier(name))
            }
            TypeIr::List(element) | TypeIr::Array { element, .. } => {
                format!(
                    "reader.read_list(lambda: {})",
                    self.python_decode_expr(ir, element)
                )
            }
            TypeIr::Ref { table, field } => self
                .ref_target_type(ir, table, field)
                .map(|ty| self.python_decode_expr(ir, ty))
                .unwrap_or_else(|| "reader.read_i32()".to_owned()),
            TypeIr::Optional(element) => {
                format!(
                    "reader.read_optional(lambda: {})",
                    self.python_decode_expr(ir, element)
                )
            }
        }
    }

    fn ref_target_type<'a>(
        &self,
        ir: &'a ConfigIr,
        table: &str,
        field: &str,
    ) -> Option<&'a TypeIr> {
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
        assert!(runtime.contains("def read_i64(self) -> int:"));
        assert!(runtime.contains("duplicate map key"));
        assert!(config.contains("class ItemTable"));
        assert!(config.contains("def get(self, key: int) -> Item | None:"));
        assert!(config.contains(") -> Item | None:"));
        assert!(config.contains(") -> list[Item]:"));
        assert!(config.contains("class SoraConfig:"));
        assert!(config.contains("def from_bytes(bytes_data: bytes) -> SoraConfig:"));
        assert!(init.contains("from .sora_config import SoraConfig"));

        let _ = std::fs::remove_dir_all(base);
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
required = true

[[tables.fields]]
name = "name"
type = "string"
required = true

[[tables.fields]]
name = "item_type"
type = "enum<ItemType>"
required = true

[[tables.fields]]
name = "large_id"
type = "i64"
required = true

[[tables.fields]]
name = "action"
type = "union<Action>"
required = true

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
