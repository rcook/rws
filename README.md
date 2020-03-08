# Richard's Workspace Tool

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

Create a `Cargo.toml` file using sed or equivalent:

```bash
cd /path/to/repos/rws
sed _rbbt_templates/Cargo.toml -e 's/$cargo_version\|$full_version/0.1.0/g' > Cargo.toml
```

Or use [RBBT][rbbt] to generate it for you:

```bash
cd /path/to/repos/rws
curl https://gitlab.com/rcook/rbbt/-/raw/v0.3/rbbt | bash
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
