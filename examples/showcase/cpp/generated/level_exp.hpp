#pragma once

#include "sora_runtime.hpp"

namespace sora::showcase {

struct LevelExp {
    std::int32_t level;
    std::int64_t exp;
    std::optional<std::string> unlock_feature;

    static LevelExp decode(SoraReader& reader) {
        return LevelExp{
            reader.read_i32(),
            reader.read_i64(),
            reader.read_optional<std::string>(),
        };
    }
};

} // namespace sora::showcase
