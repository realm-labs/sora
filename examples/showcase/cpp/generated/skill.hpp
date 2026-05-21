#pragma once

#include "sora_runtime.hpp"
#include "element_type.hpp"
#include "resource_cost.hpp"
#include "skill_effect.hpp"
#include "vec3.hpp"

namespace sora::showcase {

struct Skill {
    std::int32_t id;
    std::string name;
    ElementType element;
    // Tuple cost, e.g. Gold,0,150
    ResourceCost cost;
    // JSON object with element/power/radius
    SkillEffect effect;
    std::int32_t required_level;
    // Optional item requirement
    std::optional<std::int32_t> required_item;
    Vec3 cast_origin;

    static Skill decode(SoraReader& reader) {
        return Skill{
            reader.read_i32(),
            reader.read_string(),
            decode_value<ElementType>(reader),
            ResourceCost::decode(reader),
            SkillEffect::decode(reader),
            reader.read_i32(),
            reader.read_optional<std::int32_t>(),
            Vec3::decode(reader),
        };
    }
};

} // namespace sora::showcase
