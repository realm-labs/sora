use std::path::{Path, PathBuf};

use sora_data::model::ConfigData;
use sora_diagnostics::Result;
use sora_input::traits::DataInput;
use sora_ir::model::ConfigIr;

use crate::data::load_config_data;

#[derive(Debug, Clone)]
pub struct TomlDataInput {
    data_root: PathBuf,
}

impl TomlDataInput {
    pub fn new(data_root: impl Into<PathBuf>) -> Self {
        Self {
            data_root: data_root.into(),
        }
    }

    pub fn data_root(&self) -> &Path {
        &self.data_root
    }
}

impl DataInput for TomlDataInput {
    fn load_data(&self, ir: &ConfigIr) -> Result<ConfigData> {
        load_config_data(ir, &self.data_root)
    }
}
