use sora_diagnostics::{Result, SoraError};

use crate::{
    exporter::{DataExporter, ExportOutput, ExportRequest, OutputKind},
    fs_util::{create_dir_all, deterministic_json_bytes, write_file},
};

const BINARY_MAGIC: &[u8; 4] = b"SORA";
const BINARY_VERSION: u32 = 1;

pub struct BinaryBundleExporter;

impl DataExporter for BinaryBundleExporter {
    fn format_name(&self) -> &'static str {
        "binary"
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

        if let Some(parent) = path.parent() {
            create_dir_all(parent)?;
        }

        let schema_payload = deterministic_json_bytes(request.ir)?;
        let data_payload = deterministic_json_bytes(request.data)?;
        let mut bundle = Vec::with_capacity(16 + schema_payload.len() + data_payload.len());
        bundle.extend_from_slice(BINARY_MAGIC);
        bundle.extend_from_slice(&BINARY_VERSION.to_le_bytes());
        bundle.extend_from_slice(&(schema_payload.len() as u32).to_le_bytes());
        bundle.extend_from_slice(&(data_payload.len() as u32).to_le_bytes());
        bundle.extend_from_slice(&schema_payload);
        bundle.extend_from_slice(&data_payload);

        write_file(path, bundle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::exporter::ExportOutput;
    use sora_data::model::{ConfigData, RowData, TableData, Value};
    use sora_ir::{model::ConfigIr, normalize::normalize_schema};
    use sora_schema::model::SchemaFile;
    use std::{
        collections::BTreeMap,
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    #[test]
    fn binary_bundle_has_expected_header() {
        let ir = example_ir();
        let data = example_data();
        let path = temp_dir().join("config.sora");

        BinaryBundleExporter
            .export(ExportRequest {
                ir: &ir,
                data: &data,
                output: ExportOutput::File(path.clone()),
            })
            .unwrap();

        let bytes = fs::read(&path).unwrap();
        assert_eq!(&bytes[0..4], b"SORA");
        assert_eq!(u32::from_le_bytes(bytes[4..8].try_into().unwrap()), 1);
        assert!(u32::from_le_bytes(bytes[8..12].try_into().unwrap()) > 0);
        assert!(u32::from_le_bytes(bytes[12..16].try_into().unwrap()) > 0);

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
