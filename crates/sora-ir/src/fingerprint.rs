use sora_diagnostics::{Result, SoraError};

use crate::model::ConfigIr;

pub fn schema_fingerprint(ir: &ConfigIr) -> Result<String> {
    let schema = ir.data_schema();
    let bytes = serde_json::to_vec(&schema).map_err(SoraError::SerializeData)?;
    Ok(fingerprint_hex(&bytes))
}

pub fn fingerprint_hex(bytes: &[u8]) -> String {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }

    format!("{hash:016x}")
}
