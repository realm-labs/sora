use serde::Serialize;
use sora_data::model::TableData;
use sora_diagnostics::{Result, SoraError};

use crate::{
    exporter::{DataExporter, ExportOutput, ExportRequest, OutputKind},
    fs_util::{create_dir_all, create_parent_dir, write_file},
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
            let mut file_path = path.clone();
            let mut segments = table.name.split('.').peekable();
            while let Some(segment) = segments.next() {
                if segments.peek().is_some() {
                    file_path.push(segment);
                } else {
                    file_path.push(format!("{segment}.json"));
                }
            }
            create_parent_dir(&file_path)?;
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
    use sora_schema::model::ProjectSchema;
    use std::{
        collections::BTreeMap,
        fs,
        path::PathBuf,
        sync::atomic::{AtomicUsize, Ordering},
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
                locale_catalog: None,
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

    #[test]
    fn debug_json_exporter_uses_namespace_directories() {
        let ir = example_ir();
        let mut data = example_data();
        data.tables[0].name = "items.Item".to_owned();
        let out_dir = temp_dir();

        DebugJsonExporter
            .export(ExportRequest {
                ir: &ir,
                data: &data,
                locale_catalog: None,
                execution: &sora_execution::ExecutionContext::default(),
                options: Default::default(),
                output: ExportOutput::Directory(out_dir.clone()),
            })
            .unwrap();

        assert!(out_dir.join("items/Item.json").is_file());
        let _ = fs::remove_dir_all(out_dir);
    }

    fn example_ir() -> ConfigIr {
        let schema: ProjectSchema = toml::from_str(
            r#"
project = { id = "game_config" }
groups = { common = { default = true } }
views = { default = { contract = "game_config/default", groups = ["common"] } }

[[tables]]
id = "item"
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
        std::env::temp_dir().join(format!("sora-export-debug-json-test-{unique}-{id}"))
    }
}
