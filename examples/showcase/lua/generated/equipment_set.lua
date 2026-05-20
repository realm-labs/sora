
local SkillEffect = require("generated.skill_effect")

---@class EquipmentSet
---@field id integer
---@field name string
---@field itemIds integer[]
---@field bonusEffect SkillEffect

local EquipmentSet = {}

---@param reader SoraReader
---@return EquipmentSet
function EquipmentSet.decode(reader)
    return {
        id = reader:read_i32(),
        name = reader:read_string(),
        itemIds = reader:read_list(function() return reader:read_i32() end),
        bonusEffect = SkillEffect.decode(reader),
    }
end

return EquipmentSet
