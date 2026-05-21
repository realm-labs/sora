#pragma once

#include "sora_runtime.hpp"
#include "resource_kind.hpp"

namespace sora::showcase {

struct ResourceCost {
    ResourceKind kind;
    std::int32_t id;
    std::int32_t count;

    static ResourceCost decode(SoraReader& reader) {
        return ResourceCost{
            decode_value<ResourceKind>(reader),
            reader.read_i32(),
            reader.read_i32(),
        };
    }
};

} // namespace sora::showcase
