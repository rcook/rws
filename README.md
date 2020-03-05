# Richard's Workspace Tool

_Manages Git-based workspaces_

[Official home page][home]

## `git` command

_Runs Git command in each project directory_

## `info` command

_Prints workspace information_

## `run` command

_Runs command in each project directory_

## `rws-workspace.yaml` configuration

Schema:

```yaml
# (Optional)
default-language: lua

# (Optional)
lua-config:
  # (Optional)
  preamble: |
    #dofile("shared.lua")
    local DEPS_FILE_NAME = "_DEPS"
    local function parse_config_lines(lines)
      local result = {}
      for _, line in ipairs(lines) do
        local temp = prelude.trim_string(line)
        if string.len(temp:len()) > 0 and string.find(temp, "#") ~= 1 then
          result[#result + 1] = temp
        end
      end
      return result
    end
  # (Optional)
  use-prelude: true

# (Optional) (or specify "dependency-command")
dependencies:
  aaa:
  - bbb
  - ccc
  ccc:
  - ddd
  - eee

# (Optional) (or specify "dependencies")
dependency-command:
  # (Optional)
  language: lua
  # (Optional)
  use-prelude: true
  # (Required)
  script: |
    if prelude.is_file(DEPS_FILE_NAME) then
      return parse_config_lines(prelude.read_file_lines(DEPS_FILE_NAME))
    else
      return { }
    end

# (Optional)
excluded-projects:
- fff
- ggg
```

## Licence

[MIT License][licence]

[home]: https://github.com/rcook/rws
[licence]: LICENSE
