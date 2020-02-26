import contextlib
import os
import subprocess

@contextlib.contextmanager
def saved_cwd(dir):
    saved_dir = os.getcwd()
    try:
        os.chdir(dir)
        yield
    finally:
        os.chdir(saved_dir)

def append_source(lines, indent, source):
    indent_str = "  " * indent
    if isinstance(source, str) or isinstance(source, unicode):
        lines.extend([ "{}{}".format(indent_str, x) for x in source.splitlines() ])
    elif isinstance(source, list):
        for line in source:
            lines.append("{}{}".format(indent_str, line))
    else:
        raise ValueError("Unsupported source type {}".format(type(source)))

def append_block(lines, indent, tag, source):
    append_source(lines, indent, "##### BEGIN {} #####".format(tag))
    append_source(lines, indent, source)
    append_source(lines, indent, "##### END {} #####".format(tag))

def build_python_command_func(preamble, command_script):
    lines = []
    append_block(lines, 0, "PREAMBLE", preamble)
    append_source(lines, 0, "def __user_func__():")

    temp = command_script.splitlines()
    if len(temp) > 1:
        append_block(lines, 1, "COMMAND BODY", temp[0:-1])
    if len(temp) > 0:
        append_block(lines, 1, "COMMAND RETURN", "return " + temp[-1])
    return "\n".join(lines)

def run_user_command(config, command, project_dir):
    language = command["language"]
    if language == "shell":
        script = command["script"]
        output = subprocess.check_output(script, cwd=project_dir, shell=True)
        return [ x for x in [ x.strip() for x in output.splitlines() ] if len(x) > 0 and not x.startswith("#") ]

    if language == "python":
        source = build_python_command_func(config.get("python-preamble", ""), command["script"])
        with saved_cwd(project_dir):
            scope = {}
            exec(source, scope)
            return scope["__user_func__"]()

    raise ValueError("Unsupported language {}".format(language))