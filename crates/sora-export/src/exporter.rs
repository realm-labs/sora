use std::path::PathBuf;

use sora_data::model::ConfigData;
use sora_diagnostics::Result;
use sora_execution::ExecutionContext;
use sora_ir::model::ConfigIr;

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
    pub execution: &'a ExecutionContext,
    pub output: ExportOutput,
}

pub trait DataExporter: Send + Sync {
    fn format_name(&self) -> &'static str;
    fn output_kind(&self) -> OutputKind;
    fn export(&self, request: ExportRequest<'_>) -> Result<()>;
}
