from __future__ import print_function

import colorama
import platform
from colorama import Fore, Style

TAG_BEGIN = "[["
TAG_END = "]]"
COLOURS = {
    "cyan": Fore.CYAN,
    "green": Fore.GREEN,
    "red": Fore.RED,
    "yellow": Fore.YELLOW
}

def init_markup():
    is_windows = any(platform.win32_ver())
    if is_windows:
        colorama.init(convert=True)

def print_markup(markup):
    idx = 0
    colours = []
    while True:
        tag_begin_idx = markup.find(TAG_BEGIN, idx)
        if tag_begin_idx == -1:
            print(markup[idx:] + Style.RESET_ALL)
            if len(colours) > 0:
                raise ValueError("Markup has unbalanced tags")
            return
        print(markup[idx:tag_begin_idx], end="")
        tag_end_idx = markup.find(TAG_END, tag_begin_idx + len(TAG_BEGIN))
        tag = markup[tag_begin_idx + len(TAG_END):tag_end_idx]
        if tag.startswith("/"):
            base_tag = tag[1:]
            if COLOURS[base_tag] != colours.pop():
                raise RuntimeError("Unexpected {} tag".format(base_tag))
        else:
            colours.append(COLOURS[tag])

        colour = Style.RESET_ALL if len(colours) == 0 else colours[-1]
        print(colour, end="")

        idx = tag_end_idx + 2
