#pragma once

#include "sora_runtime.hpp"

#include <cstdint>
#include <functional>

namespace sora::showcase {

enum class MailType : std::int32_t {
    System = 0,
    Event = 1,
    Compensation = 2,
};

inline MailType decode_mail_type_ordinal(std::uint32_t value) {
    switch (value) {
    case 0:
        return MailType::System;
    case 1:
        return MailType::Event;
    case 2:
        return MailType::Compensation;
    default:
        throw SoraReadException("invalid enum ordinal for MailType");
    }
}

} // namespace sora::showcase

namespace std {
template <>
struct hash<sora::showcase::MailType> {
    std::size_t operator()(const sora::showcase::MailType& value) const {
        return std::hash<std::int32_t>()(static_cast<std::int32_t>(value));
    }
};
} // namespace std

namespace sora::showcase {

template <>
inline MailType decode_value<MailType>(SoraReader& reader) {
    return decode_mail_type_ordinal(reader.read_u32());
}

} // namespace sora::showcase
