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

        let bundle = BinaryEncoder::new(request.ir, request.data, request.options.compression)
            .encode(request.execution)?;

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
        sync::atomic::{AtomicUsize, Ordering},
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
                locale_catalog: None,
                execution: &sora_execution::ExecutionContext::default(),
                options: Default::default(),
                output: ExportOutput::File(path.clone()),
            })
            .unwrap();

        let bytes = fs::read(&path).unwrap();
        assert_eq!(&bytes[0..4], b"SORA");
        assert_eq!(u32::from_le_bytes(bytes[4..8].try_into().unwrap()), 1);
        assert_eq!(u32::from_le_bytes(bytes[8..12].try_into().unwrap()), 24);
        assert!(u32::from_le_bytes(bytes[12..16].try_into().unwrap()) > 0);
        assert_eq!(u32::from_le_bytes(bytes[16..20].try_into().unwrap()), 4);

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
        assert!(manifest["data_fingerprint"].as_str().unwrap().len() > 8);

        assert_eq!(sections[1].kind, 1);
        assert_eq!(sections[1].compression, 0);
        assert_eq!(sections[1].name, "$schema");
        assert_eq!(sections[1].len, sections[1].uncompressed_len);
        assert_eq!(sections[2].kind, 3);
        assert_eq!(sections[2].compression, 0);
        assert_eq!(sections[2].name, "$strings");
        assert_eq!(sections[2].len, sections[2].uncompressed_len);
        let strings_payload = &bytes[sections[2].offset..sections[2].offset + sections[2].len];
        let (string_count, cursor) = read_var_u32(strings_payload, 0);
        let (string_len, cursor) = read_var_u32(strings_payload, cursor);
        assert_eq!(string_count, 1);
        assert_eq!(string_len, 10);
        assert_eq!(
            &strings_payload[cursor..cursor + string_len as usize],
            b"Iron Sword"
        );
        assert_eq!(sections[3].kind, 2);
        assert_eq!(sections[3].compression, 0);
        assert_eq!(sections[3].name, "Item");
        assert_eq!(sections[3].len, sections[3].uncompressed_len);

        let table_payload = &bytes[sections[3].offset..sections[3].offset + sections[3].len];
        assert_eq!(read_u32(table_payload, 0), 1);
        assert_eq!(read_u32(table_payload, 4), 0);
        assert_eq!(read_u32(table_payload, 8), 3);
        let (id, cursor) = read_var_i32(table_payload, 12);
        let (name_id, cursor) = read_var_u32(table_payload, cursor);
        assert_eq!(id, 1001);
        assert_eq!(name_id, 0);
        assert_eq!(cursor, table_payload.len());

        let _ = fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn binary_bundle_can_compress_sections_with_zstd() {
        let ir = example_ir();
        let data = example_data();
        let path = temp_dir().join("config.sora");

        BinaryBundleExporter
            .export(ExportRequest {
                ir: &ir,
                data: &data,
                locale_catalog: None,
                execution: &sora_execution::ExecutionContext::default(),
                options: crate::exporter::ExportOptions {
                    compression: crate::exporter::ExportCompression::Zstd { level: 3 },
                    locale: None,
                },
                output: ExportOutput::File(path.clone()),
            })
            .unwrap();

        let bytes = fs::read(&path).unwrap();
        let sections = read_sections(&bytes);
        assert_eq!(sections[0].compression, 0);
        assert_eq!(sections[1].compression, 0);
        assert_eq!(sections[2].compression, 1);
        assert_eq!(sections[3].compression, 1);
        assert!(sections[2].len > 0);
        assert!(sections[3].len > 0);
        assert_ne!(sections[2].len, sections[2].uncompressed_len);
        assert_ne!(sections[3].len, sections[3].uncompressed_len);

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

[[tables.fields]]
name = "name"
type = "string"
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
        std::env::temp_dir().join(format!("sora-export-binary-test-{unique}-{id}"))
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
            let offset = read_u32(bytes, cursor + 16) as usize;
            let len = read_u32(bytes, cursor + 20) as usize;
            let uncompressed_len = read_u32(bytes, cursor + 24) as usize;
            let name_start = cursor + 28;
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

    fn read_var_u32(bytes: &[u8], mut offset: usize) -> (u32, usize) {
        let mut value = 0u32;
        let mut shift = 0;
        loop {
            let byte = bytes[offset];
            offset += 1;
            value |= u32::from(byte & 0x7f) << shift;
            if byte & 0x80 == 0 {
                return (value, offset);
            }
            shift += 7;
        }
    }

    fn read_var_i32(bytes: &[u8], offset: usize) -> (i32, usize) {
        let (value, offset) = read_var_u32(bytes, offset);
        (((value >> 1) as i32) ^ (-((value & 1) as i32)), offset)
    }
}
