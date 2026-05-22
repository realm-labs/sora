use serde::Serialize;
use sora_data::model::TableData;
use sora_diagnostics::{Result, SoraError};

use crate::{
    exporter::{DataExporter, ExportOutput, ExportRequest, OutputKind},
    fs_util::{create_dir_all, write_file},
};

pub struct DebugJsonExporter;

impl DataExporter for DebugJsonExporter {
    fn format_name(&self) -> &'static str {
        "json-debug"
    }

    fn output_kind(&self) -> OutputKind {
        OutputKind::Directory
    }

    fn export(&self, request: ExportRequest<'_>) -> Result<()> {
        let ExportOutput::Directory(path) = request.output else {
            return Err(SoraError::InvalidExportOutput {
                format: self.format_name().to_owned(),
                expected: "directory",
            });
        };

        create_dir_all(&path)?;
        for table in &request.data.tables {
            let file_path = path.join(format!("{}.json", table.name));
            let content = serde_json::to_string_pretty(&DebugTableView { table })
                .map_err(SoraError::SerializeData)?;
            write_file(file_path, content)?;
        }

        Ok(())
    }
}

#[derive(Serialize)]
struct DebugTableView<'a> {
    table: &'a TableData,
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
    fn debug_json_exporter_writes_table_file() {
        let ir = example_ir();
        let data = example_data();
        let out_dir = temp_dir();

        DebugJsonExporter
            .export(ExportRequest {
                ir: &ir,
                data: &data,
                execution: &sora_execution::ExecutionContext::default(),
                options: Default::default(),
                output: ExportOutput::Directory(out_dir.clone()),
            })
            .unwrap();

        let content = fs::read_to_string(out_dir.join("Item.json")).unwrap();
        assert!(content.contains("\"name\": \"Item\""));
        assert!(content.contains("\"Iron Sword\""));

        let _ = fs::remove_dir_all(out_dir);
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
