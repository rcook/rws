from __future__ import print_function

import argparse
import os
import subprocess
from rwslib.argparse_helpers import *
from rwslib.command_builders import *
from rwslib.markup import *
from rwslib.workspace import *

COMMAND_BUILDER_TYPES = [
    ExecCommandBuilder,
    GitCommandBuilder,
    PowerShellCommandBuilder
]

TOPO_ARG_VALUE = "topo"

def run_command(project_dir, command):
    try:
        status = subprocess.call(command, cwd=project_dir)
        return None if status == 0 else status
    except OSError as e:
        return e

def format_result(project_dir, result):
    if result is None:
        return "Command succeeded for project {}".format(project_dir)
    if isinstance(result, int):
        return "Command failed for project {} with status {}".format(project_dir, result)
    if isinstance(result, Exception):
        return "Command failed for project {} with error {}".format(project_dir, result)
    raise ValueError("Invalid result {}".format(result))

def get_project_dirs(workspace, order):
    if order == "alpha":
        return workspace.project_dirs_alpha
    elif order == TOPO_ARG_VALUE:
        return workspace.project_dirs_topo
    else:
        raise ValueError("Invalid order {}".format(order))

def main():
    parser = argparse.ArgumentParser(description="Run commands in Git-based projects")
    add_boolean_switch(parser, "fail-fast", True, "fail on first error", "run command in all project directories")
    parser.add_argument(
        "--order",
        type=str,
        default=TOPO_ARG_VALUE,
        choices=[TOPO_ARG_VALUE, "alpha"],
        help="order of project traversal (default: {})".format(TOPO_ARG_VALUE)
    )

    subparsers = parser.add_subparsers()
    for t in COMMAND_BUILDER_TYPES:
        p = t.make_subparser(subparsers)
        p.set_defaults(command_builder_type=t)

    args = parser.parse_args()
    command_builder = args.command_builder_type(args)

    workspace = Workspace.find(os.getcwd())
    is_success = True

    for project_dir in get_project_dirs(workspace, args.order):
        command = command_builder.make_command(project_dir)

        print_markup("[[cyan]]Project {}[[/cyan]]".format(project_dir))
        result = run_command(project_dir, command)

        m = format_result(project_dir, result)

        if result is None:
            print_markup("[[green]]{}[[/green]]\n".format(m))
        else:
            if args.fail_fast:
                print_markup("[[red]]{}[[/red]]\n".format(m))
                exit(1)
            is_success = False
            print_markup("[[yellow]]{}[[/yellow]]\n".format(m))

    if is_success:
        print_markup("[[green]]Command succeeded for all projects[[/green]]")
        exit(0)
    else:
        print_markup("[[red]]Command failed for one or more projects[[/red]]")
        exit(1)

if __name__ == "__main__":
    init_markup()
    main()
