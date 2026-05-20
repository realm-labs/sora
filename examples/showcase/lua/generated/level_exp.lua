

---@class LevelExp
---@field level integer
---@field exp integer
---@field unlockFeature string?

local LevelExp = {}

---@param reader SoraReader
---@return LevelExp
function LevelExp.decode(reader)
    return {
        level = reader:read_i32(),
        exp = reader:read_i64(),
        unlockFeature = reader:read_optional(function() return reader:read_string() end),
    }
end

return LevelExp
