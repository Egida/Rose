import inquirer
import getpass
import tomllib
import asyncio
import os

from stone_color.messages import *

import teamserver
import console

def clear():
    if os.name == "nt":
        os.system("cls")
    else: 
        os.system("clear")


PROFILE_TEMPLATE = """
"address" = "{}"

[credentials]
"username" = "{}"
"password" = "{}"
""".strip()

def create_new_profile() -> bool:
    server_addr = input("Server address (ws://localhost:5555) :: ").strip()
    username = input("Username :: ").strip()

    while True:
        password = getpass.getpass("Password :: ").strip()
        password_confirm = getpass.getpass("Confirm password :: ").strip()
        if password_confirm == password:
            break
        else:
            errorf("Passwords don't matches")

    profile_name = input("Profile name (whitespaces will replace with '_') :: ")

    confirm = input("Are you sure? [y/N] ")
    if confirm.lower() == "y":
        with open(os.path.join("profiles", profile_name.replace(" ", "_")) + ".toml", "w") as fd:
           fd.write(PROFILE_TEMPLATE.format(server_addr, username, password)) 

        return True
    else:
        return False 

def get_profile(profile_name: str) -> tuple[str, str, str] | None:
    server_addr: str | None
    username: str | None
    password: str | None

    if not os.path.exists(os.path.join("profiles", profile_name)):
        errorf(f"profile '{profile_name}' don't exists")
        return None 

    with open(os.path.join("profiles", profile_name), "rb") as fd:
        profile = tomllib.load(fd)
        server_addr = profile.get("address")
        credentials = profile.get("credentials")

        if not credentials:
            errorf("[credentials] not found in profile configuration")
            return None

        username = credentials.get("username")
        password = credentials.get("password")

        if not server_addr or not username or not password:
            errorf("Profile configuration is invalid:")
            printf(f"\taddress = {server_addr}")
            printf(f"\tcredentials.username = {username}")
            printf(f"\tcredentials.password = {password}")
            return None
    
    return (server_addr, username, password)

def main() -> int:
    profiles = []
    for file in os.listdir("profiles"):
        if file.endswith(".toml"):
            profiles.append(file.replace("_", " "))

    questions = [
        inquirer.List(
            "menu",
            message="What profile do you want use?",
            choices=[*profiles, "Add a new profile"]
        )
    ]

    answer = inquirer.prompt(questions)

    if answer is None:
        return 1

    answer = answer["menu"]

    match answer:
        case "Add a new profile":
            if not create_new_profile():
                return 1
        case _:
            profile_data = get_profile(answer)
            if not profile_data:
                return 1

            try:
                ts = teamserver.TeamServer(*profile_data)
            except Exception as e:
                errorf(e) 
                return 1

            error = ts.auth()

            if error:
                errorf("Auth:", error)
                return 1
            
            console.run(ts)

    return 0

if __name__ == "__main__":
    quit(main())
