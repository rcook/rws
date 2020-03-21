# Richard's Workspace Tool

[![AppVeyor status for project](https://ci.appveyor.com/api/projects/status/m7bfloijbr2la3dh?svg=true)](https://ci.appveyor.com/project/rcook/rws)
[![AppVeyor status for master branch](https://ci.appveyor.com/api/projects/status/m7bfloijbr2la3dh/branch/master?svg=true)](https://ci.appveyor.com/project/rcook/rws/branch/master)

_Manages Git-based workspaces_

[Official home page][home]

This is intended to be a cross-platform Git workspace management tool. To allow users to extend its functionality via workspace configuration, RWS uses an embedded Lua scripting engine. This is intended to discourage users from writing non-portable shell script extensions. It has been tested on Ubuntu, Windows 10 and macOS 10.14.6.

## `git` command

_Runs Git command in each project directory_

## `info` command

_Prints workspace information_

## `run` command

_Runs command in each project directory_

## `rws-workspace.yaml` configuration

This is the schema for the optional `rws-workspace.yaml` configuration file that should be placed in the root directory of your multi-repo workspace:

```yaml
# (Optional)
variables:
  # (Optional)
  VARIABLE0: VALUE0
  # (Optional)
  VARIABLE1: VALUE1

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
        if temp:len() > 0 and temp:find("#") ~= 1 then
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

# (Optional)
init-command:
  # (Optional)
  language: lua
  # (Optional)
  use-prelude: true
  # (Required)
  script: |
    print("Hello from init-command")
```

## Building locally

### Install Rust

* [rustup][rustup] is recommended
* Building with rustup has been tested on Linux, Windows and macOS

### Clone workspace

```bash
cd /path/to/repos
git clone https://gitlab.com/rcook/rws.git
cd /path/to/repos/rws
cargo build
```

## Licence

[MIT License][licence]

[home]: https://github.com/rcook/rws
[licence]: LICENSE
[rustup]: https://rustup.rs/
