use std::collections::BTreeSet;

use crate::model::TypeIr;

pub const INTERVAL_START_FIELD: &str = "start";
pub const INTERVAL_END_FIELD: &str = "end";
pub const INTERVAL_START_INCLUSIVE_FIELD: &str = "start_inclusive";
pub const INTERVAL_END_INCLUSIVE_FIELD: &str = "end_inclusive";

pub fn interval_struct_name(element: &TypeIr) -> Option<&'static str> {
    match element {
        TypeIr::I8 => Some("SoraIntervalI8"),
        TypeIr::U8 => Some("SoraIntervalU8"),
        TypeIr::I16 => Some("SoraIntervalI16"),
        TypeIr::U16 => Some("SoraIntervalU16"),
        TypeIr::I32 => Some("SoraIntervalI32"),
        TypeIr::U32 => Some("SoraIntervalU32"),
        TypeIr::I64 => Some("SoraIntervalI64"),
        TypeIr::F32 => Some("SoraIntervalF32"),
        TypeIr::F64 => Some("SoraIntervalF64"),
        TypeIr::Duration => Some("SoraIntervalDuration"),
        TypeIr::DateTime => Some("SoraIntervalDateTime"),
        _ => None,
    }
}

pub fn interval_element_type(struct_name: &str) -> Option<TypeIr> {
    Some(match struct_name {
        "SoraIntervalI8" => TypeIr::I8,
        "SoraIntervalU8" => TypeIr::U8,
        "SoraIntervalI16" => TypeIr::I16,
        "SoraIntervalU16" => TypeIr::U16,
        "SoraIntervalI32" => TypeIr::I32,
        "SoraIntervalU32" => TypeIr::U32,
        "SoraIntervalI64" => TypeIr::I64,
        "SoraIntervalF32" => TypeIr::F32,
        "SoraIntervalF64" => TypeIr::F64,
        "SoraIntervalDuration" => TypeIr::Duration,
        "SoraIntervalDateTime" => TypeIr::DateTime,
        _ => return None,
    })
}

pub fn interval_element(ty: &TypeIr) -> Option<TypeIr> {
    match ty {
        TypeIr::Optional(inner) => interval_element(inner),
        TypeIr::Struct(name) => interval_element_type(name),
        _ => None,
    }
}

pub fn collect_interval_structs(ty: &TypeIr, names: &mut BTreeSet<String>) {
    match ty {
        TypeIr::Struct(name) if interval_element_type(name).is_some() => {
            names.insert(name.clone());
        }
        TypeIr::List(element)
        | TypeIr::Set(element)
        | TypeIr::Optional(element)
        | TypeIr::Array { element, .. } => collect_interval_structs(element, names),
        TypeIr::Map { key, value } => {
            collect_interval_structs(key, names);
            collect_interval_structs(value, names);
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interval_struct_names_round_trip() {
        for ty in [
            TypeIr::I8,
            TypeIr::U8,
            TypeIr::I16,
            TypeIr::U16,
            TypeIr::I32,
            TypeIr::U32,
            TypeIr::I64,
            TypeIr::F32,
            TypeIr::F64,
            TypeIr::Duration,
            TypeIr::DateTime,
        ] {
            let name = interval_struct_name(&ty).unwrap();
            assert_eq!(interval_element_type(name), Some(ty));
        }
    }
}
