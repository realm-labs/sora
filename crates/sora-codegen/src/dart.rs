use std::path::Path;

use heck::{ToLowerCamelCase, ToPascalCase, ToSnakeCase};
use minijinja::context;
use serde::Serialize;
use sora_diagnostics::{Result, SoraError};
use sora_ir::model::{ConfigIr, RuntimeFormatIr, TypeIr};

use crate::{
    generator::CodeGenerator,
    model::{
        BaseField, BaseImport, BaseIndex, BaseModel, BaseRecord, BaseTable, BaseUnion,
        BaseUnionVariant, build_base_model,
    },
    render::{ensure_dir, render_template, write_file},
    types::dart_type_name,
};

pub struct DartCodeGenerator;

impl CodeGenerator for DartCodeGenerator {
    fn generate(&self, ir: &ConfigIr, out_dir: &Path) -> Result<()> {
        if ir.codegen.dart.runtime_format != RuntimeFormatIr::Json {
            return Err(SoraError::InvalidSchema(format!(
                "dart codegen runtime_format `{}` is not implemented yet; supported runtime_format: json",
                crate::generator::runtime_format_name(ir.codegen.dart.runtime_format)
            )));
        }

        ensure_dir(out_dir)?;
        let model = DartModel::from_base_model(ir, build_base_model(ir)?);

        for item in &model.enums {
            let rendered = render_template("dart", "enum.dart.j2", context! { enum => item })?;
            write_file(&out_dir.join(format!("{}.dart", item.snake_name)), rendered)?;
        }

        for record in &model.records {
            let rendered =
                render_template("dart", "record.dart.j2", context! { record => record })?;
            write_file(
                &out_dir.join(format!("{}.dart", record.snake_name)),
                rendered,
            )?;
        }

        for union in &model.unions {
            let rendered = render_template("dart", "union.dart.j2", context! { union => union })?;
            write_file(
                &out_dir.join(format!("{}.dart", union.snake_name)),
                rendered,
            )?;
        }

        let rendered = render_template("dart", "runtime.dart.j2", context! {})?;
        write_file(&out_dir.join("runtime.dart"), rendered)?;

        let rendered = render_template("dart", "config.dart.j2", context! { model => &model })?;
        write_file(&out_dir.join("sora_config.dart"), rendered)?;

        let rendered = render_template("dart", "library.dart.j2", context! { model => &model })?;
        write_file(&out_dir.join("generated.dart"), rendered)
    }
}

#[derive(Debug, Clone, Serialize)]
struct DartModel {
    schema_fingerprint: String,
    enums: Vec<DartEnum>,
    unions: Vec<DartUnion>,
    records: Vec<DartRecord>,
    tables: Vec<DartTable>,
    modules: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct DartEnum {
    pascal_name: String,
    snake_name: String,
    values: Vec<DartEnumValue>,
}

#[derive(Debug, Clone, Serialize)]
struct DartEnumValue {
    raw_name: String,
    name: String,
}

#[derive(Debug, Clone, Serialize)]
struct DartUnion {
    pascal_name: String,
    snake_name: String,
    tag: String,
    variants: Vec<DartUnionVariant>,
    imports: Vec<DartImport>,
}

#[derive(Debug, Clone, Serialize)]
struct DartUnionVariant {
    raw_name: String,
    class_name: String,
    fields: Vec<DartField>,
}

#[derive(Debug, Clone, Serialize)]
struct DartRecord {
    pascal_name: String,
    snake_name: String,
    imports: Vec<DartImport>,
    fields: Vec<DartField>,
    table: Option<DartTable>,
}

#[derive(Debug, Clone, Serialize)]
struct DartImport {
    module: String,
    name: String,
}

#[derive(Debug, Clone, Serialize)]
struct DartField {
    raw_name: String,
    name: String,
    type_name: String,
    value_decode: String,
    comment: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct DartTable {
    name: String,
    pascal_name: String,
    camel_name: String,
    snake_name: String,
    mode: String,
    row_type: String,
    key_name: Option<String>,
    key_field_name: Option<String>,
    key_type: Option<String>,
    unique_indexes: Vec<DartIndex>,
    non_unique_indexes: Vec<DartIndex>,
}

#[derive(Debug, Clone, Serialize)]
struct DartIndex {
    pascal_name: String,
    field_name: String,
    param_name: String,
    param_type: String,
    key_type: String,
}

impl DartModel {
    fn from_base_model(ir: &ConfigIr, model: BaseModel) -> Self {
        let enums = model
            .enums
            .into_iter()
            .map(|item| DartEnum {
                pascal_name: dart_type_identifier(&item.pascal_name),
                snake_name: dart_module_name(&item.snake_name),
                values: item
                    .values
                    .into_iter()
                    .map(|value| DartEnumValue {
                        name: dart_enum_value_identifier(&value),
                        raw_name: value,
                    })
                    .collect(),
            })
            .collect::<Vec<_>>();

        let unions = model
            .unions
            .into_iter()
            .map(|item| dart_union(ir, item))
            .collect::<Vec<_>>();
        let tables = model
            .tables
            .into_iter()
            .map(|item| dart_table(ir, item))
            .collect::<Vec<_>>();
        let records = model
            .records
            .into_iter()
            .map(|item| {
                let table = tables
                    .iter()
                    .find(|table| table.row_type == dart_type_identifier(&item.pascal_name))
                    .cloned();
                dart_record(ir, item, table)
            })
            .collect::<Vec<_>>();
        let modules = enums
            .iter()
            .map(|item| item.snake_name.clone())
            .chain(records.iter().map(|item| item.snake_name.clone()))
            .chain(unions.iter().map(|item| item.snake_name.clone()))
            .collect();

        Self {
            schema_fingerprint: model.schema_fingerprint,
            enums,
            unions,
            records,
            tables,
            modules,
        }
    }
}

fn dart_record(ir: &ConfigIr, record: BaseRecord, table: Option<DartTable>) -> DartRecord {
    DartRecord {
        pascal_name: dart_type_identifier(&record.pascal_name),
        snake_name: dart_module_name(&record.snake_name),
        imports: record.imports.into_iter().map(dart_import).collect(),
        fields: record
            .fields
            .into_iter()
            .map(|field| dart_field(ir, field))
            .collect(),
        table,
    }
}

fn dart_union(ir: &ConfigIr, union: BaseUnion) -> DartUnion {
    DartUnion {
        pascal_name: dart_type_identifier(&union.pascal_name),
        snake_name: dart_module_name(&union.snake_name),
        tag: union.tag,
        variants: union
            .variants
            .into_iter()
            .map(|variant| dart_variant(ir, &union.name, variant))
            .collect(),
        imports: union.imports.into_iter().map(dart_import).collect(),
    }
}

fn dart_variant(ir: &ConfigIr, union_name: &str, variant: BaseUnionVariant) -> DartUnionVariant {
    DartUnionVariant {
        raw_name: variant.name,
        class_name: dart_type_identifier(&format!("{union_name}{}", variant.pascal_name)),
        fields: variant
            .fields
            .into_iter()
            .map(|field| dart_field(ir, field))
            .collect(),
    }
}

fn dart_table(ir: &ConfigIr, table: BaseTable) -> DartTable {
    let key_field_name = table
        .key_field
        .as_ref()
        .map(|field| dart_field_identifier(&field.camel_name));
    let key_type = table
        .key_field
        .as_ref()
        .map(|field| dart_type_name(ir, &field.ty));

    DartTable {
        name: table.name,
        pascal_name: dart_type_identifier(&table.pascal_name),
        camel_name: dart_field_identifier(&table.camel_name),
        snake_name: dart_module_name(&table.snake_name),
        mode: table.mode_name,
        row_type: dart_type_identifier(&table.pascal_name),
        key_name: table.key_name,
        key_field_name,
        key_type,
        unique_indexes: table
            .unique_indexes
            .into_iter()
            .map(|index| dart_index(ir, index))
            .collect(),
        non_unique_indexes: table
            .non_unique_indexes
            .into_iter()
            .map(|index| dart_index(ir, index))
            .collect(),
    }
}

fn dart_index(ir: &ConfigIr, index: BaseIndex) -> DartIndex {
    let field_name = dart_field_identifier(&index.field.camel_name);
    DartIndex {
        pascal_name: dart_type_identifier(&index.pascal_name),
        param_name: field_name.clone(),
        field_name,
        param_type: dart_type_name(ir, &index.field.ty),
        key_type: dart_type_name(ir, &index.field.ty),
    }
}

fn dart_field(ir: &ConfigIr, field: BaseField) -> DartField {
    DartField {
        raw_name: field.raw_name,
        name: dart_field_identifier(&field.camel_name),
        type_name: dart_type_name(ir, &field.ty),
        value_decode: dart_value_decode_expr(ir, &field.ty, "__VALUE__"),
        comment: field.comment,
    }
}

fn dart_import(import: BaseImport) -> DartImport {
    DartImport {
        module: dart_module_name(&import.module),
        name: dart_type_identifier(&import.name),
    }
}

fn dart_value_decode_expr(ir: &ConfigIr, ty: &TypeIr, value: &str) -> String {
    match ty {
        TypeIr::Bool => format!("{value}.asBool()"),
        TypeIr::I32 | TypeIr::I64 => format!("{value}.asInt()"),
        TypeIr::F32 | TypeIr::F64 => format!("{value}.asDouble()"),
        TypeIr::String => format!("{value}.asString()"),
        TypeIr::Enum(name) | TypeIr::Struct(name) | TypeIr::Union(name) => {
            format!("{}.decode({value})", dart_type_identifier(name))
        }
        TypeIr::List(element) | TypeIr::Array { element, .. } => {
            format!(
                "{value}.asList((item) => {})",
                dart_value_decode_expr(ir, element, "item")
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
            .map(|field| dart_value_decode_expr(ir, &field.ty, value))
            .unwrap_or_else(|| format!("{value}.asInt()")),
        TypeIr::Optional(element) => {
            format!(
                "{value}.isNull ? null : {}",
                dart_value_decode_expr(ir, element, value)
            )
        }
    }
}

fn dart_module_name(value: &str) -> String {
    sanitize_identifier(&value.to_snake_case(), CaseKind::Snake)
}

fn dart_type_identifier(value: &str) -> String {
    sanitize_identifier(&value.to_pascal_case(), CaseKind::Pascal)
}

fn dart_field_identifier(value: &str) -> String {
    sanitize_identifier(&value.to_lower_camel_case(), CaseKind::Camel)
}

fn dart_enum_value_identifier(value: &str) -> String {
    dart_field_identifier(value)
}

#[derive(Clone, Copy)]
enum CaseKind {
    Snake,
    Pascal,
    Camel,
}

fn sanitize_identifier(value: &str, case: CaseKind) -> String {
    let mut out = String::with_capacity(value.len());
    for (index, ch) in value.chars().enumerate() {
        let valid = ch == '_' || ch == '$' || ch.is_ascii_alphanumeric();
        let ch = if valid { ch } else { '_' };
        if index == 0 && ch.is_ascii_digit() {
            out.push(match case {
                CaseKind::Pascal => 'T',
                CaseKind::Snake | CaseKind::Camel => 'v',
            });
        }
        out.push(ch);
    }
    if out.is_empty() || out == "_" {
        out = match case {
            CaseKind::Pascal => "Value".to_owned(),
            CaseKind::Snake | CaseKind::Camel => "value".to_owned(),
        };
    }
    if is_dart_keyword(&out) {
        out.push('_');
    }
    out
}

fn is_dart_keyword(value: &str) -> bool {
    matches!(
        value,
        "abstract"
            | "as"
            | "assert"
            | "async"
            | "await"
            | "base"
            | "break"
            | "case"
            | "catch"
            | "class"
            | "const"
            | "continue"
            | "covariant"
            | "default"
            | "deferred"
            | "do"
            | "dynamic"
            | "else"
            | "enum"
            | "export"
            | "extends"
            | "extension"
            | "external"
            | "factory"
            | "false"
            | "final"
            | "finally"
            | "for"
            | "Function"
            | "get"
            | "hide"
            | "if"
            | "implements"
            | "import"
            | "in"
            | "interface"
            | "is"
            | "late"
            | "library"
            | "mixin"
            | "new"
            | "null"
            | "on"
            | "operator"
            | "part"
            | "required"
            | "rethrow"
            | "return"
            | "sealed"
            | "set"
            | "show"
            | "static"
            | "super"
            | "switch"
            | "sync"
            | "this"
            | "throw"
            | "true"
            | "try"
            | "typedef"
            | "var"
            | "void"
            | "when"
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

    #[test]
    fn generates_dart_json_runtime_files() {
        let schema: SchemaFile = toml::from_str(
            r#"
package = "game_config"

[codegen.dart]
runtime_format = "json"

[[enums]]
name = "ItemType"
values = ["Weapon", "Armor"]

[[structs]]
name = "Cost"

[[structs.fields]]
name = "item_id"
type = "i32"
required = true

[[unions]]
name = "Action"

[[unions.variants]]
name = "AddItem"

[[unions.variants.fields]]
name = "item_id"
type = "i32"
required = true

[[tables]]
name = "Item"
mode = "map"
key = "id"

[[tables.fields]]
name = "id"
type = "i32"
required = true
comment = "Item id"

[[tables.fields]]
name = "item_type"
type = "enum<ItemType>"
required = true

[[tables.fields]]
name = "tags"
type = "list<string>"

[[tables.fields]]
name = "action"
type = "union<Action>"
"#,
        )
        .unwrap();
        let ir = normalize_schema(schema).unwrap();
        let base = std::env::temp_dir().join("sora-codegen-dart-test");
        let _ = std::fs::remove_dir_all(&base);

        DartCodeGenerator.generate(&ir, &base).unwrap();

        let item = std::fs::read_to_string(base.join("item.dart")).unwrap();
        let action = std::fs::read_to_string(base.join("action.dart")).unwrap();
        let config = std::fs::read_to_string(base.join("sora_config.dart")).unwrap();
        let runtime = std::fs::read_to_string(base.join("runtime.dart")).unwrap();

        assert!(item.starts_with("// This file was generated by Sora. Do not edit manually."));
        assert!(item.contains("final class Item"));
        assert!(item.contains("/// Item id"));
        assert!(item.contains("itemType: ItemType.decode(obj.get(\"item_type\"))"));
        assert!(action.contains("sealed class Action"));
        assert!(action.contains("final class ActionAddItem extends Action"));
        assert!(item.contains("final class ItemTable extends Iterable<Item>"));
        assert!(!config.contains("final class ItemTable extends Iterable<Item>"));
        assert!(runtime.contains("SoraValueBundle parseJson"));
        assert!(runtime.contains("abstract interface class SoraConfigTable"));

        let _ = std::fs::remove_dir_all(base);
    }
}
