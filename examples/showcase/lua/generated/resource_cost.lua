
local ResourceKind = require("generated.resource_kind")

---@class ResourceCost
---@field kind ResourceKind
---@field id integer
---@field count integer

local ResourceCost = {}

---@param reader SoraReader
---@return ResourceCost
function ResourceCost.decode(reader)
    return {
        kind = ResourceKind.decode(reader),
        id = reader:read_i32(),
        count = reader:read_i32(),
    }
end

return ResourceCost
