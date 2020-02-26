from rwslib.argparse_helpers import *

class ExecCommandBuilder(object):
    @staticmethod
    def make_subparser(subparsers):
        p = make_subparser(subparsers, "exec", "run command in each project directory")
        add_command_argument(p)
        return p

    def __init__(self, args):
        self._args = args

    def make_command(self, dir):
        return self._args.command

class GitCommandBuilder(object):
    @staticmethod
    def make_subparser(subparsers):
        p = make_subparser(subparsers, "git", "run Git command in each project directory")
        add_command_argument(p)
        return p

    def __init__(self, args):
        self._args = args

    def make_command(self, dir):
        return [ "git" ] + self._args.command

class PowerShellCommandBuilder(object):
    @staticmethod
    def make_subparser(subparsers):
        p = make_subparser(subparsers, "ps", "run PowerShell command in each project directory")
        add_boolean_switch(p, "profile", False, "load profile in each shell", "do not load profile in each shell")
        add_command_argument(p)
        return p

    def __init__(self, args):
        self._args = args

    def make_command(self, dir):
        if self._args.profile:
            return [ "powershell.exe" ] + self._args.command
        else:
            return [ "powershell.exe", "-NoProfile" ] + self._args.command
