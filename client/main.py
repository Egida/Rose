import stone_color
import json
from websockets.sync.client import connect

from enum import Enum

class Methods: 
    Auth = "Auth"
    ServerInfo = "ServerInfo"

def main():
    with connect("ws://localhost:5555") as stream:
        stream.send(json.dumps({"method": "Auth", "parameters": {"username": "test123", "password": "SecurePassword1234"}}))
        print(stream.recv())

if __name__ == "__main__":
    main()
