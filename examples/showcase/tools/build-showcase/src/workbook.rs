use std::path::Path;

use anyhow::Result;
use sora_excel::generator::ExcelTemplateGenerator;
use sora_ir::model::ConfigIr;

use crate::rows::showcase_rows;

pub(crate) fn write_workbooks(ir: &ConfigIr, data_root: &Path) -> Result<()> {
    ExcelTemplateGenerator.generate_with_rows(ir, data_root, |table| showcase_rows(&table.name))?;
    Ok(())
}
