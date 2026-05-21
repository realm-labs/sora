#pragma once

#include "sora_runtime.hpp"
#include "event_condition.hpp"
#include "reward_action.hpp"

namespace sora::showcase {

struct EventRule {
    std::int32_t id;
    std::string name;
    EventCondition condition;
    std::vector<RewardAction> actions;

    static EventRule decode(SoraReader& reader) {
        return EventRule{
            reader.read_i32(),
            reader.read_string(),
            EventCondition::decode(reader),
            reader.read_vector<RewardAction>(),
        };
    }
};

} // namespace sora::showcase
