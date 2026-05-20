use std::borrow::Cow;

use calamine::Data;
use sora_data::model::Value;
use sora_diagnostics::Result;
use sora_input::cell::{CellContext, CellValue, cell_to_value as parse_cell_value};
use sora_ir::model::TypeIr;

pub(crate) fn cell_to_value(cell: &Data, ty: &TypeIr, context: &CellContext<'_>) -> Result<Value> {
    parse_cell_value(&xlsx_cell_value(cell), ty, context)
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
