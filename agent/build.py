import os
import tomllib

def get_rust_bin(app: str, target, suffix, mode = "debug"):
    return os.path.join("target", target, mode, app + suffix)

def main():
    cargo_toml = tomllib.load(open("Cargo.toml", "rb"))
    config = tomllib.load(open(".cargo/config.toml", "rb"))
    target = config["build"]["target"]

    if "windows" in target:
        suffix = ".exe"
    else:
        suffix = ""

    path = get_rust_bin(cargo_toml["package"]["name"], target, suffix)

if __name__ == "__main__":
    main()
