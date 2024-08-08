import asyncio
import json
import websocket
from enum import Enum

class Method(Enum): 
    Auth        = 1 
    Jobs        = 2 
    ListAgents  = 3

class ClientError(Enum):
    InvalidJsonFormat       = 1
    InvalidJsonDatatype     = 2 
    NotFoundJsonParameter   = 3 
    Invalidusername         = 4
    InvalidPassword         = 5 
    NoAuth                  = 6 

class CError:
    def __init__(self, type: ClientError, message: str) -> None:
        self.type = type
        self.message = message

    def __str__(self) -> str:
        return self.message

class TeamServer:
    def __init__(self, address: str, username: str, password: str, connect = True) -> None:
        self.address = address
        self.username = username
        self.password = password

        self.ws = websocket.WebSocket()

        if connect:
            self.ws.connect(self.address)

    def __empty__(): 
        return TeamServer("", "", "", False)

    def send2(self, method: Method, parameters = {}) -> int:
        return self.ws.send(json.dumps({
            "method": method,
            "parameters": parameters
        }))

    def recv2(self) -> dict | CError:
        data = self.ws.recv()
        jsondata: dict = json.loads(data)
        jsondata_error = jsondata.get("error")

        if jsondata_error:
            return CError(ClientError[jsondata_error], jsondata["message"])
        else:
            return jsondata
       
    # Return: Nonetype means no errors
    def auth(self) -> None | CError:
        self.ws.send(json.dumps(
            {
                "method": Method.Auth.name, 
                "parameters": {
                    "username": self.username,
                    "password": self.password 
                }
            }
        ))

        data = self.recv2()

        if not isinstance(data, CError):
            return None

        return data
        
    def get_jobs(self) -> list[dict] | CError:
        self.ws.send(json.dumps({
            "method": Method.Jobs.name,
            "parameters": {}
        }))

        data = self.ws.recv()

        jsondata: dict = json.loads(data)
        jsondata_error = jsondata.get("error")

        if jsondata_error:
            return CError(ClientError[jsondata_error], jsondata["message"])
        else:
            return jsondata["data"] 

    def get_agents(self) -> list[dict] | CError:
        self.ws.send(json.dumps({
            "method": Method.ListAgents.name,
            "parameters": {}
        }))

        data = self.ws.recv()

        jsondata: dict = json.loads(data)
        jsondata_error = jsondata.get("error")

        if jsondata_error:
            return CError(ClientError[jsondata_error], jsondata["message"])
        else:
            return jsondata["data"] 
    
    def close(self):
        self.ws.close()
