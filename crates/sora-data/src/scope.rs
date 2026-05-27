use std::collections::BTreeMap;

use sora_ir::model::{ConfigIr, FieldIr, TypeIr, UnionIr};

use crate::model::{ConfigData, RowData, TableData, Value};

pub fn filter_config_data_by_ir(ir: &ConfigIr, data: &ConfigData) -> ConfigData {
    ConfigData {
        tables: ir
            .tables
            .iter()
            .filter_map(|table| {
                let source = data.tables.iter().find(|item| item.name == table.name)?;
                Some(TableData {
                    name: table.name.clone(),
                    rows: source
                        .rows
                        .iter()
                        .map(|row| filter_row(ir, &table.fields, row))
                        .collect(),
                })
            })
            .collect(),
    }
}

fn filter_row(ir: &ConfigIr, fields: &[FieldIr], row: &RowData) -> RowData {
    RowData {
        values: fields
            .iter()
            .filter_map(|field| {
                let value = row.values.get(&field.name)?;
                Some((
                    field.name.clone(),
                    filter_value(ir, &field.ty, value).unwrap_or_else(|| value.clone()),
                ))
            })
            .collect(),
    }
}

fn filter_value(ir: &ConfigIr, ty: &TypeIr, value: &Value) -> Option<Value> {
    match ty {
        TypeIr::Struct(name) => {
            let struct_ir = ir.structs.iter().find(|item| item.name == *name)?;
            let Value::Object(object) = value else {
                return Some(value.clone());
            };
            Some(Value::Object(filter_object(ir, &struct_ir.fields, object)))
        }
        TypeIr::Union(name) => {
            let union_ir = ir.unions.iter().find(|item| item.name == *name)?;
            filter_union(ir, union_ir, value)
        }
        TypeIr::List(element) | TypeIr::Set(element) | TypeIr::Array { element, .. } => {
            let Value::List(values) = value else {
                return Some(value.clone());
            };
            Some(Value::List(
                values
                    .iter()
                    .map(|value| filter_value(ir, element, value).unwrap_or_else(|| value.clone()))
                    .collect(),
            ))
        }
        TypeIr::Map {
            key,
            value: element,
        } => {
            let Value::List(values) = value else {
                return Some(value.clone());
            };
            Some(Value::List(
                values
                    .iter()
                    .map(|entry| filter_map_entry(ir, key, element, entry))
                    .collect(),
            ))
        }
        TypeIr::Optional(element) => {
            if matches!(value, Value::Null) {
                Some(Value::Null)
            } else {
                filter_value(ir, element, value)
            }
        }
        TypeIr::Bool
        | TypeIr::I8
        | TypeIr::U8
        | TypeIr::I16
        | TypeIr::U16
        | TypeIr::I32
        | TypeIr::U32
        | TypeIr::I64
        | TypeIr::Duration
        | TypeIr::F32
        | TypeIr::F64
        | TypeIr::String
        | TypeIr::Text
        | TypeIr::Enum(_)
        | TypeIr::Ref { .. } => Some(value.clone()),
    }
}

fn filter_map_entry(ir: &ConfigIr, key_ty: &TypeIr, value_ty: &TypeIr, entry: &Value) -> Value {
    let Value::List(items) = entry else {
        return entry.clone();
    };
    if items.len() != 2 {
        return entry.clone();
    }

    Value::List(vec![
        filter_value(ir, key_ty, &items[0]).unwrap_or_else(|| items[0].clone()),
        filter_value(ir, value_ty, &items[1]).unwrap_or_else(|| items[1].clone()),
    ])
}

fn filter_union(ir: &ConfigIr, union_ir: &UnionIr, value: &Value) -> Option<Value> {
    let Value::Object(object) = value else {
        return Some(value.clone());
    };
    let Some(Value::String(variant_name)) = object.get(&union_ir.tag) else {
        return Some(value.clone());
    };
    let Some(variant) = union_ir
        .variants
        .iter()
        .find(|item| item.name == *variant_name)
    else {
        return Some(value.clone());
    };

    let mut filtered = filter_object(ir, &variant.fields, object);
    filtered.insert(union_ir.tag.clone(), Value::String(variant_name.clone()));
    Some(Value::Object(filtered))
}

fn filter_object(
    ir: &ConfigIr,
    fields: &[FieldIr],
    object: &BTreeMap<String, Value>,
) -> BTreeMap<String, Value> {
    fields
        .iter()
        .filter_map(|field| {
            let value = object.get(&field.name)?;
            Some((
                field.name.clone(),
                filter_value(ir, &field.ty, value).unwrap_or_else(|| value.clone()),
            ))
        })
        .collect()
}
