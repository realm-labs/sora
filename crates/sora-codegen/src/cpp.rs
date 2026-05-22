use std::path::Path;

use heck::ToSnakeCase;
use minijinja::context;
use serde::Serialize;
use sora_diagnostics::{Result, SoraError};
use sora_ir::model::{ConfigIr, CppStandardIr, TableModeIr, TypeIr};

use crate::{
    generator::{CodeGenerator, ensure_sora_runtime_format},
    model::{
        BaseField, BaseImport, BaseIndex, BaseModel, BaseRecord, BaseTable, BaseUnion,
        BaseUnionVariant, build_base_model,
    },
    render::{ensure_dir, render_template, write_file},
};

pub struct CppCodeGenerator;

impl CodeGenerator for CppCodeGenerator {
    fn generate(&self, ir: &ConfigIr, out_dir: &Path) -> Result<()> {
        ensure_sora_runtime_format("cpp", ir.codegen.cpp.runtime_format)?;
        ensure_dir(out_dir)?;

        let options = CppOptionsView::new(ir)?;
        let model = CppModel::from_base_model(ir, build_base_model(ir)?, &options);

        for item in &model.enums {
            let rendered = render_template(
                "cpp",
                "enum.hpp.j2",
                context! { enum => item, options => &options },
            )?;
            write_file(
                &out_dir.join(format!("{}.hpp", item.name.to_snake_case())),
                rendered,
            )?;
        }

        for record in &model.records {
            let rendered = render_template(
                "cpp",
                "record.hpp.j2",
                context! { record => record, options => &options },
            )?;
            write_file(
                &out_dir.join(format!("{}.hpp", record.snake_name)),
                rendered,
            )?;
        }

        for union in &model.unions {
            let rendered = render_template(
                "cpp",
                "union.hpp.j2",
                context! { union => union, options => &options },
            )?;
            write_file(&out_dir.join(format!("{}.hpp", union.snake_name)), rendered)?;
        }

        let rendered = render_template("cpp", "runtime.hpp.j2", context! { options => &options })?;
        write_file(&out_dir.join("sora_runtime.hpp"), rendered)?;

        let rendered = render_template(
            "cpp",
            "config.hpp.j2",
            context! { model => &model, options => &options },
        )?;
        write_file(&out_dir.join("sora_config.hpp"), rendered)
    }
}

#[derive(Debug, Clone, Serialize)]
struct CppOptionsView {
    standard_name: &'static str,
    namespace_name: String,
    namespace_open: String,
    namespace_close: String,
    has_std_optional: bool,
    has_std_variant: bool,
}

impl CppOptionsView {
    fn new(ir: &ConfigIr) -> Result<Self> {
        let namespace = ir
            .codegen
            .cpp
            .namespace
            .clone()
            .unwrap_or_else(|| ir.package.replace('.', "::"));
        let namespace_segments = parse_cpp_namespace(&namespace)?;
        let standard = ir.codegen.cpp.cpp_standard;
        Ok(Self {
            standard_name: cpp_standard_name(standard),
            namespace_name: namespace_segments.join("::"),
            namespace_open: namespace_open(&namespace_segments, standard),
            namespace_close: namespace_close(&namespace_segments, standard),
            has_std_optional: matches!(
                standard,
                CppStandardIr::Cpp17 | CppStandardIr::Cpp20 | CppStandardIr::Cpp23
            ),
            has_std_variant: matches!(
                standard,
                CppStandardIr::Cpp17 | CppStandardIr::Cpp20 | CppStandardIr::Cpp23
            ),
        })
    }
}

#[derive(Debug, Clone, Serialize)]
struct CppModel {
    schema_fingerprint: String,
    enums: Vec<CppEnum>,
    unions: Vec<CppUnion>,
    records: Vec<CppRecord>,
    tables: Vec<CppTable>,
    modules: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct CppEnum {
    name: String,
    snake_name: String,
    values: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct CppUnion {
    pascal_name: String,
    snake_name: String,
    tag: String,
    variants: Vec<CppUnionVariant>,
    imports: Vec<CppImport>,
}

#[derive(Debug, Clone, Serialize)]
struct CppUnionVariant {
    name: String,
    method_name: String,
    fields: Vec<CppField>,
}

#[derive(Debug, Clone, Serialize)]
struct CppRecord {
    pascal_name: String,
    snake_name: String,
    imports: Vec<CppImport>,
    fields: Vec<CppField>,
    table: Option<CppTable>,
}

#[derive(Debug, Clone, Serialize)]
struct CppImport {
    module: String,
}

#[derive(Debug, Clone, Serialize)]
struct CppTable {
    name: String,
    pascal_name: String,
    snake_name: String,
    mode: String,
    container_type: String,
    row_type: String,
    key_name: Option<String>,
    key_field_name: Option<String>,
    key_type: Option<String>,
    unique_indexes: Vec<CppIndex>,
    non_unique_indexes: Vec<CppIndex>,
}

#[derive(Debug, Clone, Serialize)]
struct CppIndex {
    name: String,
    method_name: String,
    field_name: String,
    param_name: String,
    key_type: String,
}

#[derive(Debug, Clone, Serialize)]
struct CppField {
    raw_name: String,
    name: String,
    type_name: String,
    decode: String,
    comment: Option<String>,
}

impl CppModel {
    fn from_base_model(ir: &ConfigIr, model: BaseModel, options: &CppOptionsView) -> Self {
        let enums = model
            .enums
            .into_iter()
            .map(|item| CppEnum {
                name: item.pascal_name,
                snake_name: item.snake_name,
                values: item.values,
            })
            .collect();
        let tables = model
            .tables
            .into_iter()
            .map(|item| cpp_table(ir, item, options))
            .collect::<Vec<_>>();
        let records = model
            .records
            .into_iter()
            .map(|item| {
                let table = tables
                    .iter()
                    .find(|table| table.row_type == item.pascal_name)
                    .cloned();
                cpp_record(ir, item, options, table)
            })
            .collect();
        let unions = model
            .unions
            .into_iter()
            .map(|item| cpp_union(ir, item, options))
            .collect();

        Self {
            schema_fingerprint: model.schema_fingerprint,
            enums,
            unions,
            records,
            tables,
            modules: model.modules,
        }
    }
}

fn cpp_union(ir: &ConfigIr, union: BaseUnion, options: &CppOptionsView) -> CppUnion {
    CppUnion {
        pascal_name: union.pascal_name,
        snake_name: union.snake_name,
        tag: union.tag,
        variants: union
            .variants
            .into_iter()
            .map(|variant| cpp_variant(ir, variant, options))
            .collect(),
        imports: union.imports.into_iter().map(cpp_import).collect(),
    }
}

fn cpp_variant(
    ir: &ConfigIr,
    variant: BaseUnionVariant,
    options: &CppOptionsView,
) -> CppUnionVariant {
    CppUnionVariant {
        name: variant.pascal_name,
        method_name: variant.snake_name,
        fields: variant
            .fields
            .into_iter()
            .map(|field| cpp_field(ir, field, options))
            .collect(),
    }
}

fn cpp_record(
    ir: &ConfigIr,
    record: BaseRecord,
    options: &CppOptionsView,
    table: Option<CppTable>,
) -> CppRecord {
    CppRecord {
        pascal_name: record.pascal_name,
        snake_name: record.snake_name,
        imports: record.imports.into_iter().map(cpp_import).collect(),
        fields: record
            .fields
            .into_iter()
            .map(|field| cpp_field(ir, field, options))
            .collect(),
        table,
    }
}

fn cpp_table(ir: &ConfigIr, table: BaseTable, options: &CppOptionsView) -> CppTable {
    let row_type = table.pascal_name.clone();
    let key_type = table
        .key_field
        .as_ref()
        .map(|field| cpp_type_name(ir, &field.ty, options));
    let container_type = cpp_container_type(table.mode, &row_type, key_type.as_deref());
    let key_field_name = table
        .key_field
        .as_ref()
        .map(|field| field.snake_name.clone());

    CppTable {
        name: table.name,
        pascal_name: table.pascal_name,
        snake_name: table.snake_name,
        mode: table.mode_name,
        container_type,
        row_type,
        key_name: table.key_name,
        key_field_name,
        key_type,
        unique_indexes: table
            .unique_indexes
            .into_iter()
            .map(|index| cpp_index(ir, index, options))
            .collect(),
        non_unique_indexes: table
            .non_unique_indexes
            .into_iter()
            .map(|index| cpp_index(ir, index, options))
            .collect(),
    }
}

fn cpp_index(ir: &ConfigIr, index: BaseIndex, options: &CppOptionsView) -> CppIndex {
    CppIndex {
        name: index.snake_name,
        method_name: index.method_name,
        field_name: index.field.snake_name.clone(),
        param_name: index.field.snake_name.clone(),
        key_type: cpp_type_name(ir, &index.field.ty, options),
    }
}

fn cpp_field(ir: &ConfigIr, field: BaseField, options: &CppOptionsView) -> CppField {
    CppField {
        raw_name: field.raw_name,
        name: field.snake_name,
        type_name: cpp_type_name(ir, &field.ty, options),
        decode: cpp_decode_expr(ir, &field.ty, options),
        comment: field.comment,
    }
}

fn cpp_import(import: BaseImport) -> CppImport {
    CppImport {
        module: import.module,
    }
}

fn cpp_container_type(mode: TableModeIr, row_type: &str, key_type: Option<&str>) -> String {
    match mode {
        TableModeIr::List => format!("std::vector<{row_type}>"),
        TableModeIr::Map => match key_type {
            Some(key_type) => format!("std::unordered_map<{key_type}, {row_type}>"),
            None => format!("std::vector<{row_type}>"),
        },
        TableModeIr::Singleton => row_type.to_owned(),
    }
}

fn cpp_type_name(ir: &ConfigIr, ty: &TypeIr, options: &CppOptionsView) -> String {
    match ty {
        TypeIr::Bool => "bool".to_owned(),
        TypeIr::I32 => "std::int32_t".to_owned(),
        TypeIr::I64 => "std::int64_t".to_owned(),
        TypeIr::F32 => "float".to_owned(),
        TypeIr::F64 => "double".to_owned(),
        TypeIr::String => "std::string".to_owned(),
        TypeIr::Enum(name) | TypeIr::Struct(name) | TypeIr::Union(name) => name.clone(),
        TypeIr::List(element) | TypeIr::Set(element) => {
            format!("std::vector<{}>", cpp_type_name(ir, element, options))
        }
        TypeIr::Map { key, value } => format!(
            "std::unordered_map<{}, {}>",
            cpp_type_name(ir, key, options),
            cpp_type_name(ir, value, options)
        ),
        TypeIr::Array { element, len } => {
            format!("std::array<{}, {len}>", cpp_type_name(ir, element, options))
        }
        TypeIr::Ref { table, field } => ir
            .tables
            .iter()
            .find(|candidate| candidate.name == *table)
            .and_then(|table| {
                table
                    .fields
                    .iter()
                    .find(|candidate| candidate.name == *field)
            })
            .map(|field| cpp_type_name(ir, &field.ty, options))
            .unwrap_or_else(|| "std::int32_t".to_owned()),
        TypeIr::Optional(element) => {
            let inner = cpp_type_name(ir, element, options);
            if options.has_std_optional {
                format!("std::optional<{inner}>")
            } else {
                format!("SoraOptional<{inner}>")
            }
        }
    }
}

fn cpp_decode_expr(ir: &ConfigIr, ty: &TypeIr, options: &CppOptionsView) -> String {
    match ty {
        TypeIr::Bool => "reader.read_bool()".to_owned(),
        TypeIr::I32 => "reader.read_i32()".to_owned(),
        TypeIr::I64 => "reader.read_i64()".to_owned(),
        TypeIr::F32 => "reader.read_f32()".to_owned(),
        TypeIr::F64 => "reader.read_f64()".to_owned(),
        TypeIr::String => "reader.read_string()".to_owned(),
        TypeIr::Enum(name) => format!("decode_value<{name}>(reader)"),
        TypeIr::Struct(name) | TypeIr::Union(name) => format!("{name}::decode(reader)"),
        TypeIr::List(element) | TypeIr::Set(element) => {
            format!(
                "reader.read_vector<{}>()",
                cpp_type_name(ir, element, options)
            )
        }
        TypeIr::Map { key, value } => format!(
            "reader.read_map<{}, {}>()",
            cpp_type_name(ir, key, options),
            cpp_type_name(ir, value, options)
        ),
        TypeIr::Array { element, len } => {
            format!(
                "reader.read_array<{}, {len}>()",
                cpp_type_name(ir, element, options)
            )
        }
        TypeIr::Ref { table, field } => ir
            .tables
            .iter()
            .find(|candidate| candidate.name == *table)
            .and_then(|table| {
                table
                    .fields
                    .iter()
                    .find(|candidate| candidate.name == *field)
            })
            .map(|field| cpp_decode_expr(ir, &field.ty, options))
            .unwrap_or_else(|| "reader.read_i32()".to_owned()),
        TypeIr::Optional(element) => {
            format!(
                "reader.read_optional<{}>()",
                cpp_type_name(ir, element, options)
            )
        }
    }
}

fn cpp_standard_name(standard: CppStandardIr) -> &'static str {
    match standard {
        CppStandardIr::Cpp11 => "c++11",
        CppStandardIr::Cpp14 => "c++14",
        CppStandardIr::Cpp17 => "c++17",
        CppStandardIr::Cpp20 => "c++20",
        CppStandardIr::Cpp23 => "c++23",
    }
}

fn parse_cpp_namespace(namespace: &str) -> Result<Vec<String>> {
    let segments = namespace.split("::").collect::<Vec<_>>();
    if segments.is_empty() || segments.iter().any(|segment| !is_cpp_identifier(segment)) {
        return Err(SoraError::InvalidSchema(format!(
            "cpp namespace `{namespace}` must use `::` separated C++ identifiers"
        )));
    }
    Ok(segments.into_iter().map(str::to_owned).collect())
}

fn is_cpp_identifier(segment: &str) -> bool {
    let mut chars = segment.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || first == '_')
        && chars.all(|value| value.is_ascii_alphanumeric() || value == '_')
}

fn namespace_open(segments: &[String], standard: CppStandardIr) -> String {
    if matches!(
        standard,
        CppStandardIr::Cpp17 | CppStandardIr::Cpp20 | CppStandardIr::Cpp23
    ) {
        format!("namespace {} {{", segments.join("::"))
    } else {
        segments
            .iter()
            .map(|segment| format!("namespace {segment} {{"))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

fn namespace_close(segments: &[String], standard: CppStandardIr) -> String {
    let close_count = if matches!(
        standard,
        CppStandardIr::Cpp17 | CppStandardIr::Cpp20 | CppStandardIr::Cpp23
    ) {
        1
    } else {
        segments.len()
    };
    let closes = "}".repeat(close_count);
    format!("{closes} // namespace {}", segments.join("::"))
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
    fn generates_cpp_files() {
        let ir = example_ir();
        let base = temp_dir();

        CppCodeGenerator.generate(&ir, &base).unwrap();

        let item = std::fs::read_to_string(base.join("item.hpp")).unwrap();
        let action = std::fs::read_to_string(base.join("action.hpp")).unwrap();
        let config = std::fs::read_to_string(base.join("sora_config.hpp")).unwrap();
        let runtime = std::fs::read_to_string(base.join("sora_runtime.hpp")).unwrap();

        assert!(item.contains("struct Item"));
        assert!(item.contains("ItemType item_type;"));
        assert!(item.contains("Action action;"));
        assert!(item.contains("static Item decode(SoraReader& reader)"));
        assert!(action.contains("std::variant<"));
        assert!(action.contains("AddItem"));
        assert!(config.contains("class SoraConfig"));
        assert!(item.contains("class ItemTable final : public SoraTable"));
        assert!(item.contains("std::unordered_map<std::int32_t, Item> rows_;"));
        assert!(item.contains("const Item* get(const std::int32_t& key) const"));
        assert!(config.contains("const ItemTable& item() const"));
        assert!(!config.contains("std::unordered_map<std::int32_t, Item> item_;"));
        assert!(runtime.contains("template <typename T>"));
        assert!(runtime.contains("class SoraTable"));
        assert!(runtime.contains("std::optional<T> read_optional()"));

        let _ = std::fs::remove_dir_all(base);
    }

    #[test]
    fn cpp11_uses_fallback_optional_and_union_storage() {
        let mut ir = example_ir();
        ir.codegen.cpp.cpp_standard = CppStandardIr::Cpp11;
        let base = temp_dir();

        CppCodeGenerator.generate(&ir, &base).unwrap();

        let action = std::fs::read_to_string(base.join("action.hpp")).unwrap();
        let runtime = std::fs::read_to_string(base.join("sora_runtime.hpp")).unwrap();

        assert!(action.contains("std::shared_ptr<Holder> value_;"));
        assert!(!action.contains("std::variant<"));
        assert!(runtime.contains("class SoraOptional"));
        assert!(runtime.contains("SoraOptional<T> read_optional()"));

        let _ = std::fs::remove_dir_all(base);
    }

    fn example_ir() -> ConfigIr {
        let schema: SchemaFile = toml::from_str(
            r#"
package = "game.config"

[codegen.cpp]
namespace = "sora::game_config"

[[enums]]
name = "ItemType"
values = ["Weapon", "Armor"]

[[unions]]
name = "Action"
tag = "kind"

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
key = true

[[tables.fields]]
name = "name"
type = "string"

[[tables.fields]]
name = "item_type"
type = "enum<ItemType>"

[[tables.fields]]
name = "action"
type = "union<Action>"

[[tables.fields]]
name = "tags"
type = "list<string>"
parser = { kind = "split", separator = "|" }

[[tables.fields]]
name = "maybe_count"
type = "optional<i32>"

[[tables.indexes]]
name = "by_name"
fields = ["name"]
unique = true
"#,
        )
        .unwrap();
        normalize_schema(schema).unwrap()
    }

    fn temp_dir() -> PathBuf {
        let unique = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("sora-cpp-codegen-test-{unique}"))
    }
}
