
local StatModifier = require("generated.stat_modifier")

---@class Buff
---@field id integer
---@field name string
---@field duration number
---@field modifiers StatModifier[]

local Buff = {}

---@param reader SoraReader
---@return Buff
function Buff.decode(reader)
    return {
        id = reader:read_i32(),
        name = reader:read_string(),
        duration = reader:read_f32(),
        modifiers = reader:read_list(function() return StatModifier.decode(reader) end),
    }
end

return Buff
