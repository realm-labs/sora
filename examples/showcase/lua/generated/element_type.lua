---@alias ElementType
---| '"Fire"'
---| '"Ice"'
---| '"Lightning"'
---| '"Physical"'

---@class ElementTypeValues
---@field Fire ElementType
---@field Ice ElementType
---@field Lightning ElementType
---@field Physical ElementType

---@type ElementTypeValues
local ElementType = {
    Fire = "Fire",
    Ice = "Ice",
    Lightning = "Lightning",
    Physical = "Physical",
}
local values = {
    [0] = ElementType.Fire,
    [1] = ElementType.Ice,
    [2] = ElementType.Lightning,
    [3] = ElementType.Physical,
}

---@param reader SoraReader
---@return ElementType
function ElementType.decode(reader)
    local ordinal = reader:read_u32()
    local value = values[ordinal]
    if value == nil then
        error("invalid enum ordinal " .. tostring(ordinal) .. " for ElementType")
    end
    return value
end

return ElementType
