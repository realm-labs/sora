#pragma once

#include "sora_runtime.hpp"

#include <cstdint>
#include <functional>

namespace sora::showcase {

enum class Rarity : std::int32_t {
    Common = 0,
    Uncommon = 1,
    Rare = 2,
    Epic = 3,
    Legendary = 4,
};

inline Rarity decode_rarity_ordinal(std::uint32_t value) {
    switch (value) {
    case 0:
        return Rarity::Common;
    case 1:
        return Rarity::Uncommon;
    case 2:
        return Rarity::Rare;
    case 3:
        return Rarity::Epic;
    case 4:
        return Rarity::Legendary;
    default:
        throw SoraReadException("invalid enum ordinal for Rarity");
    }
}

} // namespace sora::showcase

namespace std {
template <>
struct hash<sora::showcase::Rarity> {
    std::size_t operator()(const sora::showcase::Rarity& value) const {
        return std::hash<std::int32_t>()(static_cast<std::int32_t>(value));
    }
};
} // namespace std

namespace sora::showcase {

template <>
inline Rarity decode_value<Rarity>(SoraReader& reader) {
    return decode_rarity_ordinal(reader.read_u32());
}

} // namespace sora::showcase
