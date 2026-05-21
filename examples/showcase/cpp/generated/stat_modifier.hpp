#pragma once

#include "sora_runtime.hpp"
#include "stat_type.hpp"

namespace sora::showcase {

struct StatModifier {
    StatType stat;
    float value;
    bool is_percent;

    static StatModifier decode(SoraReader& reader) {
        return StatModifier{
            decode_value<StatType>(reader),
            reader.read_f32(),
            reader.read_bool(),
        };
    }
};

} // namespace sora::showcase
