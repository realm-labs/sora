use crate::model::TypeIr;

pub const INTERVAL_START_FIELD: &str = "start";
pub const INTERVAL_END_FIELD: &str = "end";
pub const INTERVAL_START_INCLUSIVE_FIELD: &str = "start_inclusive";
pub const INTERVAL_END_INCLUSIVE_FIELD: &str = "end_inclusive";

pub fn interval_record_name(element: &TypeIr) -> Option<&'static str> {
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

pub fn is_interval_element(element: &TypeIr) -> bool {
    matches!(
        element,
        TypeIr::I8
            | TypeIr::U8
            | TypeIr::I16
            | TypeIr::U16
            | TypeIr::I32
            | TypeIr::U32
            | TypeIr::I64
            | TypeIr::F32
            | TypeIr::F64
            | TypeIr::Duration
            | TypeIr::DateTime
    )
}

pub fn interval_element(ty: &TypeIr) -> Option<&TypeIr> {
    match ty {
        TypeIr::Optional(inner) => interval_element(inner),
        TypeIr::Interval(element) => Some(element),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_supported_interval_elements() {
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
            assert!(is_interval_element(&ty));
            assert!(interval_record_name(&ty).is_some());
        }
        assert!(!is_interval_element(&TypeIr::String));
    }
}
