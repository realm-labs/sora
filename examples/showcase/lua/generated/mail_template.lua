
local MailType = require("generated.mail_type")
local Reward = require("generated.reward")

---@class MailTemplate
---@field id integer
---@field mailType MailType
---@field titleKey string
---@field bodyKey string
---@field rewards Reward[]

local MailTemplate = {}

---@param reader SoraReader
---@return MailTemplate
function MailTemplate.decode(reader)
    return {
        id = reader:read_i32(),
        mailType = MailType.decode(reader),
        titleKey = reader:read_string(),
        bodyKey = reader:read_string(),
        rewards = reader:read_list(function() return Reward.decode(reader) end),
    }
end

return MailTemplate
