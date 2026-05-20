use std::collections::BTreeMap;

use crate::{
    binary::BinaryBundleExporter, cbor::CborBundleExporter, debug_json::DebugJsonExporter,
    exporter::DataExporter, json::JsonBundleExporter, protobuf::ProtobufBundleExporter,
    typed_protobuf::TypedProtobufExporter,
};

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
        registry.register(CborBundleExporter);
        registry.register(DebugJsonExporter);
        registry.register(JsonBundleExporter);
        registry.register(ProtobufBundleExporter);
        registry.register(TypedProtobufExporter);
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

pub fn builtin_supported_formats() -> Vec<&'static str> {
    ExporterRegistry::with_builtin_exporters().supported_formats()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::exporter::OutputKind;

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
        assert_eq!(
            registry.get("json").unwrap().output_kind(),
            OutputKind::File
        );
        assert_eq!(
            registry.get("cbor").unwrap().output_kind(),
            OutputKind::File
        );
        assert_eq!(
            registry.get("protobuf").unwrap().output_kind(),
            OutputKind::File
        );
        assert_eq!(
            registry.get("typed-protobuf").unwrap().output_kind(),
            OutputKind::File
        );
        assert!(registry.get("unknown").is_none());
        assert_eq!(
            registry.supported_formats(),
            vec![
                "binary",
                "cbor",
                "json",
                "json-debug",
                "protobuf",
                "typed-protobuf"
            ]
        );
    }
}
