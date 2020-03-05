local m = {}

function m.trim(s)
  return s:gsub("^%s*(.-)%s*$", "%1")
end

function m.read_file(path)
  local f = assert(io.open(path, "rb"))
  local content = assert(f:read("*all"))
  f:close()
  return content
end

function m.read_file_lines(path)
  local f = assert(io.open(path, "rb"))
  local lines = {}
  local i = 1
  while true do
    local line = f:read()
    if line == nil then break end
    lines[i] = line
    i = i + 1
  end
  f:close()
  return lines
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
