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
    else:
        headers = list(map(str.capitalize, list(js[1].keys())))
        data = []
        for attack in js:
            data.append(list(attack.values()))
        
        printf("\n", tables.ascii_table(headers, data)[1:])

def quit():
    ts.close()
    sys.exit(0)
