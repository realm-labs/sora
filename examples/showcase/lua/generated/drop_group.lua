

---@class DropGroup
---@field id integer
---@field name string

local DropGroup = {}

---@param reader SoraReader
---@return DropGroup
function DropGroup.decode(reader)
    return {
        id = reader:read_i32(),
        name = reader:read_string(),
    }
end

return DropGroup
