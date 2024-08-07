from collections.abc import Callable
from stone_color.messages import *
from stone_color.color import *

import teamserver
import commands

COMMANDS: dict[str, Callable] = {
    "jobs": commands.jobs,
    "quit": commands.quit,
}

def run(ts: teamserver.TeamServer):
    commands.init(ts)

    while True:
        try:
            prompt = input(formatf("{#underline}", ts.username, "{#reset}", sep="") + " >> ").split()

            if len(prompt) <= 0:
                continue

            if prompt[0] in list(COMMANDS.keys()):
                COMMANDS[prompt[0]]()

        except (KeyboardInterrupt, EOFError):
            pass 
