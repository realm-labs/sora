
local Rarity = require("generated.rarity")
local Vec3 = require("generated.vec3")

---@class Character
---@field id integer
---@field name string
---@field rarity Rarity
---@field baseLevel integer
---@field baseSkill integer
---@field starterItems integer[]
---@field spawnPos Vec3

local Character = {}

---@param reader SoraReader
---@return Character
function Character.decode(reader)
    return {
        id = reader:read_i32(),
        name = reader:read_string(),
        rarity = Rarity.decode(reader),
        baseLevel = reader:read_i32(),
        baseSkill = reader:read_i32(),
        starterItems = reader:read_list(function() return reader:read_i32() end),
        spawnPos = Vec3.decode(reader),
    }
end

return Character
