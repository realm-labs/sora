use std::{borrow::Cow, collections::BTreeMap, path::Path};

use sora_data::model::{ConfigData, RowData, Value};
use sora_diagnostics::Result;
use sora_ir::model::{ConfigIr, FieldIr, StructIr, TypeIr};

use crate::cell::{CellContext, CellLocation, CellValue, cell_to_value};

pub fn materialize_defaults(ir: &ConfigIr, data: &ConfigData) -> Result<ConfigData> {
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
            materialize_field_defaults(ir, &table.fields, row)?;
        }
    }

    Ok(data)
}

fn materialize_field_defaults(ir: &ConfigIr, fields: &[FieldIr], row: &mut RowData) -> Result<()> {
    for field in fields {
        if !row.values.contains_key(&field.name)
            && let Some(default) = &field.default
        {
            row.values
                .insert(field.name.clone(), default_to_value(ir, field, default)?);
        }

        if let Some(value) = row.values.get_mut(&field.name) {
            materialize_nested_defaults(ir, &field.ty, value)?;
        }
    }

    Ok(())
}

fn materialize_nested_defaults(ir: &ConfigIr, ty: &TypeIr, value: &mut Value) -> Result<()> {
    match (ty, value) {
        (TypeIr::Optional(_), Value::Null) => Ok(()),
        (TypeIr::Optional(inner), value) => materialize_nested_defaults(ir, inner, value),
        (TypeIr::Struct(struct_name), Value::Object(values)) => {
            if let Some(struct_ir) = struct_ir(ir, struct_name) {
                materialize_object_defaults(ir, struct_ir, values)?;
            }
            Ok(())
        }
        (TypeIr::List(element), Value::List(values))
        | (TypeIr::Array { element, .. }, Value::List(values)) => {
            for value in values {
                materialize_nested_defaults(ir, element, value)?;
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
) -> Result<()> {
    for field in &struct_ir.fields {
        if !values.contains_key(&field.name)
            && let Some(default) = &field.default
        {
            values.insert(field.name.clone(), default_to_value(ir, field, default)?);
        }

        if let Some(value) = values.get_mut(&field.name) {
            materialize_nested_defaults(ir, &field.ty, value)?;
        }
    }

    Ok(())
}

fn default_to_value(ir: &ConfigIr, field: &FieldIr, source: &str) -> Result<Value> {
    let context = CellContext {
        path: Path::new("<schema>"),
        ir,
        location: CellLocation::Default,
        field: &field.name,
        parser: field.parser.as_deref(),
        separator: field.separator.as_deref(),
        prefix: field.prefix.as_deref(),
        suffix: field.suffix.as_deref(),
    };
    cell_to_value(
        &CellValue::Text(Cow::Owned(source.to_owned())),
        &field.ty,
        &context,
    )
}

fn struct_ir<'a>(ir: &'a ConfigIr, name: &str) -> Option<&'a StructIr> {
    ir.structs.iter().find(|candidate| candidate.name == name)
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
separator = ","
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
