use prost::Message;
use sora_data::model::{ConfigData, Value};
use sora_diagnostics::{Result, SoraError};
use sora_ir::model::ConfigIr;

use crate::{
    bundle::{FORMAT_VERSION, data_fingerprint, schema_fingerprint},
    exporter::{DataExporter, ExportOutput, ExportRequest, OutputKind},
    fs_util::{create_parent_dir, write_file},
};

pub struct ProtobufBundleExporter;

impl DataExporter for ProtobufBundleExporter {
    fn format_name(&self) -> &'static str {
        "sora-protobuf"
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
        let bundle = ProtoBundle::from_data(request.ir, request.data)?;
        write_file(path, bundle.encode_to_vec())
    }
}

#[derive(Clone, PartialEq, Message)]
struct ProtoBundle {
    #[prost(uint32, tag = "1")]
    format_version: u32,
    #[prost(string, tag = "2")]
    format: String,
    #[prost(string, tag = "3")]
    package: String,
    #[prost(bytes = "vec", tag = "4")]
    schema_json: Vec<u8>,
    #[prost(message, repeated, tag = "5")]
    tables: Vec<ProtoTable>,
    #[prost(string, tag = "6")]
    schema_fingerprint: String,
    #[prost(string, tag = "7")]
    data_fingerprint: String,
}

#[derive(Clone, PartialEq, Message)]
struct ProtoTable {
    #[prost(string, tag = "1")]
    name: String,
    #[prost(message, repeated, tag = "2")]
    rows: Vec<ProtoRow>,
}

#[derive(Clone, PartialEq, Message)]
struct ProtoRow {
    #[prost(message, repeated, tag = "1")]
    fields: Vec<ProtoField>,
}

#[derive(Clone, PartialEq, Message)]
struct ProtoField {
    #[prost(string, tag = "1")]
    name: String,
    #[prost(message, optional, tag = "2")]
    value: Option<ProtoValue>,
}

#[derive(Clone, PartialEq, Message)]
struct ProtoValue {
    #[prost(oneof = "proto_value::Kind", tags = "1, 2, 3, 4, 5, 6, 7")]
    kind: Option<proto_value::Kind>,
}

mod proto_value {
    #[derive(Clone, PartialEq, prost::Oneof)]
    pub enum Kind {
        #[prost(bool, tag = "1")]
        Bool(bool),
        #[prost(int64, tag = "2")]
        Integer(i64),
        #[prost(double, tag = "3")]
        Float(f64),
        #[prost(string, tag = "4")]
        String(String),
        #[prost(message, tag = "5")]
        List(super::ProtoList),
        #[prost(message, tag = "6")]
        Object(super::ProtoObject),
        #[prost(bool, tag = "7")]
        Null(bool),
    }
}

#[derive(Clone, PartialEq, Message)]
struct ProtoList {
    #[prost(message, repeated, tag = "1")]
    values: Vec<ProtoValue>,
}

#[derive(Clone, PartialEq, Message)]
struct ProtoObject {
    #[prost(message, repeated, tag = "1")]
    fields: Vec<ProtoField>,
}

impl ProtoBundle {
    fn from_data(ir: &ConfigIr, data: &ConfigData) -> Result<Self> {
        let schema = ir.data_schema();
        let schema_json = serde_json::to_vec(&schema).map_err(SoraError::SerializeData)?;
        Ok(Self {
            format_version: FORMAT_VERSION,
            format: "sora-protobuf".to_owned(),
            package: schema.package.clone(),
            schema_json,
            schema_fingerprint: schema_fingerprint(ir)?,
            data_fingerprint: data_fingerprint(data)?,
            tables: data
                .tables
                .iter()
                .map(|table| ProtoTable {
                    name: table.name.clone(),
                    rows: table
                        .rows
                        .iter()
                        .map(|row| ProtoRow {
                            fields: row
                                .values
                                .iter()
                                .map(|(name, value)| ProtoField {
                                    name: name.clone(),
                                    value: Some(ProtoValue::from(value)),
                                })
                                .collect(),
                        })
                        .collect(),
                })
                .collect(),
        })
    }
}

impl From<&Value> for ProtoValue {
    fn from(value: &Value) -> Self {
        let kind = match value {
            Value::Bool(value) => proto_value::Kind::Bool(*value),
            Value::Integer(value) => proto_value::Kind::Integer(*value),
            Value::Float(value) => proto_value::Kind::Float(*value),
            Value::String(value) => proto_value::Kind::String(value.clone()),
            Value::List(values) => proto_value::Kind::List(ProtoList {
                values: values.iter().map(ProtoValue::from).collect(),
            }),
            Value::Object(fields) => proto_value::Kind::Object(ProtoObject {
                fields: fields
                    .iter()
                    .map(|(name, value)| ProtoField {
                        name: name.clone(),
                        value: Some(ProtoValue::from(value)),
                    })
                    .collect(),
            }),
            Value::Null => proto_value::Kind::Null(true),
        };

        Self { kind: Some(kind) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::exporter::ExportOutput;
    use sora_data::model::{RowData, TableData};
    use sora_ir::{model::ConfigIr, normalize::normalize_schema};
    use sora_schema::model::SchemaFile;
    use std::{
        collections::BTreeMap,
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    #[test]
    fn protobuf_exporter_writes_bundle_file() {
        let ir = example_ir();
        let data = example_data();
        let path = temp_dir().join("config.sora.pb");

        ProtobufBundleExporter
            .export(ExportRequest {
                ir: &ir,
                data: &data,
                execution: &sora_execution::ExecutionContext::default(),
                options: Default::default(),
                output: ExportOutput::File(path.clone()),
            })
            .unwrap();

        let bundle = ProtoBundle::decode(fs::read(&path).unwrap().as_slice()).unwrap();
        assert_eq!(bundle.format_version, 1);
        assert_eq!(bundle.format, "sora-protobuf");
        assert_eq!(bundle.package, "game_config");
        assert!(bundle.schema_fingerprint.len() > 8);
        assert!(bundle.data_fingerprint.len() > 8);
        assert_eq!(bundle.tables[0].name, "Item");
        assert_eq!(bundle.tables[0].rows[0].fields[0].name, "id");

        let _ = fs::remove_dir_all(path.parent().unwrap());
    }

    fn example_ir() -> ConfigIr {
        let schema: SchemaFile = toml::from_str(
            r#"
package = "game_config"

[[tables]]
name = "Item"
mode = "map"
key = "id"

[[tables.fields]]
name = "id"
type = "i32"
required = true
"#,
        )
        .unwrap();
        normalize_schema(schema).unwrap()
    }

    fn example_data() -> ConfigData {
        ConfigData {
            tables: vec![TableData {
                name: "Item".to_owned(),
                rows: vec![RowData {
                    values: BTreeMap::from([
                        ("id".to_owned(), Value::Integer(1001)),
                        ("name".to_owned(), Value::String("Iron Sword".to_owned())),
                    ]),
                }],
            }],
        }
    }

    fn temp_dir() -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("sora-export-test-{unique}"))
    }
}
