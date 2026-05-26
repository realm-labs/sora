use std::path::Path;

use rust_xlsxwriter::{
    Color, DataValidation, DataValidationRule, Format, FormatAlign, FormatBorder, Note, Workbook,
};
use sora_diagnostics::{Result, SoraError};
use sora_ir::{
    input_projection::{TaggedColumnKind, struct_columns, tagged_columns, tagged_columns_union},
    model::{ConfigIr, FieldIr, TableIr, TypeIr},
};

use crate::projection::{
    DATA_START_ROW, DESC_ROW, FIELD_ROW, FIELD_START_COLUMN, NAME_ROW, TYPE_ROW, TemplateColumn,
    TemplateColumnGroupRole, table_template_columns, table_template_rows, tuple_shape,
};

const DATA_VALIDATION_ROWS: u32 = 1000;

pub(crate) fn write_workbook_with_rows(
    ir: &ConfigIr,
    tables: &[&TableIr],
    path: &Path,
    rows_for_table: impl Fn(&TableIr) -> Vec<Vec<String>>,
) -> Result<()> {
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

        let columns = table_template_columns(ir, table);
        for (row_index, row) in table_template_rows(ir, table).iter().enumerate() {
            for (column_index, value) in row.iter().enumerate() {
                let column_info = column_index
                    .checked_sub(FIELD_START_COLUMN as usize)
                    .and_then(|index| columns.get(index));
                if value.is_empty() {
                    worksheet
                        .write_blank(
                            row_index as u32,
                            column_index as u16,
                            formats.for_cell(row_index, column_index, column_info),
                        )
                        .map_err(|source| excel_error(path, source))?;
                } else {
                    worksheet
                        .write_with_format(
                            row_index as u32,
                            column_index as u16,
                            value,
                            formats.for_cell(row_index, column_index, column_info),
                        )
                        .map_err(|source| excel_error(path, source))?;
                }
            }
        }

        apply_sheet_layout(ir, table, worksheet, &formats, path)?;
        apply_field_notes(ir, table, worksheet, path)?;
        apply_data_validations(ir, table, worksheet, path)?;
        write_data_rows(worksheet, &formats, path, &columns, rows_for_table(table))?;
        worksheet
            .set_freeze_panes(DATA_START_ROW, 1)
            .map_err(|source| excel_error(path, source))?;
    }

    workbook
        .save(path)
        .map_err(|source| excel_error(path, source))
}

pub(crate) struct SyncedTableSheet<'a> {
    pub table: &'a TableIr,
    pub sheet_name: String,
    pub rows: Vec<Vec<String>>,
    pub legacy_columns: Vec<LegacyColumn>,
}

pub(crate) struct LegacyColumn {
    pub headers: Vec<String>,
    pub values: Vec<String>,
}

pub(crate) struct PreservedSheet {
    pub sheet_name: String,
    pub rows: Vec<Vec<String>>,
}

pub(crate) fn write_synced_workbook(
    ir: &ConfigIr,
    table_sheets: &[SyncedTableSheet<'_>],
    preserved_sheets: &[PreservedSheet],
    path: &Path,
) -> Result<()> {
    let mut workbook = Workbook::new();
    let formats = TemplateFormats::new();

    for sheet in table_sheets {
        let worksheet = workbook.add_worksheet();
        worksheet
            .set_name(&sheet.sheet_name)
            .map_err(|source| excel_error(path, source))?;

        let columns = table_template_columns(ir, sheet.table);
        write_synced_headers(ir, sheet, worksheet, &formats, path)?;
        apply_sheet_layout(ir, sheet.table, worksheet, &formats, path)?;
        apply_legacy_column_layout(&columns, &sheet.legacy_columns, worksheet, &formats, path)?;
        apply_field_notes(ir, sheet.table, worksheet, path)?;
        apply_data_validations(ir, sheet.table, worksheet, path)?;
        write_synced_data_rows(worksheet, &formats, path, &columns, sheet)?;
        worksheet
            .set_freeze_panes(DATA_START_ROW, 1)
            .map_err(|source| excel_error(path, source))?;
    }

    for sheet in preserved_sheets {
        let worksheet = workbook.add_worksheet();
        worksheet
            .set_name(&sheet.sheet_name)
            .map_err(|source| excel_error(path, source))?;
        for (row_index, row) in sheet.rows.iter().enumerate() {
            for (column_index, value) in row.iter().enumerate() {
                if !value.is_empty() {
                    worksheet
                        .write_string(row_index as u32, column_index as u16, value)
                        .map_err(|source| excel_error(path, source))?;
                }
            }
        }
    }

    workbook
        .save(path)
        .map_err(|source| excel_error(path, source))
}

fn write_synced_headers(
    ir: &ConfigIr,
    sheet: &SyncedTableSheet<'_>,
    worksheet: &mut rust_xlsxwriter::Worksheet,
    formats: &TemplateFormats,
    path: &Path,
) -> Result<()> {
    let columns = table_template_columns(ir, sheet.table);
    for (row_index, row) in table_template_rows(ir, sheet.table).iter().enumerate() {
        for (column_index, value) in row.iter().enumerate() {
            let column_info = column_index
                .checked_sub(FIELD_START_COLUMN as usize)
                .and_then(|index| columns.get(index));
            write_header_cell(
                worksheet,
                row_index,
                column_index,
                value,
                formats.for_cell(row_index, column_index, column_info),
                path,
            )?;
        }

        let legacy_start = FIELD_START_COLUMN as usize + columns.len();
        for (legacy_index, legacy) in sheet.legacy_columns.iter().enumerate() {
            let value = legacy
                .headers
                .get(row_index)
                .map(String::as_str)
                .unwrap_or_default();
            write_header_cell(
                worksheet,
                row_index,
                legacy_start + legacy_index,
                value,
                formats.legacy_cell(row_index),
                path,
            )?;
        }
    }

    Ok(())
}

fn write_header_cell(
    worksheet: &mut rust_xlsxwriter::Worksheet,
    row: usize,
    column: usize,
    value: &str,
    format: &Format,
    path: &Path,
) -> Result<()> {
    if value.is_empty() {
        worksheet
            .write_blank(row as u32, column as u16, format)
            .map_err(|source| excel_error(path, source))?;
    } else {
        worksheet
            .write_with_format(row as u32, column as u16, value, format)
            .map_err(|source| excel_error(path, source))?;
    }
    Ok(())
}

fn apply_legacy_column_layout(
    columns: &[TemplateColumn],
    legacy_columns: &[LegacyColumn],
    worksheet: &mut rust_xlsxwriter::Worksheet,
    formats: &TemplateFormats,
    path: &Path,
) -> Result<()> {
    let start = FIELD_START_COLUMN + columns.len() as u16;
    for (index, legacy) in legacy_columns.iter().enumerate() {
        let column = start + index as u16;
        let name = legacy
            .headers
            .get(DESC_ROW as usize)
            .filter(|value| !value.is_empty())
            .or_else(|| legacy.headers.get(FIELD_ROW as usize))
            .map(String::as_str)
            .unwrap_or("legacy");
        worksheet
            .set_column_width(column, field_width(name, None, "legacy"))
            .map_err(|source| excel_error(path, source))?;
        worksheet
            .set_column_format(column, &formats.legacy_data)
            .map_err(|source| excel_error(path, source))?;
    }
    Ok(())
}

fn write_synced_data_rows(
    worksheet: &mut rust_xlsxwriter::Worksheet,
    formats: &TemplateFormats,
    path: &Path,
    columns: &[TemplateColumn],
    sheet: &SyncedTableSheet<'_>,
) -> Result<()> {
    let legacy_row_count = sheet
        .legacy_columns
        .iter()
        .map(|column| column.values.len())
        .max()
        .unwrap_or_default();
    let row_count = sheet.rows.len().max(legacy_row_count);
    for row_offset in 0..row_count {
        let row = sheet.rows.get(row_offset);
        for (column, column_info) in columns.iter().enumerate() {
            let value = row
                .and_then(|row| row.get(column))
                .map(String::as_str)
                .unwrap_or_default();
            let value = if column_info.derived && value.is_empty() {
                derived_placeholder(Some(column_info))
            } else {
                value.to_owned()
            };
            worksheet
                .write_with_format(
                    DATA_START_ROW + row_offset as u32,
                    FIELD_START_COLUMN + column as u16,
                    &value,
                    formats.data_cell(column_info.derived),
                )
                .map_err(|source| excel_error(path, source))?;
        }

        let legacy_start = FIELD_START_COLUMN + columns.len() as u16;
        for (legacy_index, legacy) in sheet.legacy_columns.iter().enumerate() {
            let value = legacy
                .values
                .get(row_offset)
                .map(String::as_str)
                .unwrap_or_default();
            if !value.is_empty() {
                worksheet
                    .write_with_format(
                        DATA_START_ROW + row_offset as u32,
                        legacy_start + legacy_index as u16,
                        value,
                        &formats.legacy_data,
                    )
                    .map_err(|source| excel_error(path, source))?;
            }
        }
    }

    Ok(())
}

fn write_data_rows(
    worksheet: &mut rust_xlsxwriter::Worksheet,
    formats: &TemplateFormats,
    path: &Path,
    columns: &[TemplateColumn],
    rows: Vec<Vec<String>>,
) -> Result<()> {
    for (row_offset, row) in rows.iter().enumerate() {
        let column_count = row.len().max(columns.len());
        for column in 0..column_count {
            let value = row.get(column).map(String::as_str).unwrap_or_default();
            let derived = columns.get(column).is_some_and(|column| column.derived);
            let value = if derived && value.is_empty() {
                derived_placeholder(columns.get(column))
            } else {
                value.to_owned()
            };
            worksheet
                .write_with_format(
                    DATA_START_ROW + row_offset as u32,
                    FIELD_START_COLUMN + column as u16,
                    &value,
                    formats.data_cell(derived),
                )
                .map_err(|source| excel_error(path, source))?;
        }
    }

    Ok(())
}

fn derived_placeholder(column: Option<&TemplateColumn>) -> String {
    column
        .and_then(|column| (!column.input.is_empty()).then(|| column.input.clone()))
        .unwrap_or_else(|| "generated".to_owned())
}

fn apply_data_validations(
    ir: &ConfigIr,
    table: &TableIr,
    worksheet: &mut rust_xlsxwriter::Worksheet,
    path: &Path,
) -> Result<()> {
    let mut column = FIELD_START_COLUMN;
    for field in &table.fields {
        if field.derived_from.is_some() {
            column += tagged_columns(ir, field)
                .map(|columns| columns.len() as u16)
                .or_else(|| struct_columns(ir, field).map(|columns| columns.len() as u16))
                .unwrap_or(1);
            continue;
        }
        if let Some(columns) = struct_columns(ir, field) {
            for struct_column in columns {
                let data_validation = data_validation_for_field(ir, struct_column.field, path)?;
                if let Some(data_validation) = data_validation {
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
                column += 1;
            }
            continue;
        }
        if let Some(columns) = tagged_columns(ir, field) {
            for tagged_column in columns {
                let data_validation = match tagged_column.kind {
                    TaggedColumnKind::Tag => {
                        if let Some(union) = tagged_columns_union(ir, field) {
                            union_tag_validation(
                                union.variants.iter().map(|variant| variant.name.as_str()),
                                path,
                            )?
                        } else {
                            None
                        }
                    }
                    TaggedColumnKind::VariantField(variant_field) => {
                        data_validation_for_field(ir, variant_field, path)?
                    }
                };
                if let Some(data_validation) = data_validation {
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
                column += 1;
            }
            continue;
        }

        let data_validation = data_validation_for_field(ir, field, path)?;
        if let Some(data_validation) = data_validation {
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

        column += 1;
    }

    Ok(())
}

fn apply_field_notes(
    ir: &ConfigIr,
    table: &TableIr,
    worksheet: &mut rust_xlsxwriter::Worksheet,
    path: &Path,
) -> Result<()> {
    let mut column = FIELD_START_COLUMN;
    for field in &table.fields {
        if let Some(columns) = struct_columns(ir, field) {
            for struct_column in columns {
                insert_note(
                    worksheet,
                    NAME_ROW,
                    column,
                    struct_column_note_text(ir, field, struct_column.field, &struct_column.name),
                    path,
                )?;
                insert_note(
                    worksheet,
                    TYPE_ROW,
                    column,
                    root_type_note_text(field, "columns"),
                    path,
                )?;
                column += 1;
            }
            continue;
        }

        if let Some(columns) = tagged_columns(ir, field) {
            for tagged_column in columns {
                insert_note(
                    worksheet,
                    NAME_ROW,
                    column,
                    tagged_column_note_text(ir, field, &tagged_column),
                    path,
                )?;
                insert_note(
                    worksheet,
                    TYPE_ROW,
                    column,
                    root_type_note_text(field, "tagged_columns"),
                    path,
                )?;
                column += 1;
            }
            continue;
        }

        let note_text = field_note_text(ir, field);
        if let Some(note_text) = note_text {
            let note = Note::new(note_text).set_author("Sora");
            worksheet
                .insert_note(NAME_ROW, column, &note)
                .map_err(|source| excel_error(path, source))?;
        }

        column += 1;
    }

    Ok(())
}

fn insert_note(
    worksheet: &mut rust_xlsxwriter::Worksheet,
    row: u32,
    column: u16,
    text: String,
    path: &Path,
) -> Result<()> {
    let note = Note::new(text).set_author("Sora");
    worksheet
        .insert_note(row, column, &note)
        .map_err(|source| excel_error(path, source))?;
    Ok(())
}

fn struct_column_note_text(
    ir: &ConfigIr,
    root_field: &FieldIr,
    struct_field: &FieldIr,
    column: &str,
) -> String {
    let mut note_text = field_note_text(ir, struct_field).unwrap_or_default();
    append_section(
        &mut note_text,
        &format!(
            "Field: {}.{}\nType: {}\nRoot field: {}\nRoot type: {}\nParser: columns\nColumn: {}",
            root_field.name,
            struct_field.name,
            struct_field.ty,
            root_field.name,
            root_field.ty,
            column
        ),
    );
    note_text
}

fn tagged_column_note_text(
    ir: &ConfigIr,
    root_field: &FieldIr,
    tagged_column: &sora_ir::input_projection::TaggedColumn<'_>,
) -> String {
    match tagged_column.kind {
        TaggedColumnKind::Tag => format!(
            "Field: {}\nType: {} tag\nRoot field: {}\nRoot type: {}\nParser: tagged_columns\nColumn: {}",
            root_field.name, root_field.ty, root_field.name, root_field.ty, tagged_column.name
        ),
        TaggedColumnKind::VariantField(variant_field) => {
            let mut note_text = field_note_text(ir, variant_field).unwrap_or_default();
            append_section(
                &mut note_text,
                &format!(
                    "Field: {}.{}\nType: {}\nRoot field: {}\nRoot type: {}\nParser: tagged_columns\nColumn: {}",
                    root_field.name,
                    variant_field.name,
                    variant_field.ty,
                    root_field.name,
                    root_field.ty,
                    tagged_column.name
                ),
            );
            note_text
        }
    }
}

fn root_type_note_text(field: &FieldIr, parser: &str) -> String {
    format!(
        "Root field: {}\nRoot type: {}\nParser: {}",
        field.name, field.ty, parser
    )
}

fn append_section(text: &mut String, section: &str) {
    if !text.is_empty() {
        text.push_str("\n\n");
    }
    text.push_str(section);
}

fn field_note_text(ir: &ConfigIr, field: &FieldIr) -> Option<String> {
    let comment = field.comment.as_deref().map(str::trim).unwrap_or_default();
    let tuple_shape = tuple_shape(ir, field);
    if comment.is_empty() && tuple_shape.is_none() {
        return None;
    }

    let mut lines = Vec::new();
    if !comment.is_empty() {
        lines.push(comment.to_owned());
        lines.push(String::new());
    }
    lines.push(format!("Field: {}", field.name));
    lines.push(format!("Type: {}", field.ty));
    lines.push(format!("Scope: {}", field.scope.display()));
    if let Some(tuple_shape) = tuple_shape {
        lines.push(format!("Tuple fields: {tuple_shape}"));
    }

    if field.key {
        lines.push("Key: yes".to_owned());
    }
    if let Some(default) = &field.default {
        lines.push(format!("Default: {default}"));
    }
    if let Some(parser) = &field.parser {
        lines.push(format!("Parser: {}", parser.kind));
        for (key, value) in &parser.options {
            lines.push(format!("Parser {key}: {value}"));
        }
    }
    if let Some([min, max]) = field.range {
        lines.push(format!("Range: {min}..{max}"));
    }
    if let Some([min, max]) = field.length {
        lines.push(format!("Length: {min}..{max}"));
    }
    if let Some(derived_from) = &field.derived_from {
        lines.push("Generated: yes; do not fill this column.".to_owned());
        lines.push(format!(
            "From: {}.{} -> {}",
            derived_from.source_table, derived_from.child_key, derived_from.parent_key
        ));
        if let Some(value_field) = &derived_from.value_field {
            lines.push(format!("From field: {value_field}"));
        }
    }

    Some(lines.join("\n"))
}

fn data_validation_for_field(
    ir: &ConfigIr,
    field: &FieldIr,
    path: &Path,
) -> Result<Option<DataValidation>> {
    Ok(match &field.ty {
        TypeIr::Bool => Some(bool_validation(path)?),
        TypeIr::Enum(enum_name) => enum_validation(ir, enum_name, path)?,
        TypeIr::I8
        | TypeIr::U8
        | TypeIr::I16
        | TypeIr::U16
        | TypeIr::I32
        | TypeIr::U32
        | TypeIr::I64 => integer_validation(field, path)?,
        TypeIr::F32 | TypeIr::F64 => decimal_validation(field, path)?,
        TypeIr::Optional(inner) => data_validation_for_type(ir, inner, field, path)?,
        _ => None,
    })
}

fn data_validation_for_type(
    ir: &ConfigIr,
    ty: &TypeIr,
    field: &FieldIr,
    path: &Path,
) -> Result<Option<DataValidation>> {
    Ok(match ty {
        TypeIr::Bool => Some(bool_validation(path)?),
        TypeIr::Enum(enum_name) => enum_validation(ir, enum_name, path)?,
        TypeIr::I8
        | TypeIr::U8
        | TypeIr::I16
        | TypeIr::U16
        | TypeIr::I32
        | TypeIr::U32
        | TypeIr::I64 => integer_validation(field, path)?,
        TypeIr::F32 | TypeIr::F64 => decimal_validation(field, path)?,
        _ => None,
    })
}

fn bool_validation(path: &Path) -> Result<DataValidation> {
    DataValidation::new()
        .allow_list_strings(&["true", "false"])
        .map_err(|source| excel_error(path, source))?
        .set_input_title("Boolean")
        .map_err(|source| excel_error(path, source))?
        .set_input_message("Select true or false.")
        .map_err(|source| excel_error(path, source))?
        .set_error_title("Invalid boolean value")
        .map_err(|source| excel_error(path, source))?
        .set_error_message("Value must be true or false.")
        .map_err(|source| excel_error(path, source))
}

fn enum_validation(ir: &ConfigIr, enum_name: &str, path: &Path) -> Result<Option<DataValidation>> {
    let Some(enum_values) = enum_values(ir, enum_name) else {
        return Ok(None);
    };
    if enum_values.is_empty() {
        return Ok(None);
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
    Ok(Some(data_validation))
}

fn union_tag_validation<'a>(
    values: impl IntoIterator<Item = &'a str>,
    path: &Path,
) -> Result<Option<DataValidation>> {
    let values = values.into_iter().collect::<Vec<_>>();
    if values.is_empty() {
        return Ok(None);
    }

    let data_validation = DataValidation::new()
        .allow_list_strings(&values)
        .map_err(|source| excel_error(path, source))?
        .set_input_title("Union tag")
        .map_err(|source| excel_error(path, source))?
        .set_input_message("Select a union variant.")
        .map_err(|source| excel_error(path, source))?
        .set_error_title("Invalid union variant")
        .map_err(|source| excel_error(path, source))?
        .set_error_message(format!("Value must be one of: {}", values.join(", ")))
        .map_err(|source| excel_error(path, source))?;
    Ok(Some(data_validation))
}

fn integer_validation(field: &FieldIr, path: &Path) -> Result<Option<DataValidation>> {
    let Some([min, max]) = field.range else {
        return Ok(None);
    };
    let (Ok(min), Ok(max)) = (i32::try_from(min), i32::try_from(max)) else {
        return Ok(None);
    };

    let data_validation = DataValidation::new()
        .allow_whole_number(DataValidationRule::Between(min, max))
        .set_input_title("Integer range")
        .map_err(|source| excel_error(path, source))?
        .set_input_message(format!("Enter an integer from {min} to {max}."))
        .map_err(|source| excel_error(path, source))?
        .set_error_title("Value outside range")
        .map_err(|source| excel_error(path, source))?
        .set_error_message(format!("Value must be an integer from {min} to {max}."))
        .map_err(|source| excel_error(path, source))?;
    Ok(Some(data_validation))
}

fn decimal_validation(field: &FieldIr, path: &Path) -> Result<Option<DataValidation>> {
    let Some([min, max]) = field.range else {
        return Ok(None);
    };
    let min = min as f64;
    let max = max as f64;

    let data_validation = DataValidation::new()
        .allow_decimal_number(DataValidationRule::Between(min, max))
        .set_input_title("Number range")
        .map_err(|source| excel_error(path, source))?
        .set_input_message(format!("Enter a number from {min} to {max}."))
        .map_err(|source| excel_error(path, source))?
        .set_error_title("Value outside range")
        .map_err(|source| excel_error(path, source))?
        .set_error_message(format!("Value must be a number from {min} to {max}."))
        .map_err(|source| excel_error(path, source))?;
    Ok(Some(data_validation))
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
    grouped_header: Vec<Format>,
    grouped_tag_header: Vec<Format>,
    schema_label: Format,
    schema_value: Format,
    grouped_schema_value: Vec<Format>,
    description: Format,
    grouped_description: Vec<Format>,
    derived_data: Format,
    legacy_header: Format,
    legacy_data: Format,
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
            grouped_header: group_palette()
                .into_iter()
                .map(|color| {
                    Format::new()
                        .set_bold()
                        .set_font_color(Color::RGB(0x0F172A))
                        .set_background_color(color)
                        .set_border(FormatBorder::Thin)
                        .set_align(FormatAlign::Center)
                        .set_align(FormatAlign::VerticalCenter)
                })
                .collect(),
            grouped_tag_header: group_palette()
                .into_iter()
                .map(|color| {
                    Format::new()
                        .set_bold()
                        .set_font_color(Color::RGB(0x0F172A))
                        .set_background_color(color)
                        .set_border(FormatBorder::Thin)
                        .set_align(FormatAlign::Center)
                        .set_align(FormatAlign::VerticalCenter)
                        .set_italic()
                })
                .collect(),
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
            grouped_schema_value: group_palette()
                .into_iter()
                .map(|color| {
                    Format::new()
                        .set_background_color(color)
                        .set_border(FormatBorder::Thin)
                        .set_align(FormatAlign::VerticalCenter)
                })
                .collect(),
            description: Format::new()
                .set_background_color(Color::RGB(0xFFF7DB))
                .set_border(FormatBorder::Thin)
                .set_text_wrap()
                .set_align(FormatAlign::Top),
            grouped_description: group_palette()
                .into_iter()
                .map(|color| {
                    Format::new()
                        .set_background_color(color)
                        .set_border(FormatBorder::Thin)
                        .set_text_wrap()
                        .set_align(FormatAlign::Top)
                })
                .collect(),
            derived_data: Format::new()
                .set_background_color(Color::RGB(0xE5E7EB))
                .set_font_color(Color::RGB(0x6B7280))
                .set_border(FormatBorder::Thin)
                .set_italic()
                .set_align(FormatAlign::VerticalCenter),
            legacy_header: Format::new()
                .set_background_color(Color::RGB(0xF3F4F6))
                .set_font_color(Color::RGB(0x6B7280))
                .set_border(FormatBorder::Thin)
                .set_italic()
                .set_align(FormatAlign::VerticalCenter),
            legacy_data: Format::new()
                .set_background_color(Color::RGB(0xFAFAFA))
                .set_font_color(Color::RGB(0x6B7280))
                .set_border(FormatBorder::Thin)
                .set_italic()
                .set_align(FormatAlign::VerticalCenter),
        }
    }

    fn for_cell(&self, row: usize, column: usize, column_info: Option<&TemplateColumn>) -> &Format {
        let row = row as u32;
        if row != 0
            && column != 0
            && let Some(group) = column_info.and_then(|column| column.group)
        {
            let index = group.index % self.grouped_header.len();
            return match row {
                NAME_ROW if group.role == TemplateColumnGroupRole::Tag => {
                    &self.grouped_tag_header[index]
                }
                NAME_ROW => &self.grouped_header[index],
                row if row == DESC_ROW => &self.grouped_description[index],
                _ => &self.grouped_schema_value[index],
            };
        }
        match row {
            0 if column.is_multiple_of(2) => &self.metadata_label,
            0 => &self.metadata_value,
            NAME_ROW => &self.display_header,
            row if row == DESC_ROW && column == 0 => &self.schema_label,
            row if row == DESC_ROW => &self.description,
            _ if column == 0 => &self.schema_label,
            _ => &self.schema_value,
        }
    }

    fn data_cell(&self, derived: bool) -> &Format {
        if derived {
            &self.derived_data
        } else {
            &self.schema_value
        }
    }

    fn legacy_cell(&self, row: usize) -> &Format {
        if row == DESC_ROW as usize {
            &self.description
        } else {
            &self.legacy_header
        }
    }
}

fn group_palette() -> [Color; 6] {
    [
        Color::RGB(0xDDF7EE),
        Color::RGB(0xDBEAFE),
        Color::RGB(0xFCE7F3),
        Color::RGB(0xFEF3C7),
        Color::RGB(0xEDE9FE),
        Color::RGB(0xE0F2FE),
    ]
}

fn apply_sheet_layout(
    ir: &ConfigIr,
    table: &TableIr,
    worksheet: &mut rust_xlsxwriter::Worksheet,
    formats: &TemplateFormats,
    path: &Path,
) -> Result<()> {
    worksheet
        .set_row_height(NAME_ROW, 24)
        .map_err(|source| excel_error(path, source))?;
    worksheet
        .set_row_height(DESC_ROW, 42)
        .map_err(|source| excel_error(path, source))?;
    worksheet
        .set_column_width(0, 12)
        .map_err(|source| excel_error(path, source))?;

    for (index, column_info) in table_template_columns(ir, table).iter().enumerate() {
        let column = (index + 1) as u16;
        let width = field_width(
            column_info.name.as_str(),
            Some(&column_info.comment),
            &column_info.type_hint,
        );
        worksheet
            .set_column_width(column, width)
            .map_err(|source| excel_error(path, source))?;
        worksheet
            .set_column_format(
                column,
                if column_info.derived {
                    &formats.derived_data
                } else {
                    &formats.schema_value
                },
            )
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

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use sora_ir::model::{ConfigIr, DerivedFieldIr, EnumIr, ScopeIr, StructIr, TypeIr};

    #[test]
    fn field_note_text_includes_comment_and_metadata() {
        let field = FieldIr {
            name: "rewards".to_owned(),
            ty: TypeIr::List(Box::new(TypeIr::Struct("Reward".to_owned()))),
            scope: ScopeIr::default(),
            key: false,
            comment: Some("Reward rows".to_owned()),
            default: Some("[]".to_owned()),
            range: Some([1, 99]),
            length: Some([1, 3]),
            parser: None,
            derived_from: Some(DerivedFieldIr {
                source_table: "Reward".to_owned(),
                parent_key: "id".to_owned(),
                child_key: "item_id".to_owned(),
                value_field: Some("value".to_owned()),
                order_by: Some("rank".to_owned()),
            }),
        };

        let ir = empty_ir();
        let note_text = field_note_text(&ir, &field).unwrap();

        assert!(note_text.contains("Reward rows"));
        assert!(note_text.contains("Field: rewards"));
        assert!(note_text.contains("Type: list<struct<Reward>>"));
        assert!(note_text.contains("Scope: all"));
        assert!(note_text.contains("Default: []"));
        assert!(note_text.contains("Range: 1..99"));
        assert!(note_text.contains("Length: 1..3"));
        assert!(note_text.contains("From: Reward.item_id -> id"));
        assert!(note_text.contains("From field: value"));
    }

    #[test]
    fn field_note_text_skips_empty_comments() {
        let field = FieldIr {
            name: "name".to_owned(),
            ty: TypeIr::String,
            scope: ScopeIr::default(),
            key: false,
            comment: Some("   ".to_owned()),
            default: None,
            range: None,
            length: None,
            parser: None,
            derived_from: None,
        };

        let ir = empty_ir();
        assert_eq!(field_note_text(&ir, &field), None);
    }

    #[test]
    fn field_note_text_includes_tuple_shape_without_comment() {
        let ir = ConfigIr {
            package: "game_config".to_owned(),
            localization: None,
            enums: vec![EnumIr {
                name: "ResourceType".to_owned(),
                scope: ScopeIr::default(),
                values: vec!["Item".to_owned()],
                aliases: Vec::new(),
            }],
            structs: vec![StructIr {
                name: "ResourceCost".to_owned(),
                scope: ScopeIr::default(),
                fields: vec![
                    FieldIr {
                        name: "kind".to_owned(),
                        ty: TypeIr::Enum("ResourceType".to_owned()),
                        scope: ScopeIr::default(),
                        key: false,
                        comment: None,
                        default: None,
                        range: None,
                        length: None,
                        parser: None,
                        derived_from: None,
                    },
                    FieldIr {
                        name: "id".to_owned(),
                        ty: TypeIr::I32,
                        scope: ScopeIr::default(),
                        key: false,
                        comment: None,
                        default: None,
                        range: None,
                        length: None,
                        parser: None,
                        derived_from: None,
                    },
                    FieldIr {
                        name: "count".to_owned(),
                        ty: TypeIr::I32,
                        scope: ScopeIr::default(),
                        key: false,
                        comment: None,
                        default: None,
                        range: None,
                        length: None,
                        parser: None,
                        derived_from: None,
                    },
                ],
            }],
            unions: Vec::new(),
            tables: Vec::new(),
        };
        let field = FieldIr {
            name: "cost".to_owned(),
            ty: TypeIr::Struct("ResourceCost".to_owned()),
            scope: ScopeIr::default(),
            key: false,
            comment: None,
            default: None,
            range: None,
            length: None,
            parser: Some(sora_ir::model::ParserIr {
                kind: "tuple".to_owned(),
                options: BTreeMap::new(),
            }),
            derived_from: None,
        };

        let note_text = field_note_text(&ir, &field).unwrap();

        assert!(note_text.contains("Tuple fields: kind: enum<ResourceType>, id: i32, count: i32"));
    }

    #[test]
    fn expanded_struct_column_note_includes_root_type() {
        let root_field = FieldIr {
            name: "cost".to_owned(),
            ty: TypeIr::Struct("ResourceCost".to_owned()),
            scope: ScopeIr::default(),
            key: false,
            comment: None,
            default: None,
            range: None,
            length: None,
            parser: Some(sora_ir::model::ParserIr {
                kind: "columns".to_owned(),
                options: BTreeMap::new(),
            }),
            derived_from: None,
        };
        let struct_field = FieldIr {
            name: "kind".to_owned(),
            ty: TypeIr::Enum("ResourceType".to_owned()),
            scope: ScopeIr::default(),
            key: false,
            comment: Some("Resource kind".to_owned()),
            default: None,
            range: None,
            length: None,
            parser: None,
            derived_from: None,
        };

        let note_text =
            struct_column_note_text(&empty_ir(), &root_field, &struct_field, "cost_kind");

        assert!(note_text.contains("Resource kind"));
        assert!(note_text.contains("Field: cost.kind"));
        assert!(note_text.contains("Type: enum<ResourceType>"));
        assert!(note_text.contains("Root field: cost"));
        assert!(note_text.contains("Root type: struct<ResourceCost>"));
        assert!(note_text.contains("Parser: columns"));
    }

    #[test]
    fn expanded_tagged_column_note_includes_root_type_without_variant_comment() {
        let root_field = FieldIr {
            name: "value".to_owned(),
            ty: TypeIr::Union("EventCondition".to_owned()),
            scope: ScopeIr::default(),
            key: false,
            comment: None,
            default: None,
            range: None,
            length: None,
            parser: Some(sora_ir::model::ParserIr {
                kind: "tagged_columns".to_owned(),
                options: BTreeMap::new(),
            }),
            derived_from: None,
        };
        let variant_field = FieldIr {
            name: "quest_id".to_owned(),
            ty: TypeIr::I32,
            scope: ScopeIr::default(),
            key: false,
            comment: None,
            default: None,
            range: None,
            length: None,
            parser: None,
            derived_from: None,
        };
        let tagged_column = sora_ir::input_projection::TaggedColumn {
            name: "quest_id".to_owned(),
            kind: sora_ir::input_projection::TaggedColumnKind::VariantField(&variant_field),
        };

        let note_text = tagged_column_note_text(&empty_ir(), &root_field, &tagged_column);

        assert!(note_text.contains("Field: value.quest_id"));
        assert!(note_text.contains("Type: i32"));
        assert!(note_text.contains("Root field: value"));
        assert!(note_text.contains("Root type: union<EventCondition>"));
        assert!(note_text.contains("Parser: tagged_columns"));
    }

    #[test]
    fn root_type_note_targets_type_row_content() {
        let field = FieldIr {
            name: "cost".to_owned(),
            ty: TypeIr::Struct("ResourceCost".to_owned()),
            scope: ScopeIr::default(),
            key: false,
            comment: None,
            default: None,
            range: None,
            length: None,
            parser: None,
            derived_from: None,
        };

        assert_eq!(
            root_type_note_text(&field, "columns"),
            "Root field: cost\nRoot type: struct<ResourceCost>\nParser: columns"
        );
    }

    fn empty_ir() -> ConfigIr {
        ConfigIr {
            package: "game_config".to_owned(),
            localization: None,
            enums: Vec::new(),
            structs: Vec::new(),
            unions: Vec::new(),
            tables: Vec::new(),
        }
    }
}
