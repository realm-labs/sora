
local ResourceKind = require("generated.resource_kind")

---@class Shop
---@field id integer
---@field name string
---@field currency ResourceKind

local Shop = {}

---@param reader SoraReader
---@return Shop
function Shop.decode(reader)
    return {
        id = reader:read_i32(),
        name = reader:read_string(),
        currency = ResourceKind.decode(reader),
    }
end

return Shop
