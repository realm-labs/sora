use sora_diagnostics::{Result, SoraError};

use crate::{
    bundle::DataBundleView,
    exporter::{DataExporter, ExportOutput, ExportRequest, OutputKind},
    fs_util::{create_parent_dir, write_file},
};

pub struct CborBundleExporter;

impl DataExporter for CborBundleExporter {
    fn format_name(&self) -> &'static str {
        "cbor"
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
        let content = serde_cbor::to_vec(&DataBundleView::new("cbor", request.ir, request.data)?)
            .map_err(|error| SoraError::SerializeDataFormat {
            format: self.format_name(),
            message: error.to_string(),
        })?;
        write_file(path, content)
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
        sync::atomic::{AtomicUsize, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    #[test]
    fn cbor_exporter_writes_bundle_file() {
        let ir = example_ir();
        let data = example_data();
        let path = temp_dir().join("config.cbor");

        CborBundleExporter
            .export(ExportRequest {
                ir: &ir,
                data: &data,
                execution: &sora_execution::ExecutionContext::default(),
                options: Default::default(),
                output: ExportOutput::File(path.clone()),
            })
            .unwrap();

        let value: serde_cbor::Value = serde_cbor::from_slice(&fs::read(&path).unwrap()).unwrap();
        let serde_cbor::Value::Map(fields) = value else {
            panic!("expected cbor map");
        };
        assert!(fields.iter().any(|(key, value)| {
            matches!(key, serde_cbor::Value::Text(key) if key == "format")
                && matches!(value, serde_cbor::Value::Text(value) if value == "cbor")
        }));
        assert!(fields.iter().any(|(key, value)| {
            matches!(key, serde_cbor::Value::Text(key) if key == "schema_fingerprint")
                && matches!(value, serde_cbor::Value::Text(value) if value.len() > 8)
        }));
        assert!(fields.iter().any(|(key, value)| {
            matches!(key, serde_cbor::Value::Text(key) if key == "data_fingerprint")
                && matches!(value, serde_cbor::Value::Text(value) if value.len() > 8)
        }));

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
        static NEXT_ID: AtomicUsize = AtomicUsize::new(0);
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("sora-export-test-{unique}-{id}"))
    }
}
