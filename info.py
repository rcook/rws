from __future__ import print_function

import argparse
from rwslib.markup import *
from rwslib.workspace import *

def main():
    parser = argparse.ArgumentParser(description="Show Git-based workspace information")

    _ = parser.parse_args()

    workspace = Workspace(os.getcwd())

    if workspace.config_path is None:
        print_markup("Workspace configuration file: [[cyan]](none)[[/cyan]]")
    else:
        print_markup("Workspace configuration file: [[cyan]]{}[[/cyan]]".format(workspace.config_path))

    print("Project directories (alpha order):")
    for project_dir in workspace.project_dirs_alpha:
        print_markup("  [[cyan]]{}[[/cyan]]".format(project_dir))

    print("Project directories (topo order):")
    for project_dir in workspace.project_dirs_topo:
        print_markup("  [[cyan]]{}[[/cyan]]".format(project_dir))

if __name__ == "__main__":
    init_markup()
    main()
