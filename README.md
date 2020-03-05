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
  preamble: dofile("../my-shared-script.lua")
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
    if prelude.is_file("Config") then
      return prelude.parse_config(prelude.read_file_lines("Config"))
    else
      return {}
    end
```

## Licence

[MIT License][licence]

[home]: https://github.com/rcook/rws
[licence]: LICENSE
