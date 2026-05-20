
local ResourceCost = require("generated.resource_cost")

---@class Achievement
---@field id integer
---@field titleKey string
---@field targetCount string
---@field reward ResourceCost

local Achievement = {}

---@param reader SoraReader
---@return Achievement
function Achievement.decode(reader)
    return {
        id = reader:read_i32(),
        titleKey = reader:read_string(),
        targetCount = reader:read_i64_string(),
        reward = ResourceCost.decode(reader),
    }
end

return Achievement
