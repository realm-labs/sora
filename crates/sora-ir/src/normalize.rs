use sora_diagnostics::{Result, SoraError};
use sora_schema::model::{
    CodegenSchema, EnumReprSchema, ErlangCodegenSchema, ErlangEnumReprSchema, FieldSchema,
    IndexSchema, JavaScriptCodegenSchema, LanguageCodegenSchema, LuaCodegenSchema,
    LuaEnumReprSchema, LuaVersionSchema, RuntimeFormatSchema, RustMapTypeSchema, SchemaFile,
    TableModeSchema, TableSchema, TableSourceSchema, TypeScriptCodegenSchema, UnionSchema,
    UnionVariantSchema,
};

use crate::{
    model::{
        AggregationIr, CodegenIr, ConfigIr, EnumIr, EnumReprIr, ErlangCodegenIr, ErlangEnumReprIr,
        FieldIr, IndexIr, JavaScriptCodegenIr, LanguageCodegenIr, LuaCodegenIr, LuaEnumReprIr,
        LuaVersionIr, RuntimeFormatIr, RustCodegenIr, RustMapTypeIr, StructIr, TableIr,
        TableModeIr, TableSourceIr, TypeIr, TypeScriptCodegenIr, UnionIr, UnionVariantIr,
    },
    parse::parse_type,
};

pub fn normalize_schema(schema: SchemaFile) -> Result<ConfigIr> {
    ConfigIr::try_from(schema)
}

impl TryFrom<SchemaFile> for ConfigIr {
    type Error = SoraError;

    fn try_from(schema: SchemaFile) -> Result<Self> {
        Ok(Self {
            package: schema.package,
            codegen: CodegenIr::from(schema.codegen),
            enums: schema
                .enums
                .into_iter()
                .map(|item| EnumIr {
                    name: item.name,
                    values: item.values,
                })
                .collect(),
            structs: schema
                .structs
                .into_iter()
                .map(|item| {
                    Ok(StructIr {
                        name: item.name,
                        fields: convert_fields(item.fields)?,
                    })
                })
                .collect::<Result<Vec<_>>>()?,
            unions: schema
                .unions
                .into_iter()
                .map(UnionIr::try_from)
                .collect::<Result<Vec<_>>>()?,
            tables: schema
                .tables
                .into_iter()
                .map(TableIr::try_from)
                .collect::<Result<Vec<_>>>()?,
        })
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
            typescript: TypeScriptCodegenIr::from(value.typescript),
            javascript: JavaScriptCodegenIr::from(value.javascript),
            erlang: ErlangCodegenIr::from(value.erlang),
            lua: LuaCodegenIr::from(value.lua),
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
            RuntimeFormatSchema::Protobuf => Self::Protobuf,
            RuntimeFormatSchema::Cbor => Self::Cbor,
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
            fields: convert_fields(variant.fields)?,
        })
    }
}

impl TryFrom<TableSchema> for TableIr {
    type Error = SoraError;

    fn try_from(table: TableSchema) -> Result<Self> {
        Ok(Self {
            name: table.name,
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
        if aggregation.is_none() {
            validate_parser_format(
                &field.name,
                &ty,
                field.parser.as_deref(),
                field.separator.as_deref(),
                field.prefix.as_deref(),
                field.suffix.as_deref(),
            )?;
            if field.parser.is_none() {
                validate_collection_format(
                    &field.name,
                    &ty,
                    field.separator.as_deref(),
                    field.prefix.as_deref(),
                    field.suffix.as_deref(),
                )?;
            }
        } else {
            validate_optional_non_empty(&field.name, "parser", field.parser.as_deref())?;
            validate_optional_non_empty(&field.name, "separator", field.separator.as_deref())?;
            validate_optional_non_empty(&field.name, "prefix", field.prefix.as_deref())?;
            validate_optional_non_empty(&field.name, "suffix", field.suffix.as_deref())?;
        }

        Ok(Self {
            name: field.name,
            ty,
            key: field.key,
            comment: field.comment,
            required: field.required.unwrap_or(false),
            default: field.default,
            range: field.range,
            length: field.length,
            parser: field.parser,
            separator: field.separator,
            prefix: field.prefix,
            suffix: field.suffix,
            aggregation,
        })
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
    fields.into_iter().map(FieldIr::try_from).collect()
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

fn validate_parser_format(
    field_name: &str,
    ty: &TypeIr,
    parser: Option<&str>,
    separator: Option<&str>,
    prefix: Option<&str>,
    suffix: Option<&str>,
) -> Result<()> {
    let Some(parser) = parser else {
        return Ok(());
    };

    match parser {
        "tuple" => {
            validate_tuple_target(field_name, ty)?;
            validate_required_non_empty(field_name, ty, "separator", separator)?;
            validate_optional_non_empty(field_name, "prefix", prefix)?;
            validate_optional_non_empty(field_name, "suffix", suffix)
        }
        _ => Err(SoraError::InvalidSchema(format!(
            "field `{field_name}` declares unsupported parser `{parser}`"
        ))),
    }
}

fn validate_tuple_target(field_name: &str, ty: &TypeIr) -> Result<()> {
    match ty {
        TypeIr::Struct(_) => Ok(()),
        TypeIr::Optional(inner) => validate_tuple_target(field_name, inner),
        _ => Err(SoraError::InvalidSchema(format!(
            "field `{field_name}` declares parser `tuple` but type `{ty}` is not struct"
        ))),
    }
}

fn validate_collection_format(
    field_name: &str,
    ty: &TypeIr,
    separator: Option<&str>,
    prefix: Option<&str>,
    suffix: Option<&str>,
) -> Result<()> {
    match ty {
        TypeIr::List(_) | TypeIr::Array { .. } => {
            validate_required_non_empty(field_name, ty, "separator", separator)?;
            validate_optional_non_empty(field_name, "prefix", prefix)?;
            validate_optional_non_empty(field_name, "suffix", suffix)?;
            Ok(())
        }
        TypeIr::Optional(inner) => {
            validate_collection_format(field_name, inner, separator, prefix, suffix)
        }
        _ if separator.is_some() || prefix.is_some() || suffix.is_some() => {
            Err(SoraError::InvalidSchema(format!(
                "field `{field_name}` declares collection format metadata but type `{ty}` is not list or array"
            )))
        }
        _ => Ok(()),
    }
}

fn validate_required_non_empty(
    field_name: &str,
    ty: &TypeIr,
    key: &'static str,
    value: Option<&str>,
) -> Result<()> {
    match value {
        Some(value) if !value.is_empty() => Ok(()),
        _ => Err(SoraError::InvalidSchema(format!(
            "field `{field_name}` with type `{ty}` must declare non-empty `{key}`"
        ))),
    }
}

fn validate_optional_non_empty(
    field_name: &str,
    key: &'static str,
    value: Option<&str>,
) -> Result<()> {
    match value {
        Some("") => Err(SoraError::InvalidSchema(format!(
            "field `{field_name}` declares empty `{key}`"
        ))),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        EnumReprIr, ErlangEnumReprIr, LuaEnumReprIr, LuaVersionIr, RuntimeFormatIr, TableModeIr,
        TypeIr,
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
separator = "|"
prefix = "["
suffix = "]"
default = "[\"starter\"]"
length = [1, 3]
"#,
        )
        .unwrap();

        let ir = normalize_schema(schema).unwrap();
        assert_eq!(ir.package, "game_config");
        assert_eq!(ir.codegen.rust.runtime_format, RuntimeFormatIr::Sora);
        assert_eq!(ir.codegen.kotlin.runtime_format, RuntimeFormatIr::Sora);
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
        assert_eq!(ir.tables[0].fields[1].separator.as_deref(), Some("|"));
        assert_eq!(ir.tables[0].fields[1].prefix.as_deref(), Some("["));
        assert_eq!(ir.tables[0].fields[1].suffix.as_deref(), Some("]"));
        assert_eq!(
            ir.tables[0].fields[1].default.as_deref(),
            Some("[\"starter\"]")
        );
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
parser = "tuple"
separator = ","
"#,
        )
        .unwrap();

        let ir = normalize_schema(schema).unwrap();
        assert_eq!(ir.tables[0].fields[0].parser.as_deref(), Some("tuple"));
        assert_eq!(ir.tables[0].fields[0].separator.as_deref(), Some(","));
    }

    #[test]
    fn validates_collection_separator_metadata() {
        let missing_separator: SchemaFile = toml::from_str(
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
        assert!(matches!(
            normalize_schema(missing_separator).unwrap_err(),
            SoraError::InvalidSchema(message) if message.contains("must declare non-empty `separator`")
        ));

        let invalid_separator: SchemaFile = toml::from_str(
            r#"
package = "game_config"

[[tables]]
name = "Item"
mode = "list"

[[tables.fields]]
name = "name"
type = "string"
separator = "|"
"#,
        )
        .unwrap();
        assert!(matches!(
            normalize_schema(invalid_separator).unwrap_err(),
            SoraError::InvalidSchema(message) if message.contains("collection format metadata")
        ));

        let scalar_prefix: SchemaFile = toml::from_str(
            r#"
package = "game_config"

[[tables]]
name = "Item"
mode = "list"

[[tables.fields]]
name = "name"
type = "string"
prefix = "["
"#,
        )
        .unwrap();
        assert!(matches!(
            normalize_schema(scalar_prefix).unwrap_err(),
            SoraError::InvalidSchema(message) if message.contains("collection format metadata")
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
        let missing_separator: SchemaFile = toml::from_str(
            r#"
package = "game_config"

[[structs]]
name = "Vec2"

[[tables]]
name = "Spawn"
mode = "list"

[[tables.fields]]
name = "pos"
type = "struct<Vec2>"
parser = "tuple"
"#,
        )
        .unwrap();
        assert!(matches!(
            normalize_schema(missing_separator).unwrap_err(),
            SoraError::InvalidSchema(message) if message.contains("must declare non-empty `separator`")
        ));

        let scalar_parser: SchemaFile = toml::from_str(
            r#"
package = "game_config"

[[tables]]
name = "Spawn"
mode = "list"

[[tables.fields]]
name = "pos"
type = "string"
parser = "tuple"
separator = ","
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
        assert_eq!(ir.tables[0].fields[1].separator, None);
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
