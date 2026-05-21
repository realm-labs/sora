use sora_diagnostics::{Result, SoraError};
use sora_schema::model::{
    CStandardSchema, CodegenSchema, CppStandardSchema, EnumReprSchema, ErlangCodegenSchema,
    ErlangEnumReprSchema, FieldSchema, IndexSchema, JavaScriptCodegenSchema, LanguageCodegenSchema,
    LuaCodegenSchema, LuaEnumReprSchema, LuaVersionSchema, ParserSchema, RuntimeFormatSchema,
    RustMapTypeSchema, SchemaFile, ScopeSchema, TableModeSchema, TableSchema, TableSourceSchema,
    TypeScriptCodegenSchema, UnionSchema, UnionVariantSchema,
};

use crate::{
    model::{
        AggregationIr, CCodegenIr, CStandardIr, CodegenIr, ConfigIr, CppCodegenIr, CppStandardIr,
        EnumIr, EnumReprIr, ErlangCodegenIr, ErlangEnumReprIr, FieldIr, IndexIr,
        JavaScriptCodegenIr, LanguageCodegenIr, LuaCodegenIr, LuaEnumReprIr, LuaVersionIr,
        ParserIr, RuntimeFormatIr, RustCodegenIr, RustMapTypeIr, ScopeIr, StructIr, TableIr,
        TableModeIr, TableSourceIr, TypeIr, TypeScriptCodegenIr, UnionIr, UnionVariantIr,
    },
    parse::parse_type,
    parser::ParserRegistry,
};

pub fn normalize_schema(schema: SchemaFile) -> Result<ConfigIr> {
    normalize_schema_with_parsers(schema, &ParserRegistry::builtin())
}

pub fn normalize_schema_with_parsers(
    schema: SchemaFile,
    parser_registry: &ParserRegistry,
) -> Result<ConfigIr> {
    Ok(ConfigIr {
        package: schema.package,
        codegen: CodegenIr::from(schema.codegen),
        enums: schema
            .enums
            .into_iter()
            .map(|item| {
                Ok(EnumIr {
                    name: item.name,
                    scope: ScopeIr::try_from(item.scope)?,
                    values: item.values,
                })
            })
            .collect::<Result<Vec<_>>>()?,
        structs: schema
            .structs
            .into_iter()
            .map(|item| {
                Ok(StructIr {
                    name: item.name,
                    scope: ScopeIr::try_from(item.scope)?,
                    fields: convert_fields_with_parsers(item.fields, parser_registry)?,
                })
            })
            .collect::<Result<Vec<_>>>()?,
        unions: schema
            .unions
            .into_iter()
            .map(|union| convert_union_with_parsers(union, parser_registry))
            .collect::<Result<Vec<_>>>()?,
        tables: schema
            .tables
            .into_iter()
            .map(|table| convert_table_with_parsers(table, parser_registry))
            .collect::<Result<Vec<_>>>()?,
    })
}

impl TryFrom<SchemaFile> for ConfigIr {
    type Error = SoraError;

    fn try_from(schema: SchemaFile) -> Result<Self> {
        normalize_schema(schema)
    }
}

impl From<CodegenSchema> for CodegenIr {
    fn from(value: CodegenSchema) -> Self {
        Self {
            rust: RustCodegenIr {
                runtime_format: RuntimeFormatIr::from(value.rust.runtime_format),
                map_type: match value.rust.map_type {
                    RustMapTypeSchema::Std => RustMapTypeIr::Std,
                    RustMapTypeSchema::FxHashMap => RustMapTypeIr::FxHashMap,
                },
            },
            kotlin: LanguageCodegenIr::from(value.kotlin),
            csharp: LanguageCodegenIr::from(value.csharp),
            java: LanguageCodegenIr::from(value.java),
            go: LanguageCodegenIr::from(value.go),
            dart: LanguageCodegenIr::from(value.dart),
            godot: LanguageCodegenIr::from(value.godot),
            c: CCodegenIr {
                runtime_format: RuntimeFormatIr::from(value.c.runtime_format),
                c_standard: CStandardIr::from(value.c.c_standard),
                prefix: value.c.prefix,
            },
            cpp: CppCodegenIr {
                runtime_format: RuntimeFormatIr::from(value.cpp.runtime_format),
                cpp_standard: CppStandardIr::from(value.cpp.cpp_standard),
                namespace: value.cpp.namespace,
            },
            typescript: TypeScriptCodegenIr::from(value.typescript),
            javascript: JavaScriptCodegenIr::from(value.javascript),
            erlang: ErlangCodegenIr::from(value.erlang),
            lua: LuaCodegenIr::from(value.lua),
            python: LanguageCodegenIr::from(value.python),
        }
    }
}

impl From<LanguageCodegenSchema> for LanguageCodegenIr {
    fn from(value: LanguageCodegenSchema) -> Self {
        Self {
            runtime_format: RuntimeFormatIr::from(value.runtime_format),
        }
    }
}

impl From<TypeScriptCodegenSchema> for TypeScriptCodegenIr {
    fn from(value: TypeScriptCodegenSchema) -> Self {
        Self {
            runtime_format: RuntimeFormatIr::from(value.runtime_format),
            enum_repr: EnumReprIr::from(value.enum_repr),
        }
    }
}

impl From<JavaScriptCodegenSchema> for JavaScriptCodegenIr {
    fn from(value: JavaScriptCodegenSchema) -> Self {
        Self {
            runtime_format: RuntimeFormatIr::from(value.runtime_format),
            enum_repr: EnumReprIr::from(value.enum_repr),
            emit_dts: value.emit_dts,
        }
    }
}

impl From<ErlangCodegenSchema> for ErlangCodegenIr {
    fn from(value: ErlangCodegenSchema) -> Self {
        Self {
            runtime_format: RuntimeFormatIr::from(value.runtime_format),
            enum_repr: ErlangEnumReprIr::from(value.enum_repr),
        }
    }
}

impl From<LuaCodegenSchema> for LuaCodegenIr {
    fn from(value: LuaCodegenSchema) -> Self {
        Self {
            runtime_format: RuntimeFormatIr::from(value.runtime_format),
            module: value.module,
            lua_version: LuaVersionIr::from(value.lua_version),
            enum_repr: LuaEnumReprIr::from(value.enum_repr),
        }
    }
}

impl From<RuntimeFormatSchema> for RuntimeFormatIr {
    fn from(value: RuntimeFormatSchema) -> Self {
        match value {
            RuntimeFormatSchema::Sora => Self::Sora,
            RuntimeFormatSchema::Json => Self::Json,
            RuntimeFormatSchema::SoraProtobuf => Self::SoraProtobuf,
            RuntimeFormatSchema::Cbor => Self::Cbor,
        }
    }
}

impl From<CStandardSchema> for CStandardIr {
    fn from(value: CStandardSchema) -> Self {
        match value {
            CStandardSchema::C99 => Self::C99,
            CStandardSchema::C11 => Self::C11,
            CStandardSchema::C17 => Self::C17,
            CStandardSchema::C23 => Self::C23,
        }
    }
}

impl From<CppStandardSchema> for CppStandardIr {
    fn from(value: CppStandardSchema) -> Self {
        match value {
            CppStandardSchema::Cpp11 => Self::Cpp11,
            CppStandardSchema::Cpp14 => Self::Cpp14,
            CppStandardSchema::Cpp17 => Self::Cpp17,
            CppStandardSchema::Cpp20 => Self::Cpp20,
            CppStandardSchema::Cpp23 => Self::Cpp23,
        }
    }
}

impl From<LuaVersionSchema> for LuaVersionIr {
    fn from(value: LuaVersionSchema) -> Self {
        match value {
            LuaVersionSchema::Lua51 => Self::Lua51,
            LuaVersionSchema::Lua52 => Self::Lua52,
            LuaVersionSchema::Lua53 => Self::Lua53,
            LuaVersionSchema::Lua54 => Self::Lua54,
            LuaVersionSchema::LuaJit => Self::LuaJit,
        }
    }
}

impl From<LuaEnumReprSchema> for LuaEnumReprIr {
    fn from(value: LuaEnumReprSchema) -> Self {
        match value {
            LuaEnumReprSchema::Integer => Self::Integer,
            LuaEnumReprSchema::String => Self::String,
        }
    }
}

impl From<EnumReprSchema> for EnumReprIr {
    fn from(value: EnumReprSchema) -> Self {
        match value {
            EnumReprSchema::Integer => Self::Integer,
            EnumReprSchema::String => Self::String,
        }
    }
}

impl From<ErlangEnumReprSchema> for ErlangEnumReprIr {
    fn from(value: ErlangEnumReprSchema) -> Self {
        match value {
            ErlangEnumReprSchema::Integer => Self::Integer,
            ErlangEnumReprSchema::Atom => Self::Atom,
        }
    }
}

impl TryFrom<UnionSchema> for UnionIr {
    type Error = SoraError;

    fn try_from(union: UnionSchema) -> Result<Self> {
        if union.tag.is_empty() {
            return Err(SoraError::InvalidSchema(format!(
                "union `{}` declares empty `tag`",
                union.name
            )));
        }

        Ok(Self {
            name: union.name,
            scope: ScopeIr::try_from(union.scope)?,
            tag: union.tag,
            variants: union
                .variants
                .into_iter()
                .map(UnionVariantIr::try_from)
                .collect::<Result<Vec<_>>>()?,
        })
    }
}

impl TryFrom<UnionVariantSchema> for UnionVariantIr {
    type Error = SoraError;

    fn try_from(variant: UnionVariantSchema) -> Result<Self> {
        Ok(Self {
            name: variant.name,
            scope: ScopeIr::try_from(variant.scope)?,
            fields: convert_fields(variant.fields)?,
        })
    }
}

impl TryFrom<TableSchema> for TableIr {
    type Error = SoraError;

    fn try_from(table: TableSchema) -> Result<Self> {
        Ok(Self {
            name: table.name,
            scope: ScopeIr::try_from(table.scope)?,
            mode: table.mode.into(),
            key: table.key,
            source: table.source.map(Into::into),
            fields: convert_fields(table.fields)?,
            indexes: table.indexes.into_iter().map(IndexIr::from).collect(),
        })
    }
}

impl TryFrom<FieldSchema> for FieldIr {
    type Error = SoraError;

    fn try_from(field: FieldSchema) -> Result<Self> {
        convert_field_with_parsers(field, &ParserRegistry::builtin())
    }
}

impl From<ParserSchema> for ParserIr {
    fn from(parser: ParserSchema) -> Self {
        Self {
            kind: parser.kind,
            options: parser.options,
        }
    }
}

impl TryFrom<ScopeSchema> for ScopeIr {
    type Error = SoraError;

    fn try_from(scope: ScopeSchema) -> Result<Self> {
        if scope.values.is_empty() {
            return Err(SoraError::InvalidSchema(
                "scope must contain at least one value".to_owned(),
            ));
        }

        let mut values = Vec::with_capacity(scope.values.len());
        for value in scope.values {
            let value = value.trim();
            if value.is_empty() {
                return Err(SoraError::InvalidSchema(
                    "scope values must not be empty".to_owned(),
                ));
            }
            if !value
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
            {
                return Err(SoraError::InvalidSchema(format!(
                    "scope `{value}` must contain only ASCII letters, digits, `_`, or `-`"
                )));
            }
            if !values.iter().any(|item| item == value) {
                values.push(value.to_owned());
            }
        }

        Ok(Self { values })
    }
}

impl From<TableModeSchema> for TableModeIr {
    fn from(mode: TableModeSchema) -> Self {
        match mode {
            TableModeSchema::List => Self::List,
            TableModeSchema::Map => Self::Map,
            TableModeSchema::Singleton => Self::Singleton,
        }
    }
}

impl From<IndexSchema> for IndexIr {
    fn from(index: IndexSchema) -> Self {
        Self {
            name: index.name,
            fields: index.fields,
            unique: index.unique,
        }
    }
}

impl From<TableSourceSchema> for TableSourceIr {
    fn from(source: TableSourceSchema) -> Self {
        Self {
            format: source.format,
            file: source.file,
            sheet: source.sheet,
        }
    }
}

fn convert_fields(fields: Vec<FieldSchema>) -> Result<Vec<FieldIr>> {
    convert_fields_with_parsers(fields, &ParserRegistry::builtin())
}

fn convert_fields_with_parsers(
    fields: Vec<FieldSchema>,
    parser_registry: &ParserRegistry,
) -> Result<Vec<FieldIr>> {
    fields
        .into_iter()
        .map(|field| convert_field_with_parsers(field, parser_registry))
        .collect()
}

fn convert_field_with_parsers(
    field: FieldSchema,
    parser_registry: &ParserRegistry,
) -> Result<FieldIr> {
    let aggregation = match (field.source_table, field.parent_key, field.child_key) {
        (None, None, None) => None,
        (Some(source_table), Some(parent_key), Some(child_key)) => Some(AggregationIr {
            source_table,
            parent_key,
            child_key,
            order_by: field.order_by,
        }),
        _ => {
            return Err(SoraError::InvalidSchema(format!(
                "field `{}` has incomplete aggregation metadata",
                field.name
            )));
        }
    };

    let ty = parse_type(&field.ty)?;
    validate_length_constraint(&field.name, &ty, field.length)?;
    if field.default.is_some() && aggregation.is_some() {
        return Err(SoraError::InvalidSchema(format!(
            "field `{}` declares both `default` and aggregation metadata",
            field.name
        )));
    }
    parser_registry.validate_field_parser(&field.name, &ty, field.parser.as_ref())?;

    Ok(FieldIr {
        name: field.name,
        ty,
        scope: ScopeIr::try_from(field.scope)?,
        key: field.key,
        comment: field.comment,
        required: field.required.unwrap_or(false),
        default: field.default,
        range: field.range,
        length: field.length,
        parser: field.parser.map(ParserIr::from),
        aggregation,
    })
}

fn convert_union_with_parsers(
    union: UnionSchema,
    parser_registry: &ParserRegistry,
) -> Result<UnionIr> {
    if union.tag.is_empty() {
        return Err(SoraError::InvalidSchema(format!(
            "union `{}` declares empty `tag`",
            union.name
        )));
    }

    Ok(UnionIr {
        name: union.name,
        scope: ScopeIr::try_from(union.scope)?,
        tag: union.tag,
        variants: union
            .variants
            .into_iter()
            .map(|variant| convert_union_variant_with_parsers(variant, parser_registry))
            .collect::<Result<Vec<_>>>()?,
    })
}

fn convert_union_variant_with_parsers(
    variant: UnionVariantSchema,
    parser_registry: &ParserRegistry,
) -> Result<UnionVariantIr> {
    Ok(UnionVariantIr {
        name: variant.name,
        scope: ScopeIr::try_from(variant.scope)?,
        fields: convert_fields_with_parsers(variant.fields, parser_registry)?,
    })
}

fn convert_table_with_parsers(
    table: TableSchema,
    parser_registry: &ParserRegistry,
) -> Result<TableIr> {
    Ok(TableIr {
        name: table.name,
        scope: ScopeIr::try_from(table.scope)?,
        mode: table.mode.into(),
        key: table.key,
        source: table.source.map(Into::into),
        fields: convert_fields_with_parsers(table.fields, parser_registry)?,
        indexes: table.indexes.into_iter().map(IndexIr::from).collect(),
    })
}

fn validate_length_constraint(
    field_name: &str,
    ty: &TypeIr,
    length: Option<[usize; 2]>,
) -> Result<()> {
    let Some([min, max]) = length else {
        return Ok(());
    };
    if min > max {
        return Err(SoraError::InvalidSchema(format!(
            "field `{field_name}` declares invalid `length` [{min}, {max}]"
        )));
    }

    match ty {
        TypeIr::String | TypeIr::List(_) | TypeIr::Array { .. } => Ok(()),
        TypeIr::Optional(inner) => validate_length_constraint(field_name, inner, length),
        _ => Err(SoraError::InvalidSchema(format!(
            "field `{field_name}` declares `length` but type `{ty}` is not string, list, or array"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        CStandardIr, CppStandardIr, EnumReprIr, ErlangEnumReprIr, LuaEnumReprIr, LuaVersionIr,
        RuntimeFormatIr, TableModeIr, TypeIr,
    };

    #[test]
    fn normalizes_schema() {
        let schema: SchemaFile = toml::from_str(
            r#"
package = "game_config"

[[enums]]
name = "ItemType"
values = ["Weapon"]

[[tables]]
name = "Item"
mode = "map"
key = "id"

[[tables.fields]]
name = "id"
type = "i32"
key = true
required = true

[[tables.fields]]
name = "tags"
type = "list<string>"
parser = { kind = "split", separator = "|" }
default = "starter"
length = [1, 3]
"#,
        )
        .unwrap();

        let ir = normalize_schema(schema).unwrap();
        assert_eq!(ir.package, "game_config");
        assert_eq!(ir.codegen.rust.runtime_format, RuntimeFormatIr::Sora);
        assert_eq!(ir.codegen.kotlin.runtime_format, RuntimeFormatIr::Sora);
        assert_eq!(ir.codegen.dart.runtime_format, RuntimeFormatIr::Sora);
        assert_eq!(ir.codegen.godot.runtime_format, RuntimeFormatIr::Sora);
        assert_eq!(ir.codegen.c.runtime_format, RuntimeFormatIr::Sora);
        assert_eq!(ir.codegen.c.c_standard, CStandardIr::C11);
        assert_eq!(ir.codegen.cpp.runtime_format, RuntimeFormatIr::Sora);
        assert_eq!(ir.codegen.cpp.cpp_standard, CppStandardIr::Cpp17);
        assert_eq!(ir.codegen.typescript.runtime_format, RuntimeFormatIr::Sora);
        assert_eq!(ir.codegen.typescript.enum_repr, EnumReprIr::String);
        assert_eq!(ir.codegen.javascript.runtime_format, RuntimeFormatIr::Sora);
        assert_eq!(ir.codegen.javascript.enum_repr, EnumReprIr::String);
        assert!(ir.codegen.javascript.emit_dts);
        assert_eq!(ir.codegen.erlang.runtime_format, RuntimeFormatIr::Sora);
        assert_eq!(ir.codegen.erlang.enum_repr, ErlangEnumReprIr::Atom);
        assert_eq!(ir.codegen.lua.runtime_format, RuntimeFormatIr::Sora);
        assert_eq!(ir.codegen.lua.lua_version, LuaVersionIr::Lua54);
        assert_eq!(ir.codegen.lua.enum_repr, LuaEnumReprIr::String);
        assert_eq!(ir.enums[0].name, "ItemType");
        assert_eq!(ir.tables[0].mode, TableModeIr::Map);
        assert!(ir.tables[0].fields[0].required);
        assert_eq!(ir.tables[0].fields[0].ty, TypeIr::I32);
        let parser = ir.tables[0].fields[1].parser.as_ref().unwrap();
        assert_eq!(parser.kind, "split");
        assert_eq!(parser.options["separator"], "|");
        assert_eq!(ir.tables[0].fields[1].default.as_deref(), Some("starter"));
        assert_eq!(ir.tables[0].fields[1].length, Some([1, 3]));
    }

    #[test]
    fn normalizes_tuple_struct_parser() {
        let schema: SchemaFile = toml::from_str(
            r#"
package = "game_config"

[[structs]]
name = "Vec3"

[[structs.fields]]
name = "x"
type = "f32"

[[structs.fields]]
name = "y"
type = "f32"

[[structs.fields]]
name = "z"
type = "f32"

[[tables]]
name = "Spawn"
mode = "list"

[[tables.fields]]
name = "pos"
type = "struct<Vec3>"
parser = { kind = "tuple" }
"#,
        )
        .unwrap();

        let ir = normalize_schema(schema).unwrap();
        assert_eq!(
            ir.tables[0].fields[0]
                .parser
                .as_ref()
                .map(|parser| parser.kind.as_str()),
            Some("tuple")
        );
    }

    #[test]
    fn normalizes_tuple_list_parser() {
        let schema: SchemaFile = toml::from_str(
            r#"
package = "game_config"

[[structs]]
name = "ResourceCost"

[[structs.fields]]
name = "kind"
type = "string"

[[structs.fields]]
name = "id"
type = "i32"

[[structs.fields]]
name = "count"
type = "i32"

[[tables]]
name = "Recipe"
mode = "list"

[[tables.fields]]
name = "materials"
type = "list<ResourceCost>"
parser = { kind = "tuple_list", item_separator = ";", separator = "," }
"#,
        )
        .unwrap();

        let ir = normalize_schema(schema).unwrap();
        let parser = ir.tables[0].fields[0].parser.as_ref().unwrap();
        assert_eq!(parser.kind, "tuple_list");
        assert_eq!(parser.options["item_separator"], ";");
        assert_eq!(parser.options["separator"], ",");
    }

    #[test]
    fn default_collections_do_not_need_parser_metadata() {
        let schema: SchemaFile = toml::from_str(
            r#"
package = "game_config"

[[tables]]
name = "Item"
mode = "list"

[[tables.fields]]
name = "tags"
type = "list<string>"
"#,
        )
        .unwrap();

        let ir = normalize_schema(schema).unwrap();
        assert_eq!(ir.tables[0].fields[0].parser, None);
    }

    #[test]
    fn rejects_invalid_parser_metadata() {
        let scalar_split: SchemaFile = toml::from_str(
            r#"
package = "game_config"

[[tables]]
name = "Item"
mode = "list"

[[tables.fields]]
name = "name"
type = "string"
parser = { kind = "split", separator = "|" }
"#,
        )
        .unwrap();
        assert!(matches!(
            normalize_schema(scalar_split).unwrap_err(),
            SoraError::InvalidSchema(message) if message.contains("is not list or array")
        ));

        let scalar_tuple_list: SchemaFile = toml::from_str(
            r#"
package = "game_config"

[[tables]]
name = "Recipe"
mode = "list"

[[tables.fields]]
name = "materials"
type = "list<string>"
parser = { kind = "tuple_list" }
"#,
        )
        .unwrap();
        assert!(matches!(
            normalize_schema(scalar_tuple_list).unwrap_err(),
            SoraError::InvalidSchema(message) if message.contains("not list or array of struct")
        ));

        let unknown_parser: SchemaFile = toml::from_str(
            r#"
package = "game_config"

[[tables]]
name = "Item"
mode = "list"

[[tables.fields]]
name = "name"
type = "string"
parser = { kind = "lua" }
"#,
        )
        .unwrap();
        assert!(matches!(
            normalize_schema(unknown_parser).unwrap_err(),
            SoraError::InvalidSchema(message) if message.contains("unsupported parser")
        ));
    }

    #[test]
    fn validates_length_constraints() {
        let invalid_type: SchemaFile = toml::from_str(
            r#"
package = "game_config"

[[tables]]
name = "Item"
mode = "list"

[[tables.fields]]
name = "id"
type = "i32"
length = [1, 4]
"#,
        )
        .unwrap();
        assert!(matches!(
            normalize_schema(invalid_type).unwrap_err(),
            SoraError::InvalidSchema(message) if message.contains("declares `length`")
        ));

        let invalid_range: SchemaFile = toml::from_str(
            r#"
package = "game_config"

[[tables]]
name = "Item"
mode = "list"

[[tables.fields]]
name = "name"
type = "string"
length = [4, 1]
"#,
        )
        .unwrap();
        assert!(matches!(
            normalize_schema(invalid_range).unwrap_err(),
            SoraError::InvalidSchema(message) if message.contains("invalid `length`")
        ));
    }

    #[test]
    fn rejects_invalid_tuple_parser_metadata() {
        let scalar_parser: SchemaFile = toml::from_str(
            r#"
package = "game_config"

[[tables]]
name = "Spawn"
mode = "list"

[[tables.fields]]
name = "pos"
type = "string"
parser = { kind = "tuple" }
"#,
        )
        .unwrap();
        assert!(matches!(
            normalize_schema(scalar_parser).unwrap_err(),
            SoraError::InvalidSchema(message) if message.contains("is not struct")
        ));
    }

    #[test]
    fn aggregation_list_fields_do_not_need_separator_metadata() {
        let schema: SchemaFile = toml::from_str(
            r#"
package = "game_config"

[[structs]]
name = "Reward"

[[structs.fields]]
name = "count"
type = "i32"

[[tables]]
name = "Item"
mode = "map"
key = "id"

[[tables.fields]]
name = "id"
type = "i32"

[[tables.fields]]
name = "rewards"
type = "list<Reward>"
source_table = "ItemReward"
parent_key = "id"
child_key = "item_id"

[[tables]]
name = "ItemReward"
mode = "list"
"#,
        )
        .unwrap();

        let ir = normalize_schema(schema).unwrap();
        assert!(ir.tables[0].fields[1].aggregation.is_some());
        assert_eq!(ir.tables[0].fields[1].parser, None);
    }

    #[test]
    fn rejects_default_on_aggregation_fields() {
        let schema: SchemaFile = toml::from_str(
            r#"
package = "game_config"

[[structs]]
name = "Reward"

[[tables]]
name = "Item"
mode = "map"
key = "id"

[[tables.fields]]
name = "id"
type = "i32"

[[tables.fields]]
name = "rewards"
type = "list<Reward>"
source_table = "ItemReward"
parent_key = "id"
child_key = "item_id"
default = "[]"

[[tables]]
name = "ItemReward"
mode = "list"
"#,
        )
        .unwrap();

        assert!(matches!(
            normalize_schema(schema).unwrap_err(),
            SoraError::InvalidSchema(message)
                if message.contains("declares both `default` and aggregation")
        ));
    }
}
