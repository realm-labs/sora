use sora_data::model::{ConfigData, TableData, Value};
use sora_diagnostics::{Result, SoraError};
use sora_execution::ExecutionContext;
use sora_ir::model::{ConfigIr, FieldIr, StructIr, TableIr, TypeIr, UnionIr};

use crate::bundle::{data_fingerprint, schema_fingerprint};

const MAGIC: &[u8; 4] = b"SORA";
const VERSION: u32 = 1;
const HEADER_LEN: u32 = 24;
const SECTION_KIND_MANIFEST: u32 = 0;
const SECTION_KIND_SCHEMA: u32 = 1;
const SECTION_KIND_TABLE: u32 = 2;
const COMPRESSION_NONE: u32 = 0;

pub(crate) struct BinaryEncoder<'a> {
    ir: &'a ConfigIr,
    data: &'a ConfigData,
}

impl<'a> BinaryEncoder<'a> {
    pub(crate) fn new(ir: &'a ConfigIr, data: &'a ConfigData) -> Self {
        Self { ir, data }
    }

    pub(crate) fn encode(&self, execution: &ExecutionContext) -> Result<Vec<u8>> {
        let schema = self.ir.data_schema();
        let mut sections = Vec::new();
        sections.push(Section {
            kind: SECTION_KIND_MANIFEST,
            compression: COMPRESSION_NONE,
            name: "$manifest".to_owned(),
            payload: serde_json::to_vec(&self.manifest()?).map_err(SoraError::SerializeData)?,
        });
        sections.push(Section {
            kind: SECTION_KIND_SCHEMA,
            compression: COMPRESSION_NONE,
            name: "$schema".to_owned(),
            payload: serde_json::to_vec(&schema).map_err(SoraError::SerializeData)?,
        });

        let tables = self.ir.tables.iter().collect::<Vec<_>>();
        let mut table_sections = execution.map(tables, |table| {
            let table_data = self.table_data(&table.name)?;
            Ok(Section {
                kind: SECTION_KIND_TABLE,
                compression: COMPRESSION_NONE,
                name: table.name.clone(),
                payload: self.encode_table(table, table_data)?,
            })
        })?;
        sections.append(&mut table_sections);

        encode_bundle(sections)
    }

    fn manifest(&self) -> Result<BundleManifest> {
        let mut tables = Vec::new();
        for table in &self.ir.tables {
            let table_data = self.table_data(&table.name)?;
            tables.push(ManifestTable {
                name: table.name.clone(),
                rows: table_data.rows.len(),
            });
        }

        Ok(BundleManifest {
            format_version: VERSION,
            package: self.ir.package.clone(),
            schema_fingerprint: schema_fingerprint(self.ir)?,
            data_fingerprint: data_fingerprint(self.data)?,
            tables,
        })
    }

    fn table_data(&self, table_name: &str) -> Result<&'a TableData> {
        self.data
            .tables
            .iter()
            .find(|table| table.name == table_name)
            .ok_or_else(|| SoraError::InvalidSchema(format!("missing table data `{table_name}`")))
    }

    fn encode_table(&self, table: &TableIr, data: &TableData) -> Result<Vec<u8>> {
        let mut rows = Vec::new();
        for row in &data.rows {
            let mut row_bytes = Vec::new();
            for field in &table.fields {
                let null = Value::Null;
                let value = row.values.get(&field.name).unwrap_or(&null);
                self.encode_value(&field.ty, value, &mut row_bytes)?;
            }
            rows.push(row_bytes);
        }

        let row_count = rows.len();
        let offsets_len = row_count + 1;
        let row_data_start = 4 + offsets_len * 8;
        let mut payload = Vec::new();
        write_u32(&mut payload, checked_u32(row_count, "row count")?);

        let mut offset = 0_u64;
        for row in &rows {
            write_u64(&mut payload, offset);
            offset = offset
                .checked_add(row.len() as u64)
                .ok_or_else(|| binary_error("table row payload is too large"))?;
        }
        write_u64(&mut payload, offset);

        debug_assert_eq!(payload.len(), row_data_start);
        for row in rows {
            payload.extend_from_slice(&row);
        }

        Ok(payload)
    }

    fn encode_value(&self, ty: &TypeIr, value: &Value, out: &mut Vec<u8>) -> Result<()> {
        match ty {
            TypeIr::Optional(inner) => {
                if matches!(value, Value::Null) {
                    write_u8(out, 0);
                } else {
                    write_u8(out, 1);
                    self.encode_value(inner, value, out)?;
                }
            }
            TypeIr::Bool => {
                let Value::Bool(value) = value else {
                    return Err(type_error(ty, value));
                };
                write_u8(out, u8::from(*value));
            }
            TypeIr::I32 => {
                let Value::Integer(value) = value else {
                    return Err(type_error(ty, value));
                };
                let value = i32::try_from(*value).map_err(|_| {
                    binary_error(format!("cannot encode integer `{value}` as `{ty}`"))
                })?;
                write_i32(out, value);
            }
            TypeIr::I64 => {
                let Value::Integer(value) = value else {
                    return Err(type_error(ty, value));
                };
                write_i64(out, *value);
            }
            TypeIr::F32 => match value {
                Value::Integer(value) => write_f32(out, *value as f32),
                Value::Float(value) => write_f32(out, *value as f32),
                _ => return Err(type_error(ty, value)),
            },
            TypeIr::F64 => match value {
                Value::Integer(value) => write_f64(out, *value as f64),
                Value::Float(value) => write_f64(out, *value),
                _ => return Err(type_error(ty, value)),
            },
            TypeIr::String => {
                let Value::String(value) = value else {
                    return Err(type_error(ty, value));
                };
                write_string(out, value)?;
            }
            TypeIr::Enum(enum_name) => {
                let Value::String(value) = value else {
                    return Err(type_error(ty, value));
                };
                let ordinal = self.enum_ordinal(enum_name, value)?;
                write_u32(out, ordinal);
            }
            TypeIr::Struct(struct_name) => {
                let Value::Object(values) = value else {
                    return Err(type_error(ty, value));
                };
                let struct_ir = self.struct_ir(struct_name)?;
                for field in &struct_ir.fields {
                    let null = Value::Null;
                    let value = values.get(&field.name).unwrap_or(&null);
                    self.encode_value(&field.ty, value, out)?;
                }
            }
            TypeIr::Union(union_name) => {
                let Value::Object(values) = value else {
                    return Err(type_error(ty, value));
                };
                let union_ir = self.union_ir(union_name)?;
                let Some(Value::String(variant_name)) = values.get(&union_ir.tag) else {
                    return Err(binary_error(format!(
                        "union `{union_name}` value is missing string tag `{}`",
                        union_ir.tag
                    )));
                };
                let Some((ordinal, variant)) = union_ir
                    .variants
                    .iter()
                    .enumerate()
                    .find(|(_, candidate)| candidate.name == *variant_name)
                else {
                    return Err(binary_error(format!(
                        "unknown union variant `{union_name}.{variant_name}`"
                    )));
                };
                write_u32(out, checked_u32(ordinal, "union ordinal")?);
                for field in &variant.fields {
                    let null = Value::Null;
                    let value = values.get(&field.name).unwrap_or(&null);
                    self.encode_value(&field.ty, value, out)?;
                }
            }
            TypeIr::List(element) | TypeIr::Set(element) => {
                let Value::List(values) = value else {
                    return Err(type_error(ty, value));
                };
                write_u32(out, checked_u32(values.len(), "list length")?);
                for value in values {
                    self.encode_value(element, value, out)?;
                }
            }
            TypeIr::Map {
                key,
                value: element,
            } => {
                let Value::List(values) = value else {
                    return Err(type_error(ty, value));
                };
                write_u32(out, checked_u32(values.len(), "map length")?);
                for entry in values {
                    let Value::List(pair) = entry else {
                        return Err(type_error(ty, entry));
                    };
                    let [entry_key, entry_value] = pair.as_slice() else {
                        return Err(type_error(ty, entry));
                    };
                    self.encode_value(key, entry_key, out)?;
                    self.encode_value(element, entry_value, out)?;
                }
            }
            TypeIr::Array { element, len } => {
                let Value::List(values) = value else {
                    return Err(type_error(ty, value));
                };
                if values.len() != *len {
                    return Err(type_error(ty, value));
                }
                write_u32(out, checked_u32(values.len(), "array length")?);
                for value in values {
                    self.encode_value(element, value, out)?;
                }
            }
            TypeIr::Ref { table, field } => {
                let target_ty = self.ref_target_type(table, field)?;
                self.encode_value(target_ty, value, out)?;
            }
        }

        Ok(())
    }

    fn enum_ordinal(&self, enum_name: &str, value: &str) -> Result<u32> {
        let enum_ir = self
            .ir
            .enums
            .iter()
            .find(|candidate| candidate.name == enum_name)
            .ok_or_else(|| binary_error(format!("unknown enum `{enum_name}`")))?;
        if let Some(entry) = enum_ir
            .aliases
            .iter()
            .find(|candidate| candidate.name == value || candidate.alias == value)
        {
            let ordinal = enum_ir
                .values
                .iter()
                .position(|candidate| candidate == &entry.name)
                .ok_or_else(|| binary_error(format!("unknown enum value `{enum_name}.{value}`")))?;
            return checked_u32(ordinal, "enum ordinal");
        }

        let ordinal = enum_ir
            .values
            .iter()
            .position(|candidate| candidate == value)
            .ok_or_else(|| binary_error(format!("unknown enum value `{enum_name}.{value}`")))?;

        checked_u32(ordinal, "enum ordinal")
    }

    fn struct_ir(&self, struct_name: &str) -> Result<&StructIr> {
        self.ir
            .structs
            .iter()
            .find(|candidate| candidate.name == struct_name)
            .ok_or_else(|| binary_error(format!("unknown struct `{struct_name}`")))
    }

    fn union_ir(&self, union_name: &str) -> Result<&UnionIr> {
        self.ir
            .unions
            .iter()
            .find(|candidate| candidate.name == union_name)
            .ok_or_else(|| binary_error(format!("unknown union `{union_name}`")))
    }

    fn ref_target_type(&self, table_name: &str, field_name: &str) -> Result<&TypeIr> {
        self.ir
            .tables
            .iter()
            .find(|candidate| candidate.name == table_name)
            .and_then(|table| table.fields.iter().find(|field| field.name == field_name))
            .map(|field: &FieldIr| &field.ty)
            .ok_or_else(|| binary_error(format!("unknown ref target `{table_name}.{field_name}`")))
    }
}

#[derive(serde::Serialize)]
struct BundleManifest {
    format_version: u32,
    package: String,
    schema_fingerprint: String,
    data_fingerprint: String,
    tables: Vec<ManifestTable>,
}

#[derive(serde::Serialize)]
struct ManifestTable {
    name: String,
    rows: usize,
}

struct Section {
    kind: u32,
    compression: u32,
    name: String,
    payload: Vec<u8>,
}

fn encode_bundle(sections: Vec<Section>) -> Result<Vec<u8>> {
    let section_count = sections.len();
    let directory_len = sections
        .iter()
        .map(|section| 40_usize + section.name.len())
        .sum::<usize>();
    let mut offset = u64::from(HEADER_LEN)
        .checked_add(directory_len as u64)
        .ok_or_else(|| binary_error("binary directory is too large"))?;

    let mut directory = Vec::with_capacity(directory_len);
    let mut payload = Vec::new();
    for section in sections {
        write_u32(&mut directory, section.kind);
        write_u32(&mut directory, section.compression);
        write_u32(
            &mut directory,
            checked_u32(section.name.len(), "section name length")?,
        );
        write_u32(&mut directory, 0);
        write_u64(&mut directory, offset);
        write_u64(&mut directory, section.payload.len() as u64);
        write_u64(&mut directory, section.payload.len() as u64);
        directory.extend_from_slice(section.name.as_bytes());

        offset = offset
            .checked_add(section.payload.len() as u64)
            .ok_or_else(|| binary_error("binary payload is too large"))?;
        payload.extend_from_slice(&section.payload);
    }

    let mut bundle = Vec::with_capacity(HEADER_LEN as usize + directory.len() + payload.len());
    bundle.extend_from_slice(MAGIC);
    write_u32(&mut bundle, VERSION);
    write_u32(&mut bundle, HEADER_LEN);
    write_u32(
        &mut bundle,
        checked_u32(directory_len, "section directory length")?,
    );
    write_u32(&mut bundle, checked_u32(section_count, "section count")?);
    write_u32(&mut bundle, 0);
    bundle.extend_from_slice(&directory);
    bundle.extend_from_slice(&payload);

    Ok(bundle)
}

fn write_u8(out: &mut Vec<u8>, value: u8) {
    out.push(value);
}

fn write_u32(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn write_i32(out: &mut Vec<u8>, value: i32) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn write_u64(out: &mut Vec<u8>, value: u64) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn write_i64(out: &mut Vec<u8>, value: i64) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn write_f32(out: &mut Vec<u8>, value: f32) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn write_f64(out: &mut Vec<u8>, value: f64) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn write_string(out: &mut Vec<u8>, value: &str) -> Result<()> {
    write_u32(out, checked_u32(value.len(), "string length")?);
    out.extend_from_slice(value.as_bytes());
    Ok(())
}

fn checked_u32(value: usize, kind: &'static str) -> Result<u32> {
    u32::try_from(value).map_err(|_| binary_error(format!("{kind} exceeds u32::MAX")))
}

fn type_error(ty: &TypeIr, value: &Value) -> SoraError {
    binary_error(format!(
        "cannot encode `{}` value as `{}`",
        value_kind_name(value),
        ty
    ))
}

fn binary_error(message: impl Into<String>) -> SoraError {
    SoraError::InvalidSchema(format!("binary export: {}", message.into()))
}

fn value_kind_name(value: &Value) -> &'static str {
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
