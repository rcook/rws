local m = {}

function m.trim(s)
  return s:gsub("^%s*(.-)%s*$", "%1")
end

function m.parse_config(lines)
  local result = {}
  local line
  for _, line in ipairs(lines) do
    local temp = m.trim(line)
    if temp:find("#") ~= 1 and temp:len() > 0 then
      result[#result + 1] = temp
    end
  end
  return result
end

return m
