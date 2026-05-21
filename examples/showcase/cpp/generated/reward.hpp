#pragma once

#include "sora_runtime.hpp"

namespace sora::showcase {

struct Reward {
    std::int32_t item_id;
    std::int32_t count;

    static Reward decode(SoraReader& reader) {
        return Reward{
            reader.read_i32(),
            reader.read_i32(),
        };
    }
};

} // namespace sora::showcase
