use std::path::Path;

use heck::{ToPascalCase, ToShoutySnakeCase, ToSnakeCase};
use minijinja::context;
use serde::Serialize;
use sora_diagnostics::{Result, SoraError};
use sora_ir::model::{ConfigIr, TypeIr};

use crate::{
    generator::{CodeGenerator, CodegenContext, runtime_format_name},
    model::{
        BaseField, BaseIndex, BaseModel, BaseRecord, BaseTable, BaseUnion, BaseUnionVariant,
        build_base_model,
    },
    options::{GodotCodegenOptions, RuntimeFormat},
    render::{ensure_dir, render_template, write_file},
    type_mapping::{TypeMapping, TypeMappingContext, TypeMappingRegistry},
};

pub struct GodotCodeGenerator;
crate::impl_test_codegen_generate!(GodotCodeGenerator, "godot");

impl CodeGenerator for GodotCodeGenerator {
    fn generate(&self, context: CodegenContext<'_>, out_dir: &Path) -> Result<()> {
        let ir = context.ir;
        let options = context.options::<GodotCodegenOptions>()?;
        if options.runtime_format != RuntimeFormat::Json {
            return Err(SoraError::InvalidSchema(format!(
                "godot codegen runtime_format `{}` is not implemented yet; supported runtime_format: json",
                runtime_format_name(options.runtime_format)
            )));
        }

        let typed_dictionaries = supports_typed_dictionaries(&options.godot_version)?;
        ensure_dir(out_dir)?;
        let mapper = GodotTypeMapper::new(
            context.target,
            ir,
            context.type_mappings,
            typed_dictionaries,
        );
        let model = GodotModel::from_base_model(ir, build_base_model(ir)?, &mapper);

        for item in &model.enums {
            let rendered = render_template("godot", "enum.gd.j2", context! { enum => item })?;
            write_file(&out_dir.join(format!("{}.gd", item.file_name)), rendered)?;
        }

        for record in &model.records {
            let rendered = render_template("godot", "record.gd.j2", context! { record => record })?;
            write_file(&out_dir.join(format!("{}.gd", record.file_name)), rendered)?;
        }

        for union in &model.unions {
            let rendered = render_template("godot", "union.gd.j2", context! { union => union })?;
            write_file(&out_dir.join(format!("{}.gd", union.file_name)), rendered)?;
            for variant in &union.variants {
                let rendered = render_template(
                    "godot",
                    "union_variant.gd.j2",
                    context! { union => union, variant => variant },
                )?;
                write_file(&out_dir.join(format!("{}.gd", variant.file_name)), rendered)?;
            }
            let rendered =
                render_template("godot", "union_codec.gd.j2", context! { union => union })?;
            write_file(
                &out_dir.join(format!("{}.gd", union.codec_file_name)),
                rendered,
            )?;
        }

        let rendered = render_template("godot", "runtime.gd.j2", context! {})?;
        write_file(&out_dir.join("sora_runtime.gd"), rendered)?;

        let rendered = render_template("godot", "config.gd.j2", context! { model => &model })?;
        write_file(&out_dir.join("sora_config.gd"), rendered)
    }
}

#[derive(Debug, Clone, Serialize)]
struct GodotModel {
    schema_fingerprint: String,
    typed_dictionaries: bool,
    enums: Vec<GodotEnum>,
    records: Vec<GodotRecord>,
    unions: Vec<GodotUnion>,
    tables: Vec<GodotTable>,
}

#[derive(Debug, Clone, Serialize)]
struct GodotEnum {
    class_name: String,
    file_name: String,
    comment: Option<String>,
    values: Vec<GodotEnumValue>,
}

#[derive(Debug, Clone, Serialize)]
struct GodotEnumValue {
    raw_name: String,
    const_name: String,
    comment: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct GodotRecord {
    class_name: String,
    file_name: String,
    fields: Vec<GodotField>,
    table: Option<GodotTable>,
    custom_imports: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct GodotUnion {
    class_name: String,
    file_name: String,
    codec_class_name: String,
    codec_file_name: String,
    raw_tag: String,
    tag: String,
    variants: Vec<GodotUnionVariant>,
}

#[derive(Debug, Clone, Serialize)]
struct GodotUnionVariant {
    raw_name: String,
    class_name: String,
    file_name: String,
    fields: Vec<GodotField>,
    custom_imports: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct GodotField {
    raw_name: String,
    name: String,
    type_name: String,
    default_value: String,
    value_decode: String,
    decode_assign: bool,
    imports: Vec<String>,
    comment: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct GodotTable {
    name: String,
    class_name: String,
    field_name: String,
    mode: String,
    row_type: String,
    key_name: Option<String>,
    key_field_name: Option<String>,
    key_type: String,
    key_array_type: String,
    row_array_type: String,
    rows_dictionary_type: String,
    typed_rows_dictionary: bool,
    unique_indexes: Vec<GodotIndex>,
    non_unique_indexes: Vec<GodotIndex>,
}

#[derive(Debug, Clone, Serialize)]
struct GodotIndex {
    method_name: String,
    field_name: String,
    param_name: String,
    param_type: String,
    dictionary_type: String,
    typed_dictionary: bool,
    result_array_type: String,
}

impl GodotModel {
    fn from_base_model(ir: &ConfigIr, model: BaseModel, mapper: &GodotTypeMapper<'_>) -> Self {
        let enums = model
            .enums
            .into_iter()
            .map(|item| GodotEnum {
                class_name: godot_type_identifier(&item.pascal_name),
                file_name: godot_file_name(&item.snake_name),
                comment: item.comment,
                values: item
                    .values
                    .into_iter()
                    .map(|value| GodotEnumValue {
                        const_name: godot_const_identifier(&value.name),
                        raw_name: value.name,
                        comment: value.comment,
                    })
                    .collect(),
            })
            .collect();
        let tables = model
            .tables
            .into_iter()
            .map(|table| godot_table(table, mapper))
            .collect::<Vec<_>>();
        let records = model
            .records
            .into_iter()
            .map(|item| {
                let class_name = godot_type_identifier(&item.pascal_name);
                let table = tables
                    .iter()
                    .find(|table| table.row_type == class_name)
                    .cloned();
                godot_record(ir, item, table, mapper)
            })
            .collect();
        let unions = model
            .unions
            .into_iter()
            .map(|item| godot_union(ir, item, mapper))
            .collect();

        Self {
            schema_fingerprint: model.schema_fingerprint,
            typed_dictionaries: mapper.typed_dictionaries,
            enums,
            records,
            unions,
            tables,
        }
    }
}

fn godot_record(
    ir: &ConfigIr,
    record: BaseRecord,
    table: Option<GodotTable>,
    mapper: &GodotTypeMapper<'_>,
) -> GodotRecord {
    let fields = record
        .fields
        .into_iter()
        .map(|field| godot_field(ir, field, mapper))
        .collect::<Vec<_>>();
    let custom_imports = collect_godot_imports(fields.iter());
    GodotRecord {
        class_name: godot_type_identifier(&record.pascal_name),
        file_name: godot_file_name(&record.snake_name),
        fields,
        table,
        custom_imports,
    }
}

fn godot_union(ir: &ConfigIr, union: BaseUnion, mapper: &GodotTypeMapper<'_>) -> GodotUnion {
    let class_name = godot_type_identifier(&union.pascal_name);
    let variants = union
        .variants
        .into_iter()
        .map(|variant| godot_union_variant(ir, &class_name, variant, mapper))
        .collect::<Vec<_>>();

    GodotUnion {
        codec_class_name: format!("{class_name}Codec"),
        codec_file_name: godot_file_name(&format!("{}_codec", union.snake_name)),
        class_name,
        file_name: godot_file_name(&union.snake_name),
        raw_tag: union.tag.clone(),
        tag: godot_field_identifier(&union.tag),
        variants,
    }
}

fn godot_union_variant(
    ir: &ConfigIr,
    union_class_name: &str,
    variant: BaseUnionVariant,
    mapper: &GodotTypeMapper<'_>,
) -> GodotUnionVariant {
    let fields = variant
        .fields
        .into_iter()
        .map(|field| godot_field(ir, field, mapper))
        .collect::<Vec<_>>();
    let custom_imports = collect_godot_imports(fields.iter());
    let class_name = format!(
        "{union_class_name}{}",
        godot_type_identifier(&variant.pascal_name)
    );
    GodotUnionVariant {
        raw_name: variant.name,
        file_name: godot_file_name(&class_name),
        class_name,
        fields,
        custom_imports,
    }
}

fn godot_table(table: BaseTable, mapper: &GodotTypeMapper<'_>) -> GodotTable {
    let key_field_name = table
        .key_field
        .as_ref()
        .map(|field| godot_field_identifier(&field.snake_name));
    let key_type = table
        .key_field
        .as_ref()
        .map(|field| mapper.type_name(&field.ty))
        .unwrap_or_else(|| "Variant".to_owned());
    let row_type = godot_type_identifier(&table.pascal_name);
    let row_array_type = format!("Array[{row_type}]");
    let key_array_type = typed_array_type(&key_type);
    let rows_dictionary_type = if mapper.typed_dictionaries {
        format!("Dictionary[{key_type}, {row_type}]")
    } else {
        "Dictionary".to_owned()
    };
    let typed_rows_dictionary = mapper.typed_dictionaries;

    GodotTable {
        name: table.name,
        class_name: godot_type_identifier(&format!("{}Table", table.pascal_name)),
        field_name: godot_field_identifier(&table.snake_name),
        mode: table.mode_name,
        row_type: row_type.clone(),
        key_name: table.key_name,
        key_field_name,
        key_type,
        key_array_type,
        row_array_type: row_array_type.clone(),
        rows_dictionary_type,
        typed_rows_dictionary,
        unique_indexes: table
            .unique_indexes
            .into_iter()
            .map(|index| godot_index(index, &row_type, mapper, true))
            .collect(),
        non_unique_indexes: table
            .non_unique_indexes
            .into_iter()
            .map(|index| godot_index(index, &row_type, mapper, false))
            .collect(),
    }
}

fn godot_index(
    index: BaseIndex,
    row_type: &str,
    mapper: &GodotTypeMapper<'_>,
    unique: bool,
) -> GodotIndex {
    let field_name = godot_field_identifier(&index.field.snake_name);
    let param_type = mapper.type_name(&index.field.ty);
    let value_type = if unique { row_type } else { "Array" };
    GodotIndex {
        method_name: godot_field_identifier(&index.snake_name),
        param_name: field_name.clone(),
        field_name,
        dictionary_type: if mapper.typed_dictionaries {
            format!("Dictionary[{param_type}, {value_type}]")
        } else {
            "Dictionary".to_owned()
        },
        typed_dictionary: mapper.typed_dictionaries,
        param_type,
        result_array_type: format!("Array[{row_type}]"),
    }
}

fn godot_field(ir: &ConfigIr, field: BaseField, mapper: &GodotTypeMapper<'_>) -> GodotField {
    GodotField {
        raw_name: field.raw_name,
        name: godot_field_identifier(&field.snake_name),
        type_name: mapper.type_name(&field.ty),
        default_value: godot_default_value(ir, &field.ty),
        value_decode: godot_value_decode_expr(ir, &field.ty, "__VALUE__", mapper),
        decode_assign: mapper.is_typed_collection(&field.ty),
        imports: mapper.imports(&field.ty),
        comment: field.comment,
    }
}

fn collect_godot_imports<'a>(fields: impl Iterator<Item = &'a GodotField>) -> Vec<String> {
    let mut imports = fields
        .flat_map(|field| field.imports.iter().cloned())
        .collect::<Vec<_>>();
    imports.sort();
    imports.dedup();
    imports
}

struct GodotTypeMapper<'a> {
    target: &'a str,
    ir: &'a ConfigIr,
    mappings: &'a TypeMappingRegistry,
    typed_dictionaries: bool,
}

impl<'a> GodotTypeMapper<'a> {
    fn new(
        target: &'a str,
        ir: &'a ConfigIr,
        mappings: &'a TypeMappingRegistry,
        typed_dictionaries: bool,
    ) -> Self {
        Self {
            target,
            ir,
            mappings,
            typed_dictionaries,
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
            | TypeIr::I64
            | TypeIr::Duration
            | TypeIr::DateTime => "int".to_owned(),
            TypeIr::F32 | TypeIr::F64 => "float".to_owned(),
            TypeIr::String | TypeIr::Enum(_) => "String".to_owned(),
            TypeIr::Text => "SoraRuntime.TextKey".to_owned(),
            TypeIr::Struct(name) | TypeIr::Union(name) => godot_type_identifier(name),
            TypeIr::List(element) | TypeIr::Set(element) | TypeIr::Array { element, .. } => {
                typed_array_type(&self.nested_collection_type_name(element))
            }
            TypeIr::Map { key, value } if self.typed_dictionaries => format!(
                "Dictionary[{}, {}]",
                self.nested_collection_type_name(key),
                self.nested_collection_type_name(value)
            ),
            TypeIr::Map { .. } => "Dictionary".to_owned(),
            TypeIr::Ref { table, field } => ref_target_type(self.ir, table, field)
                .map(|ty| self.type_name(ty))
                .unwrap_or_else(|| "int".to_owned()),
            TypeIr::Optional(element) => self.optional_type_name(element),
        }
    }

    fn nested_collection_type_name(&self, ty: &TypeIr) -> String {
        if let Some(mapping) = self.mapping(ty) {
            return mapping.type_name;
        }
        match ty {
            TypeIr::List(_) | TypeIr::Set(_) | TypeIr::Array { .. } => "Array".to_owned(),
            TypeIr::Map { .. } => "Dictionary".to_owned(),
            TypeIr::Optional(_) => "Variant".to_owned(),
            _ => self.type_name(ty),
        }
    }

    fn optional_type_name(&self, element: &TypeIr) -> String {
        if let Some(mapping) = self.mapping(element) {
            return mapping
                .nullable_type_name
                .unwrap_or_else(|| "Variant".to_owned());
        }
        match element {
            TypeIr::Struct(name) | TypeIr::Union(name) => godot_type_identifier(name),
            TypeIr::Text => "SoraRuntime.TextKey".to_owned(),
            _ => "Variant".to_owned(),
        }
    }

    fn is_typed_collection(&self, ty: &TypeIr) -> bool {
        if self.mapping(ty).is_some() {
            return false;
        }
        match ty {
            TypeIr::List(_) | TypeIr::Set(_) | TypeIr::Array { .. } => {
                self.type_name(ty) != "Array"
            }
            TypeIr::Map { .. } => self.typed_dictionaries,
            _ => false,
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

fn godot_value_decode_expr(
    ir: &ConfigIr,
    ty: &TypeIr,
    value: &str,
    mapper: &GodotTypeMapper<'_>,
) -> String {
    match ty {
        TypeIr::Bool => format!("bool({value})"),
        TypeIr::I8 | TypeIr::U8 | TypeIr::I16 | TypeIr::U16 | TypeIr::I32 | TypeIr::U32 => {
            format!("SoraRuntime.decode_int({value})")
        }
        TypeIr::I64 | TypeIr::Duration | TypeIr::DateTime => {
            format!("SoraRuntime.decode_int({value})")
        }
        TypeIr::F32 | TypeIr::F64 => format!("float({value})"),
        TypeIr::String => format!("str({value})"),
        TypeIr::Text => format!("SoraRuntime.TextKey.new(str({value}))"),
        TypeIr::Enum(name) => mapper.wrap_value_decode(
            ty,
            format!("{}.decode({value})", godot_type_identifier(name)),
        ),
        TypeIr::Struct(name) => mapper.wrap_value_decode(
            ty,
            format!("{}.decode({value})", godot_type_identifier(name)),
        ),
        TypeIr::Union(name) => mapper.wrap_value_decode(
            ty,
            format!("{}Codec.decode({value})", godot_type_identifier(name)),
        ),
        TypeIr::List(element) | TypeIr::Set(element) | TypeIr::Array { element, .. } => format!(
            "SoraRuntime.decode_array({value}, func(item): return {})",
            godot_value_decode_expr(ir, element, "item", mapper)
        ),
        TypeIr::Map {
            key,
            value: element,
        } => format!(
            "SoraRuntime.decode_map({value}, func(item): return {}, func(item): return {})",
            godot_value_decode_expr(ir, key, "item", mapper),
            godot_value_decode_expr(ir, element, "item", mapper)
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
            .map(|field| godot_value_decode_expr(ir, &field.ty, value, mapper))
            .unwrap_or_else(|| format!("{value}.as_int()")),
        TypeIr::Optional(element) => {
            format!(
                "null if {value} == null else {}",
                godot_value_decode_expr(ir, element, value, mapper)
            )
        }
    }
}

fn godot_default_value(ir: &ConfigIr, ty: &TypeIr) -> String {
    match ty {
        TypeIr::Bool => "false".to_owned(),
        TypeIr::I8
        | TypeIr::U8
        | TypeIr::I16
        | TypeIr::U16
        | TypeIr::I32
        | TypeIr::U32
        | TypeIr::I64
        | TypeIr::Duration
        | TypeIr::DateTime => "0".to_owned(),
        TypeIr::F32 | TypeIr::F64 => "0.0".to_owned(),
        TypeIr::String | TypeIr::Enum(_) => "\"\"".to_owned(),
        TypeIr::Text => "null".to_owned(),
        TypeIr::List(_) | TypeIr::Set(_) | TypeIr::Array { .. } => "[]".to_owned(),
        TypeIr::Map { .. } => "{}".to_owned(),
        TypeIr::Optional(_) | TypeIr::Struct(_) | TypeIr::Union(_) => "null".to_owned(),
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
            .map(|field| godot_default_value(ir, &field.ty))
            .unwrap_or_else(|| "0".to_owned()),
    }
}

fn godot_file_name(value: &str) -> String {
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

fn typed_array_type(element_type: &str) -> String {
    if element_type == "Variant" {
        "Array".to_owned()
    } else {
        format!("Array[{element_type}]")
    }
}

fn supports_typed_dictionaries(version: &str) -> Result<bool> {
    let Some((major, minor)) = version.split_once('.') else {
        return Err(SoraError::InvalidSchema(format!(
            "invalid Godot version `{version}`; expected `4.3` or newer"
        )));
    };
    let major = major.parse::<u32>().map_err(|_| {
        SoraError::InvalidSchema(format!(
            "invalid Godot version `{version}`; expected `4.3` or newer"
        ))
    })?;
    let minor = minor.parse::<u32>().map_err(|_| {
        SoraError::InvalidSchema(format!(
            "invalid Godot version `{version}`; expected `4.3` or newer"
        ))
    })?;
    if major != 4 || minor < 3 {
        return Err(SoraError::InvalidSchema(format!(
            "unsupported Godot version `{version}`; expected `4.3` or newer"
        )));
    }
    Ok(minor >= 4)
}

fn godot_type_identifier(value: &str) -> String {
    sanitize_identifier(&value.to_pascal_case(), CaseKind::Pascal)
}

fn godot_field_identifier(value: &str) -> String {
    sanitize_identifier(&value.to_snake_case(), CaseKind::Snake)
}

fn godot_const_identifier(value: &str) -> String {
    sanitize_identifier(&value.to_shouty_snake_case(), CaseKind::Const)
}

#[derive(Clone, Copy)]
enum CaseKind {
    Snake,
    Pascal,
    Const,
}

fn sanitize_identifier(value: &str, case: CaseKind) -> String {
    let mut out = String::with_capacity(value.len());
    for (index, ch) in value.chars().enumerate() {
        let valid = ch == '_' || ch.is_ascii_alphanumeric();
        let ch = if valid { ch } else { '_' };
        if index == 0 && ch.is_ascii_digit() {
            out.push(match case {
                CaseKind::Pascal => 'T',
                CaseKind::Snake => 'v',
                CaseKind::Const => '_',
            });
        }
        out.push(ch);
    }
    if out.is_empty() || out == "_" {
        out = match case {
            CaseKind::Pascal => "Value".to_owned(),
            CaseKind::Snake => "value".to_owned(),
            CaseKind::Const => "VALUE".to_owned(),
        };
    }
    if is_godot_keyword(&out) {
        out.push('_');
    }
    out
}

fn is_godot_keyword(value: &str) -> bool {
    matches!(
        value,
        "Array"
            | "Dictionary"
            | "String"
            | "Variant"
            | "bool"
            | "break"
            | "class"
            | "class_name"
            | "const"
            | "continue"
            | "else"
            | "extends"
            | "false"
            | "float"
            | "for"
            | "func"
            | "if"
            | "in"
            | "int"
            | "match"
            | "null"
            | "pass"
            | "return"
            | "self"
            | "static"
            | "true"
            | "var"
            | "void"
            | "while"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::options::{GodotCodegenOptions, RuntimeFormat};
    use sora_ir::normalize::normalize_schema;
    use sora_schema::model::ProjectSchema;

    #[test]
    fn generates_godot_json_runtime_files() {
        let schema: ProjectSchema = toml::from_str(
            r#"
project = { id = "game_config" }
groups = { common = { default = true } }
views = { default = { contract = "game_config/default", groups = ["common"] } }

[codegen.godot]
runtime_format = "json"
godot_version = "4.3"

[[enums]]
name = "ItemType"
values = [{ id = 0, name = "Weapon" }, { id = 1, name = "Armor" }]

[[structs]]
name = "Cost"

[[structs.fields]]
name = "item_id"
type = "i32"

[[unions]]
name = "RewardAction"

[[unions.variants]]
name = "AddItem"

[[unions.variants.fields]]
name = "item_id"
type = "i32"

[[tables]]
id = "item"
name = "Item"
mode = "map"
key = "id"

[[tables.fields]]
name = "id"
type = "i32"

[[tables.fields]]
name = "item_type"
type = "enum<ItemType>"

[[tables.fields]]
name = "attributes"
type = "map<string,i32>"

[[tables.fields]]
name = "tags"
type = "list<string>"

[[tables.fields]]
name = "large_counter"
type = "i64"

[[tables.indexes]]
name = "by_item_type"
fields = ["item_type"]
"#,
        )
        .unwrap();
        let ir = normalize_schema(schema).unwrap();
        let out = std::env::temp_dir().join("sora-codegen-godot-test");
        let _ = std::fs::remove_dir_all(&out);

        GodotCodeGenerator
            .generate_with_options(
                &ir,
                GodotCodegenOptions {
                    runtime_format: RuntimeFormat::Json,
                    godot_version: "4.3".to_owned(),
                },
                &out,
            )
            .unwrap();

        assert!(out.join("sora_runtime.gd").exists());
        assert!(out.join("sora_config.gd").exists());
        assert!(out.join("item.gd").exists());
        assert!(out.join("item_type.gd").exists());
        assert!(out.join("reward_action.gd").exists());
        assert!(out.join("reward_action_add_item.gd").exists());
        assert!(out.join("reward_action_codec.gd").exists());
        let item = std::fs::read_to_string(out.join("item.gd")).unwrap();
        let config = std::fs::read_to_string(out.join("sora_config.gd")).unwrap();
        let runtime = std::fs::read_to_string(out.join("sora_runtime.gd")).unwrap();
        let union_codec = std::fs::read_to_string(out.join("reward_action_codec.gd")).unwrap();
        assert!(item.contains("class ItemTable"));
        assert!(item.contains("func get_row(key_value: int) -> Item"));
        assert!(item.contains("func find_by_item_type(item_type: String) -> Array[Item]"));
        assert!(item.contains("func ordered_rows() -> Array[Item]"));
        assert!(item.contains("var tags: Array[String] = []"));
        assert!(item.contains("out.tags.assign(SoraRuntime.decode_array"));
        assert!(item.contains("var attributes: Dictionary = {}"));
        assert!(item.contains("out.large_counter = SoraRuntime.decode_int"));
        assert!(!config.contains("class ItemTable"));
        assert!(config.contains("return null"));
        assert!(runtime.contains("_quote_unsafe_json_integers"));
        assert!(runtime.contains("static func decode_int"));
        assert!(union_codec.contains("var out := RewardActionAddItem.new()"));
        assert!(union_codec.contains("return out"));

        let out_44 = std::env::temp_dir().join("sora-codegen-godot-44-test");
        let _ = std::fs::remove_dir_all(&out_44);
        GodotCodeGenerator
            .generate_with_options(
                &ir,
                GodotCodegenOptions {
                    runtime_format: RuntimeFormat::Json,
                    godot_version: "4.4".to_owned(),
                },
                &out_44,
            )
            .unwrap();
        let item_44 = std::fs::read_to_string(out_44.join("item.gd")).unwrap();
        let config_44 = std::fs::read_to_string(out_44.join("sora_config.gd")).unwrap();
        assert!(item_44.contains("var attributes: Dictionary[String, int] = {}"));
        assert!(item_44.contains("var _rows: Dictionary[int, Item] = {}"));
        assert!(item_44.contains("var _item_type: Dictionary[String, Array] = {}"));
        assert!(
            config_44.contains("var _tables: Dictionary[String, SoraRuntime.SoraConfigTable] = {}")
        );

        let _ = std::fs::remove_dir_all(out);
        let _ = std::fs::remove_dir_all(out_44);
    }

    #[test]
    fn validates_supported_godot_versions() {
        assert!(!supports_typed_dictionaries("4.3").unwrap());
        assert!(supports_typed_dictionaries("4.4").unwrap());
        assert!(supports_typed_dictionaries("4.7").unwrap());
        assert!(supports_typed_dictionaries("3.5").is_err());
        assert!(supports_typed_dictionaries("latest").is_err());
    }
}
