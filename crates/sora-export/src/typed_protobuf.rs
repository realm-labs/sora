use std::collections::BTreeMap;

use sora_data::model::{ConfigData, RowData, Value};
use sora_diagnostics::{Result, SoraError};
use sora_ir::model::{ConfigIr, FieldIr, TableModeIr, TypeIr, UnionIr};

use crate::{
    exporter::{DataExporter, ExportOutput, ExportRequest, OutputKind},
    fs_util::{create_parent_dir, write_file},
};

pub struct TypedProtobufExporter;

impl DataExporter for TypedProtobufExporter {
    fn format_name(&self) -> &'static str {
        "proto"
    }

    fn output_kind(&self) -> OutputKind {
        OutputKind::File
    }

    fn export(&self, request: ExportRequest<'_>) -> Result<()> {
        let ExportOutput::File(path) = request.output else {
            return Err(SoraError::InvalidExportOutput {
                format: self.format_name().to_owned(),
                expected: "file",
            });
        };

        create_parent_dir(&path)?;
        let bytes = encode_config(request.ir, request.data)?;
        write_file(path, bytes)
    }
}

fn encode_config(ir: &ConfigIr, data: &ConfigData) -> Result<Vec<u8>> {
    let mut writer = ProtoWriter::default();
    for (index, table) in ir.tables.iter().enumerate() {
        let Some(table_data) = data
            .tables
            .iter()
            .find(|candidate| candidate.name == table.name)
        else {
            continue;
        };
        match table.mode {
            TableModeIr::Singleton => {
                if let Some(row) = table_data.rows.first() {
                    writer.message((index + 1) as u32, |writer| {
                        encode_row(ir, writer, &table.name, &table.fields, row)
                    })?;
                }
            }
            TableModeIr::List | TableModeIr::Map => {
                for row in &table_data.rows {
                    writer.message((index + 1) as u32, |writer| {
                        encode_row(ir, writer, &table.name, &table.fields, row)
                    })?;
                }
            }
        }
    }
    Ok(writer.into_bytes())
}

fn encode_row(
    ir: &ConfigIr,
    writer: &mut ProtoWriter,
    table: &str,
    fields: &[FieldIr],
    row: &RowData,
) -> Result<()> {
    for (index, field) in fields.iter().enumerate() {
        let Some(value) = row.values.get(&field.name) else {
            continue;
        };
        encode_field(
            ir,
            writer,
            table,
            field,
            (index + 1) as u32,
            &field.ty,
            value,
        )?;
    }
    Ok(())
}

fn encode_field(
    ir: &ConfigIr,
    writer: &mut ProtoWriter,
    table: &str,
    field: &FieldIr,
    tag: u32,
    ty: &TypeIr,
    value: &Value,
) -> Result<()> {
    if matches!(value, Value::Null) {
        return Ok(());
    }

    match ty {
        TypeIr::List(element) | TypeIr::Set(element) | TypeIr::Array { element, .. } => {
            let values = expect_list(table, field, value)?;
            for item in values {
                encode_value(ir, writer, table, field, tag, element, item)?;
            }
            Ok(())
        }
        TypeIr::Map {
            key,
            value: element,
        } => {
            let values = expect_list(table, field, value)?;
            for item in values {
                let pair = expect_list(table, field, item)?;
                if pair.len() == 2 {
                    writer.message(tag, |writer| {
                        encode_value(ir, writer, table, field, 1, key, &pair[0])?;
                        encode_value(ir, writer, table, field, 2, element, &pair[1])
                    })?;
                }
            }
            Ok(())
        }
        TypeIr::Optional(element) => encode_value(ir, writer, table, field, tag, element, value),
        _ => encode_value(ir, writer, table, field, tag, ty, value),
    }
}

fn encode_value(
    ir: &ConfigIr,
    writer: &mut ProtoWriter,
    table: &str,
    field: &FieldIr,
    tag: u32,
    ty: &TypeIr,
    value: &Value,
) -> Result<()> {
    match ty {
        TypeIr::Bool => writer.bool(tag, expect_bool(table, field, value)?),
        TypeIr::I8
        | TypeIr::U8
        | TypeIr::I16
        | TypeIr::U16
        | TypeIr::I32
        | TypeIr::U32
        | TypeIr::I64 => writer.int64(tag, expect_integer(table, field, value)?),
        TypeIr::F32 => writer.float(tag, expect_float(table, field, value)? as f32),
        TypeIr::F64 => writer.double(tag, expect_float(table, field, value)?),
        TypeIr::String | TypeIr::Text => writer.string(tag, expect_string(table, field, value)?),
        TypeIr::Enum(name) => {
            let value = expect_string(table, field, value)?;
            let enum_ir = ir
                .enums
                .iter()
                .find(|item| item.name == *name)
                .ok_or_else(|| SoraError::InvalidEnumValue {
                    table: table.to_owned(),
                    field: field.name.clone(),
                    value: value.to_owned(),
                })?;
            let value = enum_ir
                .aliases
                .iter()
                .find(|item| item.alias == value)
                .map(|item| item.name.as_str())
                .unwrap_or(value);
            let number = enum_ir
                .values
                .iter()
                .position(|candidate| candidate == value)
                .map(|value| value as i64)
                .ok_or_else(|| SoraError::InvalidEnumValue {
                    table: table.to_owned(),
                    field: field.name.clone(),
                    value: value.to_owned(),
                })?;
            writer.int64(tag, number);
        }
        TypeIr::Struct(name) => {
            let fields = ir
                .structs
                .iter()
                .find(|item| item.name == *name)
                .map(|item| item.fields.as_slice())
                .ok_or_else(|| SoraError::UnknownTypeReference {
                    kind: "struct",
                    name: name.clone(),
                    owner_kind: "field",
                    owner: field.name.clone(),
                    field: field.name.clone(),
                })?;
            writer.message(tag, |writer| {
                encode_object(ir, writer, table, field, fields, value)
            })?;
        }
        TypeIr::Union(name) => {
            let union = ir
                .unions
                .iter()
                .find(|item| item.name == *name)
                .ok_or_else(|| SoraError::UnknownTypeReference {
                    kind: "union",
                    name: name.clone(),
                    owner_kind: "field",
                    owner: field.name.clone(),
                    field: field.name.clone(),
                })?;
            writer.message(tag, |writer| {
                encode_union(ir, writer, table, field, union, value)
            })?;
        }
        TypeIr::List(_)
        | TypeIr::Set(_)
        | TypeIr::Map { .. }
        | TypeIr::Array { .. }
        | TypeIr::Optional(_) => encode_field(ir, writer, table, field, tag, ty, value)?,
        TypeIr::Ref {
            table: ref_table,
            field: ref_field,
        } => {
            let target_ty = ref_type(ir, ref_table, ref_field);
            encode_value(ir, writer, table, field, tag, target_ty, value)?;
        }
    }
    Ok(())
}

fn encode_object(
    ir: &ConfigIr,
    writer: &mut ProtoWriter,
    table: &str,
    field: &FieldIr,
    fields: &[FieldIr],
    value: &Value,
) -> Result<()> {
    let object = expect_object(table, field, value)?;
    for (index, child_field) in fields.iter().enumerate() {
        let Some(child_value) = object.get(&child_field.name) else {
            continue;
        };
        encode_field(
            ir,
            writer,
            table,
            child_field,
            (index + 1) as u32,
            &child_field.ty,
            child_value,
        )?;
    }
    Ok(())
}

fn encode_union(
    ir: &ConfigIr,
    writer: &mut ProtoWriter,
    table: &str,
    field: &FieldIr,
    union: &UnionIr,
    value: &Value,
) -> Result<()> {
    let object = expect_object(table, field, value)?;
    let Some(Value::String(tag)) = object.get(&union.tag) else {
        return Err(SoraError::TypeMismatch {
            table: table.to_owned(),
            field: field.name.clone(),
            expected: format!("union tag `{}`", union.tag),
            actual: kind_name(value).to_owned(),
        });
    };
    let (variant_index, variant) = union
        .variants
        .iter()
        .enumerate()
        .find(|(_, variant)| variant.name == *tag)
        .ok_or_else(|| SoraError::InvalidSchema(format!("unknown union variant `{tag}`")))?;
    writer.message((variant_index + 1) as u32, |writer| {
        let row = RowData {
            values: object
                .iter()
                .filter(|(name, _)| *name != &union.tag)
                .map(|(name, value)| (name.clone(), value.clone()))
                .collect(),
        };
        encode_row(ir, writer, table, &variant.fields, &row)
    })
}

fn ref_type<'a>(ir: &'a ConfigIr, table: &str, field: &str) -> &'a TypeIr {
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
        .unwrap_or(&TypeIr::I32)
}

fn expect_bool(table: &str, field: &FieldIr, value: &Value) -> Result<bool> {
    match value {
        Value::Bool(value) => Ok(*value),
        _ => Err(type_mismatch(table, field, "bool", value)),
    }
}

fn expect_integer(table: &str, field: &FieldIr, value: &Value) -> Result<i64> {
    match value {
        Value::Integer(value) => Ok(*value),
        _ => Err(type_mismatch(table, field, "integer", value)),
    }
}

fn expect_float(table: &str, field: &FieldIr, value: &Value) -> Result<f64> {
    match value {
        Value::Float(value) => Ok(*value),
        Value::Integer(value) => Ok(*value as f64),
        _ => Err(type_mismatch(table, field, "float", value)),
    }
}

fn expect_string<'a>(table: &str, field: &FieldIr, value: &'a Value) -> Result<&'a str> {
    match value {
        Value::String(value) => Ok(value),
        _ => Err(type_mismatch(table, field, "string", value)),
    }
}

fn expect_list<'a>(table: &str, field: &FieldIr, value: &'a Value) -> Result<&'a [Value]> {
    match value {
        Value::List(values) => Ok(values),
        _ => Err(type_mismatch(table, field, "list", value)),
    }
}

fn expect_object<'a>(
    table: &str,
    field: &FieldIr,
    value: &'a Value,
) -> Result<&'a BTreeMap<String, Value>> {
    match value {
        Value::Object(fields) => Ok(fields),
        _ => Err(type_mismatch(table, field, "object", value)),
    }
}

fn type_mismatch(table: &str, field: &FieldIr, expected: &str, actual: &Value) -> SoraError {
    SoraError::TypeMismatch {
        table: table.to_owned(),
        field: field.name.clone(),
        expected: expected.to_owned(),
        actual: kind_name(actual).to_owned(),
    }
}

fn kind_name(value: &Value) -> &'static str {
    match value {
        Value::Bool(_) => "bool",
        Value::Integer(_) => "integer",
        Value::Float(_) => "float",
        Value::String(_) => "string",
        Value::List(_) => "list",
        Value::Object(_) => "object",
        Value::Null => "null",
    }
}

#[derive(Default)]
struct ProtoWriter {
    bytes: Vec<u8>,
}

impl ProtoWriter {
    fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }

    fn bool(&mut self, tag: u32, value: bool) {
        self.uint64(tag, u64::from(value));
    }

    fn uint64(&mut self, tag: u32, value: u64) {
        self.key(tag, 0);
        self.varint(value);
    }

    fn int64(&mut self, tag: u32, value: i64) {
        self.uint64(tag, value as u64);
    }

    fn float(&mut self, tag: u32, value: f32) {
        self.key(tag, 5);
        self.bytes.extend(value.to_le_bytes());
    }

    fn double(&mut self, tag: u32, value: f64) {
        self.key(tag, 1);
        self.bytes.extend(value.to_le_bytes());
    }

    fn string(&mut self, tag: u32, value: &str) {
        self.bytes(tag, value.as_bytes());
    }

    fn message(
        &mut self,
        tag: u32,
        write: impl FnOnce(&mut ProtoWriter) -> Result<()>,
    ) -> Result<()> {
        let mut nested = ProtoWriter::default();
        write(&mut nested)?;
        self.bytes(tag, &nested.bytes);
        Ok(())
    }

    fn bytes(&mut self, tag: u32, value: &[u8]) {
        self.key(tag, 2);
        self.varint(value.len() as u64);
        self.bytes.extend(value);
    }

    fn key(&mut self, tag: u32, wire_type: u8) {
        self.varint(((tag as u64) << 3) | u64::from(wire_type));
    }

    fn varint(&mut self, mut value: u64) {
        while value >= 0x80 {
            self.bytes.push((value as u8) | 0x80);
            value >>= 7;
        }
        self.bytes.push(value as u8);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sora_data::model::TableData;
    use sora_ir::normalize::normalize_schema;
    use sora_schema::model::SchemaFile;

    #[test]
    fn typed_protobuf_exporter_writes_business_payload() {
        let schema: SchemaFile = toml::from_str(
            r#"
package = "game_config"

[[enums]]
name = "ItemType"
values = ["Weapon", "Armor"]

[[tables]]
name = "Item"
mode = "map"
key = "id"

[[tables.fields]]
name = "id"
type = "i32"

[[tables.fields]]
name = "name"
type = "string"

[[tables.fields]]
name = "item_type"
type = "enum<ItemType>"
"#,
        )
        .unwrap();
        let ir = normalize_schema(schema).unwrap();
        let data = ConfigData {
            tables: vec![TableData {
                name: "Item".to_owned(),
                rows: vec![RowData {
                    values: BTreeMap::from([
                        ("id".to_owned(), Value::Integer(1001)),
                        ("name".to_owned(), Value::String("Iron Sword".to_owned())),
                        ("item_type".to_owned(), Value::String("Weapon".to_owned())),
                    ]),
                }],
            }],
        };

        let bytes = encode_config(&ir, &data).unwrap();

        assert!(!bytes.is_empty());
        assert_eq!(bytes[0], 0x0a);
    }
}
