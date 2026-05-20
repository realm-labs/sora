
local EventCondition = require("generated.event_condition")
local RewardAction = require("generated.reward_action")

---@class EventRule
---@field id integer
---@field name string
---@field condition EventCondition
---@field actions RewardAction[]

local EventRule = {}

---@param reader SoraReader
---@return EventRule
function EventRule.decode(reader)
    return {
        id = reader:read_i32(),
        name = reader:read_string(),
        condition = EventCondition.decode(reader),
        actions = reader:read_list(function() return RewardAction.decode(reader) end),
    }
end

return EventRule
