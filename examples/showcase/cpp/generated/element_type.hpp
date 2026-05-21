#pragma once

#include "sora_runtime.hpp"

#include <cstdint>
#include <functional>

namespace sora::showcase {

enum class ElementType : std::int32_t {
    Fire = 0,
    Ice = 1,
    Lightning = 2,
    Physical = 3,
};

inline ElementType decode_element_type_ordinal(std::uint32_t value) {
    switch (value) {
    case 0:
        return ElementType::Fire;
    case 1:
        return ElementType::Ice;
    case 2:
        return ElementType::Lightning;
    case 3:
        return ElementType::Physical;
    default:
        throw SoraReadException("invalid enum ordinal for ElementType");
    }
}

} // namespace sora::showcase

namespace std {
template <>
struct hash<sora::showcase::ElementType> {
    std::size_t operator()(const sora::showcase::ElementType& value) const {
        return std::hash<std::int32_t>()(static_cast<std::int32_t>(value));
    }
};
} // namespace std

namespace sora::showcase {

template <>
inline ElementType decode_value<ElementType>(SoraReader& reader) {
    return decode_element_type_ordinal(reader.read_u32());
}

} // namespace sora::showcase
