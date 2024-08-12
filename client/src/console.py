import os
import readline
from collections.abc import Callable
from stone_color.messages import *
from stone_color.color import *

import teamserver
import commands

COMMANDS: dict[str, Callable] = {
    "jobs": commands.jobs,
    "list": commands.listagents,
    "quit": commands.quit,
}

HISTORY_PATH = ".history"
COMPLETIONS = list(COMMANDS.keys())
current_candidates = []

def readline_init():
    if not os.path.exists(HISTORY_PATH):
        open(HISTORY_PATH, "w")

    readline.read_history_file(HISTORY_PATH)
    readline.set_completer(readline_completer)
    readline.parse_and_bind("tab: complete")

def readline_completer(text: str, state):
    if text:
        matches = [s for s in COMPLETIONS if s and s.startswith(text)]
    else:
        matches = COMPLETIONS[:]

    try:
        return matches[state]
    except IndexError:
        return None

def run(ts: teamserver.TeamServer):
    readline_init()
    commands.init(ts)

    while True:
        try:
            prompt = input(formatf("{#underline}", ts.username, "{#reset}", sep="") + " >> ").split()

            if len(prompt) <= 0:
                continue

            readline.write_history_file(HISTORY_PATH)

            if prompt[0] in list(COMMANDS.keys()):
                COMMANDS[prompt[0]]()

        except KeyboardInterrupt:
            if len(readline.get_line_buffer()) <= 0:
                printf("To quit, type 'quit'")
            else:
                printf()

        except EOFError:
            printf()

