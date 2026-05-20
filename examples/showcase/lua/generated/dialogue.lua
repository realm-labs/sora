

---@class Dialogue
---@field id integer
---@field speakerKey string
---@field lines string[]

local Dialogue = {}

---@param reader SoraReader
---@return Dialogue
function Dialogue.decode(reader)
    return {
        id = reader:read_i32(),
        speakerKey = reader:read_string(),
        lines = reader:read_list(function() return reader:read_string() end),
    }
end

return Dialogue
