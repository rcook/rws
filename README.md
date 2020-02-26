# Richard's Workspace Tools

## `each` helper

_Run commands in Git-based projects_

### `exec` subcommand

_Run command in each project directory_

### `git` subcommand

_Run Git command in each project directory_

### `ps` subcommand

_Run PowerShell command in each project directory_

## `projects` helper

_List Git-based projects_

## Example `rws-workspace.yaml` configuration

```yaml
#dependency-command:
#  language: shell
#  script: if [ -f Config ]; then cat Config; fi

python-preamble: |
  from __future__ import print_function
  import os

  def parse_config_lines(lines):
    return [ x for x in [ x.strip() for x in lines ] if len(x) > 0 and not x.startswith("#") ]

  def read_config_lines(path, default_value=None):
    if os.path.isfile(path):
      with open(path) as f:
        return parse_config_lines(f.readlines())
    else:
      if default_value is None:
        raise RuntimeError("Configuration file {} was not found".format(path))
      return default_value

dependency-command:
  language: python
  script: read_config_lines("Config", [])

excluded-projects:
- rws
```

* `dependency-command`: shell command run in order to yield a workspace's dependency graph
* `excluded-projects`: list of projects to exclude from workspace

## Licence

[MIT License][licence]

[licence]: LICENSE
