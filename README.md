# Richard's Workspace Tools

[Official home page][home]

## `each` helper

_Run commands in Git-based projects_

### `exec` subcommand

_Run command in each project directory_

### `git` subcommand

_Run Git command in each project directory_

### `ps` subcommand

_Run PowerShell command in each project directory_

## `info` helper

_Show Git-based workspace information_

## `topo` helper

_List Git-based projects in topological order_

## `rws-workspace.yaml` configuration

* `dependency-command`: shell command run in order to yield a workspace's dependency graph
* `excluded-projects`: list of projects to exclude from workspace

## Example `rws-workspace.yaml` configurations

* [Example 1][example-1]: Unix-style shell dependency command
* [Example 2][example-2]: Python-based dependency command (`DEPENDENCIES` file in each project)
* [Example 3][example-3]: Dependencies specified in workspace configuration

## Install dependencies

```
$ pip install --user colorama pyyaml
```

## Licence

[MIT License][licence]

[home]: https://github.com/rcook/rws
[licence]: LICENSE
[example-1]: examples/rws-workspace1.yaml
[example-2]: examples/rws-workspace2.yaml
