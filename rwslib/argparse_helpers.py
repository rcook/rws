from rwslib.string_helpers import *

def add_boolean_switch(parser, name, default_value, true_help, false_help):
    group = parser.add_mutually_exclusive_group()
    dest = name.replace("-", "_")
    if default_value:
        group.add_argument("--{}".format(name), dest=dest, action="store_true", help="{} (default)".format(true_help))
        group.add_argument("--no-{}".format(name), dest=dest, action="store_false", help=false_help)
    else:
        group.add_argument("--{}".format(name), dest=dest, action="store_true", help=true_help)
        group.add_argument("--no-{}".format(name), dest=dest, action="store_false", help="{} (default)".format(false_help))
    group.set_defaults(**{ dest: default_value })

def add_command_argument(parser):
    parser.add_argument("command", nargs="+", metavar="COMMAND", help="commands and arguments")

def make_subparser(subparsers, name, help):
    return subparsers.add_parser(name, description=capitalize_initial(help), help=help)
