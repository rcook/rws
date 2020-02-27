import os
import yaml
from rwslib.graph import *
from rwslib.scripting import *
from sets import Set

DEFAULT_CONFIG = {}
CONFIG_FILE_NAME = "rws-workspace.yaml"

def read_config(config_path):
    with open(config_path, "rt") as f:
        return yaml.load(f, Loader=yaml.BaseLoader)

def resolve_project_dirs(workspace_dir, lines):
    return [ os.path.join(workspace_dir, line) for line in lines ]

def get_project_dirs(workspace_dir, excluded_project_dirs):
    paths = [ os.path.join(workspace_dir, d) for d in os.listdir(workspace_dir) ]
    all_project_dirs = [ p for p in paths if os.path.isdir(p) and os.path.isdir(os.path.join(p, ".git")) ]
    return [ d for d in all_project_dirs if d not in excluded_project_dirs ]

def topo_from_dependency_command(workspace_dir, config, project_dirs_alpha, dependency_command):
    g = MappedGraph()
    for project_dir in reversed(project_dirs_alpha):
        g.add_edge(project_dir, project_dir)
        deps = resolve_project_dirs(workspace_dir, run_user_command(config, dependency_command, project_dir))
        for dep in deps:
            g.add_edge(dep, project_dir)
    return g.topo_sort()

def topo_from_dependencies(workspace_dir, config, project_dirs_alpha, dependencies):
    all_deps = {}
    for k, v in dependencies.items():
        all_deps[os.path.join(workspace_dir, k)] = [ os.path.join(workspace_dir, x) for x in v ]

    g = MappedGraph()
    for project_dir in reversed(project_dirs_alpha):
        g.add_edge(project_dir, project_dir)
        for dep in all_deps.get(project_dir, []):
            g.add_edge(dep, project_dir)
    return g.topo_sort()

class Workspace(object):
    @staticmethod
    def find(start_dir):
        current_dir = start_dir
        while True:
            config_path = os.path.join(current_dir, CONFIG_FILE_NAME)
            if os.path.isfile(config_path):
                return Workspace(current_dir)
            next_dir = os.path.dirname(current_dir)
            if next_dir == current_dir:
                return Workspace(start_dir)
            current_dir = next_dir

    @property
    def workspace_dir(self):
        return self._workspace_dir

    @property
    def config_path(self):
        return self._config_path

    @property
    def project_dirs_alpha(self):
        return self._project_dirs_alpha

    @property
    def project_dirs_topo(self):
        return self._project_dirs_topo

    def __init__(self, workspace_dir):
        self._workspace_dir = os.path.abspath(workspace_dir)
        config_path = os.path.join(self._workspace_dir, CONFIG_FILE_NAME)
        self._config_path = config_path if os.path.isfile(config_path) else None
        config = DEFAULT_CONFIG if self._config_path is None else read_config(self._config_path)

        dependency_command = config.get("dependency-command")
        dependencies = config.get("dependencies")
        excluded_project_dirs = resolve_project_dirs(workspace_dir, config.get("excluded-projects", []))
        self._project_dirs_alpha = sorted(get_project_dirs(self._workspace_dir, excluded_project_dirs))

        if dependency_command is not None and dependencies is not None:
            raise RuntimeError("Workspace cannot specify both dependency-command and dependencies")
        elif dependency_command is not None:
            # Dependencies yielded dynamically by running external command/script
            self._project_dirs_topo = topo_from_dependency_command(workspace_dir, config, self._project_dirs_alpha, dependency_command)
        elif dependencies is not None:
            # Dependencies expressed in workspace configuration
            self._project_dirs_topo = topo_from_dependencies(workspace_dir, config, self._project_dirs_alpha, dependencies)
        else:
            # No dependencies
            self._project_dirs_topo = self._project_dirs_alpha
