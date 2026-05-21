#pragma once

#include "sora_runtime.hpp"

namespace sora::showcase {

struct QuestReward {
    std::int32_t quest_id;
    std::int32_t seq;
    std::int32_t item_id;
    std::int32_t count;

    static QuestReward decode(SoraReader& reader) {
        return QuestReward{
            reader.read_i32(),
            reader.read_i32(),
            reader.read_i32(),
            reader.read_i32(),
        };
    }
};

} // namespace sora::showcase
