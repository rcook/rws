# Richard's Workspace Tool

[![AppVeyor build status](https://ci.appveyor.com/api/projects/status/w2nmlj9ljfkp10kh/branch/master?svg=true)](https://ci.appveyor.com/project/rcook/rws/branch/master)

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

## CI/CD

* [AppVeyor][appveyor-rws]
* [GitLab][gitlab-rws]

## Building locally

### Install Rust

* [rustup][rustup] is recommended
* Building with rustup has been tested on Linux, Windows and macOS

### Clone workspace

```bash
cd /path/to/repos
git clone https://gitlab.com/rcook/rws.git
```

### Create `Cargo.toml`

Create a `Cargo.toml` file using sed, PowerShell or my [rbbt][rbbt] tool.

#### sed

```bash
cd /path/to/repos/rws
sed -e 's/$cargo_version/0.1.0/g' -e 's/$full_version/0.1.0/g' _rbbt_templates/Cargo.toml > Cargo.toml
```

#### PowerShell

```ps
cd C:\path\to\repos\rws
(Get-Content -Path .\_rbbt_templates\Cargo.toml) -replace '\$cargo_version|\$full_version', '0.1.0' | Out-File -Path Cargo.toml -Encoding ASCII
```

#### Using rbbt

From Bash etc.:

```bash
cd /path/to/repos/rws
curl -sS https://gitlab.com/rcook/rbbt/-/raw/v0.4.4/rbbt | bash
```

From PowerShell:

```ps
cd C:\path\to\repos\rws
Invoke-WebRequest -Uri https://gitlab.com/rcook/rbbt/-/raw/v0.4.4/rbbt.ps1 | Invoke-Expression
```

### Build

```bash
cd /path/to/repos/rws
cargo build
```

## Licence

[MIT License][licence]

[appveyor-rws]: https://ci.appveyor.com/project/rcook/rws
[gitlab-rws]: https://gitlab.com/rcook/rws/pipelines
[home]: https://gitlab.com/rcook/rws
[licence]: LICENSE
[rbbt]: https://gitlab.com/rcook/rbbt
[rustup]: https://rustup.rs/
