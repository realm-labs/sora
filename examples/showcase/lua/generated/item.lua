
local ItemType = require("generated.item_type")
local ResourceCost = require("generated.resource_cost")

---@class Item
---@field id integer
---@field name string
---@field itemType ItemType
---@field maxStack integer
---@field price ResourceCost
---@field tags string[]

local Item = {}

---@param reader SoraReader
---@return Item
function Item.decode(reader)
    return {
        id = reader:read_i32(),
        name = reader:read_string(),
        itemType = ItemType.decode(reader),
        maxStack = reader:read_i32(),
        price = ResourceCost.decode(reader),
        tags = reader:read_list(function() return reader:read_string() end),
    }
end

return Item
