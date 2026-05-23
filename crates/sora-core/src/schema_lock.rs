use std::{fs, path::Path};

use serde::{Deserialize, Serialize};
use sora_diagnostics::{Result, SoraError};
use sora_ir::model::ConfigIr;

const SCHEMA_LOCK_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchemaLock {
    pub version: u32,
    pub package: String,
    pub fingerprint: String,
    pub schema: ConfigIr,
}

impl SchemaLock {
    pub fn from_ir(ir: &ConfigIr) -> Result<Self> {
        let schema = ir.data_schema();
        Ok(Self {
            version: SCHEMA_LOCK_VERSION,
            package: ir.package.clone(),
            fingerprint: schema_fingerprint(&schema)?,
            schema,
        })
    }
}

pub fn write_schema_lock_file(ir: &ConfigIr, path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| SoraError::CreateDir {
            path: parent.to_path_buf(),
            source,
        })?;
    }

    let lock = SchemaLock::from_ir(ir)?;
    let content = serde_json::to_string_pretty(&lock).map_err(SoraError::SerializeData)?;
    fs::write(path, content).map_err(|source| SoraError::WriteFile {
        path: path.to_path_buf(),
        source,
    })
}

pub fn read_schema_lock_file(path: &Path) -> Result<SchemaLock> {
    let content = fs::read_to_string(path).map_err(|source| SoraError::ReadFile {
        path: path.to_path_buf(),
        source,
    })?;
    serde_json::from_str(&content).map_err(|source| SoraError::ParseSchema {
        path: path.to_path_buf(),
        message: source.to_string(),
    })
}

pub fn verify_schema_lock(ir: &ConfigIr, lock: &SchemaLock) -> Result<()> {
    let current = SchemaLock::from_ir(ir)?;
    if lock.version != SCHEMA_LOCK_VERSION {
        return Err(SoraError::InvalidSchema(format!(
            "schema.lock version {} is not supported; expected {}",
            lock.version, SCHEMA_LOCK_VERSION
        )));
    }
    if lock.fingerprint != current.fingerprint || lock.schema != current.schema {
        return Err(SoraError::InvalidSchema(format!(
            "schema.lock mismatch: expected fingerprint `{}`, current fingerprint `{}`",
            lock.fingerprint, current.fingerprint
        )));
    }
    Ok(())
}

pub fn schema_fingerprint(ir: &ConfigIr) -> Result<String> {
    let bytes = serde_json::to_vec(ir).map_err(SoraError::SerializeData)?;
    Ok(fnv1a64_hex(&bytes))
}

fn fnv1a64_hex(bytes: &[u8]) -> String {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use sora_ir::normalize::normalize_schema;
    use sora_schema::model::SchemaFile;

    #[test]
    fn schema_lock_detects_schema_changes() {
        let ir = example_ir("i32");
        let lock = SchemaLock::from_ir(&ir).unwrap();
        verify_schema_lock(&ir, &lock).unwrap();

        let changed = example_ir("i64");
        let error = verify_schema_lock(&changed, &lock).unwrap_err();

        assert!(error.to_string().contains("schema.lock mismatch"));
    }

    fn example_ir(id_type: &str) -> ConfigIr {
        let schema: SchemaFile = toml::from_str(&format!(
            r#"
package = "game_config"

[[tables]]
name = "Item"
mode = "map"
key = "id"

[[tables.fields]]
name = "id"
type = "{id_type}"
key = true
"#
        ))
        .unwrap();
        normalize_schema(schema).unwrap()
    }
}
