use sora_diagnostics::{Result, SoraError};

mod encoder;

use crate::{
    exporter::{DataExporter, ExportOutput, ExportRequest, OutputKind},
    fs_util::{create_dir_all, write_file},
};

use self::encoder::BinaryEncoder;

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

        let bundle = BinaryEncoder::new(request.ir, request.data).encode(request.execution)?;

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
                execution: &sora_execution::ExecutionContext::default(),
                output: ExportOutput::File(path.clone()),
            })
            .unwrap();

        let bytes = fs::read(&path).unwrap();
        assert_eq!(&bytes[0..4], b"SORA");
        assert_eq!(u32::from_le_bytes(bytes[4..8].try_into().unwrap()), 1);
        assert_eq!(u32::from_le_bytes(bytes[8..12].try_into().unwrap()), 24);
        assert!(u32::from_le_bytes(bytes[12..16].try_into().unwrap()) > 0);
        assert_eq!(u32::from_le_bytes(bytes[16..20].try_into().unwrap()), 3);

        let sections = read_sections(&bytes);
        assert_eq!(sections[0].kind, 0);
        assert_eq!(sections[0].compression, 0);
        assert_eq!(sections[0].name, "$manifest");
        assert_eq!(sections[0].len, sections[0].uncompressed_len);
        let manifest: serde_json::Value = serde_json::from_slice(
            &bytes[sections[0].offset..sections[0].offset + sections[0].len],
        )
        .unwrap();
        assert_eq!(manifest["format_version"], 1);
        assert_eq!(manifest["package"], "game_config");
        assert_eq!(manifest["tables"][0]["name"], "Item");
        assert_eq!(manifest["tables"][0]["rows"], 1);
        assert!(manifest["schema_fingerprint"].as_str().unwrap().len() > 8);

        assert_eq!(sections[1].kind, 1);
        assert_eq!(sections[1].compression, 0);
        assert_eq!(sections[1].name, "$schema");
        assert_eq!(sections[1].len, sections[1].uncompressed_len);
        assert_eq!(sections[2].kind, 2);
        assert_eq!(sections[2].compression, 0);
        assert_eq!(sections[2].name, "Item");
        assert_eq!(sections[2].len, sections[2].uncompressed_len);

        let table_payload = &bytes[sections[2].offset..sections[2].offset + sections[2].len];
        assert_eq!(read_u32(table_payload, 0), 1);
        assert_eq!(read_u64(table_payload, 4), 0);
        assert_eq!(read_u64(table_payload, 12), 4);
        assert_eq!(
            i32::from_le_bytes(table_payload[20..24].try_into().unwrap()),
            1001
        );

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

    #[derive(Debug)]
    struct TestSection {
        kind: u32,
        compression: u32,
        name: String,
        offset: usize,
        len: usize,
        uncompressed_len: usize,
    }

    fn read_sections(bytes: &[u8]) -> Vec<TestSection> {
        let directory_len = read_u32(bytes, 12) as usize;
        let section_count = read_u32(bytes, 16) as usize;
        let mut cursor = 24;
        let directory_end = cursor + directory_len;
        let mut sections = Vec::new();
        while cursor < directory_end {
            let kind = read_u32(bytes, cursor);
            let compression = read_u32(bytes, cursor + 4);
            let name_len = read_u32(bytes, cursor + 8) as usize;
            let offset = read_u64(bytes, cursor + 16) as usize;
            let len = read_u64(bytes, cursor + 24) as usize;
            let uncompressed_len = read_u64(bytes, cursor + 32) as usize;
            let name_start = cursor + 40;
            let name = std::str::from_utf8(&bytes[name_start..name_start + name_len])
                .unwrap()
                .to_owned();
            sections.push(TestSection {
                kind,
                compression,
                name,
                offset,
                len,
                uncompressed_len,
            });
            cursor = name_start + name_len;
        }
        assert_eq!(sections.len(), section_count);
        sections
    }

    fn read_u32(bytes: &[u8], offset: usize) -> u32 {
        u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
    }

    fn read_u64(bytes: &[u8], offset: usize) -> u64 {
        u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap())
    }
}
