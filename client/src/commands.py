import sys

from stone_color import tables
from stone_color import processbar
from stone_color.messages import *

from teamserver import CError, TeamServer

global ts
ts: TeamServer = TeamServer.__empty__() 

def init(_ts: TeamServer):
    global ts
    ts = _ts

def jobs():
    js = ts.get_jobs()
    if isinstance(js, CError):
        errorf(js)
    elif len(js) <= 0:
        errorf("No active jobs found")
    else:
        headers = list(map(str.capitalize, list(js[0].keys())))
        data = []
        for attack in js:
            data.append(list(attack.values()))
        
        printf("\n", tables.ascii_table(headers, data), sep="")

def listagents():
    agents = ts.get_agents()
    if isinstance(agents, CError):
        errorf(agents)
    elif len(agents) <= 0:
        errorf("No active agents found")
    else:
        headers = list(map(str.capitalize, list(agents[0].keys())))
        data = []
        for attack in agents:
            data.append(list(attack.values()))
        
        printf("\n", tables.ascii_table(headers, data), sep="")


def quit():
    ts.close()
    sys.exit(0)
