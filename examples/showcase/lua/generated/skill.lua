
local ElementType = require("generated.element_type")
local ResourceCost = require("generated.resource_cost")
local SkillEffect = require("generated.skill_effect")
local Vec3 = require("generated.vec3")

---@class Skill
---@field id integer
---@field name string
---@field element ElementType
---@field cost ResourceCost
---@field effect SkillEffect
---@field requiredLevel integer
---@field requiredItem integer?
---@field castOrigin Vec3

local Skill = {}

---@param reader SoraReader
---@return Skill
function Skill.decode(reader)
    return {
        id = reader:read_i32(),
        name = reader:read_string(),
        element = ElementType.decode(reader),
        cost = ResourceCost.decode(reader),
        effect = SkillEffect.decode(reader),
        requiredLevel = reader:read_i32(),
        requiredItem = reader:read_optional(function() return reader:read_i32() end),
        castOrigin = Vec3.decode(reader),
    }
end

return Skill
