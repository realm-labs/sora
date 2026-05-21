use std::borrow::Cow;

use calamine::Data;
use sora_data::model::Value;
use sora_diagnostics::Result;
use sora_input::{
    cell::{CellContext, CellValue, cell_to_value_with_parsers},
    parser::ParserRegistry,
};
use sora_ir::model::TypeIr;

pub(crate) fn cell_to_value_with_registry(
    cell: &Data,
    ty: &TypeIr,
    context: &CellContext<'_>,
    parser_registry: &ParserRegistry,
) -> Result<Value> {
    cell_to_value_with_parsers(&xlsx_cell_value(cell), ty, context, parser_registry)
}

pub(crate) fn cell_is_empty(cell: &Data) -> bool {
    xlsx_cell_value(cell).is_empty()
}

pub(crate) fn cell_to_string(cell: &Data) -> String {
    xlsx_cell_value(cell).display_text()
}

fn xlsx_cell_value(cell: &Data) -> CellValue<'_> {
    match cell {
        Data::Empty => CellValue::Empty,
        Data::String(value) => CellValue::Text(Cow::Borrowed(value)),
        Data::Float(value) => CellValue::Float(*value),
        Data::Int(value) => CellValue::Integer(*value),
        Data::Bool(value) => CellValue::Bool(*value),
        Data::DateTime(value) => CellValue::Text(Cow::Owned(value.to_string())),
        Data::DateTimeIso(value) => CellValue::Text(Cow::Borrowed(value)),
        Data::DurationIso(value) => CellValue::Text(Cow::Borrowed(value)),
        Data::Error(value) => CellValue::Error(Cow::Owned(value.to_string())),
    }
}
