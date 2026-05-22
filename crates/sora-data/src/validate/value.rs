use std::collections::{BTreeMap, BTreeSet};

use sora_diagnostics::{Result, SoraError};
use sora_ir::model::{ConfigIr, FieldIr, StructIr, TableModeIr, TypeIr, UnionIr};

use crate::model::{ConfigData, Value};
pub(super) fn validate_field_value(
    ir: &ConfigIr,
    config_data: &ConfigData,
    table: &str,
    field: &FieldIr,
    path: &str,
    value: &Value,
) -> Result<()> {
    validate_typed_value(
        ir,
        config_data,
        table,
        path,
        &field.ty,
        ValueConstraints {
            range: field.range,
            length: field.length,
        },
        value,
    )
}

#[derive(Clone, Copy)]
struct ValueConstraints {
    range: Option<[i64; 2]>,
    length: Option<[usize; 2]>,
}

fn validate_typed_value(
    ir: &ConfigIr,
    config_data: &ConfigData,
    table: &str,
    path: &str,
    ty: &TypeIr,
    constraints: ValueConstraints,
    value: &Value,
) -> Result<()> {
    match ty {
        TypeIr::Optional(element) if matches!(value, Value::Null) => Ok(()),
        TypeIr::Optional(element) => {
            validate_typed_value(ir, config_data, table, path, element, constraints, value)
        }
        TypeIr::Bool => expect_type(table, path, ty, value, matches!(value, Value::Bool(_))),
        TypeIr::I32 => match value {
            Value::Integer(number) if i32::try_from(*number).is_ok() => {
                validate_integer_range(table, path, *number, constraints.range)
            }
            _ => type_mismatch(table, path, ty, value),
        },
        TypeIr::I64 => match value {
            Value::Integer(number) => {
                validate_integer_range(table, path, *number, constraints.range)
            }
            _ => type_mismatch(table, path, ty, value),
        },
        TypeIr::F32 | TypeIr::F64 => match value {
            Value::Integer(number) => {
                validate_float_range(table, path, *number as f64, constraints.range)
            }
            Value::Float(number) => validate_float_range(table, path, *number, constraints.range),
            _ => type_mismatch(table, path, ty, value),
        },
        TypeIr::String => match value {
            Value::String(value) => {
                validate_length(table, path, value.chars().count(), constraints.length)
            }
            _ => type_mismatch(table, path, ty, value),
        },
        TypeIr::Enum(enum_name) => validate_enum(ir, table, path, ty, enum_name, value),
        TypeIr::Struct(struct_name) => {
            validate_struct(ir, config_data, table, path, ty, struct_name, value)
        }
        TypeIr::Union(union_name) => {
            validate_union(ir, config_data, table, path, ty, union_name, value)
        }
        TypeIr::List(element) => {
            validate_list(ir, config_data, table, path, element, constraints, value)
        }
        TypeIr::Set(element) => {
            validate_list(ir, config_data, table, path, element, constraints, value)
        }
        TypeIr::Map {
            key,
            value: element,
        } => validate_map(
            ir,
            config_data,
            table,
            path,
            MapExpectation {
                key,
                value: element,
            },
            constraints,
            value,
        ),
        TypeIr::Array { element, len } => validate_array(
            ir,
            config_data,
            table,
            path,
            ArrayExpectation {
                ty,
                element,
                len: *len,
            },
            constraints,
            value,
        ),
        TypeIr::Ref {
            table: ref_table,
            field: ref_field,
        } => validate_ref(config_data, table, path, ty, ref_table, ref_field, value),
    }
}

fn validate_enum(
    ir: &ConfigIr,
    table: &str,
    path: &str,
    ty: &TypeIr,
    enum_name: &str,
    value: &Value,
) -> Result<()> {
    let Value::String(item) = value else {
        return type_mismatch(table, path, ty, value);
    };

    let is_valid = ir
        .enums
        .iter()
        .find(|candidate| candidate.name == enum_name)
        .is_some_and(|candidate| {
            candidate.values.contains(item)
                || candidate.aliases.iter().any(|entry| entry.alias == *item)
        });
    if is_valid {
        Ok(())
    } else {
        Err(SoraError::InvalidEnumValue {
            table: table.to_owned(),
            field: path.to_owned(),
            value: item.clone(),
        })
    }
}

fn validate_struct(
    ir: &ConfigIr,
    config_data: &ConfigData,
    table: &str,
    path: &str,
    ty: &TypeIr,
    struct_name: &str,
    value: &Value,
) -> Result<()> {
    let Value::Object(values) = value else {
        return type_mismatch(table, path, ty, value);
    };
    let Some(struct_ir) = ir
        .structs
        .iter()
        .find(|candidate| candidate.name == struct_name)
    else {
        return type_mismatch(table, path, ty, value);
    };

    validate_object_fields(ir, config_data, table, path, struct_ir, values)
}

fn validate_object_fields(
    ir: &ConfigIr,
    config_data: &ConfigData,
    table: &str,
    path: &str,
    struct_ir: &StructIr,
    values: &BTreeMap<String, Value>,
) -> Result<()> {
    let field_names = struct_ir
        .fields
        .iter()
        .map(|field| field.name.as_str())
        .collect::<BTreeSet<_>>();
    for field_name in values.keys() {
        if !field_names.contains(field_name.as_str()) {
            return Err(SoraError::UnknownField {
                table: table.to_owned(),
                field: format!("{path}.{field_name}"),
            });
        }
    }

    for field in &struct_ir.fields {
        let child_path = format!("{path}.{}", field.name);
        match values.get(&field.name) {
            Some(value) => {
                validate_field_value(ir, config_data, table, field, &child_path, value)?;
            }
            None if field.required => {
                return Err(SoraError::MissingRequiredField {
                    table: table.to_owned(),
                    field: child_path,
                });
            }
            None => {}
        }
    }

    Ok(())
}

fn validate_union(
    ir: &ConfigIr,
    config_data: &ConfigData,
    table: &str,
    path: &str,
    ty: &TypeIr,
    union_name: &str,
    value: &Value,
) -> Result<()> {
    let Value::Object(values) = value else {
        return type_mismatch(table, path, ty, value);
    };
    let Some(union_ir) = ir
        .unions
        .iter()
        .find(|candidate| candidate.name == union_name)
    else {
        return type_mismatch(table, path, ty, value);
    };
    validate_union_object(ir, config_data, table, path, ty, union_ir, values)
}

fn validate_union_object(
    ir: &ConfigIr,
    config_data: &ConfigData,
    table: &str,
    path: &str,
    ty: &TypeIr,
    union_ir: &UnionIr,
    values: &BTreeMap<String, Value>,
) -> Result<()> {
    let Some(Value::String(variant_name)) = values.get(&union_ir.tag) else {
        return type_mismatch(table, &format!("{path}.{}", union_ir.tag), ty, &Value::Null);
    };
    let Some(variant) = union_ir
        .variants
        .iter()
        .find(|candidate| candidate.name == *variant_name)
    else {
        return Err(SoraError::InvalidEnumValue {
            table: table.to_owned(),
            field: format!("{path}.{}", union_ir.tag),
            value: variant_name.clone(),
        });
    };

    let field_names = variant
        .fields
        .iter()
        .map(|field| field.name.as_str())
        .chain(std::iter::once(union_ir.tag.as_str()))
        .collect::<BTreeSet<_>>();
    for field_name in values.keys() {
        if !field_names.contains(field_name.as_str()) {
            return Err(SoraError::UnknownField {
                table: table.to_owned(),
                field: format!("{path}.{field_name}"),
            });
        }
    }

    for field in &variant.fields {
        let child_path = format!("{path}.{}", field.name);
        match values.get(&field.name) {
            Some(value) => {
                validate_field_value(ir, config_data, table, field, &child_path, value)?;
            }
            None if field.required => {
                return Err(SoraError::MissingRequiredField {
                    table: table.to_owned(),
                    field: child_path,
                });
            }
            None => {}
        }
    }

    Ok(())
}

fn validate_list(
    ir: &ConfigIr,
    config_data: &ConfigData,
    table: &str,
    path: &str,
    element: &TypeIr,
    constraints: ValueConstraints,
    value: &Value,
) -> Result<()> {
    let Value::List(items) = value else {
        return type_mismatch(table, path, &TypeIr::List(Box::new(element.clone())), value);
    };
    validate_length(table, path, items.len(), constraints.length)?;

    for (index, item) in items.iter().enumerate() {
        validate_typed_value(
            ir,
            config_data,
            table,
            &format!("{path}[{index}]"),
            element,
            ValueConstraints {
                range: constraints.range,
                length: None,
            },
            item,
        )?;
    }

    Ok(())
}

struct MapExpectation<'a> {
    key: &'a TypeIr,
    value: &'a TypeIr,
}

fn validate_map(
    ir: &ConfigIr,
    config_data: &ConfigData,
    table: &str,
    path: &str,
    expectation: MapExpectation<'_>,
    constraints: ValueConstraints,
    value: &Value,
) -> Result<()> {
    let Value::List(items) = value else {
        return type_mismatch(
            table,
            path,
            &TypeIr::Map {
                key: Box::new(expectation.key.clone()),
                value: Box::new(expectation.value.clone()),
            },
            value,
        );
    };
    validate_length(table, path, items.len(), constraints.length)?;
    for (index, item) in items.iter().enumerate() {
        let Value::List(pair) = item else {
            return type_mismatch(table, &format!("{path}[{index}]"), expectation.value, item);
        };
        if pair.len() != 2 {
            return Err(SoraError::InvalidSchema(format!(
                "map value `{path}[{index}]` must contain exactly two elements"
            )));
        }
        validate_typed_value(
            ir,
            config_data,
            table,
            &format!("{path}[{index}].key"),
            expectation.key,
            ValueConstraints {
                range: None,
                length: None,
            },
            &pair[0],
        )?;
        validate_typed_value(
            ir,
            config_data,
            table,
            &format!("{path}[{index}].value"),
            expectation.value,
            ValueConstraints {
                range: None,
                length: None,
            },
            &pair[1],
        )?;
    }
    Ok(())
}

struct ArrayExpectation<'a> {
    ty: &'a TypeIr,
    element: &'a TypeIr,
    len: usize,
}

fn validate_array(
    ir: &ConfigIr,
    config_data: &ConfigData,
    table: &str,
    path: &str,
    expectation: ArrayExpectation<'_>,
    constraints: ValueConstraints,
    value: &Value,
) -> Result<()> {
    let Value::List(items) = value else {
        return type_mismatch(table, path, expectation.ty, value);
    };
    if items.len() != expectation.len {
        return type_mismatch(table, path, expectation.ty, value);
    }
    validate_length(table, path, items.len(), constraints.length)?;

    for (index, item) in items.iter().enumerate() {
        validate_typed_value(
            ir,
            config_data,
            table,
            &format!("{path}[{index}]"),
            expectation.element,
            ValueConstraints {
                range: constraints.range,
                length: None,
            },
            item,
        )?;
    }

    Ok(())
}

fn validate_ref(
    config_data: &ConfigData,
    table: &str,
    path: &str,
    ty: &TypeIr,
    ref_table: &str,
    ref_field: &str,
    value: &Value,
) -> Result<()> {
    if !matches!(value, Value::Integer(_) | Value::String(_)) {
        return type_mismatch(table, path, ty, value);
    }

    let key = stable_key(value);
    let exists = config_data
        .tables
        .iter()
        .find(|candidate| candidate.name == ref_table)
        .is_some_and(|table_data| {
            table_data
                .rows
                .iter()
                .filter_map(|row| row.values.get(ref_field))
                .any(|candidate| stable_key(candidate) == key)
        });

    if exists {
        Ok(())
    } else {
        Err(SoraError::MissingReference {
            table: table.to_owned(),
            field: path.to_owned(),
            ref_table: ref_table.to_owned(),
            ref_field: ref_field.to_owned(),
            value: key,
        })
    }
}

fn expect_type(table: &str, path: &str, ty: &TypeIr, value: &Value, matches: bool) -> Result<()> {
    if matches {
        Ok(())
    } else {
        type_mismatch(table, path, ty, value)
    }
}

fn type_mismatch(table: &str, path: &str, ty: &TypeIr, value: &Value) -> Result<()> {
    Err(SoraError::TypeMismatch {
        table: table.to_owned(),
        field: path.to_owned(),
        expected: ty.to_string(),
        actual: value.kind_name().to_owned(),
    })
}

fn validate_length(
    table: &str,
    path: &str,
    actual: usize,
    length: Option<[usize; 2]>,
) -> Result<()> {
    let Some([min, max]) = length else {
        return Ok(());
    };
    if actual < min || actual > max {
        Err(SoraError::LengthOutOfBounds {
            table: table.to_owned(),
            field: path.to_owned(),
            actual,
            min,
            max,
        })
    } else {
        Ok(())
    }
}

fn validate_integer_range(
    table: &str,
    path: &str,
    value: i64,
    range: Option<[i64; 2]>,
) -> Result<()> {
    let Some([min, max]) = range else {
        return Ok(());
    };
    if value < min || value > max {
        Err(SoraError::RangeOutOfBounds {
            table: table.to_owned(),
            field: path.to_owned(),
            value: value.to_string(),
            min,
            max,
        })
    } else {
        Ok(())
    }
}

fn validate_float_range(
    table: &str,
    path: &str,
    value: f64,
    range: Option<[i64; 2]>,
) -> Result<()> {
    let Some([min, max]) = range else {
        return Ok(());
    };
    if value < min as f64 || value > max as f64 {
        Err(SoraError::RangeOutOfBounds {
            table: table.to_owned(),
            field: path.to_owned(),
            value: value.to_string(),
            min,
            max,
        })
    } else {
        Ok(())
    }
}

pub(super) fn stable_key(value: &Value) -> String {
    match value {
        Value::Bool(value) => value.to_string(),
        Value::Integer(value) => value.to_string(),
        Value::Float(value) => value.to_string(),
        Value::String(value) => value.clone(),
        Value::List(_) => "<list>".to_owned(),
        Value::Object(_) => "<object>".to_owned(),
        Value::Null => "<null>".to_owned(),
    }
}

pub(super) fn table_mode_name(mode: TableModeIr) -> &'static str {
    match mode {
        TableModeIr::List => "list",
        TableModeIr::Map => "map",
        TableModeIr::Singleton => "singleton",
    }
}
