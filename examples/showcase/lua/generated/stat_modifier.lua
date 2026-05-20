
local StatType = require("generated.stat_type")

---@class StatModifier
---@field stat StatType
---@field value number
---@field isPercent boolean

local StatModifier = {}

---@param reader SoraReader
---@return StatModifier
function StatModifier.decode(reader)
    return {
        stat = StatType.decode(reader),
        value = reader:read_f32(),
        isPercent = reader:read_bool(),
    }
end

return StatModifier
