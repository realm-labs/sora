
local ResourceCost = require("generated.resource_cost")

---@class VipLevel
---@field level integer
---@field cost ResourceCost
---@field perks string[]

local VipLevel = {}

---@param reader SoraReader
---@return VipLevel
function VipLevel.decode(reader)
    return {
        level = reader:read_i32(),
        cost = ResourceCost.decode(reader),
        perks = reader:read_list(function() return reader:read_string() end),
    }
end

return VipLevel
