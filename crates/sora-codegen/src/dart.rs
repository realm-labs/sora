use std::path::Path;

use heck::{ToLowerCamelCase, ToPascalCase, ToSnakeCase};
use minijinja::context;
use serde::Serialize;
use sora_diagnostics::Result;
use sora_ir::model::{ConfigIr, TypeIr};

use crate::{
    generator::{CodeGenerator, CodegenContext, runtime_format_name},
    model::{
        BaseField, BaseImport, BaseIndex, BaseModel, BaseRecord, BaseTable, BaseUnion,
        BaseUnionVariant, build_base_model,
    },
    options::LanguageCodegenOptions,
    render::{ensure_dir, render_template, write_file},
    type_mapping::{TypeMapping, TypeMappingContext, TypeMappingRegistry},
};

pub struct DartCodeGenerator;
crate::impl_test_codegen_generate!(DartCodeGenerator, "dart");

impl CodeGenerator for DartCodeGenerator {
    fn generate(&self, context: CodegenContext<'_>, out_dir: &Path) -> Result<()> {
        let ir = context.ir;
        let options = context.options::<LanguageCodegenOptions>()?;
        let runtime_format = runtime_format_name(options.runtime_format);

        ensure_dir(out_dir)?;
        let mapper = DartTypeMapper::new(context.target, ir, context.type_mappings);
        let model = DartModel::from_base_model(ir, build_base_model(ir)?, &mapper);

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

        let rendered = render_template(
            "dart",
            "runtime.dart.j2",
            context! { runtime_format => runtime_format },
        )?;
        write_file(&out_dir.join("runtime.dart"), rendered)?;

        let rendered = render_template(
            "dart",
            "config.dart.j2",
            context! { model => &model, runtime_format => runtime_format },
        )?;
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
    has_localization: bool,
    locales: Vec<String>,
    default_locale: String,
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
    custom_imports: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct DartUnionVariant {
    raw_name: String,
    class_name: String,
    fields: Vec<DartField>,
    has_text_keys: bool,
}

#[derive(Debug, Clone, Serialize)]
struct DartRecord {
    pascal_name: String,
    snake_name: String,
    imports: Vec<DartImport>,
    custom_imports: Vec<String>,
    fields: Vec<DartField>,
    has_text_keys: bool,
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
    collect_text_keys: String,
    imports: Vec<String>,
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
    fn from_base_model(ir: &ConfigIr, model: BaseModel, mapper: &DartTypeMapper<'_>) -> Self {
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
            .map(|item| dart_union(ir, item, mapper))
            .collect::<Vec<_>>();
        let tables = model
            .tables
            .into_iter()
            .map(|item| dart_table(ir, item, mapper))
            .collect::<Vec<_>>();
        let records = model
            .records
            .into_iter()
            .map(|item| {
                let table = tables
                    .iter()
                    .find(|table| table.row_type == dart_type_identifier(&item.pascal_name))
                    .cloned();
                dart_record(ir, item, table, mapper)
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

fn dart_record(
    ir: &ConfigIr,
    record: BaseRecord,
    table: Option<DartTable>,
    mapper: &DartTypeMapper<'_>,
) -> DartRecord {
    let fields = record
        .fields
        .into_iter()
        .map(|field| dart_field(ir, field, mapper))
        .collect::<Vec<_>>();
    let has_text_keys = fields
        .iter()
        .any(|field| !field.collect_text_keys.is_empty());
    DartRecord {
        pascal_name: dart_type_identifier(&record.pascal_name),
        snake_name: dart_module_name(&record.snake_name),
        imports: record.imports.into_iter().map(dart_import).collect(),
        custom_imports: collect_dart_imports(fields.iter()),
        fields,
        has_text_keys,
        table,
    }
}

fn dart_union(ir: &ConfigIr, union: BaseUnion, mapper: &DartTypeMapper<'_>) -> DartUnion {
    let variants = union
        .variants
        .into_iter()
        .map(|variant| dart_variant(ir, &union.name, variant, mapper))
        .collect::<Vec<_>>();
    let custom_imports = collect_dart_imports(variants.iter().flat_map(|variant| &variant.fields));
    DartUnion {
        pascal_name: dart_type_identifier(&union.pascal_name),
        snake_name: dart_module_name(&union.snake_name),
        tag: union.tag,
        variants,
        imports: union.imports.into_iter().map(dart_import).collect(),
        custom_imports,
    }
}

fn dart_variant(
    ir: &ConfigIr,
    union_name: &str,
    variant: BaseUnionVariant,
    mapper: &DartTypeMapper<'_>,
) -> DartUnionVariant {
    let fields = variant
        .fields
        .into_iter()
        .map(|field| dart_field(ir, field, mapper))
        .collect::<Vec<_>>();
    let has_text_keys = fields
        .iter()
        .any(|field| !field.collect_text_keys.is_empty());
    DartUnionVariant {
        raw_name: variant.name,
        class_name: dart_type_identifier(&format!("{union_name}{}", variant.pascal_name)),
        fields,
        has_text_keys,
    }
}

fn dart_table(ir: &ConfigIr, table: BaseTable, mapper: &DartTypeMapper<'_>) -> DartTable {
    let key_field_name = table
        .key_field
        .as_ref()
        .map(|field| dart_field_identifier(&field.camel_name));
    let key_type = table
        .key_field
        .as_ref()
        .map(|field| mapper.type_name(&field.ty));

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
            .map(|index| dart_index(ir, index, mapper))
            .collect(),
        non_unique_indexes: table
            .non_unique_indexes
            .into_iter()
            .map(|index| dart_index(ir, index, mapper))
            .collect(),
    }
}

fn dart_index(_ir: &ConfigIr, index: BaseIndex, mapper: &DartTypeMapper<'_>) -> DartIndex {
    let field_name = dart_field_identifier(&index.field.camel_name);
    DartIndex {
        pascal_name: dart_type_identifier(&index.pascal_name),
        param_name: field_name.clone(),
        field_name,
        param_type: mapper.type_name(&index.field.ty),
        key_type: mapper.type_name(&index.field.ty),
    }
}

fn dart_field(ir: &ConfigIr, field: BaseField, mapper: &DartTypeMapper<'_>) -> DartField {
    let collect_text_keys = dart_collect_text_keys(
        ir,
        &field.ty,
        &format!("this.{}", field.camel_name),
        4,
        mapper,
    );
    DartField {
        raw_name: field.raw_name,
        name: dart_field_identifier(&field.camel_name),
        type_name: mapper.type_name(&field.ty),
        value_decode: dart_value_decode_expr(ir, &field.ty, "__VALUE__", mapper),
        collect_text_keys,
        imports: mapper.imports(&field.ty),
        comment: field.comment,
    }
}

fn collect_dart_imports<'a>(fields: impl Iterator<Item = &'a DartField>) -> Vec<String> {
    let mut imports = fields
        .flat_map(|field| field.imports.iter().cloned())
        .collect::<Vec<_>>();
    imports.sort();
    imports.dedup();
    imports
}

fn dart_import(import: BaseImport) -> DartImport {
    DartImport {
        module: dart_module_name(&import.module),
        name: dart_type_identifier(&import.name),
    }
}

struct DartTypeMapper<'a> {
    target: &'a str,
    ir: &'a ConfigIr,
    mappings: &'a TypeMappingRegistry,
}

impl<'a> DartTypeMapper<'a> {
    fn new(target: &'a str, ir: &'a ConfigIr, mappings: &'a TypeMappingRegistry) -> Self {
        Self {
            target,
            ir,
            mappings,
        }
    }

    fn type_name(&self, ty: &TypeIr) -> String {
        if let Some(mapping) = self.mapping(ty) {
            return mapping.type_name;
        }

        match ty {
            TypeIr::Bool => "bool".to_owned(),
            TypeIr::I8
            | TypeIr::U8
            | TypeIr::I16
            | TypeIr::U16
            | TypeIr::I32
            | TypeIr::U32
            | TypeIr::I64 => "int".to_owned(),
            TypeIr::Duration => "Duration".to_owned(),
            TypeIr::DateTime => "DateTime".to_owned(),
            TypeIr::F32 | TypeIr::F64 => "double".to_owned(),
            TypeIr::String => "String".to_owned(),
            TypeIr::Text => "TextKey".to_owned(),
            TypeIr::Enum(name) | TypeIr::Struct(name) | TypeIr::Union(name) => {
                dart_type_identifier(name)
            }
            TypeIr::List(element) | TypeIr::Set(element) | TypeIr::Array { element, .. } => {
                format!("List<{}>", self.type_name(element))
            }
            TypeIr::Map { key, value } => {
                format!("Map<{}, {}>", self.type_name(key), self.type_name(value))
            }
            TypeIr::Ref { table, field } => ref_target_type(self.ir, table, field)
                .map(|ty| self.type_name(ty))
                .unwrap_or_else(|| "int".to_owned()),
            TypeIr::Optional(element) => format!("{}?", self.type_name(element)),
        }
    }

    fn imports(&self, ty: &TypeIr) -> Vec<String> {
        self.mappings.imports_for(self.target, self.ir, ty)
    }

    fn mapping(&self, ty: &TypeIr) -> Option<TypeMapping> {
        self.mappings.map_type(TypeMappingContext {
            target: self.target,
            ir: self.ir,
            ty,
        })
    }

    fn wrap_value_decode(&self, ty: &TypeIr, base_expr: String) -> String {
        self.mapping(ty)
            .map(|mapping| mapping.wrap_value_decode(&base_expr))
            .unwrap_or(base_expr)
    }
}

fn dart_value_decode_expr(
    ir: &ConfigIr,
    ty: &TypeIr,
    value: &str,
    mapper: &DartTypeMapper<'_>,
) -> String {
    match ty {
        TypeIr::Bool => format!("{value}.asBool()"),
        TypeIr::I8
        | TypeIr::U8
        | TypeIr::I16
        | TypeIr::U16
        | TypeIr::I32
        | TypeIr::U32
        | TypeIr::I64 => format!("{value}.asInt()"),
        TypeIr::Duration => format!("Duration(milliseconds: {value}.asInt())"),
        TypeIr::DateTime => {
            format!("DateTime.fromMillisecondsSinceEpoch({value}.asInt(), isUtc: true)")
        }
        TypeIr::F32 | TypeIr::F64 => format!("{value}.asDouble()"),
        TypeIr::String => format!("{value}.asString()"),
        TypeIr::Text => format!("TextKey({value}.asString())"),
        TypeIr::Enum(name) | TypeIr::Struct(name) | TypeIr::Union(name) => mapper
            .wrap_value_decode(
                ty,
                format!("{}.decode({value})", dart_type_identifier(name)),
            ),
        TypeIr::List(element) | TypeIr::Set(element) | TypeIr::Array { element, .. } => {
            format!(
                "{value}.asList((item) => {})",
                dart_value_decode_expr(ir, element, "item", mapper)
            )
        }
        TypeIr::Map {
            key,
            value: element,
        } => format!(
            "{value}.asMap((item) => {}, (item) => {})",
            dart_value_decode_expr(ir, key, "item", mapper),
            dart_value_decode_expr(ir, element, "item", mapper)
        ),
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
            .map(|field| dart_value_decode_expr(ir, &field.ty, value, mapper))
            .unwrap_or_else(|| format!("{value}.asInt()")),
        TypeIr::Optional(element) => {
            format!(
                "{value}.isNull ? null : {}",
                dart_value_decode_expr(ir, element, value, mapper)
            )
        }
    }
}

fn dart_collect_text_keys(
    ir: &ConfigIr,
    ty: &TypeIr,
    value: &str,
    indent: usize,
    mapper: &DartTypeMapper<'_>,
) -> String {
    if mapper.mapping(ty).is_some() {
        return String::new();
    }
    let pad = " ".repeat(indent);
    match ty {
        TypeIr::Text => format!("{pad}out.add({value});"),
        TypeIr::Optional(element) => {
            let inner = dart_collect_text_keys(ir, element, "item", indent + 2, mapper);
            if inner.is_empty() {
                String::new()
            } else {
                format!(
                    "{pad}{{\n{pad}  final item = {value};\n{pad}  if (item != null) {{\n{inner}\n{pad}  }}\n{pad}}}"
                )
            }
        }
        TypeIr::List(element) | TypeIr::Set(element) | TypeIr::Array { element, .. } => {
            let inner = dart_collect_text_keys(ir, element, "item", indent + 2, mapper);
            if inner.is_empty() {
                String::new()
            } else {
                format!("{pad}for (final item in {value}) {{\n{inner}\n{pad}}}")
            }
        }
        TypeIr::Map {
            key,
            value: element,
        } => {
            let key_inner = dart_collect_text_keys(ir, key, "entry.key", indent + 2, mapper);
            let value_inner =
                dart_collect_text_keys(ir, element, "entry.value", indent + 2, mapper);
            if key_inner.is_empty() && value_inner.is_empty() {
                String::new()
            } else {
                format!(
                    "{pad}for (final entry in {value}.entries) {{\n{key_inner}\n{value_inner}\n{pad}}}"
                )
            }
        }
        TypeIr::Struct(_) => format!("{pad}{value}.collectTextKeys(out);"),
        TypeIr::Union(_) => format!("{pad}{value}.collectTextKeys(out);"),
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
            .map(|field| dart_collect_text_keys(ir, &field.ty, value, indent, mapper))
            .unwrap_or_default(),
        TypeIr::Bool
        | TypeIr::I8
        | TypeIr::U8
        | TypeIr::I16
        | TypeIr::U16
        | TypeIr::I32
        | TypeIr::U32
        | TypeIr::I64
        | TypeIr::Duration
        | TypeIr::DateTime
        | TypeIr::F32
        | TypeIr::F64
        | TypeIr::String
        | TypeIr::Enum(_) => String::new(),
    }
}

fn dart_module_name(value: &str) -> String {
    sanitize_identifier(&value.to_snake_case(), CaseKind::Snake)
}

fn ref_target_type<'a>(ir: &'a ConfigIr, table: &str, field: &str) -> Option<&'a TypeIr> {
    ir.tables
        .iter()
        .find(|candidate| candidate.name == table)
        .and_then(|table| {
            table
                .fields
                .iter()
                .find(|candidate| candidate.name == field)
        })
        .map(|field| &field.ty)
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
    use crate::options::{LanguageCodegenOptions, RuntimeFormat};
    use sora_ir::normalize::normalize_schema;
    use sora_schema::model::SchemaFile;

    #[test]
    fn generates_dart_export_runtime_files() {
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
comment = "Item id"

[[tables.fields]]
name = "item_type"
type = "enum<ItemType>"

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

        for (runtime_format, parse_function, adapter) in [
            (RuntimeFormat::Json, "parseJson", None),
            (RuntimeFormat::Cbor, "parseCbor", Some("decodeCbor")),
            (
                RuntimeFormat::SoraProtobuf,
                "parseProtobuf",
                Some("decodeProtobuf"),
            ),
        ] {
            let base =
                std::env::temp_dir().join(format!("sora-codegen-dart-test-{parse_function}"));
            let _ = std::fs::remove_dir_all(&base);

            DartCodeGenerator
                .generate_with_options(&ir, LanguageCodegenOptions { runtime_format }, &base)
                .unwrap();

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
            assert!(runtime.contains(&format!("SoraValueBundle {parse_function}")));
            assert!(runtime.contains("abstract interface class SoraConfigTable"));
            assert!(config.contains("static SoraConfig fromBundle(SoraValueBundle bundle)"));
            if let Some(adapter) = adapter {
                assert!(config.contains(&format!(
                    "required Object? Function(List<int> bytes) {adapter}"
                )));
            }

            let _ = std::fs::remove_dir_all(base);
        }
    }
}
