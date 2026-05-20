---@alias ResourceKind
---| '"Item"'
---| '"Gold"'
---| '"Diamond"'

---@class ResourceKindValues
---@field Item ResourceKind
---@field Gold ResourceKind
---@field Diamond ResourceKind

---@type ResourceKindValues
local ResourceKind = {
    Item = "Item",
    Gold = "Gold",
    Diamond = "Diamond",
}

local values = {
    [0] = ResourceKind.Item,
    [1] = ResourceKind.Gold,
    [2] = ResourceKind.Diamond,
}

---@param reader SoraReader
---@return ResourceKind
function ResourceKind.decode(reader)
    local ordinal = reader:read_u32()
    local value = values[ordinal]
    if value == nil then
        error("invalid enum ordinal " .. tostring(ordinal) .. " for ResourceKind")
    end
    return value
end

return ResourceKind
