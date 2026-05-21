use std::collections::BTreeMap;

use serde::Serialize;
use sora_data::model::{ConfigData, RowData, TableData, Value};
use sora_diagnostics::{Result, SoraError};
use sora_ir::model::ConfigIr;

pub(crate) const FORMAT_VERSION: u32 = 1;

#[derive(Serialize)]
pub(crate) struct DataBundleView<'a> {
    pub format: &'static str,
    pub format_version: u32,
    pub schema_fingerprint: String,
    pub data_fingerprint: String,
    pub schema: ConfigIr,
    pub data: NaturalConfigDataView<'a>,
}

impl<'a> DataBundleView<'a> {
    pub(crate) fn new(
        format: &'static str,
        ir: &'a ConfigIr,
        data: &'a ConfigData,
    ) -> Result<Self> {
        Ok(Self {
            format,
            format_version: FORMAT_VERSION,
            schema_fingerprint: schema_fingerprint(ir)?,
            data_fingerprint: data_fingerprint(data)?,
            schema: ir.data_schema(),
            data: NaturalConfigDataView(data),
        })
    }
}

pub(crate) fn schema_fingerprint(ir: &ConfigIr) -> Result<String> {
    let schema = ir.data_schema();
    let bytes = serde_json::to_vec(&schema).map_err(SoraError::SerializeData)?;
    Ok(fingerprint_hex(&bytes))
}

pub(crate) fn data_fingerprint(data: &ConfigData) -> Result<String> {
    let bytes =
        serde_json::to_vec(&NaturalConfigDataView(data)).map_err(SoraError::SerializeData)?;
    Ok(fingerprint_hex(&bytes))
}

pub(crate) fn fingerprint_hex(bytes: &[u8]) -> String {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }

    format!("{hash:016x}")
}

pub(crate) struct NaturalConfigDataView<'a>(&'a ConfigData);

impl Serialize for NaturalConfigDataView<'_> {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        NaturalConfigData {
            tables: self.0.tables.iter().map(NaturalTableDataView).collect(),
        }
        .serialize(serializer)
    }
}

#[derive(Serialize)]
struct NaturalConfigData<'a> {
    tables: Vec<NaturalTableDataView<'a>>,
}

struct NaturalTableDataView<'a>(&'a TableData);

impl Serialize for NaturalTableDataView<'_> {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        NaturalTableData {
            name: &self.0.name,
            rows: self.0.rows.iter().map(NaturalRowDataView).collect(),
        }
        .serialize(serializer)
    }
}

#[derive(Serialize)]
struct NaturalTableData<'a> {
    name: &'a str,
    rows: Vec<NaturalRowDataView<'a>>,
}

struct NaturalRowDataView<'a>(&'a RowData);

impl Serialize for NaturalRowDataView<'_> {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let values = self
            .0
            .values
            .iter()
            .map(|(key, value)| (key.as_str(), NaturalValueView(value)))
            .collect::<BTreeMap<_, _>>();
        values.serialize(serializer)
    }
}

struct NaturalValueView<'a>(&'a Value);

impl Serialize for NaturalValueView<'_> {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self.0 {
            Value::Bool(value) => value.serialize(serializer),
            Value::Integer(value) => value.serialize(serializer),
            Value::Float(value) => value.serialize(serializer),
            Value::String(value) => value.serialize(serializer),
            Value::List(values) => serializer.collect_seq(values.iter().map(NaturalValueView)),
            Value::Object(fields) => {
                let values = fields
                    .iter()
                    .map(|(key, value)| (key.as_str(), NaturalValueView(value)))
                    .collect::<BTreeMap<_, _>>();
                values.serialize(serializer)
            }
            Value::Null => serializer.serialize_none(),
        }
    }
}
