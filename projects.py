from __future__ import print_function

import argparse
import os
from rwslib.markup import *
from rwslib.workspace import *

def main():
    parser = argparse.ArgumentParser(description="List Git-based projects")

    _ = parser.parse_args()

    workspace = Workspace(os.getcwd())
    for project_dir in workspace.project_dirs_topo:
        print(project_dir)

if __name__ == "__main__":
    init_markup()
    main()
