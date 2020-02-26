import os
import yaml
from rwslib.scripting import *
from rwslib.topo import *
from sets import Set

def read_config(workspace_dir):
    config_path = os.path.join(workspace_dir, "rws-workspace.yaml")
    with open(config_path, "rt") as f:
        return yaml.load(f, Loader=yaml.BaseLoader)

def resolve_project_dirs(workspace_dir, lines):
    return [ os.path.join(workspace_dir, line) for line in lines ]

def get_project_dirs(workspace_dir, excluded_project_dirs):
    paths = [ os.path.join(workspace_dir, d) for d in os.listdir(workspace_dir) ]
    all_project_dirs = [ p for p in paths if os.path.isdir(p) and os.path.isdir(os.path.join(p, ".git")) ]
    return [ d for d in all_project_dirs if d not in excluded_project_dirs ]

class Workspace(object):
    @property
    def project_dirs(self):
        return self._project_dirs

    @property
    def ordered_project_dirs(self):
        return self._ordered_project_dirs

    def __init__(self, workspace_dir):
        self._workspace_dir = os.path.abspath(workspace_dir)
        config = read_config(workspace_dir)
        dependency_command = config.get("dependency-command")
        excluded_project_dirs = resolve_project_dirs(workspace_dir, config["excluded-projects"])

        self._project_dirs = sorted(get_project_dirs(self._workspace_dir, excluded_project_dirs))
        g = MappedGraph()
        for project_dir in reversed(self._project_dirs):
            g.add_edge(project_dir, project_dir)
            if dependency_command is not None:
                deps = resolve_project_dirs(workspace_dir, run_user_command(config, dependency_command, project_dir))
                for dep in deps:
                    g.add_edge(dep, project_dir)
        self._ordered_project_dirs = g.topo_sort()
