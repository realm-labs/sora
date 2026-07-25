use std::path::Path;

use anyhow::Result;
use sora_excel::generator::{ExcelAdditionalSheet, ExcelTemplateGenerator};
use sora_ir::model::ConfigIr;

use crate::rows::showcase_rows;

pub(crate) fn write_workbooks(ir: &ConfigIr, data_root: &Path) -> Result<()> {
    let mut localization_rows = vec![
        vec!["@localization".to_owned(), "Localization".to_owned()],
        vec![
            "#name".to_owned(),
            "key".to_owned(),
            "zh_cn".to_owned(),
            "en_us".to_owned(),
        ],
        vec![
            "#field".to_owned(),
            "key".to_owned(),
            "zh_cn".to_owned(),
            "en_us".to_owned(),
        ],
        vec![
            "#type".to_owned(),
            "string".to_owned(),
            "string".to_owned(),
            "string".to_owned(),
        ],
        vec!["#groups".to_owned()],
        vec!["#input".to_owned()],
        vec!["#desc".to_owned()],
    ];
    localization_rows.extend(showcase_rows("Localization").into_iter().map(|row| {
        std::iter::once(String::new())
            .chain(row)
            .collect::<Vec<_>>()
    }));
    ExcelTemplateGenerator.generate_with_rows_and_sheets(
        ir,
        data_root,
        |table| showcase_rows(&table.name),
        &[ExcelAdditionalSheet {
            file: "Core.xlsx".to_owned(),
            sheet: "Localization".to_owned(),
            rows: localization_rows,
        }],
    )?;
    Ok(())
}
