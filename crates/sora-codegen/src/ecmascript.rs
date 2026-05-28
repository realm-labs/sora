use heck::ToLowerCamelCase;
use serde::Serialize;
use sora_ir::model::{ConfigIr, TypeIr};

use crate::{
    model::{
        BaseField, BaseImport, BaseIndex, BaseModel, BaseRecord, BaseTable, BaseUnion,
        BaseUnionVariant,
    },
    options::EnumRepr,
    type_mapping::{TypeMapping, TypeMappingContext, TypeMappingRegistry},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EcmaScriptTarget {
    TypeScript,
    JavaScript,
}

#[derive(Debug, Clone, Serialize)]
pub struct EcmaScriptOptionsView {
    pub import_ext: &'static str,
    pub enum_is_integer: bool,
    pub emit_dts: bool,
}

impl EcmaScriptOptionsView {
    pub fn new(target: EcmaScriptTarget, enum_repr: EnumRepr, emit_dts: bool) -> Self {
        Self {
            import_ext: ".js",
            enum_is_integer: enum_repr == EnumRepr::Integer,
            emit_dts: target == EcmaScriptTarget::JavaScript && emit_dts,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct EcmaScriptModel {
    pub package: String,
    pub schema_fingerprint: String,
    pub enums: Vec<EcmaScriptEnum>,
    pub unions: Vec<EcmaScriptUnion>,
    pub records: Vec<EcmaScriptRecord>,
    pub tables: Vec<EcmaScriptTable>,
    pub modules: Vec<String>,
    pub has_localization: bool,
    pub locales: Vec<String>,
    pub default_locale: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct EcmaScriptEnum {
    pub name: String,
    pub snake_name: String,
    pub values: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EcmaScriptUnion {
    pub pascal_name: String,
    pub snake_name: String,
    pub tag: String,
    pub variants: Vec<EcmaScriptUnionVariant>,
    pub imports: Vec<EcmaScriptImport>,
    pub custom_imports: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EcmaScriptUnionVariant {
    pub raw_name: String,
    pub name: String,
    pub fields: Vec<EcmaScriptField>,
    pub has_text_keys: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct EcmaScriptRecord {
    pub pascal_name: String,
    pub snake_name: String,
    pub imports: Vec<EcmaScriptImport>,
    pub custom_imports: Vec<String>,
    pub fields: Vec<EcmaScriptField>,
    pub uses_text_key: bool,
    pub table: Option<EcmaScriptTable>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EcmaScriptImport {
    pub module: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct EcmaScriptTable {
    pub name: String,
    pub pascal_name: String,
    pub camel_name: String,
    pub snake_name: String,
    pub mode: String,
    pub row_type: String,
    pub key_name: Option<String>,
    pub key_field_name: Option<String>,
    pub key_type: Option<String>,
    pub unique_indexes: Vec<EcmaScriptIndex>,
    pub non_unique_indexes: Vec<EcmaScriptIndex>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EcmaScriptIndex {
    pub name: String,
    pub pascal_name: String,
    pub camel_name: String,
    pub field_name: String,
    pub param_camel_name: String,
    pub param_type: String,
    pub key_type: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct EcmaScriptField {
    pub raw_name: String,
    pub name: String,
    pub type_name: String,
    pub decode: String,
    pub value_decode: String,
    pub collect_text_keys: String,
    pub imports: Vec<String>,
    pub comment: Option<String>,
}

impl EcmaScriptModel {
    pub fn from_base_model(
        target: &str,
        ir: &ConfigIr,
        model: BaseModel,
        mappings: &TypeMappingRegistry,
    ) -> Self {
        let mapper = EcmaScriptTypeMapper::new(target, ir, mappings);
        let tables = model
            .tables
            .into_iter()
            .map(|item| ecmascript_table(ir, item, &mapper))
            .collect::<Vec<_>>();
        Self {
            package: model.package,
            schema_fingerprint: model.schema_fingerprint,
            enums: model
                .enums
                .into_iter()
                .map(|item| EcmaScriptEnum {
                    name: item.pascal_name,
                    snake_name: item.snake_name,
                    values: item.values,
                })
                .collect(),
            unions: model
                .unions
                .into_iter()
                .map(|item| ecmascript_union(ir, item, &mapper))
                .collect(),
            records: model
                .records
                .into_iter()
                .map(|item| {
                    let table = tables
                        .iter()
                        .find(|table| table.row_type == item.pascal_name)
                        .cloned();
                    ecmascript_record(ir, item, table, &mapper)
                })
                .collect(),
            tables,
            modules: model.modules,
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

fn ecmascript_union(
    ir: &ConfigIr,
    union: BaseUnion,
    mapper: &EcmaScriptTypeMapper<'_>,
) -> EcmaScriptUnion {
    let variants = union
        .variants
        .into_iter()
        .map(|variant| ecmascript_variant(ir, variant, mapper))
        .collect::<Vec<_>>();
    let custom_imports =
        collect_ecmascript_imports(variants.iter().flat_map(|variant| &variant.fields));
    EcmaScriptUnion {
        pascal_name: union.pascal_name,
        snake_name: union.snake_name,
        tag: union.tag.to_lower_camel_case(),
        variants,
        imports: union.imports.into_iter().map(ecmascript_import).collect(),
        custom_imports,
    }
}

fn ecmascript_variant(
    ir: &ConfigIr,
    variant: BaseUnionVariant,
    mapper: &EcmaScriptTypeMapper<'_>,
) -> EcmaScriptUnionVariant {
    let fields = variant
        .fields
        .into_iter()
        .map(|field| ecmascript_field(ir, field, mapper))
        .collect::<Vec<_>>();
    let has_text_keys = fields
        .iter()
        .any(|field| !field.collect_text_keys.is_empty());
    EcmaScriptUnionVariant {
        raw_name: variant.name,
        name: variant.pascal_name,
        fields,
        has_text_keys,
    }
}

fn ecmascript_record(
    ir: &ConfigIr,
    record: BaseRecord,
    table: Option<EcmaScriptTable>,
    mapper: &EcmaScriptTypeMapper<'_>,
) -> EcmaScriptRecord {
    let fields = record
        .fields
        .into_iter()
        .map(|field| ecmascript_field(ir, field, mapper))
        .collect::<Vec<_>>();
    let uses_text_key = fields
        .iter()
        .any(|field| field.type_name.contains("TextKey"));
    EcmaScriptRecord {
        pascal_name: record.pascal_name,
        snake_name: record.snake_name,
        imports: record.imports.into_iter().map(ecmascript_import).collect(),
        custom_imports: collect_ecmascript_imports(fields.iter()),
        fields,
        uses_text_key,
        table,
    }
}

fn ecmascript_table(
    ir: &ConfigIr,
    table: BaseTable,
    mapper: &EcmaScriptTypeMapper<'_>,
) -> EcmaScriptTable {
    let row_type = table.pascal_name.clone();
    let key_type = table
        .key_field
        .as_ref()
        .map(|field| mapper.type_name(&field.ty));
    let key_field_name = table
        .key_field
        .as_ref()
        .map(|field| field.camel_name.clone());

    EcmaScriptTable {
        name: table.name,
        pascal_name: table.pascal_name,
        camel_name: table.camel_name,
        snake_name: table.snake_name,
        mode: table.mode_name,
        row_type,
        key_name: table.key_name,
        key_field_name,
        key_type,
        unique_indexes: table
            .unique_indexes
            .into_iter()
            .map(|index| ecmascript_index(ir, index, mapper))
            .collect(),
        non_unique_indexes: table
            .non_unique_indexes
            .into_iter()
            .map(|index| ecmascript_index(ir, index, mapper))
            .collect(),
    }
}

fn ecmascript_index(
    _ir: &ConfigIr,
    index: BaseIndex,
    mapper: &EcmaScriptTypeMapper<'_>,
) -> EcmaScriptIndex {
    let key_type = mapper.type_name(&index.field.ty);
    EcmaScriptIndex {
        name: index.snake_name,
        pascal_name: index.pascal_name,
        camel_name: index.camel_name,
        field_name: index.field.camel_name.clone(),
        param_camel_name: index.field.camel_name,
        param_type: key_type.clone(),
        key_type,
    }
}

fn ecmascript_field(
    ir: &ConfigIr,
    field: BaseField,
    mapper: &EcmaScriptTypeMapper<'_>,
) -> EcmaScriptField {
    let collect_text_keys = ecmascript_collect_text_keys(
        ir,
        &field.ty,
        &format!("value.{}", field.camel_name),
        mapper,
    );
    EcmaScriptField {
        raw_name: field.raw_name,
        name: field.camel_name,
        type_name: mapper.type_name(&field.ty),
        decode: ecmascript_decode_expr(ir, &field.ty, mapper),
        value_decode: ecmascript_value_decode_expr(ir, &field.ty, "__VALUE__", mapper),
        collect_text_keys,
        imports: mapper.imports(&field.ty),
        comment: field.comment,
    }
}

fn collect_ecmascript_imports<'a>(
    fields: impl Iterator<Item = &'a EcmaScriptField>,
) -> Vec<String> {
    let mut imports = fields
        .flat_map(|field| field.imports.iter().cloned())
        .collect::<Vec<_>>();
    imports.sort();
    imports.dedup();
    imports
}

fn ecmascript_import(import: BaseImport) -> EcmaScriptImport {
    EcmaScriptImport {
        module: import.module,
        name: import.name,
    }
}

pub fn ecmascript_type_name(ir: &ConfigIr, ty: &TypeIr) -> String {
    match ty {
        TypeIr::Bool => "boolean".to_owned(),
        TypeIr::I8 | TypeIr::U8 | TypeIr::I16 | TypeIr::U16 | TypeIr::I32 | TypeIr::U32 => {
            "number".to_owned()
        }
        TypeIr::I64 | TypeIr::Duration => "bigint".to_owned(),
        TypeIr::DateTime => "Date".to_owned(),
        TypeIr::F32 | TypeIr::F64 => "number".to_owned(),
        TypeIr::String => "string".to_owned(),
        TypeIr::Text => "TextKey".to_owned(),
        TypeIr::Enum(name) | TypeIr::Struct(name) | TypeIr::Union(name) => name.clone(),
        TypeIr::List(element) | TypeIr::Set(element) | TypeIr::Array { element, .. } => {
            format!("{}[]", array_element_type(ir, element))
        }
        TypeIr::Map { key, value } => format!(
            "Map<{}, {}>",
            ecmascript_type_name(ir, key),
            ecmascript_type_name(ir, value)
        ),
        TypeIr::Ref { table, field } => ref_type(ir, table, field)
            .map(|ty| ecmascript_type_name(ir, ty))
            .unwrap_or_else(|| "number".to_owned()),
        TypeIr::Optional(element) => format!("{} | undefined", ecmascript_type_name(ir, element)),
    }
}

struct EcmaScriptTypeMapper<'a> {
    target: &'a str,
    ir: &'a ConfigIr,
    mappings: &'a TypeMappingRegistry,
}

impl<'a> EcmaScriptTypeMapper<'a> {
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
            TypeIr::Bool => "boolean".to_owned(),
            TypeIr::I8 | TypeIr::U8 | TypeIr::I16 | TypeIr::U16 | TypeIr::I32 | TypeIr::U32 => {
                "number".to_owned()
            }
            TypeIr::I64 | TypeIr::Duration => "bigint".to_owned(),
            TypeIr::DateTime => "Date".to_owned(),
            TypeIr::F32 | TypeIr::F64 => "number".to_owned(),
            TypeIr::String => "string".to_owned(),
            TypeIr::Text => "TextKey".to_owned(),
            TypeIr::Enum(name) | TypeIr::Struct(name) | TypeIr::Union(name) => name.clone(),
            TypeIr::List(element) | TypeIr::Set(element) | TypeIr::Array { element, .. } => {
                format!("{}[]", self.array_element_type(element))
            }
            TypeIr::Map { key, value } => {
                format!("Map<{}, {}>", self.type_name(key), self.type_name(value))
            }
            TypeIr::Ref { table, field } => ref_type(self.ir, table, field)
                .map(|ty| self.type_name(ty))
                .unwrap_or_else(|| "number".to_owned()),
            TypeIr::Optional(element) => format!("{} | undefined", self.type_name(element)),
        }
    }

    fn array_element_type(&self, ty: &TypeIr) -> String {
        let name = self.type_name(ty);
        if name.contains(" | ") {
            format!("({name})")
        } else {
            name
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

fn ecmascript_decode_expr(ir: &ConfigIr, ty: &TypeIr, mapper: &EcmaScriptTypeMapper<'_>) -> String {
    match ty {
        TypeIr::Bool => "reader.readBool()".to_owned(),
        TypeIr::I8 | TypeIr::I16 | TypeIr::I32 => "reader.readI32()".to_owned(),
        TypeIr::U8 | TypeIr::U16 | TypeIr::U32 => "reader.readU32()".to_owned(),
        TypeIr::I64 | TypeIr::Duration => "reader.readI64()".to_owned(),
        TypeIr::DateTime => "new Date(Number(reader.readI64()))".to_owned(),
        TypeIr::F32 => "reader.readF32()".to_owned(),
        TypeIr::F64 => "reader.readF64()".to_owned(),
        TypeIr::String => "reader.readString()".to_owned(),
        TypeIr::Text => "new TextKey(reader.readString())".to_owned(),
        TypeIr::Enum(name) | TypeIr::Struct(name) | TypeIr::Union(name) => {
            mapper.wrap_decode(ty, format!("decode{name}(reader)"))
        }
        TypeIr::List(element) | TypeIr::Set(element) | TypeIr::Array { element, .. } => {
            format!(
                "reader.readList(() => {})",
                ecmascript_decode_expr(ir, element, mapper)
            )
        }
        TypeIr::Map { key, value } => format!(
            "reader.readMap(() => {}, () => {})",
            ecmascript_decode_expr(ir, key, mapper),
            ecmascript_decode_expr(ir, value, mapper)
        ),
        TypeIr::Ref { table, field } => ref_type(ir, table, field)
            .map(|ty| ecmascript_decode_expr(ir, ty, mapper))
            .unwrap_or_else(|| "reader.readI32()".to_owned()),
        TypeIr::Optional(element) => {
            format!(
                "reader.readOptional(() => {})",
                ecmascript_decode_expr(ir, element, mapper)
            )
        }
    }
}

fn ecmascript_value_decode_expr(
    ir: &ConfigIr,
    ty: &TypeIr,
    value: &str,
    mapper: &EcmaScriptTypeMapper<'_>,
) -> String {
    match ty {
        TypeIr::Bool => format!("{value}.asBool()"),
        TypeIr::I8 | TypeIr::U8 | TypeIr::I16 | TypeIr::U16 | TypeIr::I32 | TypeIr::U32 => {
            format!("{value}.asInt()")
        }
        TypeIr::I64 | TypeIr::Duration => format!("{value}.asBigInt()"),
        TypeIr::DateTime => format!("new Date(Number({value}.asBigInt()))"),
        TypeIr::F32 | TypeIr::F64 => format!("{value}.asNumber()"),
        TypeIr::String => format!("{value}.asString()"),
        TypeIr::Text => format!("new TextKey({value}.asString())"),
        TypeIr::Enum(name) | TypeIr::Struct(name) | TypeIr::Union(name) => {
            mapper.wrap_value_decode(ty, format!("decode{name}Value({value})"))
        }
        TypeIr::List(element) | TypeIr::Set(element) | TypeIr::Array { element, .. } => {
            let item_decode = ecmascript_value_decode_expr(ir, element, "item", mapper);
            format!("{value}.asList((item) => {item_decode})")
        }
        TypeIr::Map {
            key,
            value: element,
        } => {
            let key_decode = ecmascript_value_decode_expr(ir, key, "item", mapper);
            let value_decode = ecmascript_value_decode_expr(ir, element, "item", mapper);
            format!("{value}.asMap((item) => {key_decode}, (item) => {value_decode})")
        }
        TypeIr::Ref { table, field } => ref_type(ir, table, field)
            .map(|ty| ecmascript_value_decode_expr(ir, ty, value, mapper))
            .unwrap_or_else(|| format!("{value}.asInt()")),
        TypeIr::Optional(element) => {
            let item_decode = ecmascript_value_decode_expr(ir, element, value, mapper);
            format!("{value}.isNull() ? undefined : {item_decode}")
        }
    }
}

fn ecmascript_collect_text_keys(
    ir: &ConfigIr,
    ty: &TypeIr,
    value: &str,
    mapper: &EcmaScriptTypeMapper<'_>,
) -> String {
    if mapper.mapping(ty).is_some() {
        return String::new();
    }
    match ty {
        TypeIr::Text => format!("out.push({value});"),
        TypeIr::Optional(element) => {
            let inner = ecmascript_collect_text_keys(ir, element, "item", mapper);
            if inner.is_empty() {
                String::new()
            } else {
                format!("if ({value} !== undefined) {{ const item = {value}; {inner} }}")
            }
        }
        TypeIr::List(element) | TypeIr::Set(element) | TypeIr::Array { element, .. } => {
            let inner = ecmascript_collect_text_keys(ir, element, "item", mapper);
            if inner.is_empty() {
                String::new()
            } else {
                format!("for (const item of {value}) {{ {inner} }}")
            }
        }
        TypeIr::Map {
            key,
            value: element,
        } => {
            let key_inner = ecmascript_collect_text_keys(ir, key, "mapKey", mapper);
            let value_inner = ecmascript_collect_text_keys(ir, element, "item", mapper);
            if key_inner.is_empty() && value_inner.is_empty() {
                String::new()
            } else {
                format!("for (const [mapKey, item] of {value}) {{ {key_inner} {value_inner} }}")
            }
        }
        TypeIr::Struct(name) => format!("collect{name}TextKeys({value}, out);"),
        TypeIr::Union(name) => format!("collect{name}TextKeys({value}, out);"),
        TypeIr::Ref { table, field } => ref_type(ir, table, field)
            .map(|ty| ecmascript_collect_text_keys(ir, ty, value, mapper))
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

fn array_element_type(ir: &ConfigIr, ty: &TypeIr) -> String {
    let name = ecmascript_type_name(ir, ty);
    if name.contains(" | ") {
        format!("({name})")
    } else {
        name
    }
}

fn ref_type<'a>(ir: &'a ConfigIr, table_name: &str, field_name: &str) -> Option<&'a TypeIr> {
    ir.tables
        .iter()
        .find(|table| table.name == table_name)
        .and_then(|table| table.fields.iter().find(|field| field.name == field_name))
        .map(|field| &field.ty)
}
