#pragma once

#include "sora_runtime.hpp"
#include "quest_type.hpp"
#include "reward.hpp"
#include "vec3.hpp"

namespace sora::showcase {

struct Quest {
    std::int32_t id;
    QuestType quest_type;
    std::string title;
    std::int32_t required_item;
    std::vector<std::int32_t> unlock_skills;
    Vec3 start_pos;
    // Materialized from QuestReward child rows
    std::vector<Reward> rewards;

    static Quest decode(SoraReader& reader) {
        return Quest{
            reader.read_i32(),
            decode_value<QuestType>(reader),
            reader.read_string(),
            reader.read_i32(),
            reader.read_vector<std::int32_t>(),
            Vec3::decode(reader),
            reader.read_vector<Reward>(),
        };
    }
};

} // namespace sora::showcase
