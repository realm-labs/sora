use sora_diagnostics::{Result, SoraError};

use crate::{
    bundle::DataBundleView,
    exporter::{DataExporter, ExportOutput, ExportRequest, OutputKind},
    fs_util::{create_parent_dir, write_file},
};

pub struct JsonBundleExporter;

impl DataExporter for JsonBundleExporter {
    fn format_name(&self) -> &'static str {
        "json"
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
        let content =
            serde_json::to_vec_pretty(&DataBundleView::new("json", request.ir, request.data)?)
                .map_err(SoraError::SerializeData)?;
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
    fn json_exporter_writes_bundle_file() {
        let ir = example_ir();
        let data = example_data();
        let path = temp_dir().join("config.json");

        JsonBundleExporter
            .export(ExportRequest {
                ir: &ir,
                data: &data,
                locale_catalog: None,
                execution: &sora_execution::ExecutionContext::default(),
                options: Default::default(),
                output: ExportOutput::File(path.clone()),
            })
            .unwrap();

        let value: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        assert_eq!(value["format"], "json");
        assert_eq!(value["format_version"], 1);
        assert!(value["schema_fingerprint"].as_str().unwrap().len() > 8);
        assert!(value["data_fingerprint"].as_str().unwrap().len() > 8);
        assert_eq!(value["schema"]["package"], "game_config");
        assert_eq!(value["data"]["tables"][0]["name"], "Item");

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
        std::env::temp_dir().join(format!("sora-export-json-test-{unique}-{id}"))
    }
}
