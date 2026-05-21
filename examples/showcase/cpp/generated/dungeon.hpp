#pragma once

#include "sora_runtime.hpp"
#include "resource_cost.hpp"

namespace sora::showcase {

struct Dungeon {
    std::int32_t id;
    std::string name;
    std::vector<std::int32_t> stage_ids;
    ResourceCost entry_cost;

    static Dungeon decode(SoraReader& reader) {
        return Dungeon{
            reader.read_i32(),
            reader.read_string(),
            reader.read_vector<std::int32_t>(),
            ResourceCost::decode(reader),
        };
    }
};

} // namespace sora::showcase
