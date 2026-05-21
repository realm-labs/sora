#pragma once

#include "sora_runtime.hpp"
#include "skill_effect.hpp"

namespace sora::showcase {

struct EquipmentSet {
    std::int32_t id;
    std::string name;
    std::vector<std::int32_t> item_ids;
    SkillEffect bonus_effect;

    static EquipmentSet decode(SoraReader& reader) {
        return EquipmentSet{
            reader.read_i32(),
            reader.read_string(),
            reader.read_vector<std::int32_t>(),
            SkillEffect::decode(reader),
        };
    }
};

} // namespace sora::showcase
