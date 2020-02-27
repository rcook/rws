from __future__ import print_function

import argparse
from rwslib.markup import *
from rwslib.workspace import *

def show_project_dirs(order, project_dirs):
    if len(project_dirs) > 0:
        print("Project directories ({} order):".format(order))
        for project_dir in project_dirs:
            print_markup("  [[cyan]]{}[[/cyan]]".format(project_dir))
    else:
        print_markup("Project directories ({} order): [[cyan]](none)[[/cyan]]".format(order))

def main():
    parser = argparse.ArgumentParser(description="Show Git-based workspace information")

    _ = parser.parse_args()

    workspace = Workspace.find(os.getcwd())

    print_markup("Workspace directory: [[cyan]]{}[[/cyan]]".format(workspace.workspace_dir))

    if workspace.config_path is None:
        print_markup("Workspace configuration file: [[cyan]](none)[[/cyan]]")
    else:
        print_markup("Workspace configuration file: [[cyan]]{}[[/cyan]]".format(workspace.config_path))

    show_project_dirs("alpha", workspace.project_dirs_alpha)
    show_project_dirs("topo", workspace.project_dirs_topo)

if __name__ == "__main__":
    init_markup()
    main()
