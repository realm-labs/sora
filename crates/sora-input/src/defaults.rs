use std::{borrow::Cow, collections::BTreeMap, path::Path};

use sora_data::model::{ConfigData, RowData, Value};
use sora_diagnostics::Result;
use sora_ir::model::{ConfigIr, FieldIr, StructIr, TypeIr, UnionIr};

use crate::{
    cell::{CellContext, CellLocation, CellValue, cell_to_value_with_parsers},
    parser::{ParserRegistry, builtin_registry},
};

pub fn materialize_defaults(ir: &ConfigIr, data: &ConfigData) -> Result<ConfigData> {
    materialize_defaults_with_parsers(ir, data, builtin_registry())
}

pub fn materialize_defaults_with_parsers(
    ir: &ConfigIr,
    data: &ConfigData,
    parser_registry: &ParserRegistry,
) -> Result<ConfigData> {
    let mut data = data.clone();
    for table in &ir.tables {
        let Some(table_data) = data
            .tables
            .iter_mut()
            .find(|candidate| candidate.name == table.name)
        else {
            continue;
        };

        for row in &mut table_data.rows {
            materialize_field_defaults(ir, &table.fields, row, parser_registry)?;
        }
    }

    Ok(data)
}

fn materialize_field_defaults(
    ir: &ConfigIr,
    fields: &[FieldIr],
    row: &mut RowData,
    parser_registry: &ParserRegistry,
) -> Result<()> {
    for field in fields {
        if !row.values.contains_key(&field.name)
            && let Some(default) = &field.default
        {
            row.values.insert(
                field.name.clone(),
                default_to_value(ir, field, default, parser_registry)?,
            );
        }

        if let Some(value) = row.values.get_mut(&field.name) {
            materialize_nested_defaults(ir, &field.ty, value, parser_registry)?;
        }
    }

    Ok(())
}

fn materialize_nested_defaults(
    ir: &ConfigIr,
    ty: &TypeIr,
    value: &mut Value,
    parser_registry: &ParserRegistry,
) -> Result<()> {
    match (ty, value) {
        (TypeIr::Optional(_), Value::Null) => Ok(()),
        (TypeIr::Optional(inner), value) => {
            materialize_nested_defaults(ir, inner, value, parser_registry)
        }
        (TypeIr::Struct(struct_name), Value::Object(values)) => {
            if let Some(struct_ir) = struct_ir(ir, struct_name) {
                materialize_object_defaults(ir, struct_ir, values, parser_registry)?;
            }
            Ok(())
        }
        (TypeIr::Union(union_name), Value::Object(values)) => {
            if let Some(union_ir) = union_ir(ir, union_name)
                && let Some(Value::String(variant_name)) = values.get(&union_ir.tag)
                && let Some(variant) = union_ir
                    .variants
                    .iter()
                    .find(|candidate| candidate.name == *variant_name)
            {
                materialize_field_defaults_in_object(ir, &variant.fields, values, parser_registry)?;
            }
            Ok(())
        }
        (TypeIr::List(element), Value::List(values))
        | (TypeIr::Set(element), Value::List(values))
        | (TypeIr::Array { element, .. }, Value::List(values)) => {
            for value in values {
                materialize_nested_defaults(ir, element, value, parser_registry)?;
            }
            Ok(())
        }
        (
            TypeIr::Map {
                key,
                value: element,
            },
            Value::List(values),
        ) => {
            for value in values {
                let Value::List(pair) = value else {
                    continue;
                };
                if pair.len() == 2 {
                    materialize_nested_defaults(ir, key, &mut pair[0], parser_registry)?;
                    materialize_nested_defaults(ir, element, &mut pair[1], parser_registry)?;
                }
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

fn materialize_object_defaults(
    ir: &ConfigIr,
    struct_ir: &StructIr,
    values: &mut BTreeMap<String, Value>,
    parser_registry: &ParserRegistry,
) -> Result<()> {
    materialize_field_defaults_in_object(ir, &struct_ir.fields, values, parser_registry)
}

fn materialize_field_defaults_in_object(
    ir: &ConfigIr,
    fields: &[FieldIr],
    values: &mut BTreeMap<String, Value>,
    parser_registry: &ParserRegistry,
) -> Result<()> {
    for field in fields {
        if !values.contains_key(&field.name)
            && let Some(default) = &field.default
        {
            values.insert(
                field.name.clone(),
                default_to_value(ir, field, default, parser_registry)?,
            );
        }

        if let Some(value) = values.get_mut(&field.name) {
            materialize_nested_defaults(ir, &field.ty, value, parser_registry)?;
        }
    }

    Ok(())
}

fn default_to_value(
    ir: &ConfigIr,
    field: &FieldIr,
    source: &str,
    parser_registry: &ParserRegistry,
) -> Result<Value> {
    let context = CellContext {
        path: Path::new("<schema>"),
        ir,
        location: CellLocation::Default,
        field: &field.name,
        parser: field.parser.as_ref(),
    };
    cell_to_value_with_parsers(
        &CellValue::Text(Cow::Owned(source.to_owned())),
        &field.ty,
        &context,
        parser_registry,
    )
}

fn struct_ir<'a>(ir: &'a ConfigIr, name: &str) -> Option<&'a StructIr> {
    ir.structs.iter().find(|candidate| candidate.name == name)
}

fn union_ir<'a>(ir: &'a ConfigIr, name: &str) -> Option<&'a UnionIr> {
    ir.unions.iter().find(|candidate| candidate.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sora_data::model::{RowData, TableData};
    use sora_ir::{normalize::normalize_schema, validate::validate_config_ir};
    use sora_schema::model::SchemaFile;

    #[test]
    fn materializes_table_and_nested_struct_defaults() {
        let ir = example_ir();
        let data = ConfigData {
            tables: vec![TableData {
                name: "Item".to_owned(),
                rows: vec![RowData {
                    values: BTreeMap::from([(
                        "reward".to_owned(),
                        Value::Object(BTreeMap::from([(
                            "item_id".to_owned(),
                            Value::Integer(1001),
                        )])),
                    )]),
                }],
            }],
        };

        let data = materialize_defaults(&ir, &data).unwrap();
        let row = &data.tables[0].rows[0].values;

        assert_eq!(row["id"], Value::Integer(1001));
        assert_eq!(
            row["tags"],
            Value::List(vec![Value::String("new".to_owned())])
        );
        assert_eq!(
            row["reward"],
            Value::Object(BTreeMap::from([
                ("item_id".to_owned(), Value::Integer(1001)),
                ("count".to_owned(), Value::Integer(1)),
            ]))
        );
    }

    fn example_ir() -> ConfigIr {
        let schema: SchemaFile = toml::from_str(
            r#"
package = "game_config"

[[structs]]
name = "Reward"

[[structs.fields]]
name = "item_id"
type = "i32"

[[structs.fields]]
name = "count"
type = "i32"
default = "1"

[[tables]]
name = "Item"
mode = "map"
key = "id"

[[tables.fields]]
name = "id"
type = "i32"
default = "1001"

[[tables.fields]]
name = "tags"
type = "list<string>"
default = "new"

[[tables.fields]]
name = "reward"
type = "struct<Reward>"
"#,
        )
        .unwrap();
        let ir = normalize_schema(schema).unwrap();
        validate_config_ir(&ir).unwrap();
        ir
    }
}
