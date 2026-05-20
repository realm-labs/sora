
local ResourceCost = require("generated.resource_cost")

---@class Dungeon
---@field id integer
---@field name string
---@field stageIds integer[]
---@field entryCost ResourceCost

local Dungeon = {}

---@param reader SoraReader
---@return Dungeon
function Dungeon.decode(reader)
    return {
        id = reader:read_i32(),
        name = reader:read_string(),
        stageIds = reader:read_list(function() return reader:read_i32() end),
        entryCost = ResourceCost.decode(reader),
    }
end

return Dungeon
