#pragma once

#include "sora_runtime.hpp"
#include "resource_kind.hpp"

namespace sora::showcase {

struct Shop {
    std::int32_t id;
    std::string name;
    ResourceKind currency;

    static Shop decode(SoraReader& reader) {
        return Shop{
            reader.read_i32(),
            reader.read_string(),
            decode_value<ResourceKind>(reader),
        };
    }
};

} // namespace sora::showcase
