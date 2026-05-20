use std::path::Path;

use rust_xlsxwriter::{Color, DataValidation, Format, FormatAlign, FormatBorder, Workbook};
use sora_diagnostics::{Result, SoraError};
use sora_ir::model::{ConfigIr, TableIr, TypeIr};

use crate::projection::table_template_rows;

const DATA_START_ROW: u32 = 10;
const DATA_VALIDATION_ROWS: u32 = 1000;

pub(crate) fn write_workbook(ir: &ConfigIr, tables: &[&TableIr], path: &Path) -> Result<()> {
    let mut workbook = Workbook::new();
    let formats = TemplateFormats::new();

    for table in tables {
        let worksheet = workbook.add_worksheet();
        let sheet_name = table
            .source
            .as_ref()
            .and_then(|source| source.sheet.as_deref())
            .unwrap_or(&table.name);
        worksheet
            .set_name(sheet_name)
            .map_err(|source| excel_error(path, source))?;

        for (row_index, row) in table_template_rows(table).iter().enumerate() {
            for (column_index, value) in row.iter().enumerate() {
                if value.is_empty() {
                    worksheet
                        .write_blank(
                            row_index as u32,
                            column_index as u16,
                            formats.for_cell(row_index, column_index),
                        )
                        .map_err(|source| excel_error(path, source))?;
                } else {
                    worksheet
                        .write_with_format(
                            row_index as u32,
                            column_index as u16,
                            value,
                            formats.for_cell(row_index, column_index),
                        )
                        .map_err(|source| excel_error(path, source))?;
                }
            }
        }

        apply_sheet_layout(table, worksheet, &formats, path)?;
        apply_data_validations(ir, table, worksheet, path)?;
        worksheet
            .set_freeze_panes(DATA_START_ROW, 1)
            .map_err(|source| excel_error(path, source))?;
    }

    workbook
        .save(path)
        .map_err(|source| excel_error(path, source))
}

fn apply_data_validations(
    ir: &ConfigIr,
    table: &TableIr,
    worksheet: &mut rust_xlsxwriter::Worksheet,
    path: &Path,
) -> Result<()> {
    for (index, field) in table.fields.iter().enumerate() {
        let TypeIr::Enum(enum_name) = &field.ty else {
            continue;
        };
        let Some(enum_values) = enum_values(ir, enum_name) else {
            continue;
        };
        if enum_values.is_empty() {
            continue;
        }

        let values = enum_values.iter().map(String::as_str).collect::<Vec<_>>();
        let data_validation = DataValidation::new()
            .allow_list_strings(&values)
            .map_err(|source| excel_error(path, source))?
            .set_input_title(format!("{} enum", enum_name))
            .map_err(|source| excel_error(path, source))?
            .set_input_message("Select a value from the dropdown.")
            .map_err(|source| excel_error(path, source))?
            .set_error_title("Invalid enum value")
            .map_err(|source| excel_error(path, source))?
            .set_error_message(format!("Value must be one of: {}", enum_values.join(", ")))
            .map_err(|source| excel_error(path, source))?;

        let column = (index + 1) as u16;
        worksheet
            .add_data_validation(
                DATA_START_ROW,
                column,
                DATA_START_ROW + DATA_VALIDATION_ROWS - 1,
                column,
                &data_validation,
            )
            .map_err(|source| excel_error(path, source))?;
    }

    Ok(())
}

fn enum_values<'a>(ir: &'a ConfigIr, enum_name: &str) -> Option<&'a [String]> {
    ir.enums
        .iter()
        .find(|candidate| candidate.name == enum_name)
        .map(|item| item.values.as_slice())
}

struct TemplateFormats {
    metadata_label: Format,
    metadata_value: Format,
    display_header: Format,
    schema_label: Format,
    schema_value: Format,
    description: Format,
    spacer: Format,
}

impl TemplateFormats {
    fn new() -> Self {
        Self {
            metadata_label: Format::new()
                .set_bold()
                .set_font_color(Color::White)
                .set_background_color(Color::RGB(0x2F3A4A))
                .set_border(FormatBorder::Thin)
                .set_align(FormatAlign::VerticalCenter),
            metadata_value: Format::new()
                .set_background_color(Color::RGB(0xE9EEF5))
                .set_border(FormatBorder::Thin)
                .set_align(FormatAlign::VerticalCenter),
            display_header: Format::new()
                .set_bold()
                .set_font_color(Color::White)
                .set_background_color(Color::RGB(0x3D6E8F))
                .set_border(FormatBorder::Thin)
                .set_align(FormatAlign::Center)
                .set_align(FormatAlign::VerticalCenter),
            schema_label: Format::new()
                .set_bold()
                .set_font_color(Color::RGB(0x1F2937))
                .set_background_color(Color::RGB(0xDDE5EF))
                .set_border(FormatBorder::Thin)
                .set_align(FormatAlign::VerticalCenter),
            schema_value: Format::new()
                .set_background_color(Color::RGB(0xF6F8FB))
                .set_border(FormatBorder::Thin)
                .set_align(FormatAlign::VerticalCenter),
            description: Format::new()
                .set_background_color(Color::RGB(0xFFF7DB))
                .set_border(FormatBorder::Thin)
                .set_text_wrap()
                .set_align(FormatAlign::Top),
            spacer: Format::new().set_background_color(Color::RGB(0xFFFFFF)),
        }
    }

    fn for_cell(&self, row: usize, column: usize) -> &Format {
        match row {
            0..=3 if column == 0 => &self.metadata_label,
            0..=3 => &self.metadata_value,
            4 => &self.spacer,
            5 => &self.display_header,
            6..=8 if column == 0 => &self.schema_label,
            6..=8 => &self.schema_value,
            9 if column == 0 => &self.schema_label,
            9 => &self.description,
            _ => &self.schema_value,
        }
    }
}

fn apply_sheet_layout(
    table: &TableIr,
    worksheet: &mut rust_xlsxwriter::Worksheet,
    formats: &TemplateFormats,
    path: &Path,
) -> Result<()> {
    worksheet
        .set_row_height(4, 6)
        .map_err(|source| excel_error(path, source))?;
    worksheet
        .set_row_height(5, 24)
        .map_err(|source| excel_error(path, source))?;
    worksheet
        .set_row_height(9, 42)
        .map_err(|source| excel_error(path, source))?;
    worksheet
        .set_column_width(0, 12)
        .map_err(|source| excel_error(path, source))?;

    for (index, field) in table.fields.iter().enumerate() {
        let column = (index + 1) as u16;
        let width = field_width(
            field.name.as_str(),
            field.comment.as_deref(),
            &field.ty.to_string(),
        );
        worksheet
            .set_column_width(column, width)
            .map_err(|source| excel_error(path, source))?;
        worksheet
            .set_column_format(column, &formats.schema_value)
            .map_err(|source| excel_error(path, source))?;
    }

    Ok(())
}

fn field_width(name: &str, comment: Option<&str>, ty: &str) -> f64 {
    let content_width = [name.len(), ty.len(), comment.map(str::len).unwrap_or(0)]
        .into_iter()
        .max()
        .unwrap_or(12) as f64;
    content_width.clamp(12.0, 32.0)
}

fn excel_error(path: &Path, source: impl std::fmt::Display) -> SoraError {
    SoraError::ExcelTemplate {
        path: path.to_path_buf(),
        message: source.to_string(),
    }
}
