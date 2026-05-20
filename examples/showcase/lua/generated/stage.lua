
local Reward = require("generated.reward")

---@class Stage
---@field id integer
---@field name string
---@field monsterIds integer[]
---@field recommendedPower integer
---@field firstClearRewards Reward[]

local Stage = {}

---@param reader SoraReader
---@return Stage
function Stage.decode(reader)
    return {
        id = reader:read_i32(),
        name = reader:read_string(),
        monsterIds = reader:read_list(function() return reader:read_i32() end),
        recommendedPower = reader:read_i32(),
        firstClearRewards = reader:read_list(function() return Reward.decode(reader) end),
    }
end

return Stage
