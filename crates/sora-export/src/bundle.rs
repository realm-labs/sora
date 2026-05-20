use serde::Serialize;
use sora_data::model::ConfigData;
use sora_ir::model::ConfigIr;

pub(crate) const FORMAT_VERSION: u32 = 1;

#[derive(Serialize)]
pub(crate) struct DataBundleView<'a> {
    pub format: &'static str,
    pub format_version: u32,
    pub schema: ConfigIr,
    pub data: &'a ConfigData,
}

impl<'a> DataBundleView<'a> {
    pub(crate) fn new(format: &'static str, ir: &'a ConfigIr, data: &'a ConfigData) -> Self {
        Self {
            format,
            format_version: FORMAT_VERSION,
            schema: ir.data_schema(),
            data,
        }
    }
}
