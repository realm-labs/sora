use sora_diagnostics::{Result, SoraError};
use sora_schema::model::{
    FieldSchema, IndexSchema, SchemaFile, TableModeSchema, TableSchema, TableSourceSchema,
};

use crate::{
    model::{
        AggregationIr, ConfigIr, EnumIr, FieldIr, IndexIr, StructIr, TableIr, TableModeIr,
        TableSourceIr,
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
            tables: schema
                .tables
                .into_iter()
                .map(TableIr::try_from)
                .collect::<Result<Vec<_>>>()?,
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

        Ok(Self {
            name: field.name,
            ty: parse_type(&field.ty)?,
            key: field.key,
            comment: field.comment,
            required: field.required.unwrap_or(false),
            parser: field.parser,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{TableModeIr, TypeIr};

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
"#,
        )
        .unwrap();

        let ir = normalize_schema(schema).unwrap();
        assert_eq!(ir.package, "game_config");
        assert_eq!(ir.enums[0].name, "ItemType");
        assert_eq!(ir.tables[0].mode, TableModeIr::Map);
        assert!(ir.tables[0].fields[0].required);
        assert_eq!(ir.tables[0].fields[0].ty, TypeIr::I32);
    }
}
