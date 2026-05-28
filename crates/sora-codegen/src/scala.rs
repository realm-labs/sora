use std::path::{Path, PathBuf};

use minijinja::context;
use serde::Serialize;
use sora_diagnostics::{Result, SoraError};
use sora_ir::model::{ConfigIr, TableModeIr, TypeIr};

use crate::{
    generator::{CodeGenerator, CodegenContext, runtime_format_name},
    model::{
        BaseField, BaseIndex, BaseModel, BaseRecord, BaseTable, BaseUnion, BaseUnionVariant,
        build_base_model,
    },
    options::{ScalaCodegenOptions, ScalaVersion},
    render::{ensure_dir, render_template, write_file},
    type_mapping::{TypeMapping, TypeMappingContext, TypeMappingRegistry},
};

pub struct ScalaCodeGenerator;
crate::impl_test_codegen_generate!(ScalaCodeGenerator, "scala");

impl CodeGenerator for ScalaCodeGenerator {
    fn generate(&self, context: CodegenContext<'_>, out_dir: &Path) -> Result<()> {
        let ir = context.ir;
        let codegen_options = context.options::<ScalaCodegenOptions>()?;
        ensure_dir(out_dir)?;
        let runtime_format = runtime_format_name(codegen_options.runtime_format);

        let mapper = ScalaTypeMapper::new(context.target, ir, context.type_mappings);
        let model =
            ScalaModel::from_base_model(ir, build_base_model(ir)?, &codegen_options, &mapper);
        let package_dir = scala_package_dir(out_dir, &model.package)?;

        for item in &model.enums {
            let rendered = render_template(
                "scala",
                "enum.scala.j2",
                context! { package => &model.package, enum => item, runtime_format => runtime_format },
            )?;
            write_file(&package_dir.join(format!("{}.scala", item.name)), rendered)?;
        }

        for record in &model.records {
            let rendered = render_template(
                "scala",
                "record.scala.j2",
                context! { package => &model.package, record => record, runtime_format => runtime_format },
            )?;
            write_file(
                &package_dir.join(format!("{}.scala", record.pascal_name)),
                rendered,
            )?;
        }

        for union in &model.unions {
            let rendered = render_template(
                "scala",
                "union.scala.j2",
                context! { package => &model.package, union => union, runtime_format => runtime_format },
            )?;
            write_file(
                &package_dir.join(format!("{}.scala", union.pascal_name)),
                rendered,
            )?;
        }

        let rendered = render_template(
            "scala",
            "runtime.scala.j2",
            context! { package => &model.package, runtime_format => runtime_format },
        )?;
        write_file(&package_dir.join("SoraRuntime.scala"), rendered)?;

        let rendered = render_template("scala", "config.scala.j2", context! { model => &model })?;
        write_file(&package_dir.join("SoraConfig.scala"), rendered)
    }
}

#[derive(Debug, Clone, Serialize)]
struct ScalaModel {
    package: String,
    schema_fingerprint: String,
    enums: Vec<ScalaEnum>,
    unions: Vec<ScalaUnion>,
    records: Vec<ScalaRecord>,
    tables: Vec<ScalaTable>,
    has_localization: bool,
    locales: Vec<String>,
    default_locale: String,
}

#[derive(Debug, Clone, Serialize)]
struct ScalaEnum {
    name: String,
    values: Vec<String>,
    is_scala3: bool,
}

#[derive(Debug, Clone, Serialize)]
struct ScalaUnion {
    pascal_name: String,
    tag: String,
    variants: Vec<ScalaUnionVariant>,
    imports: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct ScalaUnionVariant {
    name: String,
    fields: Vec<ScalaField>,
    has_text_keys: bool,
}

#[derive(Debug, Clone, Serialize)]
struct ScalaRecord {
    pascal_name: String,
    fields: Vec<ScalaField>,
    has_text_keys: bool,
    table: Option<ScalaTable>,
    imports: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct ScalaTable {
    name: String,
    pascal_name: String,
    camel_name: String,
    mode: String,
    container_type: String,
    row_type: String,
    key_name: Option<String>,
    key_field_name: Option<String>,
    key_type: Option<String>,
    unique_indexes: Vec<ScalaIndex>,
    non_unique_indexes: Vec<ScalaIndex>,
}

#[derive(Debug, Clone, Serialize)]
struct ScalaIndex {
    pascal_name: String,
    field_name: String,
    param_camel_name: String,
    key_type: String,
}

#[derive(Debug, Clone, Serialize)]
struct ScalaField {
    raw_name: String,
    name: String,
    type_name: String,
    decode: String,
    value_decode: String,
    collect_text_keys: String,
    imports: Vec<String>,
    comment: Option<String>,
}

impl ScalaModel {
    fn from_base_model(
        ir: &ConfigIr,
        model: BaseModel,
        codegen_options: &ScalaCodegenOptions,
        mapper: &ScalaTypeMapper<'_>,
    ) -> Self {
        let is_scala3 = codegen_options.scala_version == ScalaVersion::Scala3;
        let tables = model
            .tables
            .into_iter()
            .map(|item| scala_table(ir, item, mapper))
            .collect::<Vec<_>>();

        Self {
            package: model.package,
            schema_fingerprint: model.schema_fingerprint,
            enums: model
                .enums
                .into_iter()
                .map(|item| ScalaEnum {
                    name: item.pascal_name,
                    values: item.values,
                    is_scala3,
                })
                .collect(),
            unions: model
                .unions
                .into_iter()
                .map(|item| scala_union(ir, item, mapper))
                .collect(),
            records: model
                .records
                .into_iter()
                .map(|item| {
                    let table = tables
                        .iter()
                        .find(|table| table.row_type == item.pascal_name)
                        .cloned();
                    scala_record(ir, item, table, mapper)
                })
                .collect(),
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
            tables,
        }
    }
}

fn scala_union(ir: &ConfigIr, union: BaseUnion, mapper: &ScalaTypeMapper<'_>) -> ScalaUnion {
    let variants = union
        .variants
        .into_iter()
        .map(|variant| scala_variant(ir, variant, mapper))
        .collect::<Vec<_>>();
    let imports = collect_scala_imports(variants.iter().flat_map(|variant| &variant.fields));
    ScalaUnion {
        pascal_name: union.pascal_name,
        tag: union.tag,
        variants,
        imports,
    }
}

fn scala_variant(
    ir: &ConfigIr,
    variant: BaseUnionVariant,
    mapper: &ScalaTypeMapper<'_>,
) -> ScalaUnionVariant {
    let fields = variant
        .fields
        .into_iter()
        .map(|field| scala_field(ir, field, mapper))
        .collect::<Vec<_>>();
    let has_text_keys = fields
        .iter()
        .any(|field| !field.collect_text_keys.is_empty());
    ScalaUnionVariant {
        name: variant.pascal_name,
        fields,
        has_text_keys,
    }
}

fn scala_record(
    ir: &ConfigIr,
    record: BaseRecord,
    table: Option<ScalaTable>,
    mapper: &ScalaTypeMapper<'_>,
) -> ScalaRecord {
    let fields = record
        .fields
        .into_iter()
        .map(|field| scala_field(ir, field, mapper))
        .collect::<Vec<_>>();
    let has_text_keys = fields
        .iter()
        .any(|field| !field.collect_text_keys.is_empty());
    let imports = collect_scala_imports(fields.iter());
    ScalaRecord {
        pascal_name: record.pascal_name,
        fields,
        has_text_keys,
        table,
        imports,
    }
}

fn scala_table(ir: &ConfigIr, table: BaseTable, mapper: &ScalaTypeMapper<'_>) -> ScalaTable {
    let row_type = table.pascal_name.clone();
    let key_type = table
        .key_field
        .as_ref()
        .map(|field| mapper.type_name(&field.ty));
    let container_type = scala_container_type(table.mode, &row_type, key_type.as_deref());
    let key_field_name = table
        .key_field
        .as_ref()
        .map(|field| field.camel_name.clone());

    ScalaTable {
        name: table.name,
        pascal_name: table.pascal_name,
        camel_name: table.camel_name,
        mode: table.mode_name,
        container_type,
        row_type,
        key_name: table.key_name,
        key_field_name,
        key_type,
        unique_indexes: table
            .unique_indexes
            .into_iter()
            .map(|index| scala_index(ir, index, mapper))
            .collect(),
        non_unique_indexes: table
            .non_unique_indexes
            .into_iter()
            .map(|index| scala_index(ir, index, mapper))
            .collect(),
    }
}

fn scala_index(_ir: &ConfigIr, index: BaseIndex, mapper: &ScalaTypeMapper<'_>) -> ScalaIndex {
    ScalaIndex {
        pascal_name: index.pascal_name,
        field_name: index.field.camel_name.clone(),
        param_camel_name: index.field.camel_name,
        key_type: mapper.type_name(&index.field.ty),
    }
}

fn scala_field(ir: &ConfigIr, field: BaseField, mapper: &ScalaTypeMapper<'_>) -> ScalaField {
    let collect_text_keys = scala_collect_text_keys(
        ir,
        &field.ty,
        &format!("this.{}", field.camel_name),
        4,
        mapper,
    );
    ScalaField {
        raw_name: field.raw_name,
        name: field.camel_name,
        type_name: mapper.type_name(&field.ty),
        decode: scala_decode_expr(ir, &field.ty, mapper),
        value_decode: scala_value_decode_expr(ir, &field.ty, "__VALUE__", mapper),
        collect_text_keys,
        imports: mapper.imports(&field.ty),
        comment: field.comment,
    }
}

fn collect_scala_imports<'a>(fields: impl Iterator<Item = &'a ScalaField>) -> Vec<String> {
    let mut imports = fields
        .flat_map(|field| field.imports.iter().cloned())
        .collect::<Vec<_>>();
    imports.sort();
    imports.dedup();
    imports
}

fn scala_container_type(mode: TableModeIr, row_type: &str, key_type: Option<&str>) -> String {
    match mode {
        TableModeIr::List => format!("Vector[{row_type}]"),
        TableModeIr::Map => match key_type {
            Some(key_type) => format!("Map[{key_type}, {row_type}]"),
            None => format!("Vector[{row_type}]"),
        },
        TableModeIr::Singleton => row_type.to_owned(),
    }
}

struct ScalaTypeMapper<'a> {
    target: &'a str,
    ir: &'a ConfigIr,
    mappings: &'a TypeMappingRegistry,
}

impl<'a> ScalaTypeMapper<'a> {
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
            TypeIr::Bool => "Boolean".to_owned(),
            TypeIr::I8 | TypeIr::U8 | TypeIr::I16 | TypeIr::U16 | TypeIr::I32 => "Int".to_owned(),
            TypeIr::U32 | TypeIr::I64 => "Long".to_owned(),
            TypeIr::Duration => "scala.concurrent.duration.FiniteDuration".to_owned(),
            TypeIr::DateTime => "java.time.Instant".to_owned(),
            TypeIr::F32 => "Float".to_owned(),
            TypeIr::F64 => "Double".to_owned(),
            TypeIr::String => "String".to_owned(),
            TypeIr::Text => "TextKey".to_owned(),
            TypeIr::Enum(name) | TypeIr::Struct(name) | TypeIr::Union(name) => name.clone(),
            TypeIr::List(element) | TypeIr::Set(element) | TypeIr::Array { element, .. } => {
                format!("Vector[{}]", self.type_name(element))
            }
            TypeIr::Map { key, value } => {
                format!("Map[{}, {}]", self.type_name(key), self.type_name(value))
            }
            TypeIr::Ref { table, field } => ref_target_type(self.ir, table, field)
                .map(|ty| self.type_name(ty))
                .unwrap_or_else(|| "Int".to_owned()),
            TypeIr::Optional(element) => format!("Option[{}]", self.type_name(element)),
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

    fn wrap_decode(&self, ty: &TypeIr, base_expr: String) -> String {
        self.mapping(ty)
            .map(|mapping| mapping.wrap_decode(&base_expr))
            .unwrap_or(base_expr)
    }

    fn wrap_value_decode(&self, ty: &TypeIr, base_expr: String) -> String {
        self.mapping(ty)
            .map(|mapping| mapping.wrap_value_decode(&base_expr))
            .unwrap_or(base_expr)
    }
}

fn scala_decode_expr(ir: &ConfigIr, ty: &TypeIr, mapper: &ScalaTypeMapper<'_>) -> String {
    match ty {
        TypeIr::Bool => "reader.readBool()".to_owned(),
        TypeIr::I8 | TypeIr::I16 => "reader.readI32()".to_owned(),
        TypeIr::U8 | TypeIr::U16 => "reader.readU32()".to_owned(),
        TypeIr::I32 => "reader.readI32()".to_owned(),
        TypeIr::U32 => "java.lang.Integer.toUnsignedLong(reader.readU32())".to_owned(),
        TypeIr::I64 => "reader.readI64()".to_owned(),
        TypeIr::Duration => "SoraRuntime.durationFromMillis(reader.readI64())".to_owned(),
        TypeIr::DateTime => "java.time.Instant.ofEpochMilli(reader.readI64())".to_owned(),
        TypeIr::F32 => "reader.readF32()".to_owned(),
        TypeIr::F64 => "reader.readF64()".to_owned(),
        TypeIr::String => "reader.readString()".to_owned(),
        TypeIr::Text => "TextKey(reader.readString())".to_owned(),
        TypeIr::Enum(name) | TypeIr::Struct(name) | TypeIr::Union(name) => {
            mapper.wrap_decode(ty, format!("{name}.decode(reader)"))
        }
        TypeIr::List(element) | TypeIr::Set(element) | TypeIr::Array { element, .. } => {
            format!(
                "reader.readList({})",
                scala_decode_expr(ir, element, mapper)
            )
        }
        TypeIr::Map { key, value } => format!(
            "reader.readMap({}, {})",
            scala_decode_expr(ir, key, mapper),
            scala_decode_expr(ir, value, mapper)
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
            .map(|field| scala_decode_expr(ir, &field.ty, mapper))
            .unwrap_or_else(|| "reader.readI32()".to_owned()),
        TypeIr::Optional(element) => {
            format!(
                "reader.readOptional({})",
                scala_decode_expr(ir, element, mapper)
            )
        }
    }
}

fn scala_value_decode_expr(
    ir: &ConfigIr,
    ty: &TypeIr,
    value: &str,
    mapper: &ScalaTypeMapper<'_>,
) -> String {
    match ty {
        TypeIr::Bool => format!("{value}.asBool"),
        TypeIr::I8 | TypeIr::U8 | TypeIr::I16 | TypeIr::U16 => format!("{value}.asInt"),
        TypeIr::I32 => format!("{value}.asInt"),
        TypeIr::U32 => format!("{value}.asLong"),
        TypeIr::I64 => format!("{value}.asLong"),
        TypeIr::Duration => format!("SoraRuntime.durationFromMillis({value}.asLong)"),
        TypeIr::DateTime => format!("java.time.Instant.ofEpochMilli({value}.asLong)"),
        TypeIr::F32 => format!("{value}.asFloat"),
        TypeIr::F64 => format!("{value}.asDouble"),
        TypeIr::String => format!("{value}.asString"),
        TypeIr::Text => format!("TextKey({value}.asString)"),
        TypeIr::Enum(name) | TypeIr::Struct(name) | TypeIr::Union(name) => {
            mapper.wrap_value_decode(ty, format!("{name}.decode({value})"))
        }
        TypeIr::List(element) | TypeIr::Set(element) | TypeIr::Array { element, .. } => {
            format!(
                "{value}.asList(item => {})",
                scala_value_decode_expr(ir, element, "item", mapper)
            )
        }
        TypeIr::Map {
            key,
            value: element,
        } => format!(
            "{value}.asMap(item => {}, item => {})",
            scala_value_decode_expr(ir, key, "item", mapper),
            scala_value_decode_expr(ir, element, "item", mapper)
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
            .map(|field| scala_value_decode_expr(ir, &field.ty, value, mapper))
            .unwrap_or_else(|| format!("{value}.asInt")),
        TypeIr::Optional(element) => {
            format!(
                "if ({value}.isNull) None else Some({})",
                scala_value_decode_expr(ir, element, value, mapper)
            )
        }
    }
}

fn scala_collect_text_keys(
    ir: &ConfigIr,
    ty: &TypeIr,
    value: &str,
    indent: usize,
    mapper: &ScalaTypeMapper<'_>,
) -> String {
    if mapper.mapping(ty).is_some() {
        return String::new();
    }
    let pad = " ".repeat(indent);
    match ty {
        TypeIr::Text => format!("{pad}out += {value}"),
        TypeIr::Optional(element) => {
            let inner = scala_collect_text_keys(ir, element, "item", indent + 2, mapper);
            if inner.is_empty() {
                String::new()
            } else {
                format!("{pad}{value}.foreach {{ item =>\n{inner}\n{pad}}}")
            }
        }
        TypeIr::List(element) | TypeIr::Set(element) | TypeIr::Array { element, .. } => {
            let inner = scala_collect_text_keys(ir, element, "item", indent + 2, mapper);
            if inner.is_empty() {
                String::new()
            } else {
                format!("{pad}{value}.foreach {{ item =>\n{inner}\n{pad}}}")
            }
        }
        TypeIr::Map {
            key,
            value: element,
        } => {
            let key_inner = scala_collect_text_keys(ir, key, "key", indent + 2, mapper);
            let value_inner = scala_collect_text_keys(ir, element, "item", indent + 2, mapper);
            if key_inner.is_empty() && value_inner.is_empty() {
                String::new()
            } else {
                format!(
                    "{pad}{value}.foreach {{ case (key, item) =>\n{key_inner}\n{value_inner}\n{pad}}}"
                )
            }
        }
        TypeIr::Struct(_) => format!("{pad}{value}.collectTextKeys(out)"),
        TypeIr::Union(name) => format!("{pad}{name}.collectTextKeys({value}, out)"),
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
            .map(|field| scala_collect_text_keys(ir, &field.ty, value, indent, mapper))
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

fn scala_package_dir(out_dir: &Path, package: &str) -> Result<PathBuf> {
    let mut path = out_dir.to_path_buf();
    for segment in package.split('.') {
        if !is_scala_package_segment(segment) {
            return Err(SoraError::InvalidSchema(format!(
                "scala package `{package}` must use dot-separated identifier segments"
            )));
        }
        path.push(segment);
    }
    Ok(path)
}

fn is_scala_package_segment(segment: &str) -> bool {
    let mut chars = segment.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || first == '_')
        && chars.all(|value| value.is_ascii_alphanumeric() || value == '_')
}
