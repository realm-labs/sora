use std::collections::BTreeMap;

use serde::Serialize;
use sora_diagnostics::{Result, SoraError};

use crate::{
    bundle::{FORMAT_VERSION, schema_fingerprint},
    exporter::{DataExporter, ExportOutput, ExportRequest, OutputKind},
    fs_util::{create_parent_dir, write_file},
};

const I18N_MAGIC: &[u8; 4] = b"SORI";
const I18N_VERSION: u32 = 1;

pub struct I18nJsonExporter;

impl DataExporter for I18nJsonExporter {
    fn format_name(&self) -> &'static str {
        "i18n-json"
    }

    fn output_kind(&self) -> OutputKind {
        OutputKind::File
    }

    fn export(&self, request: ExportRequest<'_>) -> Result<()> {
        let ExportOutput::File(ref path) = request.output else {
            return Err(SoraError::InvalidExportOutput {
                format: self.format_name().to_owned(),
                expected: "file",
            });
        };
        let view = locale_view(self.format_name(), &request)?;
        create_parent_dir(&path)?;
        let content = serde_json::to_vec_pretty(&view).map_err(SoraError::SerializeData)?;
        write_file(path.clone(), content)
    }
}

pub struct I18nBinaryExporter;

impl DataExporter for I18nBinaryExporter {
    fn format_name(&self) -> &'static str {
        "i18n-binary"
    }

    fn output_kind(&self) -> OutputKind {
        OutputKind::File
    }

    fn export(&self, request: ExportRequest<'_>) -> Result<()> {
        let ExportOutput::File(ref path) = request.output else {
            return Err(SoraError::InvalidExportOutput {
                format: self.format_name().to_owned(),
                expected: "file",
            });
        };
        let view = locale_view(self.format_name(), &request)?;
        create_parent_dir(&path)?;
        write_file(path.clone(), encode_i18n_binary(&view)?)
    }
}

#[derive(Serialize)]
struct I18nView {
    format: &'static str,
    format_version: u32,
    schema_fingerprint: String,
    locale: String,
    fallback_locale: Option<String>,
    translations: BTreeMap<String, String>,
}

fn locale_view(format: &'static str, request: &ExportRequest<'_>) -> Result<I18nView> {
    let locale = request.options.locale.as_deref().ok_or_else(|| {
        SoraError::InvalidSchema(format!("export format `{format}` requires `locale`"))
    })?;
    let catalog = request.locale_catalog.ok_or_else(|| {
        SoraError::InvalidSchema(format!("export format `{format}` requires [localization]"))
    })?;
    Ok(I18nView {
        format,
        format_version: FORMAT_VERSION,
        schema_fingerprint: schema_fingerprint(request.ir)?,
        locale: locale.to_owned(),
        fallback_locale: catalog.fallback_locale.clone(),
        translations: catalog.for_locale(locale)?,
    })
}

fn encode_i18n_binary(view: &I18nView) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(I18N_MAGIC);
    write_u32(&mut bytes, I18N_VERSION);
    write_len_prefixed(&mut bytes, view.schema_fingerprint.as_bytes())?;
    write_len_prefixed(&mut bytes, view.locale.as_bytes())?;
    write_u32(
        &mut bytes,
        checked_u32(view.translations.len(), "translation count")?,
    );
    for (key, value) in &view.translations {
        write_len_prefixed(&mut bytes, key.as_bytes())?;
        write_len_prefixed(&mut bytes, value.as_bytes())?;
    }
    Ok(bytes)
}

fn write_len_prefixed(out: &mut Vec<u8>, bytes: &[u8]) -> Result<()> {
    write_u32(out, checked_u32(bytes.len(), "i18n string length")?);
    out.extend_from_slice(bytes);
    Ok(())
}

fn checked_u32(value: usize, label: &str) -> Result<u32> {
    u32::try_from(value).map_err(|_| SoraError::InvalidSchema(format!("{label} exceeds u32 range")))
}

fn write_u32(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_le_bytes());
}
