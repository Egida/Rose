import json
from websockets.sync import client as wsclient
from enum import Enum

class Method(Enum): 
    Auth        = 1 
    ServerInfo  = 2 

class ClientError(Enum):
    InvalidJsonFormat       = 1
    InvalidJsonDatatype     = 2 
    NotFoundJsonParameter   = 3 
    Invalidusername         = 4
    InvalidPassword         = 5 

class TeamServer:
    def __init__(self, address: str, username: str, password: str) -> None:
        self.address = address
        self.username = username
        self.password = password

        self.stream = wsclient.connect(address)

    # Return: Nonetype means no errors
    def auth(self) -> None | tuple[ClientError, str]:
        self.stream.send(json.dumps(
            {
                "method": Method.Auth.name, 
                "parameters": {
                    "username": self.username,
                    "password": self.password 
                }
            }
        ))

        data = self.stream.recv()
        jsondata: dict = json.loads(data)
        jsondata_error = jsondata.get("error")

        if jsondata_error:
            return (ClientError[jsondata_error], jsondata["message"])
        else:
            return None

