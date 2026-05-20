
local Vec3 = require("generated.vec3")

---@class GameSettings
---@field version string
---@field dailyResetHour integer
---@field startingGold integer
---@field spawnPos Vec3
---@field starterItems integer[]

local GameSettings = {}

---@param reader SoraReader
---@return GameSettings
function GameSettings.decode(reader)
    return {
        version = reader:read_string(),
        dailyResetHour = reader:read_i32(),
        startingGold = reader:read_i32(),
        spawnPos = Vec3.decode(reader),
        starterItems = reader:read_list(function() return reader:read_i32() end),
    }
end

return GameSettings
