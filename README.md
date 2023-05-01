# Richard's Workspace Tool

[![CI](https://github.com/rcook/rws/actions/workflows/ci.yaml/badge.svg)][ci-workflow]
[![Release](https://github.com/rcook/rws/actions/workflows/release.yaml/badge.svg)][release-workflow]

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
default_language: lua

# (Optional)
lua_config:
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
  use_prelude: true

# (Optional) (or specify "dependency_command")
dependencies:
  aaa:
  - bbb
  - ccc
  ccc:
  - ddd
  - eee

# (Optional) (or specify "dependencies")
dependency_command:
  # (Optional)
  language: lua
  # (Optional)
  use_prelude: true
  # (Required)
  script: |
    if prelude.is_file(DEPS_FILE_NAME) then
      return parse_config_lines(prelude.read_file_lines(DEPS_FILE_NAME))
    else
      return { }
    end

# (Optional)
excluded_projects:
- fff
- ggg

# (Optional)
init_command:
  # (Optional)
  language: lua
  # (Optional)
  use_prelude: true
  # (Required)
  script: |
    print("Hello from init_command")
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

[ci-workflow]: https://github.com/rcook/rws/actions/workflows/ci.yaml
[home]: https://github.com/rcook/rws
[licence]: LICENSE
[release-workflow]: https://github.com/rcook/rws/actions/workflows/release.yaml
[rustup]: https://rustup.rs/
