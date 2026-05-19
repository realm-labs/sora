use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use serde::Serialize;
use sora_data::{ConfigData, TableData};
use sora_diagnostics::{Result, SoraError};
use sora_ir::ConfigIr;

const BINARY_MAGIC: &[u8; 4] = b"SORA";
const BINARY_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExportOutput {
    Directory(PathBuf),
    File(PathBuf),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputKind {
    Directory,
    File,
}

pub struct ExportRequest<'a> {
    pub ir: &'a ConfigIr,
    pub data: &'a ConfigData,
    pub output: ExportOutput,
}

pub trait DataExporter: Send + Sync {
    fn format_name(&self) -> &'static str;
    fn output_kind(&self) -> OutputKind;
    fn export(&self, request: ExportRequest<'_>) -> Result<()>;
}

#[derive(Default)]
pub struct ExporterRegistry {
    exporters: BTreeMap<String, Box<dyn DataExporter>>,
}

impl ExporterRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_builtin_exporters() -> Self {
        let mut registry = Self::new();
        registry.register(BinaryBundleExporter);
        registry.register(DebugJsonExporter);
        registry
    }

    pub fn register<E: DataExporter + 'static>(&mut self, exporter: E) {
        self.exporters
            .insert(exporter.format_name().to_owned(), Box::new(exporter));
    }

    pub fn get(&self, format_name: &str) -> Option<&dyn DataExporter> {
        self.exporters.get(format_name).map(Box::as_ref)
    }

    pub fn supported_formats(&self) -> Vec<&'static str> {
        self.exporters
            .values()
            .map(|exporter| exporter.format_name())
            .collect()
    }
}

pub struct BinaryBundleExporter;
pub struct DebugJsonExporter;

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

        fs::write(&path, bundle).map_err(|source| SoraError::WriteFile { path, source })
    }
}

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
            fs::write(&file_path, content).map_err(|source| SoraError::WriteFile {
                path: file_path,
                source,
            })?;
        }

        Ok(())
    }
}

pub fn builtin_supported_formats() -> Vec<&'static str> {
    ExporterRegistry::with_builtin_exporters().supported_formats()
}

fn deterministic_json_bytes(value: &impl Serialize) -> Result<Vec<u8>> {
    serde_json::to_vec(value).map_err(SoraError::SerializeData)
}

fn create_dir_all(path: &Path) -> Result<()> {
    fs::create_dir_all(path).map_err(|source| SoraError::CreateDir {
        path: path.to_path_buf(),
        source,
    })
}

#[derive(Serialize)]
struct DebugTableView<'a> {
    table: &'a TableData,
}

#[cfg(test)]
mod tests {
    use super::*;
    use sora_data::{RowData, TableData, Value};
    use sora_ir::{ConfigIr, normalize_schema};
    use sora_schema::SchemaFile;
    use std::{
        collections::BTreeMap,
        time::{SystemTime, UNIX_EPOCH},
    };

    #[test]
    fn registry_finds_builtin_exporters() {
        let registry = ExporterRegistry::with_builtin_exporters();

        assert_eq!(
            registry.get("binary").unwrap().output_kind(),
            OutputKind::File
        );
        assert_eq!(
            registry.get("json-debug").unwrap().output_kind(),
            OutputKind::Directory
        );
        assert!(registry.get("unknown").is_none());
        assert_eq!(registry.supported_formats(), vec!["binary", "json-debug"]);
    }

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

    #[test]
    fn debug_json_exporter_writes_table_file() {
        let ir = example_ir();
        let data = example_data();
        let out_dir = temp_dir();

        DebugJsonExporter
            .export(ExportRequest {
                ir: &ir,
                data: &data,
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
