use std::path::Path;

use rust_xlsxwriter::{
    Color, DataValidation, DataValidationRule, Format, FormatAlign, FormatBorder, Note, Workbook,
};
use sora_diagnostics::{Result, SoraError};
use sora_ir::{
    input_projection::{TaggedColumnKind, tagged_columns, tagged_columns_union},
    model::{ConfigIr, FieldIr, TableIr, TypeIr},
};

use crate::projection::{
    DATA_START_ROW, DESC_ROW, FIELD_START_COLUMN, NAME_ROW, table_template_columns,
    table_template_rows, tuple_shape,
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

        for (row_index, row) in table_template_rows(ir, table).iter().enumerate() {
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

        apply_sheet_layout(ir, table, worksheet, &formats, path)?;
        apply_field_notes(ir, table, worksheet, path)?;
        apply_data_validations(ir, table, worksheet, path)?;
        write_data_rows(worksheet, &formats, path, rows_for_table(table))?;
        worksheet
            .set_freeze_panes(DATA_START_ROW, 1)
            .map_err(|source| excel_error(path, source))?;
    }

    workbook
        .save(path)
        .map_err(|source| excel_error(path, source))
}

fn write_data_rows(
    worksheet: &mut rust_xlsxwriter::Worksheet,
    formats: &TemplateFormats,
    path: &Path,
    rows: Vec<Vec<String>>,
) -> Result<()> {
    for (row_offset, row) in rows.iter().enumerate() {
        for (column, value) in row.iter().enumerate() {
            worksheet
                .write_with_format(
                    DATA_START_ROW + row_offset as u32,
                    FIELD_START_COLUMN + column as u16,
                    value,
                    formats.data_cell(),
                )
                .map_err(|source| excel_error(path, source))?;
        }
    }

    Ok(())
}

fn apply_data_validations(
    ir: &ConfigIr,
    table: &TableIr,
    worksheet: &mut rust_xlsxwriter::Worksheet,
    path: &Path,
) -> Result<()> {
    let mut column = FIELD_START_COLUMN;
    for field in &table.fields {
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
        if let Some(columns) = tagged_columns(ir, field) {
            for tagged_column in columns {
                let note_text = match tagged_column.kind {
                    TaggedColumnKind::Tag => Some(format!(
                        "Field: {}\nType: {}\nParser: tagged_columns\nColumn: {}",
                        field.name, field.ty, tagged_column.name
                    )),
                    TaggedColumnKind::VariantField(variant_field) => {
                        field_note_text(ir, variant_field)
                    }
                };
                if let Some(note_text) = note_text {
                    let note = Note::new(note_text).set_author("Sora");
                    worksheet
                        .insert_note(NAME_ROW, column, &note)
                        .map_err(|source| excel_error(path, source))?;
                }
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
    if field.is_required() {
        lines.push("Required: yes".to_owned());
    }
    if let Some(default) = &field.default {
        lines.push(format!("Default: {default}"));
    }
    if let Some([min, max]) = field.range {
        lines.push(format!("Range: {min}..{max}"));
    }
    if let Some([min, max]) = field.length {
        lines.push(format!("Length: {min}..{max}"));
    }
    if let Some(derived_from) = &field.derived_from {
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
        TypeIr::I32 | TypeIr::I64 => integer_validation(field, path)?,
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
        TypeIr::I32 | TypeIr::I64 => integer_validation(field, path)?,
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
    schema_label: Format,
    schema_value: Format,
    description: Format,
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
        }
    }

    fn for_cell(&self, row: usize, column: usize) -> &Format {
        let row = row as u32;
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

    fn data_cell(&self) -> &Format {
        &self.schema_value
    }
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

    fn empty_ir() -> ConfigIr {
        ConfigIr {
            package: "game_config".to_owned(),
            enums: Vec::new(),
            structs: Vec::new(),
            unions: Vec::new(),
            tables: Vec::new(),
        }
    }
}
