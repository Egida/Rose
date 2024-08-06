from collections.abc import Callable
from stone_color.messages import *
from stone_color.color import *

import teamserver
import commands

COMMANDS: dict[str, Callable] = {
    "test": commands.test,
    "quit": quit
}

def run(ts: teamserver.TeamServer):
    try:
        while True:
                prompt = input(formatf("{#italic}test{#reset}") + " >> ").split()

                if len(prompt) <= 0:
                    continue

                if prompt[0] in list(COMMANDS.keys()):
                    COMMANDS[prompt[0]]()
    except (KeyboardInterrupt, EOFError):
        pass 
