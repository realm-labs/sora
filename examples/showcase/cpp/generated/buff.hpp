#pragma once

#include "sora_runtime.hpp"
#include "stat_modifier.hpp"

namespace sora::showcase {

struct Buff {
    std::int32_t id;
    std::string name;
    float duration;
    std::vector<StatModifier> modifiers;

    static Buff decode(SoraReader& reader) {
        return Buff{
            reader.read_i32(),
            reader.read_string(),
            reader.read_f32(),
            reader.read_vector<StatModifier>(),
        };
    }
};

} // namespace sora::showcase
