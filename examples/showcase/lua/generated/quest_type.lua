---@alias QuestType
---| '"Main"'
---| '"Side"'
---| '"Daily"'

---@class QuestTypeValues
---@field Main QuestType
---@field Side QuestType
---@field Daily QuestType

---@type QuestTypeValues
local QuestType = {
    Main = "Main",
    Side = "Side",
    Daily = "Daily",
}

local values = {
    [0] = QuestType.Main,
    [1] = QuestType.Side,
    [2] = QuestType.Daily,
}

---@param reader SoraReader
---@return QuestType
function QuestType.decode(reader)
    local ordinal = reader:read_u32()
    local value = values[ordinal]
    if value == nil then
        error("invalid enum ordinal " .. tostring(ordinal) .. " for QuestType")
    end
    return value
end

return QuestType
