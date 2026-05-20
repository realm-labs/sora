---@alias MailType
---| '"System"'
---| '"Event"'
---| '"Compensation"'

---@class MailTypeValues
---@field System MailType
---@field Event MailType
---@field Compensation MailType

---@type MailTypeValues
local MailType = {
    System = "System",
    Event = "Event",
    Compensation = "Compensation",
}

local values = {
    [0] = MailType.System,
    [1] = MailType.Event,
    [2] = MailType.Compensation,
}

---@param reader SoraReader
---@return MailType
function MailType.decode(reader)
    local ordinal = reader:read_u32()
    local value = values[ordinal]
    if value == nil then
        error("invalid enum ordinal " .. tostring(ordinal) .. " for MailType")
    end
    return value
end

return MailType
